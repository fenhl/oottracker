use {
    std::{
        collections::HashMap,
        fmt,
        io,
        iter,
        sync::Arc,
    },
    itertools::Itertools as _,
    lazy_static::lazy_static,
    pyo3::{
        PyDowncastError,
        prelude::*,
        types::{
            PyBool,
            PyType,
        },
    },
    regex::Regex,
    ootr::{
        Rando as _,
        access::{
            Expr,
            ForAge,
        },
        check::Check,
        item::Item,
        model::{
            Medallion,
            TimeRange,
        },
    },
    crate::{
        Rando,
        RandoErr,
    },
};

lazy_static! {
    static ref EVENT_REGEX: Regex = Regex::new("^\\w+").expect("failed to compile event regex");
}

#[derive(Debug, Default)]
struct Args<'a>(HashMap<&'a str, (&'a PyAny, &'a Args<'a>)>);

#[derive(Debug, Clone)]
pub enum ParseError {
    HelperNumArgs {
        name: String,
        expected: usize,
        found: usize,
    },
    Io(Arc<io::Error>),
    Py(&'static str, Arc<PyErr>),
    Rando(Box<RandoErr>),
    TupleLength,
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> ParseError {
        ParseError::Io(Arc::new(e))
    }
}

impl From<PyErr> for ParseError {
    fn from(e: PyErr) -> ParseError {
        ParseError::Py("unknown", Arc::new(e))
    }
}

impl From<PyDowncastError<'_>> for ParseError {
    fn from(e: PyDowncastError<'_>) -> ParseError {
        ParseError::Py("downcast", Arc::new(e.into()))
    }
}

impl From<RandoErr> for ParseError {
    fn from(e: RandoErr) -> ParseError {
        ParseError::Rando(Box::new(e))
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::HelperNumArgs { name, expected, found } => write!(f, "logic helper {} called with {} args but it takes {}", name, found, expected),
            ParseError::Io(e) => write!(f, "I/O error: {}", e),
            ParseError::Py(loc, e) => write!(f, "Python error in {}: {}", loc, e),
            ParseError::Rando(e) => e.fmt(f),
            ParseError::TupleLength => write!(f, "tuple does not have 2 values"),
        }
    }
}

trait PyResultExt {
    type Ok;

    fn at(self, loc: &'static str) -> Result<Self::Ok, ParseError>;
}

impl<T> PyResultExt for PyResult<T> {
    type Ok = T;

