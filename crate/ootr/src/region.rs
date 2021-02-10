use {
    std::{
        collections::HashMap,
        fmt,
        hash::{
            Hash,
            Hasher,
        },
    },
    async_proto::Protocol,
    quote_value::QuoteValue,
    crate::{
        access,
        model::Dungeon,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Protocol, QuoteValue)]
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

impl PartialEq for Region {
    fn eq(&self, other: &Region) -> bool {
        self.dungeon == other.dungeon && self.name == other.name
    }
}

impl Eq for Region {}

impl Hash for Region {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.dungeon.hash(state);
    }
}
