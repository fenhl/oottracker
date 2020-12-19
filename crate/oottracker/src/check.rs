use {
    std::fmt,
    crate::{
        model::{
            Dungeon,
            Medallion,
        },
        region::Mq,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Check {
    /// Constructed using `at` or `here`.
    AnonymousEvent(Box<Check>, usize),
    Event(String),
    /// What's behind an entrance.
    Exit {
        from: String,
        from_mq: Mq,
        to: String,
    },
    /// These are the things the randomizer itself considers checks.
    Location(String),
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
            Check::Exit { from, from_mq, to } => write!(f, "{} ({}) → {}", from, from_mq, to),
            Check::Location(loc) => loc.fmt(f),
            Check::Mq(dungeon) => write!(f, "is {} MQ or vanilla", dungeon),
            Check::Setting(setting) => write!(f, "setting: {}", setting), //TODO show setting's display name
            Check::TrialActive(med) => write!(f, "{} trial active", med.element()),
            Check::Trick(trick) => write!(f, "trick: {}", trick), //TODO show trick's display name
        }
    }
}
