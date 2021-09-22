use {
    std::{
        borrow::Cow,
        collections::HashMap,
        fmt,
        io,
        iter,
        sync::Arc,
    },
    derivative::Derivative,
    itertools::Itertools as _,
    lazy_regex::regex_is_match,
    pyo3::{
        PyDowncastError,
        prelude::*,
        types::{
            PyBool,
            PyType,
        },
    },
    quote_value::QuoteValue,
    wheel::FromArc,
    crate::{
        check::Check,
        item::Item,
        model::{
            Medallion,
            TimeRange,
        },
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, QuoteValue)]
pub enum ForAge {
    Child,
    Adult,
    Both,
    Either,
}

#[derive(Debug, Clone, QuoteValue)]
pub enum Expr {
    All(Vec<Expr>),
    Any(Vec<Expr>),
    Age,
    AnonymousEvent(Check, usize),
    Eq(Box<Expr>, Box<Expr>),
    Event(String),
    /// used in helper `has_projectile`. Should only compare equal to itself.
    ForAge(ForAge),
    HasDungeonRewards(Box<Expr>),
    HasMedallions(Box<Expr>),
    HasStones(Box<Expr>),
    Item(Item, Box<Expr>),
    LacsCondition,
    LitInt(u8),
    LitStr(String),
    LogicHelper(String, Vec<Expr>),
    Not(Box<Expr>),
    /// logic helper parameter
    Param(String),
    Setting(String),
    StartingAge,
    TrialActive(Medallion),
    Trick(String),
    Time(TimeRange),
    True,
}

impl Expr {
    pub fn resolve_args<'a>(&'a self, params: &[String], args: &'a [Expr]) -> Cow<'a, Expr> {
        match self {
            Expr::All(exprs) => Cow::Owned(Expr::All(exprs.iter().map(|expr| expr.resolve_args(params, args).into_owned()).collect())),
            Expr::Any(exprs) => Cow::Owned(Expr::Any(exprs.iter().map(|expr| expr.resolve_args(params, args).into_owned()).collect())),
            Expr::Eq(expr1, expr2) => Cow::Owned(Expr::Eq(Box::new(expr1.resolve_args(params, args).into_owned()), Box::new(expr2.resolve_args(params, args).into_owned()))),
            Expr::HasDungeonRewards(expr) => Cow::Owned(Expr::HasDungeonRewards(Box::new(expr.resolve_args(params, args).into_owned()))),
            Expr::HasMedallions(expr) => Cow::Owned(Expr::HasMedallions(Box::new(expr.resolve_args(params, args).into_owned()))),
            Expr::HasStones(expr) => Cow::Owned(Expr::HasStones(Box::new(expr.resolve_args(params, args).into_owned()))),
            Expr::Item(item, count) => Cow::Owned(Expr::Item(item.clone(), Box::new(count.resolve_args(params, args).into_owned()))),
            Expr::LogicHelper(helper_name, exprs) => Cow::Owned(Expr::LogicHelper(helper_name.clone(), exprs.iter().map(|expr| expr.resolve_args(params, args).into_owned()).collect())),
            Expr::Not(expr) => Cow::Owned(Expr::Not(Box::new(expr.resolve_args(params, args).into_owned()))),
            Expr::Param(param) => if let Some(pos) = params.iter().position(|iter_param| iter_param == param) {
                Cow::Borrowed(&args[pos])
            } else {
                Cow::Owned(Expr::Param(param.clone()))
            },
            _ => Cow::Borrowed(self),
        }
    }

