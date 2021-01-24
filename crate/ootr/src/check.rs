use {
    std::fmt,
    quote_value::QuoteValue,
    crate::{
        model::{
            Dungeon,
            Medallion,
        },
        region::Mq,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, QuoteValue)]
pub enum Check {
    /// Constructed using `at` or `here`.
    AnonymousEvent(Box<Check>, usize),
    Event(String),
    /// What's behind an entrance.
    Exit {
        from: String,
        from_mq: Option<Mq>,
        to: String,
    },
    /// These are the things the randomizer itself considers checks.
    Location(String),
    /// Used as the context for anonymous events in logic helpers.
    LogicHelper(String),
    /// Check whether the given dungeon is MQ or vanilla.
    Mq(Dungeon),
    Setting(String), //TODO include the partitions that can be checked
    TrialActive(Medallion),
    Trick(String),
}

impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Check::AnonymousEvent(at_check, id) => write!(f, "requirement {} for {}", id, at_check),
            Check::Event(event) => write!(f, "event: {}", event),
            Check::Exit { from, from_mq, to } => write!(f, "{} ({}) → {}", from, from_mq.map_or_else(|| format!("overworld"), |mq| mq.to_string()), to),
            Check::Location(loc) => loc.fmt(f),
            Check::LogicHelper(fn_name) => write!(f, "logic helper {:?}", fn_name),
            Check::Mq(dungeon) => write!(f, "is {} MQ or vanilla", dungeon),
            Check::Setting(setting) => write!(f, "setting: {}", setting), //TODO show setting's display name
            Check::TrialActive(med) => write!(f, "{} trial active", med.element()),
            Check::Trick(trick) => write!(f, "trick: {}", trick), //TODO show trick's display name
        }
    }
}
