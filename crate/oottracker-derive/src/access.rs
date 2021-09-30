#![allow(unused)] //TODO

use {
    std::{
        borrow::Cow,
        collections::{
            HashMap,
            HashSet,
        },
        fmt,
        io,
        iter,
        ops::{
            BitAnd,
            RangeInclusive,
        },
        sync::Arc,
    },
    derivative::Derivative,
    enum_iterator::IntoEnumIterator,
    itertools::Itertools as _,
    lazy_regex::regex_is_match,
    proc_macro2::{
        Span,
        TokenStream,
    },
    pyo3::{
        AsPyPointer as _,
        PyDowncastError,
        exceptions::PyValueError,
        prelude::*,
        types::{
            PyBool,
            PyDict,
            PyInt,
            PyType,
        },
    },
    quote::quote,
    syn::Ident,
    wheel::FromArc,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Context {
    Exit {
        from: String,
        to: String,
    },
    LogicHelper(String),
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exit { from, to } => write!(f, "{} → {}", from, to),
            Self::LogicHelper(fn_name) => write!(f, "logic helper {:?}", fn_name),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ForAge {
    Child,
    Adult,
    Both,
    Either,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoEnumIterator)]
pub(crate) enum Medallion {
    Light,
    Forest,
    Fire,
    Water,
    Shadow,
    Spirit,
}

impl Medallion {
    fn to_item_requirement(&self) -> Requirement {
        Requirement::Item(format!("{:?} Medallion", self), 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoEnumIterator)]
pub(crate) enum Stone {
    KokiriEmerald,
    GoronRuby,
    ZoraSapphire,
}

impl Stone {
    fn to_item_requirement(&self) -> Requirement {
        Requirement::Item(match self {
            Self::KokiriEmerald => format!("Kokiri Emerald"),
            Self::GoronRuby => format!("Goron Ruby"),
            Self::ZoraSapphire => format!("Zora Sapphire"),
        }, 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoEnumIterator)]
pub(crate) enum DungeonReward {
    Medallion(Medallion),
    Stone(Stone),
}

impl DungeonReward {
    fn to_item_requirement(&self) -> Requirement {
        match self {
            Self::Medallion(med) => med.to_item_requirement(),
            Self::Stone(stone) => stone.to_item_requirement(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TimeRange {
    Day,
    Night,
    Dampe,
}

#[derive(Debug, Clone)]
pub(crate) enum Expr {
    All(Vec<Expr>),
    Any(Vec<Expr>),
    Age,
    AnonymousEvent(Context, usize),
    Eq(Box<Expr>, Box<Expr>),
    Event(String),
    /// used in helper `has_projectile`. Should only compare equal to itself.
    ForAge(ForAge),
    HasDungeonRewards(Box<Expr>),
    HasMedallions(Box<Expr>),
    HasStones(Box<Expr>),
    Item(String, Box<Expr>),
    LitInt(u8),
    LitStr(String),
    LogicHelper(String, Vec<Expr>),
    Not(Box<Expr>),
    /// logic helper parameter
    Param(String),
    Setting(String),
    TrialActive(Medallion),
    Trick(String),
    Time(TimeRange),
    True,
}

impl Expr {
    pub(crate) fn parse<'p>(py: Python<'p>, ctx: &Context, logic_helpers: &HashMap<String, (Vec<String>, Expr)>, events: &HashSet<String>, expr: &str) -> Result<Expr, ParseError> {
        let helpers = logic_helpers.iter().map(|(name, (args, _))| (&**name, args.len())).collect();
        let ast = py.import("ast")?;
        let expr = Expr::parse_inner(py, ctx, &helpers, events, &mut (0..), ast, ast.call_method1("parse", (expr.trim(), ctx.to_string(), "eval")).at("ast.parse in parse")?.getattr("body").at(".body in parse")?, &[])?;
        Ok(expr.resolve_helpers(logic_helpers).into_owned())
    }

    pub(crate) fn parse_helper<'p>(py: Python<'p>, ctx: &Context, helpers: &HashMap<&str, usize>, events: &HashSet<String>, args: &[String], expr: &str) -> Result<Expr, ParseError> {
        let ast = py.import("ast")?;
        Expr::parse_inner(py, ctx, helpers, events, &mut (0..), ast, ast.call_method1("parse", (expr.trim(), ctx.to_string(), "eval")).at("ast.parse in parse_helper")?.getattr("body").at(".body in parse_helper")?, args)
    }

    fn parse_inner<'p>(py: Python<'p>, ctx: &Context, helpers: &HashMap<&str, usize>, events: &HashSet<String>, seq: &mut impl Iterator<Item = usize>, ast: &PyModule, expr: &PyAny, args: &[String]) -> Result<Expr, ParseError> {
        // based on RuleParser.py as of 4f83414c49ff65ef2eb285667bcb153f11f1f9ef
        Ok(if ast.getattr("BoolOp")?.downcast::<PyType>()?.is_instance(expr)? {
            if ast.getattr("And")?.downcast::<PyType>()?.is_instance(expr.getattr("op")?)? {
                Expr::All(expr.getattr("values")?.iter()?.map(|expr| expr.at("next(expr.values)").and_then(|expr| Expr::parse_inner(py, ctx, helpers, events, seq, ast, expr, args))).try_collect()?)
            } else if ast.getattr("Or")?.downcast::<PyType>()?.is_instance(expr.getattr("op")?)? {
                Expr::Any(expr.getattr("values")?.iter()?.map(|expr| expr.at("next(expr.values)").and_then(|expr| Expr::parse_inner(py, ctx, helpers, events, seq, ast, expr, args))).try_collect()?)
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
                    .map(|arg| Ok::<_, ParseError>(Expr::parse_inner(py, ctx, helpers, events, seq, ast, arg?, args)?))
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
                Expr::HasDungeonRewards(Box::new(Expr::parse_inner(py, ctx, helpers, events, seq, ast, count?, args)?))
            } else if name == "has_medallions" {
                let (count,) = expr.getattr("args")?.iter()?.collect_tuple().ok_or(ParseError::HelperNumArgs { name, expected: 1, found: expr.getattr("args")?.len()? })?;
                Expr::HasMedallions(Box::new(Expr::parse_inner(py, ctx, helpers, events, seq, ast, count?, args)?))
            } else if name == "has_stones" {
                let (count,) = expr.getattr("args")?.iter()?.collect_tuple().ok_or(ParseError::HelperNumArgs { name, expected: 1, found: expr.getattr("args")?.len()? })?;
                Expr::HasStones(Box::new(Expr::parse_inner(py, ctx, helpers, events, seq, ast, count?, args)?))
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
                        let left = Expr::parse_inner(py, ctx, helpers, events, seq, ast, left, args)?;
                        let right = Expr::parse_inner(py, ctx, helpers, events, seq, ast, right, args)?;
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
                if py.import("ItemList")?.getattr("item_table")?.get_item(&name)
                .map_or(Ok::<_, ParseError>(false), |item| Ok(name == "Scarecrow Song" || item.get_item(0)?.extract::<&str>()? != "Event"))? { //HACK treat Scarecrow Song as not an event since it's not defined as one in any region
                    Expr::Item(name, Box::new(Expr::LitInt(1)))
                } else if events.contains(&name) {
                    Expr::Event(name)
                } else {
                    Expr::LitStr(name)
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
            // escaped item
            else if let Some(unescaped) = py.import("RuleParser")?.getattr("escaped_items")?.get_item(&name)
            .map_or(Ok::<_, ParseError>(None), |unescaped| {
                let unescaped = unescaped.extract()?;
                Ok(if unescaped == "Scarecrow Song" || py.import("ItemList")?.getattr("item_table")?.get_item(&unescaped)?.get_item(0)?.extract::<&str>()? != "Event" { Some(unescaped) } else { None })
            })? { //HACK treat Scarecrow Song as not an event since it's not defined as one in any region
                Expr::Item(unescaped, Box::new(Expr::LitInt(1)))
            }
            // World helper attribute (from the “rename a few attributes...” section of World.py)
            else if name == "disable_trade_revert" {
                Expr::Any(vec![
                    Expr::Eq(Box::new(Expr::Setting(format!("shuffle_interior_entrances"))), Box::new(Expr::LitStr(format!("simple")))),
                    Expr::Eq(Box::new(Expr::Setting(format!("shuffle_interior_entrances"))), Box::new(Expr::LitStr(format!("all")))),
                    Expr::Setting(format!("shuffle_overworld_entrances")),
                ])
            } else if name == "entrance_shuffle" {
                Expr::Any(vec![
                    Expr::Eq(Box::new(Expr::Setting(format!("shuffle_interior_entrances"))), Box::new(Expr::LitStr(format!("simple")))),
                    Expr::Eq(Box::new(Expr::Setting(format!("shuffle_interior_entrances"))), Box::new(Expr::LitStr(format!("all")))),
                    Expr::Setting(format!("shuffle_grotto_entrances")),
                    Expr::Setting(format!("shuffle_dungeon_entrances")),
                    Expr::Setting(format!("shuffle_overworld_entrances")),
                    Expr::Setting(format!("owl_drops")),
                    Expr::Setting(format!("warp_songs")),
                    Expr::Setting(format!("spawn_positions")),
                ])
            } else if name == "keysanity" {
                Expr::Any(vec![
                    Expr::Eq(Box::new(Expr::Setting(format!("shuffle_smallkeys"))), Box::new(Expr::LitStr(format!("keysanity")))),
                    Expr::Eq(Box::new(Expr::Setting(format!("shuffle_smallkeys"))), Box::new(Expr::LitStr(format!("remove")))),
                    Expr::Eq(Box::new(Expr::Setting(format!("shuffle_smallkeys"))), Box::new(Expr::LitStr(format!("any_dungeon")))),
                    Expr::Eq(Box::new(Expr::Setting(format!("shuffle_smallkeys"))), Box::new(Expr::LitStr(format!("overworld")))),
                ])
            }
            // setting or trick (SettingsList.py)
            else if py.import("SettingsList")?.getattr("si_dict")?.downcast::<PyDict>()?.get_item(&name).is_some() {
                Expr::Setting(name)
            } else if py.import("SettingsList")?.getattr("logic_tricks")?.call_method0("values")?.iter()?.map(|trick| Ok::<_, ParseError>(trick?.get_item("name")?.extract::<String>()?)).any(|trick_name| trick_name.unwrap() == name) {
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
        } else if ast.getattr("Subscript")?.downcast::<PyType>()?.is_instance(expr)? {
            let value = expr.getattr("value")?.getattr("id")?.extract::<String>()?;
            let slice = expr.getattr("slice")?;
            let slice = slice.getattr("id")?.extract::<String>()?;
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
            let item = if let Some(unescaped) = py.import("RuleParser")?.getattr("escaped_items")?.get_item(&item_name)
            .map_or(Ok::<_, ParseError>(None), |unescaped| {
                let unescaped = unescaped.extract::<&str>()?;
                Ok(if unescaped == "Scarecrow Song" || py.import("ItemList")?.getattr("item_table")?.get_item(unescaped)?.get_item(0)?.extract::<&str>()? != "Event" { Some(unescaped) } else { None })
            })? {
                item_name
            } else {
                unimplemented!("unescaped item name or event") //TODO
            };
            let count = if ast.getattr("Constant")?.downcast::<PyType>()?.is_instance(count)? {
                Expr::LitInt(count.getattr("value")?.extract::<u8>().at("count.value as u8")?)
            } else if ast.getattr("Name")?.downcast::<PyType>()?.is_instance(count)? {
                Expr::Setting(count.getattr("id")?.extract::<String>().at("count.id as String")?)
            } else {
                unimplemented!("converting {} into item count", display_expr(ast, count))
            };
            Expr::Item(item, Box::new(count))
        } else if ast.getattr("UnaryOp")?.downcast::<PyType>()?.is_instance(expr)? {
            if ast.getattr("Not")?.downcast::<PyType>()?.is_instance(expr.getattr("op")?)? {
                Expr::Not(Box::new(Expr::parse_inner(py, ctx, helpers, events, seq, ast, expr.getattr("operand")?, args)?))
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
pub(crate) enum ParseError {
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

impl Expr {
    //TODO make this a mutating operation?
    fn resolve_helpers<'a>(&'a self, helpers: &'a HashMap<String, (Vec<String>, Expr)>) -> Cow<'a, Expr> {
        match self {
            Expr::All(exprs) => Cow::Owned(Expr::All(exprs.iter().map(|expr| expr.resolve_helpers(helpers).into_owned()).collect())),
            Expr::Any(exprs) => Cow::Owned(Expr::Any(exprs.iter().map(|expr| expr.resolve_helpers(helpers).into_owned()).collect())),
            Expr::Eq(expr1, expr2) => Cow::Owned(Expr::Eq(Box::new(expr1.resolve_helpers(helpers).into_owned()), Box::new(expr2.resolve_helpers(helpers).into_owned()))),
            Expr::HasDungeonRewards(expr) => Cow::Owned(Expr::HasDungeonRewards(Box::new(expr.resolve_helpers(helpers).into_owned()))),
            Expr::HasMedallions(expr) => Cow::Owned(Expr::HasMedallions(Box::new(expr.resolve_helpers(helpers).into_owned()))),
            Expr::HasStones(expr) => Cow::Owned(Expr::HasStones(Box::new(expr.resolve_helpers(helpers).into_owned()))),
            Expr::Item(item, count) => Cow::Owned(Expr::Item(item.clone(), Box::new(count.resolve_helpers(helpers).into_owned()))),
            Expr::LogicHelper(helper_name, args) => if let Some((params, body)) = helpers.get(helper_name) {
                Cow::Owned(body.resolve_args(params, args).resolve_helpers(helpers).into_owned())
            } else {
                Cow::Owned(Expr::LogicHelper(helper_name.clone(), args.iter().map(|expr| expr.resolve_helpers(helpers).into_owned()).collect()))
            },
            Expr::Not(expr) => Cow::Owned(Expr::Not(Box::new(expr.resolve_helpers(helpers).into_owned()))),
            _ => Cow::Borrowed(self),
        }
    }

    fn resolve_args<'a>(&'a self, params: &[String], args: &'a [Expr]) -> Cow<'a, Expr> {
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

    pub(crate) fn compile(&self) -> TokenStream {
        //use std::io::Write as _;

        //let mut f = std::fs::OpenOptions::new().create(true).append(true).open(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("access_exprs_debug.rs")).expect("failed to open debug log");
        //writeln!(&mut f, "compiling {:?}", self).expect("failed to write to debug log");
        let info_arms = self.info_partition().into_iter()
            .map(|info| {
                let info_test = info.quote_test();
                let reqs = self.requirements(&info);
                let repr_self = format!("{:?}", self);
                let repr_info = format!("{:?}", info);
                quote!(if #info_test {
                    // current knowledge is consistent with this seed info, so these requirements are possible
                    unimplemented!("compile {} assuming {}", #repr_self, #repr_info); //TODO
                })
            });
        let code = quote! {
            #[allow(unused)] {
                #(#info_arms)*
                unimplemented!("check if reachability is always possible, possible depending on unknowns about the seed, or impossible") //TODO
            }
        };
        //writeln!(&mut f, "{}", code).expect("failed to write to debug log");
        //writeln!(&mut f).expect("failed to write to debug log");
        code
    }
}

#[derive(Debug, Default, Clone)]
struct SeedInfo {
    bool_settings: HashMap<String, bool>,
    int_settings: HashMap<String, RangeInclusive<u8>>,
    str_settings: HashMap<String, HashSet<String>>,
    time_of_day: Option<bool>,
    trials: HashMap<Medallion, bool>,
    tricks: HashMap<String, bool>,
}

impl SeedInfo {
    /// an expression that evaluates to whether the current knowledge is *consistent* with this seed info
    fn quote_test(&self) -> TokenStream {
        let Self { bool_settings, int_settings, str_settings, time_of_day, trials, tricks } = self;
        let bool_settings = bool_settings.iter().map(|(setting, enabled)| {
            let setting_ident = Ident::new(setting, Span::call_site());
            if *enabled {
                quote!(model.knowledge.settings.#setting_ident.map_or(true, |enabled| enabled))
            } else {
                quote!(model.knowledge.settings.#setting_ident.map_or(true, |enabled| !enabled))
            }
        });
        let int_settings = int_settings.iter().map(|(setting, range)| {
            let setting_ident = Ident::new(setting, Span::call_site());
            let start = *range.start();
            let end = *range.end();
            quote!(model.knowledge.settings.#setting_ident.start().max(&#start) <= model.knowledge.settings.#setting_ident.end().min(&#end))
        });
        let str_settings = str_settings.iter().map(|(setting, values)| {
            let setting_ident = Ident::new(setting, Span::call_site());
            let value_idents = values.iter().map(|value| crate::settings::value_to_ident(value));
            quote!((false #(|| model.knowledge.settings.#setting_ident.#value_idents)*))
        });
        let time_of_day = match time_of_day {
            Some(true) => quote!(unimplemented!("check time-of-day knowledge")), //TODO needs knowledge of starting=current time of day and/or route to appropriate ToD change
            Some(false) => quote!(unimplemented!("check time-of-day knowledge")), //TODO
            None => quote!(true),
        };
        let trials = trials.iter().map(|(med, enabled)| {
            let med_ident = Ident::new(&format!("{:?}", med), Span::call_site());
            if *enabled {
                quote!(model.knowledge.trials.get(Medallion::#med_ident).map_or(true, |&enabled| enabled))
            } else {
                quote!(model.knowledge.trials.get(Medallion::#med_ident).map_or(true, |&enabled| !enabled))
            }
        });
        let tricks = tricks.iter().map(|(trick, enabled)| if *enabled {
            quote!(model.knowledge.settings.allowed_tricks.get(#trick).map_or(true, |&enabled| enabled))
        } else {
            quote!(model.knowledge.settings.allowed_tricks.get(#trick).map_or(true, |&enabled| !enabled))
        });
        quote!(#time_of_day #(&& #bool_settings)* #(&& #int_settings)* #(&& #str_settings)* #(&& #tricks)*)
    }
}

impl BitAnd for SeedInfo {
    type Output = Option<Self>;

    fn bitand(mut self, rhs: Self) -> Option<Self> {
        let Self { bool_settings, int_settings, str_settings, time_of_day, trials, tricks } = rhs;
        for (setting, enabled) in bool_settings {
            if *self.bool_settings.entry(setting).or_insert(enabled) != enabled { return None }
        }
        for (setting, range) in int_settings {
            let entry = self.int_settings.entry(setting).or_insert_with(|| range.clone());
            *entry = *entry.start().max(range.start())..=*entry.end().min(range.end());
            if entry.end() < entry.start() { return None }
        }
        for (setting, values) in str_settings {
            let entry = self.str_settings.entry(setting).or_insert_with(|| values.clone());
            entry.retain(|value| values.contains(value));
            if entry.is_empty() { return None }
        }
        if let Some(time_of_day) = time_of_day {
            if *self.time_of_day.get_or_insert(time_of_day) != time_of_day { return None }
        }
        for (med, enabled) in trials {
            if *self.trials.entry(med).or_insert(enabled) != enabled { return None }
        }
        for (trick, enabled) in tricks {
            if *self.tricks.entry(trick).or_insert(enabled) != enabled { return None }
        }
        Some(self)
    }
}

impl Expr {
    fn eq_info_partition(&self, rhs: &Self) -> Vec<SeedInfo> {
        match (self, rhs) {
            (Self::All(_), Self::All(_)) => vec![SeedInfo::default()],
            (Self::All(_), _) | (_, Self::All(_)) => vec![SeedInfo::default()],
            (Self::Any(exprs), expr) | (expr, Self::Any(exprs)) => if exprs.is_empty() {
                vec![SeedInfo::default()]
            } else {
                let mut overall_infos = Vec::default();
                'outer: for mut infos in exprs.iter().map(|iter_expr| iter_expr.eq_info_partition(expr)).multi_cartesian_product() {
                    let mut overall = infos.remove(0);
                    for info in infos {
                        overall = if let Some(overall) = overall & info { overall } else { continue 'outer };
                    }
                    overall_infos.push(overall);
                }
                overall_infos
            },
            (Self::Age, Self::LitStr(rhs)) => match &**rhs {
                "adult" | "child" => vec![SeedInfo::default()], // current age is modeled as a requirement rather than seed info
                _ => unimplemented!("Expr::Age.eq_info_partition(Expr::LitStr({:?}))", rhs),
            },
            (Self::Age, Self::Setting(setting)) => match &**setting {
                "starting_age" => vec![SeedInfo::default()],
                _ => unimplemented!("Expr::Age.eq_info_partition(Expr::Setting({:?})", setting),
            },
            (Self::Item(_, _), Self::Item(_, _)) => vec![SeedInfo::default()],
            (Self::Setting(name), Self::LitStr(value)) => {
                let choices = Python::with_gil(|py| PyResult::Ok(py
                    .import("SettingsList")?
                    .call_method1("get_setting_info", (name,))?
                    .getattr("choice_list")?
                    .iter()?
                    .map(|choice| PyResult::Ok(choice?.extract::<String>()?))
                    .try_collect::<_, Vec<_>, _>()?
                )).expect("failed to read string setting choices");
                vec![
                    SeedInfo { str_settings: iter::once((name.clone(), iter::once(value.clone()).collect())).collect(), ..SeedInfo::default() },
                    SeedInfo { str_settings: iter::once((name.clone(), choices.into_iter().filter(|choice| choice != value).collect())).collect(), ..SeedInfo::default() },
                ]
            }
            _ => unimplemented!("{:?}.eq_info_partition({:?})", self, rhs),
        }
    }

    fn info_partition(&self) -> Vec<SeedInfo> {
        match self {
            // access not dependent on seed knowledge
            Self::AnonymousEvent(_, _) | Self::Event(_) | //TODO allow events (including anonymous events) whose checked status (not access) depends on seed knowledge to propagate these dependencies to access expressions that use them?
            Self::Age | Self::ForAge(_) | Self::LitInt(_) | Self::LitStr(_) | Self::True => vec![SeedInfo::default()],
            // access dependent on subexpressions
            Self::HasDungeonRewards(expr) | Self::HasMedallions(expr) | Self::HasStones(expr) | Self::Item(_, expr) | Self::Not(expr) => expr.info_partition(),
            Self::All(exprs) | Self::Any(exprs) => if exprs.is_empty() {
                vec![SeedInfo::default()]
            } else {
                let mut overall_infos = Vec::default();
                'outer: for mut infos in exprs.iter().map(|expr| expr.info_partition()).multi_cartesian_product() {
                    let mut overall = infos.remove(0);
                    for info in infos {
                        overall = if let Some(overall) = overall & info { overall } else { continue 'outer };
                    }
                    overall_infos.push(overall);
                }
                overall_infos
            },
            Self::Eq(expr1, expr2) => expr1.eq_info_partition(expr2),
            // leaf expressions with actual info requirements
            Self::Setting(name) => Python::with_gil(|py| {
                let settings_list = py.import("SettingsList")?;
                let setting = settings_list.call_method1("get_setting_info", (name,))?;
                let setting_type = setting.getattr("type")?;
                PyResult::Ok(if setting_type.as_ptr() == py.get_type::<PyInt>().as_ptr() {
                    let min = setting.getattr("gui_params")?.downcast::<PyDict>()?.get_item("min").ok_or(PyValueError::new_err("missing int setting bound"))?.extract::<u8>()?;
                    let max = setting.getattr("gui_params")?.downcast::<PyDict>()?.get_item("max").ok_or(PyValueError::new_err("missing int setting bound"))?.extract::<u8>()?;
                    (min..=max).map(|value| SeedInfo { int_settings: iter::once((name.clone(), value..=value)).collect(), ..SeedInfo::default() }).collect() //TODO rather than testing for each value individually, check what's required based on surrounding expression
                } else {
                    vec![
                        SeedInfo { bool_settings: iter::once((name.clone(), true)).collect(), ..SeedInfo::default() },
                        SeedInfo { bool_settings: iter::once((name.clone(), false)).collect(), ..SeedInfo::default() },
                    ]
                })
            }).expect("failed to read setting type"),
            Self::Time(_) => vec![
                SeedInfo { time_of_day: Some(true), ..SeedInfo::default() },
                SeedInfo { time_of_day: Some(false), ..SeedInfo::default() },
            ],
            Self::Trick(name) => vec![
                SeedInfo { tricks: iter::once((name.clone(), true)).collect(), ..SeedInfo::default() },
                SeedInfo { tricks: iter::once((name.clone(), false)).collect(), ..SeedInfo::default() },
            ],
            Self::TrialActive(med) => vec![
                SeedInfo { trials: iter::once((*med, true)).collect(), ..SeedInfo::default() },
                SeedInfo { trials: iter::once((*med, false)).collect(), ..SeedInfo::default() },
            ],
            // nonsense
            Self::LogicHelper(_, _) | Self::Param(_) => unreachable!("info_partition must be called after resolve_helpers"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum Requirement {
    /// `true` for adult, `false` for child
    Age(bool),
    AnonymousEvent(Context, usize),
    Event(String),
    Item(String, u8),
}

impl Expr {
    fn eval_u8(&self, info: &SeedInfo) -> u8 {
        match self {
            Self::LitInt(n) => *n,
            Self::Setting(name) => {
                let range = info.int_settings.get(name).expect(&format!("eval_u8 with undetermined int setting info (present in bool_settings? {:?} / present in str_settings? {:?})", info.bool_settings.contains_key(name), info.str_settings.contains_key(name)));
                if range.start() != range.end() { panic!("eval_u8 with not fully specified int setting info") }
                *range.start()
            }
            _ => unimplemented!("{:?}.eval_u8(_)", self),
        }
    }

    /// Returns the requirements for these expressions to be equal, with the given seed info, in sum of products form.
    fn eq_requirements(&self, rhs: &Self, info: &SeedInfo) -> Vec<Vec<Requirement>> {
        match (self, rhs) {
            (Self::All(lhs), Self::All(rhs)) => if lhs.len() != rhs.len() {
                Vec::default()
            } else {
                lhs.iter()
                    .zip(rhs)
                    .map(|(lhs, rhs)| lhs.eq_requirements(rhs, info))
                    .multi_cartesian_product()
                    .map(|products| products.into_iter().flatten().unique().collect())
                    .unique()
                    .collect()
                },
            (Self::All(_), _) | (_, Self::All(_)) => Vec::default(), //TODO verify correctness
            (Self::Any(exprs), expr) | (expr, Self::Any(exprs)) => exprs.iter().flat_map(|iter_expr| iter_expr.eq_requirements(expr, info)).unique().collect(), //TODO verify correctness
            (Self::Age, Self::LitStr(rhs)) => match &**rhs {
                "adult" => vec![vec![Requirement::Age(true)]],
                "child" => vec![vec![Requirement::Age(false)]],
                _ => unimplemented!("Expr::Age.eq_requirements(Expr::LitStr({:?}), _)", rhs),
            },
            (Self::Age, Self::Setting(setting)) => match &**setting {
                "starting_age" => vec![Vec::default()], // we always assume that we started as the current age, since going to the other age requires finding the Temple of Time first
                _ => unimplemented!("Expr::Age.eq_requirements(Expr::Setting({:?}), _)", rhs),
            },
            (Self::Item(item1, count1), Self::Item(item2, count2)) => if item1 == item2 { count1.eq_requirements(count2, info) } else { Vec::default() },
            (Self::LitInt(n1), Self::LitInt(n2)) => if n1 == n2 { vec![Vec::default()] } else { Vec::default() },
            (Self::Setting(name), Self::LitStr(value)) => match info.str_settings.get(name).map(|setting| setting.contains(value)) {
                Some(true) => vec![Vec::default()],
                Some(false) => Vec::default(),
                None => unimplemented!("eq_requirements with undetermined setting info ({} == {})", name, value),
            },
            _ => unimplemented!("{:?}.eq_requirements({:?}, _)", self, rhs),
        }
    }

    fn ne_requirements(&self, rhs: &Self, info: &SeedInfo) -> Vec<Vec<Requirement>> {
        match (self, rhs) {
            (Self::Setting(name), Self::LitStr(value)) => match info.str_settings.get(name).map(|setting| setting.contains(value)) {
                Some(true) => Vec::default(),
                Some(false) => vec![Vec::default()],
                None => unimplemented!("ne_requirements with undetermined setting info ({} != {})", name, value),
            },
            _ => unimplemented!("{:?}.ne_requirements({:?}, _)", self, rhs),
        }
    }

    fn not_requirements(&self, info: &SeedInfo) -> Vec<Vec<Requirement>> {
        match self {
            Self::Any(exprs) => exprs.iter()
                .map(|expr| expr.not_requirements(info))
                .multi_cartesian_product()
                .map(|products| products.into_iter().flatten().unique().collect())
                .unique()
                .collect(),
            Self::Eq(expr1, expr2) => expr1.ne_requirements(expr2, info),
            Self::Setting(name) => match info.bool_settings.get(name) {
                Some(true) => Vec::default(),
                Some(false) => vec![Vec::default()],
                None => unreachable!("not_requirements with undetermined setting info"),
            },
            Self::TrialActive(med) => match info.trials.get(med) {
                Some(true) => Vec::default(),
                Some(false) => vec![Vec::default()],
                None => unreachable!("not_requirements with undetermined trials info"),
            },
            _ => unimplemented!("{:?}.not_requirements(_)", self),
        }
    }

    /// Returns the requirements for this expression, with the given seed info, in sum of products form.
    fn requirements(&self, info: &SeedInfo) -> Vec<Vec<Requirement>> {
        match self {
            Self::All(exprs) => exprs.iter()
                .map(|expr| expr.requirements(info))
                .multi_cartesian_product()
                .map(|products| products.into_iter().flatten().unique().collect())
                .unique()
                .collect(),
            Self::Any(exprs) => exprs.iter().flat_map(|expr| expr.requirements(info)).unique().collect(),
            Self::Age => unimplemented!("Age.requirements"), //TODO
            Self::AnonymousEvent(ctx, id) => vec![vec![Requirement::AnonymousEvent(ctx.clone(), *id)]],
            Self::Eq(expr1, expr2) => expr1.eq_requirements(expr2, info),
            Self::Event(name) => vec![vec![Requirement::Event(name.clone())]],
            Self::HasDungeonRewards(count) => DungeonReward::into_enum_iter().map(|reward| reward.to_item_requirement()).combinations(count.eval_u8(info).into()).collect(),
            Self::HasMedallions(count) => Medallion::into_enum_iter().map(|med| med.to_item_requirement()).combinations(count.eval_u8(info).into()).collect(),
            Self::HasStones(count) => Stone::into_enum_iter().map(|stone| stone.to_item_requirement()).combinations(count.eval_u8(info).into()).collect(),
            Self::Item(name, count) => vec![vec![Requirement::Item(name.clone(), count.eval_u8(info))]],
            Self::Not(expr) => expr.not_requirements(info),
            Self::Setting(name) => match info.bool_settings.get(name) {
                Some(true) => vec![Vec::default()],
                Some(false) => Vec::default(),
                None => unreachable!("requirements with undetermined setting info"),
            },
            Self::TrialActive(_) => unimplemented!("TrialActive.requirements"), //TODO
            Self::Trick(name) => match info.tricks.get(name) {
                Some(true) => vec![Vec::default()],
                Some(false) => Vec::default(),
                None => unreachable!("requirements with undetermined trick info"),
            },
            Self::Time(_) => match info.time_of_day {
                Some(true) => vec![Vec::default()],
                Some(false) => Vec::default(),
                None => unreachable!("requirements with undetermined time-of-day info"),
            },
            Self::True => vec![Vec::default()],
            // nonsense
            Self::LogicHelper(_, _) | Self::Param(_) => unreachable!("requirements must be called after resolve_helpers"),
            Self::ForAge(_) | Self::LitInt(_) | Self::LitStr(_) => unreachable!("access expression {:?} does not have a reachability meaning", self),
        }
    }
}

/*
impl ModelState {
    /// If access depends on other checks (including an event or the value of an unknown setting), those checks are returned.
    fn can_access(&self, rule: &access::Expr) -> Result<bool, HashSet<Check>> {
        Ok(match rule {
            access::Expr::Item(item, count) => self.access_expr_le_val(count, self.ram.save.amount_of_item(item))?,
            access::Expr::Time(range) => self.ram.save.time_of_day.matches(*range), //TODO take location of check into account, as well as available ways to pass time
        })
    }
}
*/ //TODO move to method on Expr to compile it to Rust code