    fn parse<'p>(py: Python<'p>, ctx: &Check, expr: &str) -> Result<Expr, ParseError> {
        let logic_helpers = rando.logic_helpers()?; //TODO (pre-parse logic helpers?)
        let helpers = logic_helpers.iter().map(|(name, (args, _))| (&**name, args.len())).collect();
        let ast = py.import("ast")?;
        Expr::parse_inner(py, ctx, &helpers, &mut (0..), ast, ast.call_method1("parse", (expr, ctx.to_string(), "eval")).at("ast.parse in parse")?.getattr("body").at(".body in parse")?, &[])
    }

    fn parse_helper<'p>(py: Python<'p>, ctx: &Check, helpers: &HashMap<&str, usize>, args: &[String], expr: &str) -> Result<Expr, ParseError> {
        let ast = py.import("ast")?;
        Expr::parse_inner(py, ctx, helpers, &mut (0..), ast, ast.call_method1("parse", (expr, ctx.to_string(), "eval")).at("ast.parse in parse_helper")?.getattr("body").at(".body in parse_helper")?, args)
    }

    fn parse_inner<'p>(py: &Python<'p>, ctx: &Check, helpers: &HashMap<&str, usize>, seq: &mut impl Iterator<Item = usize>, ast: &PyModule, expr: &PyAny, args: &[String]) -> Result<Expr, ParseError> {
        // based on RuleParser.py as of 4f83414c49ff65ef2eb285667bcb153f11f1f9ef
        Ok(if ast.getattr("BoolOp")?.downcast::<PyType>()?.is_instance(expr)? {
            if ast.getattr("And")?.downcast::<PyType>()?.is_instance(expr.getattr("op")?)? {
                Expr::All(expr.getattr("values")?.iter()?.map(|expr| expr.at("next(expr.values)").and_then(|expr| Expr::parse_inner(py, ctx, helpers, seq, ast, expr, args))).try_collect()?)
            } else if ast.getattr("Or")?.downcast::<PyType>()?.is_instance(expr.getattr("op")?)? {
                Expr::Any(expr.getattr("values")?.iter()?.map(|expr| expr.at("next(expr.values)").and_then(|expr| Expr::parse_inner(py, ctx, helpers, seq, ast, expr, args))).try_collect()?)
            } else {
                unreachable!("found BoolOp expression with neither And nor Or: {}", display_expr(ast, expr))
            }
        } else if ast.getattr("Call")?.downcast::<PyType>()?.is_instance(expr)? {
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
                    .map(|arg| Ok::<_, ParseError>(Expr::parse_inner(py, ctx, helpers, seq, ast, arg?, args)?))
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
                Expr::HasDungeonRewards(Box::new(Expr::parse_inner(py, ctx, helpers, seq, ast, count?, args)?))
            } else if name == "has_medallions" {
                let (count,) = expr.getattr("args")?.iter()?.collect_tuple().ok_or(ParseError::HelperNumArgs { name, expected: 1, found: expr.getattr("args")?.len()? })?;
                Expr::HasMedallions(Box::new(Expr::parse_inner(py, ctx, helpers, seq, ast, count?, args)?))
            } else if name == "has_stones" {
                let (count,) = expr.getattr("args")?.iter()?.collect_tuple().ok_or(ParseError::HelperNumArgs { name, expected: 1, found: expr.getattr("args")?.len()? })?;
                Expr::HasStones(Box::new(Expr::parse_inner(py, ctx, helpers, seq, ast, count?, args)?))
            }
            else {
                unimplemented!("converting call expression with name {} into Expr", name)
            }
        } else if ast.getattr("Compare")?.downcast::<PyType>()?.is_instance(expr)? {
            Expr::All(
                iter::once(expr.getattr("left")?)
                    .chain(expr.getattr("comparators")?.iter()?.collect::<PyResult<Vec<_>>>()?.into_iter())
                    .tuple_windows()
                    .zip(expr.getattr("ops")?.iter()?.collect::<PyResult<Vec<_>>>()?.into_iter())
                    .map(|((left, right), op)| {
                        let left = Expr::parse_inner(py, ctx, helpers, seq, ast, left, args)?;
                        let right = Expr::parse_inner(py, ctx, helpers, seq, ast, right, args)?;
                        Ok::<_, ParseError>(if ast.getattr("Eq")?.downcast::<PyType>()?.is_instance(op)? {
                            Expr::Eq(Box::new(left), Box::new(right))
                        } else if ast.getattr("NotEq")?.downcast::<PyType>()?.is_instance(op)? {
                            Expr::Not(Box::new(Expr::Eq(Box::new(left), Box::new(right))))
                        } else {
                            unimplemented!("found Compare expression with non-Eq operator {}", op)
                        })
                    })
                    .try_collect()?
            )
        } else if ast.getattr("Constant")?.downcast::<PyType>()?.is_instance(expr)? {
            let constant = expr.getattr("value")?;
            if constant.downcast::<PyBool>().map_or(false, |b| b == PyBool::new(py, true)) {
                Expr::True
            } else if let Ok(name) = constant.extract::<String>() {
                if let Ok(item) = Item::from_str(&name) {
                    Expr::Item(item, Box::new(Expr::LitInt(1)))
                } else {
                    Expr::LitStr(name) //TODO distinguish between events and other strings by going through world files?
                }
            } else {
                unimplemented!("converting constant expression {} into Expr", display_expr(ast, expr)) //TODO
            }
        } else if ast.getattr("Name")?.downcast::<PyType>()?.is_instance(expr)? {
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
            else if let Some(item) = Item::from_escaped(&name) {
                Expr::Item(item.clone(), Box::new(Expr::LitInt(1)))
            }
            // World helper attr
            else if name == "lacs_condition" {
                Expr::LacsCondition
            } else if name == "starting_age" {
                Expr::StartingAge
            }
            // setting or trick (SettingsList.py)
            else if rando.setting_names()?.contains_key(&name) {
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
            else if regex_is_match!("^\\w+", &name) {
                Expr::Event(name.replace('_', " "))
            }
            else {
                unimplemented!("converting name expression {} into Expr", name)
            }
        } else if ast.getattr("NameConstant")?.downcast::<PyType>()?.is_instance(expr)? {
            // Python 3.7 compat TODO remove when Debian bullseye is released
            let constant = expr.getattr("value")?;
            if constant.downcast::<PyBool>().map_or(false, |b| b == PyBool::new(py, true)) {
                Expr::True
            } else {
                unimplemented!("converting name constant expression {} into Expr", display_expr(ast, expr))
            }
        } else if ast.getattr("Str")?.downcast::<PyType>()?.is_instance(expr)? {
            // Python 3.7 compat TODO remove when Debian bullseye is released
            let name = expr.getattr("s")?.extract::<String>()?;
            if let Ok(item) = Item::from_str(&name) {
                Expr::Item(item, Box::new(Expr::LitInt(1)))
            } else {
                Expr::LitStr(name) //TODO distinguish between events and other strings by going through world files?
            }
        } else if ast.getattr("Subscript")?.downcast::<PyType>()?.is_instance(expr)? {
            let value = expr.getattr("value")?.getattr("id")?.extract::<String>()?;
            let slice = expr.getattr("slice")?;
            // “value” is Python 3.7 compat TODO remove when Debian bullseye is released
            let slice = slice.getattr("id").or_else(|_| PyResult::Ok(slice.getattr("value")?.getattr("id")?))?.extract::<String>()?;
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
        } else if ast.getattr("Tuple")?.downcast::<PyType>()?.is_instance(expr)? {
            let (item, count) = expr.getattr("elts")?.iter()?.collect_tuple().ok_or(ParseError::TupleLength)?;
            let (item, count) = (item?, count?);
            let item_name = if ast.getattr("Constant")?.downcast::<PyType>()?.is_instance(item)? {
                item.getattr("value")?.extract::<String>().at("item.value as String")?
            } else if ast.getattr("Name")?.downcast::<PyType>()?.is_instance(item)? {
                item.getattr("id")?.extract::<String>().at("item.id as String")?
            } else {
                unimplemented!("converting {} into item to be counted", display_expr(ast, item))
            };
            let item = if let Some(item) = Item::from_escaped(&item_name) {
                item.clone()
            } else {
                unimplemented!() //TODO unescaped item name or event
            };
            let count = if ast.getattr("Constant")?.downcast::<PyType>()?.is_instance(count)? {
                Expr::LitInt(count.getattr("value")?.extract::<u8>().at("count.value as u8")?)
            } else if ast.getattr("Name")?.downcast::<PyType>()?.is_instance(count)? {
                Expr::Setting(count.getattr("id")?.extract::<String>().at("count.id as String")?)
            } else if ast.getattr("Num")?.downcast::<PyType>()?.is_instance(count)? {
                // Python 3.7 compat TODO remove when Debian bullseye is released
                Expr::LitInt(count.getattr("n")?.extract::<u8>().at("count.n as u8")?)
            } else {
                unimplemented!("converting {} into item count", display_expr(ast, count))
            };
            Expr::Item(item, Box::new(count))
        } else if ast.getattr("UnaryOp")?.downcast::<PyType>()?.is_instance(expr)? {
            if ast.getattr("Not")?.downcast::<PyType>()?.is_instance(expr.getattr("op")?)? {
                Expr::Not(Box::new(Expr::parse_inner(py, ctx, helpers, seq, ast, expr.getattr("operand")?, args)?))
            } else {
                unimplemented!("found UnaryOp expression other than Not: {}", display_expr(ast, expr))
            }
        } else {
            unimplemented!("converting expression {} into Expr", display_expr(ast, expr)) //TODO
        })
    }
}

