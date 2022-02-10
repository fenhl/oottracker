use {
    std::borrow::Cow,
    derivative::Derivative,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, QuoteValue)]
pub enum ForAge {
    Child,
    Adult,
    Both,
    Either,
}

#[derive(Derivative, QuoteValue)]
#[derivative(Debug(bound = ""), Clone(bound = ""))]
#[quote_value(where(R::RegionName: QuoteValue))]
pub enum Expr<R: Rando> {
    All(Vec<Expr<R>>),
    Any(Vec<Expr<R>>),
    Age,
    AnonymousEvent(Check<R>, usize),
    Contains(Box<Expr<R>>, Box<Expr<R>>),
    Eq(Box<Expr<R>>, Box<Expr<R>>),
    Event(String),
    /// used in helper `has_projectile`. Should only compare equal to itself.
    ForAge(ForAge),
    HasDungeonRewards(Box<Expr<R>>),
    HasMedallions(Box<Expr<R>>),
    HasStones(Box<Expr<R>>),
    Item(Item, Box<Expr<R>>),
    LacsCondition,
    LitInt(u8),
    LitStr(String),
    LogicHelper(String, Vec<Expr<R>>),
    Not(Box<Expr<R>>),
    /// logic helper parameter
    Param(String),
    Setting(String),
    StartingAge,
    TrialActive(Medallion),
    Trick(String),
    Time(TimeRange),
    True,
}

impl<R: Rando> Expr<R> {
    pub fn resolve_args<'a>(&'a self, params: &[String], args: &'a [Expr<R>]) -> Cow<'a, Expr<R>> {
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
