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
    serde::{
        Deserialize,
        Serialize,
    },
    crate::{
        Rando,
        access,
        model::Dungeon,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Protocol, Deserialize, Serialize, QuoteValue)]
#[serde(rename_all = "snake_case")]
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
#[quote_value(where(R::RegionName: QuoteValue))]
pub struct Region<R: Rando> {
    pub name: R::RegionName,
    pub dungeon: Option<(Dungeon, Mq)>,
    pub scene: Option<String>, //TODO use Scene type from oottracker?
    pub hint: Option<String>,
    pub time_passes: bool,
    pub events: HashMap<String, access::Expr<R>>,
    pub locations: HashMap<String, access::Expr<R>>,
    pub exits: HashMap<R::RegionName, access::Expr<R>>,
}

impl<R: Rando> PartialEq for Region<R> {
    fn eq(&self, other: &Region<R>) -> bool {
        self.dungeon == other.dungeon && self.name == other.name
    }
}

impl<R: Rando> Eq for Region<R> {}

impl<R: Rando> Hash for Region<R> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.dungeon.hash(state);
    }
}