#[derive(Debug, Default)]
struct Args<'a>(HashMap<&'a str, (&'a PyAny, &'a Args<'a>)>);

#[derive(Debug, FromArc, Clone)]
pub enum ParseError {
    HelperNumArgs {
        name: String,
        expected: usize,
        found: usize,
    },
    #[from_arc]
    Io(Arc<io::Error>),
    Py(&'static str, Arc<PyErr>),
    TupleLength,
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

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::HelperNumArgs { name, expected, found } => write!(f, "logic helper {} called with {} args but it takes {}", name, found, expected),
            ParseError::Io(e) => write!(f, "I/O error: {}", e),
            ParseError::Py(loc, e) => write!(f, "Python error in {}: {}", loc, e),
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

fn display_expr(ast: &PyModule, expr: &PyAny) -> String {
    if let Ok(unparse) = ast.call_method1("unparse", (expr,)) {
        format!("{} ({:?})", unparse, expr)
    } else {
        // Python < 3.9 doesn't have ast.unparse (might want to use astunparse from PyPI instead)
        format!("{:?}", expr)
    }
}

impl ModelState { //TODO refactor to compile access expressions into Rust code
    /// If access depends on other checks (including an event or the value of an unknown setting), those checks are returned.
    pub(crate) fn can_access(&self, rule: &access::Expr) -> Result<bool, HashSet<Check>> {
        Ok(match rule {
            access::Expr::All(rules) => {
                let mut deps = HashSet::default();
                for rule in rules {
                    match self.can_access(rule) {
                        Ok(true) => {}
                        Ok(false) => return Ok(false),
                        Err(part_deps) => deps.extend(part_deps),
                    }
                }
                if deps.is_empty() { true } else { return Err(deps) }
            }
            access::Expr::Any(rules) => {
                let mut deps = HashSet::default();
                for rule in rules {
                    match self.can_access(rule) {
                        Ok(true) => return Ok(true),
                        Ok(false) => {}
                        Err(part_deps) => deps.extend(part_deps),
                    }
                }
                if deps.is_empty() { false } else { return Err(deps) }
            }
            access::Expr::AnonymousEvent(at_check, id) => Check::AnonymousEvent(Box::new(at_check.clone()), *id).checked(self).expect(&format!("unimplemented anonymous event check: {} for {}", id, at_check)),
            access::Expr::Eq(left, right) => self.access_exprs_eq(left, right)?,
            access::Expr::Event(event) | access::Expr::LitStr(event) => Check::Event(event.clone()).checked(self).expect(&format!("unimplemented event check: {}", event)),
            access::Expr::HasStones(count) => self.access_expr_le_val(count, self.ram.save.quest_items.num_stones())?,
            access::Expr::Item(item, count) => self.access_expr_le_val(count, self.ram.save.amount_of_item(item))?,
            access::Expr::LogicHelper(helper_name, args) => {
                let helpers = rando.logic_helpers().expect("failed to load logic helpers");
                let (params, helper) = helpers.get(helper_name).expect("no such logic helper");
                self.can_access(&helper.resolve_args(params, args))?
            }
            access::Expr::Not(inner) => !self.can_access(inner)?,
            access::Expr::Setting(setting) => if let Some(&setting_value) = self.knowledge.bool_settings.get(setting) {
                setting_value
            } else {
                return Err(collect![Check::Setting(setting.clone())])
            },
            access::Expr::TrialActive(trial) => if let Some(&trial_active) = self.knowledge.active_trials.get(trial) {
                trial_active
            } else {
                return Err(collect![Check::TrialActive(*trial)])
            },
            access::Expr::Trick(trick) => if let Some(trick_value) = self.knowledge.tricks.as_ref().map_or(Some(false), |tricks| tricks.get(trick).copied()) {
                trick_value
            } else {
                return Err(collect![Check::Trick(trick.clone())])
            },
            access::Expr::Time(range) => self.ram.save.time_of_day.matches(*range), //TODO take location of check into account, as well as available ways to pass time
            access::Expr::True => true,
            _ => unimplemented!("can_access for {:?}", rule),
        })
    }

    fn access_exprs_eq<'a>(&self, left: &'a access::Expr<R>, right: &'a access::Expr<R>) -> Result<bool, HashSet<Check<R>>> {
        Ok(match (left, right) {
            (access::Expr::All(exprs), expr) | (expr, access::Expr::All(exprs)) => {
                let mut deps = HashSet::default();
                for other in exprs {
                    match self.access_exprs_eq(expr, other) {
                        Ok(true) => {}
                        Ok(false) => return Ok(false),
                        Err(part_deps) => deps.extend(part_deps),
                    }
                }
                if deps.is_empty() { true } else { return Err(deps) }
            }
            (access::Expr::Any(exprs), expr) | (expr, access::Expr::Any(exprs)) => {
                let mut deps = HashSet::default();
                for other in exprs {
                    match self.access_exprs_eq(expr, other) {
                        Ok(true) => return Ok(true),
                        Ok(false) => {}
                        Err(part_deps) => deps.extend(part_deps),
                    }
                }
                if deps.is_empty() { false } else { return Err(deps) }
            }
            (access::Expr::Age, access::Expr::LitStr(s)) if s == "child" => !self.ram.save.is_adult,
            (access::Expr::Age, access::Expr::LitStr(s)) if s == "adult" => self.ram.save.is_adult,
            (access::Expr::Age, access::Expr::StartingAge) => true, // we always assume that we started as the current age, since going to the other age requires finding the Temple of Time first
            (access::Expr::ForAge(age1), access::Expr::ForAge(age2)) => age1 == age2,
            (access::Expr::Item(item1, count1), access::Expr::Item(item2, count2)) => item1 == item2 && self.access_exprs_eq(count1, count2)?,
            (access::Expr::Item(item, count), access::Expr::LitStr(s)) |
            (access::Expr::LitStr(s), access::Expr::Item(item, count)) => if self.access_expr_eq_val(count, 1)? {
                *item == Item::from_str(s).expect(&format!("tried to compare item with non-item string literal {}", s))
            } else {
                false // multiple items are never the same as another single item
            },
            (access::Expr::LitInt(n1), access::Expr::LitInt(n2)) => n1 == n2,
            (access::Expr::LitStr(s1), access::Expr::LitStr(s2)) => s1 == s2,
            (access::Expr::LogicHelper(helper_name, args), expr) | (expr, access::Expr::LogicHelper(helper_name, args)) => {
                let helpers = rando.logic_helpers().expect("failed to load logic helpers");
                let (params, helper) = helpers.get(helper_name).expect("no such logic helper");
                self.access_exprs_eq(&helper.resolve_args(params, args), expr)?
            }
            (access::Expr::Setting(setting), access::Expr::LitStr(_)) => return Err(collect![Check::Setting(setting.clone())]), //TODO check knowledge
            (_, _) => unimplemented!("comparison of access expressions {:?} and {:?}", left, right),
        })
    }

    fn access_expr_eq_val(&self, expr: &access::Expr<R>, value: u8) -> Result<bool, HashSet<Check<R>>> {
        Ok(match expr {
            access::Expr::LitInt(n) => *n == value,
            _ => unimplemented!("access expr {:?} == value", expr),
        })
    }

    fn access_expr_le_val(&self, expr: &access::Expr<R>, value: u8) -> Result<bool, HashSet<Check<R>>> {
        Ok(match expr {
            access::Expr::LitInt(n) => *n <= value,
            _ => unimplemented!("access expr {:?} <= value", expr),
        })
    }
}
