use {
    std::{
        collections::HashMap,
        fmt,
        io,
        iter,
        sync::Arc,
    },
    derive_more::From,
    itertools::Itertools as _,
    lazy_static::lazy_static,
    pyo3::{
        PyDowncastError,
        //exceptions::PyValueError,
        prelude::*,
        types::{
            PyBool,
            PyDict,
            PyType,
        },
    },
    regex::Regex,
    crate::{
        Check,
        Item,
        Rando,
        model::Medallion,
    },
};

lazy_static! {
    static ref EVENT_REGEX: Regex = Regex::new("^\\w+").expect("failed to compile event regex");
}

#[derive(Debug, Default)]
struct Args<'a>(HashMap<&'a str, (&'a PyAny, &'a Args<'a>)>);

#[derive(Debug, Clone, Copy)]
pub enum TimeRange {
    /// 06:00–18:00.
    ///
    /// Playing Sun's Song during `Night` sets the time to 12:00.
    Day,
    /// 18:00–06:00.
    ///
    /// Playing Sun's Song during `Day` sets the time to 00:00.
    Night,
    /// The time of day when Dampé's Heart-Pounding Gravedigging Tour is available: 18:00–21:00, a subset of `Night`.
    ///
    /// Going to outside Ganon's Castle sets the time to 18:01.
    Dampe,
}

#[derive(Debug, Clone)]
pub enum ExprParseError {
    InvalidLogicHelper,
    Io(Arc<io::Error>),
    Py(&'static str, Arc<PyErr>),
    TupleLength,
}

impl From<io::Error> for ExprParseError {
    fn from(e: io::Error) -> ExprParseError {
        ExprParseError::Io(Arc::new(e))
    }
}

impl From<PyErr> for ExprParseError {
    fn from(e: PyErr) -> ExprParseError {
        ExprParseError::Py("unknown", Arc::new(e))
    }
}

impl From<PyDowncastError<'_>> for ExprParseError {
    fn from(e: PyDowncastError<'_>) -> ExprParseError {
        ExprParseError::Py("downcast", Arc::new(e.into()))
    }
}

impl fmt::Display for ExprParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprParseError::InvalidLogicHelper => write!(f, "multiple ( found in logic helper"),
            ExprParseError::Io(e) => write!(f, "I/O error: {}", e),
            ExprParseError::Py(loc, e) => write!(f, "Python error in {}: {}", loc, e),
            ExprParseError::TupleLength => write!(f, "tuple does not have 2 values"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    All(Vec<Expr>),
    Any(Vec<Expr>),
    Age,
    Item(Item, u8),
    LitStr(String),
    Setting(String),
    StartingAge,
}

impl Expr {
    fn parse(py: Python<'_>, rando: &Rando, ast: &PyModule, expr: &PyAny, args: &Args<'_>) -> Result<Expr, ExprParseError> {
        // based on RuleParser.py as of 4f83414c49ff65ef2eb285667bcb153f11f1f9ef
        Ok(if ast.get("BoolOp")?.downcast::<PyType>()?.is_instance(expr)? {
            if ast.get("And")?.downcast::<PyType>()?.is_instance(expr.getattr("op")?)? {
                Expr::All(expr.getattr("values")?.iter()?.map(|rule| rule.at("next(expr.values)").and_then(|rule| Expr::parse(py, rando, ast, rule, args))).try_collect()?)
            } else if ast.get("Or")?.downcast::<PyType>()?.is_instance(expr.getattr("op")?)? {
                Expr::Any(expr.getattr("values")?.iter()?.map(|rule| rule.at("next(expr.values)").and_then(|rule| Expr::parse(py, rando, ast, rule, args))).try_collect()?)
            } else {
                unreachable!("found BoolOp expression with neither And nor Or: {}", display_expr(ast, expr))
            }
        } else if ast.get("Call")?.downcast::<PyType>()?.is_instance(expr)? {
            let name = expr.getattr("func")?.getattr("id")?.extract::<String>().at::<ExprParseError>("expr.func.id as String")?;
            // rule alias (LogicHelpers.json)
            if let Some((fn_def, fn_body)) = rando.logic_helpers()?.iter().find(|(fn_def, _)| fn_def.starts_with(&format!("{}(", name))) {
                let (fn_name, fn_params) = fn_def[..fn_def.len() - 1].split('(').collect_tuple().ok_or(ExprParseError::InvalidLogicHelper)?; //.ok_or_else(|| PyValueError::new_err("multiple ( found in logic helper"))?;
                assert_eq!(fn_name, name);
                let fn_params = fn_params.split(',');
                let kwargs = PyDict::new(py);
                kwargs.set_item("mode", "eval")?;
                let fn_args = fn_params.zip(expr.getattr("args")?.iter()?).map(|(k, v)| PyResult::Ok((k, (v?, args)))).try_collect()?;
                Expr::parse(py, rando, ast, ast.call_method("parse", (fn_body,), Some(kwargs)).at::<ExprParseError>("ast.parse logic helper fn expr")?.getattr("body").at::<ExprParseError>(".body logic helper fn expr")?, &Args(fn_args))?
            }
            else {
                unimplemented!("converting call expression with name {} into Expr", name)
            }
        } else if ast.get("Constant")?.downcast::<PyType>()?.is_instance(expr)? {
            Expr::LitStr(expr.getattr("value")?.extract().at::<ExprParseError>("expr.value as String")?)
        } else if ast.get("Name")?.downcast::<PyType>()?.is_instance(expr)? {
            let name = expr.getattr("id")?.extract::<String>().at::<ExprParseError>("expr.id as String")?;
            // logic helper parameter
            if let Some((expr, args)) = args.0.get(&*name) { Expr::parse(py, rando, ast, expr, args)? }
            // rule alias (LogicHelpers.json)
            else if let Some(helper) = rando.logic_helpers()?.get(&name) {
                let kwargs = PyDict::new(py);
                kwargs.set_item("mode", "eval")?;
                Expr::parse(py, rando, ast, ast.call_method("parse", (helper,), Some(kwargs)).at::<ExprParseError>("ast.parse simple logic helper expr")?.getattr("body").at::<ExprParseError>(".body simple logic helper expr")?, &Args::default())?
            }
            // escaped item (ItemList.item_table)
            else if let Some(item) = rando.escaped_items().at::<ExprParseError>("escaped_items")?.get(&name) {
                Expr::Item(item.clone(), 1)
            }
            // setting
            else if name == "starting_age" {
                Expr::StartingAge
            } else if rando.setting_infos().at::<ExprParseError>("setting_infos")?.contains(&name) {
                Expr::Setting(name)
            }
            //TODO attr of State (which ones are used?)
            // kwarg_defaults and allowed_globals
            else if name == "age" {
                Expr::Age
            }
            //TODO other kwarg_defaults and allowed_globals (spot, tod, TimeOfDay — which ones are used?)
            //TODO events (not sure how those work)
            else {
                unimplemented!("converting name expression {} into Expr", name)
            }
        } else if ast.get("Tuple")?.downcast::<PyType>()?.is_instance(expr)? {
            let (item, count) = expr.getattr("elts")?.iter()?.collect_tuple().ok_or(ExprParseError::TupleLength)?;
            let (item, count) = (item?, count?);
            let item_name = if ast.get("Constant")?.downcast::<PyType>()?.is_instance(item)? {
                item.getattr("value")?.extract::<String>().at::<ExprParseError>("item.value as String")?
            } else if ast.get("Name")?.downcast::<PyType>()?.is_instance(item)? {
                item.getattr("id")?.extract::<String>().at::<ExprParseError>("item.id as String")?
            } else {
                unimplemented!("converting {} into item to be counted", display_expr(ast, item))
            };
            let item = if let Some(item) = rando.escaped_items().at::<ExprParseError>("escaped_items")?.get(&item_name) {
                item.clone()
            } else {
                unimplemented!() //TODO unescaped item name or event
            };
            let count = if ast.get("Constant")?.downcast::<PyType>()?.is_instance(count)? {
                count.getattr("value")?.extract::<u8>().at::<ExprParseError>("count.value as u8")?
            } else if ast.get("Name")?.downcast::<PyType>()?.is_instance(count)? {
                unimplemented!() //TODO setting name
            } else {
                unimplemented!("converting {} into item count", display_expr(ast, count))
            };
            Expr::Item(item, count)
        } else {
            unimplemented!("converting expression {} into Expr", display_expr(ast, expr)) //TODO
        })
    }
}

#[derive(Debug, From, Clone)]
pub enum RuleParseError {
    AccessExprParse(ExprParseError),
    CallNumArgs,
    InvalidLogicHelper,
    Io(Arc<io::Error>),
    Py(&'static str, Arc<PyErr>),
    TupleLength,
}

impl From<io::Error> for RuleParseError {
    fn from(e: io::Error) -> RuleParseError {
        RuleParseError::Io(Arc::new(e))
    }
}

impl From<PyErr> for RuleParseError {
    fn from(e: PyErr) -> RuleParseError {
        RuleParseError::Py("unknown", Arc::new(e))
    }
}

impl From<PyDowncastError<'_>> for RuleParseError {
    fn from(e: PyDowncastError<'_>) -> RuleParseError {
        RuleParseError::Py("downcast", Arc::new(e.into()))
    }
}

impl fmt::Display for RuleParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleParseError::AccessExprParse(e) => write!(f, "failed to parse access expression: {}", e),
            RuleParseError::CallNumArgs => write!(f, "incorrect number of arguments in function call"),
            RuleParseError::InvalidLogicHelper => write!(f, "multiple ( found in logic helper"),
            RuleParseError::Io(e) => write!(f, "I/O error: {}", e),
            RuleParseError::Py(loc, e) => write!(f, "Python error in {}: {}", loc, e),
            RuleParseError::TupleLength => write!(f, "tuple does not have 2 values"),
        }
    }
}

