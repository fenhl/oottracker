use {
    std::{
        collections::HashMap,
        fmt,
    },
    quote_value::QuoteValue,
    crate::{
        access,
        model::Dungeon,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, QuoteValue)]
pub enum Mq {
    Vanilla,
    Mq,
}

impl fmt::Display for Mq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mq::Vanilla => write!(f, "vanilla"),
            Mq::Mq => write!(f, "MQ"),
        }
    }
}

#[derive(Debug, Clone, QuoteValue)]
pub struct Region {
    pub name: String,
    pub dungeon: Option<(Dungeon, Mq)>,
    pub scene: Option<String>, //TODO use Scene type from oottracker?
    pub hint: Option<String>,
    pub time_passes: bool,
    pub events: HashMap<String, access::Expr>,
    pub locations: HashMap<String, access::Expr>,
    pub exits: HashMap<String, access::Expr>,
}
