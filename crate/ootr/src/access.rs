use {
    std::borrow::Cow,
    quote_value::QuoteValue,
    crate::{
        Rando,
        check::Check,
        item::Item,
        model::{
            Medallion,
            TimeRange,
        },
    },
};

pub enum ModelState {} //TODO move from oottracker repo

trait RandoWrap {}

impl<R: Rando> RandoWrap for R {}

#[derive(Debug, Clone, PartialEq, Eq, QuoteValue)]
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
}