trait ErrorExt {
    fn from_py_and_loc(loc: &'static str, e: PyErr) -> Self;
}

impl ErrorExt for ExprParseError {
    fn from_py_and_loc(loc: &'static str, e: PyErr) -> ExprParseError {
        ExprParseError::Py(loc, Arc::new(e))
    }
}

impl ErrorExt for RuleParseError {
    fn from_py_and_loc(loc: &'static str, e: PyErr) -> RuleParseError {
        RuleParseError::Py(loc, Arc::new(e))
    }
}

trait PyResultExt {
    type Ok;

    fn at<E: ErrorExt>(self, loc: &'static str) -> Result<Self::Ok, E>;
}

impl<T> PyResultExt for PyResult<T> {
    type Ok = T;

    fn at<E: ErrorExt>(self, loc: &'static str) -> Result<T, E> {
        self.map_err(|e| E::from_py_and_loc(loc, e))
    }
}

#[derive(Debug, Clone)]
pub enum Rule {
    All(Vec<Rule>),
    Any(Vec<Rule>),
    AnonymousEvent(Check, usize),
    Eq(Expr, Expr),
    Event(String),
    HasStones(u8),
    Item(Item, u8),
    Not(Box<Rule>),
    Setting(String),
    TrialActive(Medallion),
    Trick(String),
    Time(TimeRange),
    True,
}

impl Rule {
    pub fn parse(py: Python<'_>, rando: &Rando, ctx: &Check, rule: &str) -> Result<Rule, RuleParseError> {
        let ast = py.import("ast")?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("mode", "eval")?;
        Rule::parse_inner(py, rando, ctx, &mut (0..), ast, ast.call_method("parse", (rule,), Some(kwargs)).at::<RuleParseError>("ast.parse in parse")?.getattr("body").at::<RuleParseError>(".body in parse")?, &Args::default())
    }

