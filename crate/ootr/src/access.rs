use {
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