    fn at(self, loc: &'static str) -> Result<T, ParseError> {
        self.map_err(|e| ParseError::Py(loc, Arc::new(e)))
    }
}

pub(crate) trait ExprExt {
    fn parse(rando: &Rando<'_>, ctx: &Check, expr: &str) -> Result<Expr, ParseError>;
    fn parse_helper(rando: &Rando<'_>, ctx: &Check, helpers: &HashMap<&str, usize>, args: &[String], expr: &str) -> Result<Expr, ParseError>;
}

impl ExprExt for Expr {
    fn parse(rando: &Rando<'_>, ctx: &Check, expr: &str) -> Result<Expr, ParseError> {
        let logic_helpers = rando.logic_helpers()?;
        let helpers = logic_helpers.iter().map(|(name, (args, _))| (&**name, args.len())).collect();
        let ast = rando.py.import("ast")?;
        Expr::parse_inner(rando, ctx, &helpers, &mut (0..), ast, ast.call_method1("parse", (expr, ctx.to_string(), "eval")).at("ast.parse in parse")?.getattr("body").at(".body in parse")?, &[])
    }

    fn parse_helper(rando: &Rando<'_>, ctx: &Check, helpers: &HashMap<&str, usize>, args: &[String], expr: &str) -> Result<Expr, ParseError> {
        let ast = rando.py.import("ast")?;
        Expr::parse_inner(rando, ctx, helpers, &mut (0..), ast, ast.call_method1("parse", (expr, ctx.to_string(), "eval")).at("ast.parse in parse_helper")?.getattr("body").at(".body in parse_helper")?, args)
    }
}

trait ExprExtPrivate {
    fn parse_inner(rando: &Rando<'_>, ctx: &Check, helpers: &HashMap<&str, usize>, seq: &mut impl Iterator<Item = usize>, ast: &PyModule, expr: &PyAny, args: &[String]) -> Result<Expr, ParseError>;
}

impl ExprExtPrivate for Expr {
    fn parse_inner(rando: &Rando<'_>, ctx: &Check, helpers: &HashMap<&str, usize>, seq: &mut impl Iterator<Item = usize>, ast: &PyModule, expr: &PyAny, args: &[String]) -> Result<Expr, ParseError> {
        // based on RuleParser.py as of 4f83414c49ff65ef2eb285667bcb153f11f1f9ef
        Ok(if ast.get("BoolOp")?.downcast::<PyType>()?.is_instance(expr)? {
            if ast.get("And")?.downcast::<PyType>()?.is_instance(expr.getattr("op")?)? {
                Expr::All(expr.getattr("values")?.iter()?.map(|expr| expr.at("next(expr.values)").and_then(|expr| Expr::parse_inner(rando, ctx, helpers, seq, ast, expr, args))).try_collect()?)
            } else if ast.get("Or")?.downcast::<PyType>()?.is_instance(expr.getattr("op")?)? {
                Expr::Any(expr.getattr("values")?.iter()?.map(|expr| expr.at("next(expr.values)").and_then(|expr| Expr::parse_inner(rando, ctx, helpers, seq, ast, expr, args))).try_collect()?)
            } else {
                unreachable!("found BoolOp expression with neither And nor Or: {}", display_expr(ast, expr))
            }
        } else if ast.get("Call")?.downcast::<PyType>()?.is_instance(expr)? {
            let name = expr.getattr("func")?.getattr("id")?.extract::<String>().at("expr.func.id as String")?;
            // attr of Rule_AST_Transformer (at, here)
            //TODO include region and access for the event
            if name == "at" {
                Expr::AnonymousEvent(ctx.clone(), seq.next().expect("failed to get anonymous event ID"))
            } else if name == "here" {
                Expr::AnonymousEvent(ctx.clone(), seq.next().expect("failed to get anonymous event ID"))
            }
            // expr alias (LogicHelpers.json)
            else if let Some(&num_params) = helpers.get(&*name) {
                let args = expr.getattr("args")?
                    .iter()?
                    .map(|arg| Ok::<_, ParseError>(Expr::parse_inner(rando, ctx, helpers, seq, ast, arg?, args)?))
                    .try_collect::<_, Vec<_>, _>()?;
                if args.len() == num_params {
                    Expr::LogicHelper(name, args)
                } else {
                    return Err(ParseError::HelperNumArgs { name, expected: num_params, found: args.len() })
                }
            }
            //TODO attr of State (which ones are used?)
            else if name == "has_dungeon_rewards" {
                let (count,) = expr.getattr("args")?.iter()?.collect_tuple().ok_or(ParseError::HelperNumArgs { name, expected: 1, found: expr.getattr("args")?.len()? })?;
                Expr::HasDungeonRewards(Box::new(Expr::parse_inner(rando, ctx, helpers, seq, ast, count?, args)?))
            } else if name == "has_medallions" {
                let (count,) = expr.getattr("args")?.iter()?.collect_tuple().ok_or(ParseError::HelperNumArgs { name, expected: 1, found: expr.getattr("args")?.len()? })?;
                Expr::HasMedallions(Box::new(Expr::parse_inner(rando, ctx, helpers, seq, ast, count?, args)?))
            } else if name == "has_stones" {
                let (count,) = expr.getattr("args")?.iter()?.collect_tuple().ok_or(ParseError::HelperNumArgs { name, expected: 1, found: expr.getattr("args")?.len()? })?;
                Expr::HasStones(Box::new(Expr::parse_inner(rando, ctx, helpers, seq, ast, count?, args)?))
            }
            else {
                unimplemented!("converting call expression with name {} into Expr", name)
            }
        } else if ast.get("Compare")?.downcast::<PyType>()?.is_instance(expr)? {
            Expr::All(
                iter::once(expr.getattr("left")?)
                    .chain(expr.getattr("comparators")?.iter()?.collect::<PyResult<Vec<_>>>()?.into_iter())
                    .tuple_windows()
                    .zip(expr.getattr("ops")?.iter()?.collect::<PyResult<Vec<_>>>()?.into_iter())
                    .map(|((left, right), op)| {
                        let left = Expr::parse_inner(rando, ctx, helpers, seq, ast, left, args)?;
                        let right = Expr::parse_inner(rando, ctx, helpers, seq, ast, right, args)?;
                        Ok::<_, ParseError>(if ast.get("Eq")?.downcast::<PyType>()?.is_instance(op)? {
                            Expr::Eq(Box::new(left), Box::new(right))
                        } else if ast.get("NotEq")?.downcast::<PyType>()?.is_instance(op)? {
                            Expr::Not(Box::new(Expr::Eq(Box::new(left), Box::new(right))))
                        } else {
                            unimplemented!("found Compare expression with non-Eq operator {}", op)
                        })
                    })
                    .try_collect()?
            )
        } else if ast.get("Constant")?.downcast::<PyType>()?.is_instance(expr)? {
            let constant = expr.getattr("value")?;
            if constant.downcast::<PyBool>().map_or(false, |b| b == PyBool::new(rando.py, true)) {
                Expr::True
            } else if let Ok(name) = constant.extract::<String>() {
                if let Ok(item) = Item::from_str(rando, &name) {
                    Expr::Item(item, Box::new(Expr::LitInt(1)))
                } else {
                    Expr::LitStr(name) //TODO distinguish between events and other strings by going through world files?
                }
            } else {
                unimplemented!("converting constant expression {} into Expr", display_expr(ast, expr)) //TODO
            }
        } else if ast.get("Name")?.downcast::<PyType>()?.is_instance(expr)? {
            let name = expr.getattr("id")?.extract::<String>().at("expr.id as String")?;
            // logic helper parameter
            if args.contains(&name) { Expr::Param(name) }
            // attr of Rule_AST_Transformer
            else if name == "at_day" {
                Expr::Time(TimeRange::Day)
            } else if name == "at_dampe_time" {
                Expr::Time(TimeRange::Dampe)
            } else if name == "at_night" {
                Expr::Time(TimeRange::Night)
            }
            // expr alias (LogicHelpers.json)
            else if let Some(&num_params) = helpers.get(&*name) {
                if num_params == 0 {
                    Expr::LogicHelper(name, Vec::default())
                } else {
                    return Err(ParseError::HelperNumArgs { name, expected: num_params, found: 0 })
                }
            }
            // escaped item (ItemList.item_table)
            else if let Some(item) = rando.escaped_items()?.get(&name) {
                Expr::Item(item.clone(), Box::new(Expr::LitInt(1)))
            }
            // World helper attr
            else if name == "lacs_condition" {
                Expr::LacsCondition
            } else if name == "starting_age" {
                Expr::StartingAge
            }
            // setting or trick (SettingsList.py)
            else if rando.setting_infos()?.contains(&name) {
                Expr::Setting(name)
            } else if rando.logic_tricks()?.contains(&name) {
                Expr::Trick(name)
            }
            //TODO attr of State (which ones are used?)
            // kwarg_defaults and allowed_globals
            else if name == "age" {
                Expr::Age
            }
            //TODO other kwarg_defaults and allowed_globals (spot, tod, TimeOfDay — which ones are used?)
            // arbitrary placeholders
            else if name == "adult" {
                Expr::ForAge(ForAge::Adult)
            } else if name == "both" {
                Expr::ForAge(ForAge::Both)
            } else if name == "child" {
                Expr::ForAge(ForAge::Child)
            } else if name == "either" {
                Expr::ForAge(ForAge::Either)
            }
            // event
            else if EVENT_REGEX.is_match(&name) {
                Expr::Event(name.replace('_', " "))
            }
            else {
                unimplemented!("converting name expression {} into Expr", name)
            }
        } else if ast.get("Subscript")?.downcast::<PyType>()?.is_instance(expr)? {
            let value = expr.getattr("value")?.getattr("id")?.extract::<String>()?;
            let slice = expr.getattr("slice")?.getattr("id")?.extract::<String>()?;
            if value == "skipped_trials" {
                Expr::Not(Box::new(Expr::TrialActive(match &slice[..] {
                    "Light" => Medallion::Light,
                    "Forest" => Medallion::Forest,
                    "Fire" => Medallion::Fire,
                    "Water" => Medallion::Water,
                    "Shadow" => Medallion::Shadow,
                    "Spirit" => Medallion::Spirit,
                    _ => unimplemented!("unknown trial: {}", slice),
                })))
            } else {
                unimplemented!("converting subscript expression {}[{}] into Expr", value, slice)
            }
        } else if ast.get("Tuple")?.downcast::<PyType>()?.is_instance(expr)? {
            let (item, count) = expr.getattr("elts")?.iter()?.collect_tuple().ok_or(ParseError::TupleLength)?;
            let (item, count) = (item?, count?);
            let item_name = if ast.get("Constant")?.downcast::<PyType>()?.is_instance(item)? {
                item.getattr("value")?.extract::<String>().at("item.value as String")?
            } else if ast.get("Name")?.downcast::<PyType>()?.is_instance(item)? {
                item.getattr("id")?.extract::<String>().at("item.id as String")?
            } else {
                unimplemented!("converting {} into item to be counted", display_expr(ast, item))
            };
            let item = if let Some(item) = rando.escaped_items()?.get(&item_name) {
                item.clone()
            } else {
                unimplemented!() //TODO unescaped item name or event
            };
            let count = if ast.get("Constant")?.downcast::<PyType>()?.is_instance(count)? {
                Expr::LitInt(count.getattr("value")?.extract::<u8>().at("count.value as u8")?)
            } else if ast.get("Name")?.downcast::<PyType>()?.is_instance(count)? {
                Expr::Setting(count.getattr("id")?.extract::<String>().at("count.id as String")?)
            } else {
                unimplemented!("converting {} into item count", display_expr(ast, count))
            };
            Expr::Item(item, Box::new(count))
        } else if ast.get("UnaryOp")?.downcast::<PyType>()?.is_instance(expr)? {
            if ast.get("Not")?.downcast::<PyType>()?.is_instance(expr.getattr("op")?)? {
                Expr::Not(Box::new(Expr::parse_inner(rando, ctx, helpers, seq, ast, expr.getattr("operand")?, args)?))
            } else {
                unimplemented!("found UnaryOp expression other than Not: {}", display_expr(ast, expr))
            }
        } else {
            unimplemented!("converting expression {} into Expr", display_expr(ast, expr)) //TODO
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