    fn parse_inner(py: Python<'_>, rando: &Rando, ctx: &Check, seq: &mut impl Iterator<Item = usize>, ast: &PyModule, expr: &PyAny, args: &Args<'_>) -> Result<Rule, RuleParseError> {
        // based on RuleParser.py as of commit 4f83414c49ff65ef2eb285667bcb153f11f1f9ef
        Ok(if ast.get("BoolOp")?.downcast::<PyType>()?.is_instance(expr)? {
            if ast.get("And")?.downcast::<PyType>()?.is_instance(expr.getattr("op")?)? {
                Rule::All(expr.getattr("values")?.iter()?.map(|rule| rule.at("next(expr.values)").and_then(|rule| Rule::parse_inner(py, rando, ctx, seq, ast, rule, args))).try_collect()?)
            } else if ast.get("Or")?.downcast::<PyType>()?.is_instance(expr.getattr("op")?)? {
                Rule::Any(expr.getattr("values")?.iter()?.map(|rule| rule.at("next(expr.values)").and_then(|rule| Rule::parse_inner(py, rando, ctx, seq, ast, rule, args))).try_collect()?)
            } else {
                unreachable!("found BoolOp expression with neither And nor Or: {}", display_expr(ast, expr))
            }
        } else if ast.get("Call")?.downcast::<PyType>()?.is_instance(expr)? {
            let name = expr.getattr("func")?.getattr("id")?.extract::<String>().at::<RuleParseError>("expr.func.id as String")?;
            // attr of Rule_AST_Transformer (at, here)
            if name == "at" {
                Rule::AnonymousEvent(ctx.clone(), seq.next().expect("failed to get anonymous event ID"))
            } else if name == "here" {
                Rule::AnonymousEvent(ctx.clone(), seq.next().expect("failed to get anonymous event ID"))
            }
            // rule alias (LogicHelpers.json)
            else if let Some((fn_def, fn_body)) = rando.logic_helpers()?.iter().find(|(fn_def, _)| fn_def.starts_with(&format!("{}(", name))) {
                let (fn_name, fn_params) = fn_def[..fn_def.len() - 1].split('(').collect_tuple().ok_or(RuleParseError::InvalidLogicHelper)?; //.ok_or_else(|| PyValueError::new_err("multiple ( found in logic helper"))?;
                assert_eq!(fn_name, name);
                let fn_params = fn_params.split(',');
                let kwargs = PyDict::new(py);
                kwargs.set_item("mode", "eval")?;
                let fn_args = fn_params.zip(expr.getattr("args")?.iter()?).map(|(k, v)| PyResult::Ok((k, (v?, args)))).try_collect()?;
                Rule::parse_inner(py, rando, ctx, seq, ast, ast.call_method("parse", (fn_body,), Some(kwargs)).at::<RuleParseError>("ast.parse logic helper fn")?.getattr("body").at::<RuleParseError>(".body logic helper fn")?, &Args(fn_args))?
            }
            //TODO attr of State (which ones are used?)
            else if name == "has_stones" {
                let (count,) = expr.getattr("args")?.iter()?.collect_tuple().ok_or(RuleParseError::CallNumArgs)?;
                Rule::HasStones(count?.extract()?)
            }
            else {
                unimplemented!("converting call expression with name {} into Rule", name)
            }
        } else if ast.get("Compare")?.downcast::<PyType>()?.is_instance(expr)? {
            Rule::All(
                iter::once(expr.getattr("left")?)
                    .chain(expr.getattr("comparators")?.iter()?.collect::<PyResult<Vec<_>>>()?.into_iter())
                    .tuple_windows()
                    .zip(expr.getattr("ops")?.iter()?.collect::<PyResult<Vec<_>>>()?.into_iter())
                    .map(|((left, right), op)| {
                        let left = Expr::parse(py, rando, ast, left, args)?;
                        let right = Expr::parse(py, rando, ast, right, args)?;
                        Ok::<_, RuleParseError>(if ast.get("Eq")?.downcast::<PyType>()?.is_instance(op)? {
                            Rule::Eq(left, right)
                        } else if ast.get("NotEq")?.downcast::<PyType>()?.is_instance(op)? {
                            Rule::Not(Box::new(Rule::Eq(left, right)))
                        } else {
                            unimplemented!("found Compare expression with non-Eq operator {}", op)
                        })
                    })
                    .try_collect()?
            )
        } else if ast.get("Constant")?.downcast::<PyType>()?.is_instance(expr)? {
            let constant = expr.getattr("value")?;
            if constant.downcast::<PyBool>().map_or(false, |b| b == PyBool::new(py, true)) {
                Rule::True
            } else if let Ok(name) = constant.extract::<String>() {
                if let Ok(item) = Item::from_str(py, rando, &name) {
                    Rule::Item(item, 1)
                } else {
                    Rule::Event(name)
                }
            } else {
                unimplemented!("converting constant expression {} into Rule", display_expr(ast, expr)) //TODO
            }
        } else if ast.get("Name")?.downcast::<PyType>()?.is_instance(expr)? {
            let name = expr.getattr("id")?.extract::<String>()?;
            // logic helper parameter
            if let Some((rule, args)) = args.0.get(&*name) { Rule::parse_inner(py, rando, ctx, seq, ast, rule, args)? }
            // attr of Rule_AST_Transformer
            else if name == "at_day" {
                Rule::Time(TimeRange::Day)
            } else if name == "at_dampe_time" {
                Rule::Time(TimeRange::Dampe)
            } else if name == "at_night" {
                Rule::Time(TimeRange::Night)
            }
            // alias (LogicHelpers.json)
            else if let Some(helper) = rando.logic_helpers()?.get(&name) {
                let kwargs = PyDict::new(py);
                kwargs.set_item("mode", "eval")?;
                Rule::parse_inner(py, rando, ctx, seq, ast, ast.call_method("parse", (helper,), Some(kwargs)).at::<RuleParseError>("ast.parse simple logic helper")?.getattr("body").at::<RuleParseError>(".body simple logic helper")?, &Args::default())?
            }
            // escaped item (ItemList.item_table)
            else if let Some(item) = rando.escaped_items().at::<RuleParseError>("escaped_items")?.get(&name) {
                Rule::Item(item.clone(), 1)
            }
            // setting (SettingsList.py)
            else if rando.setting_infos().at::<RuleParseError>("setting_infos")?.contains(&name) {
                Rule::Setting(name)
            } else if rando.logic_tricks().at::<RuleParseError>("logic_tricks")?.contains(&name) {
                Rule::Trick(name)
            }
            //TODO attr of State (which ones are used?)
            // event
            else if EVENT_REGEX.is_match(&name) {
                Rule::Event(name.replace('_', " ")) //TODO uncomment
            }
            else {
                unimplemented!("converting name expression {} into Rule", name)
            }
        } else if ast.get("Subscript")?.downcast::<PyType>()?.is_instance(expr)? {
            let value = expr.getattr("value")?.getattr("id")?.extract::<String>()?;
            let slice = expr.getattr("slice")?.getattr("id")?.extract::<String>()?;
            if value == "skipped_trials" {
                Rule::Not(Box::new(Rule::TrialActive(match &slice[..] {
                    "Light" => Medallion::Light,
                    "Forest" => Medallion::Forest,
                    "Fire" => Medallion::Fire,
                    "Water" => Medallion::Water,
                    "Shadow" => Medallion::Shadow,
                    "Spirit" => Medallion::Spirit,
                    _ => unimplemented!("unknown trial: {}", slice),
                })))
            } else {
                unimplemented!("converting subscript expression {}[{}] into Rule", value, slice)
            }
        } else if ast.get("Tuple")?.downcast::<PyType>()?.is_instance(expr)? {
            let (item, count) = expr.getattr("elts")?.iter()?.collect_tuple().ok_or(RuleParseError::TupleLength)?;
            let (item, count) = (item?, count?);
            let item_name = if ast.get("Constant")?.downcast::<PyType>()?.is_instance(item)? {
                item.getattr("value")?.extract::<String>().at::<RuleParseError>("item.value as String")?
            } else if ast.get("Name")?.downcast::<PyType>()?.is_instance(item)? {
                item.getattr("id")?.extract::<String>().at::<RuleParseError>("item.id as String")?
            } else {
                unimplemented!("converting {} into item to be counted", display_expr(ast, item))
            };
            let item = if let Some(item) = rando.escaped_items().at::<RuleParseError>("escaped_items")?.get(&item_name) {
                item.clone()
            } else {
                unimplemented!() //TODO unescaped item name or event
            };
            let count = if ast.get("Constant")?.downcast::<PyType>()?.is_instance(count)? {
                count.getattr("value")?.extract::<u8>().at::<RuleParseError>("count.value as u8")?
            } else if ast.get("Name")?.downcast::<PyType>()?.is_instance(count)? {
                unimplemented!() //TODO setting name
            } else {
                unimplemented!("converting {} into item count", display_expr(ast, count))
            };
            Rule::Item(item, count)
        } else if ast.get("UnaryOp")?.downcast::<PyType>()?.is_instance(expr)? {
            if ast.get("Not")?.downcast::<PyType>()?.is_instance(expr.getattr("op")?)? {
                Rule::Not(Box::new(Rule::parse_inner(py, rando, ctx, seq, ast, expr.getattr("operand")?, args)?))
            } else {
                unimplemented!("found UnaryOp expression other than Not: {}", display_expr(ast, expr))
            }
        } else {
            unimplemented!("converting expression {} into Rule", display_expr(ast, expr)) //TODO
        })
    }
}

fn display_expr(ast: &PyModule, expr: &PyAny) -> String {
    if let Ok(unparse) = ast.call_method1("unparse", (expr,)) {
        format!("{} ({:?})", unparse, expr)
    } else {
        // Python < 3.9 doesn't have ast.unparse (might want to use astunparse from PyPI instead)
        format!("{:?}", expr)
    }
}
