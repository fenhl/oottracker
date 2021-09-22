#![allow(unused)] //TODO

use {
    std::fmt,
    crate::{
        model::{
            Dungeon,
            Medallion,
        },
        region::Region,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Check {
    /// Constructed using `at` or `here`.
    AnonymousEvent(Box<Check>, usize),
    Event(String),
    /// What's behind an entrance.
    Exit { // don't merge with knowledge checks, as there should only be 1 check per exit, not 1 per possible connected entrance
        from: Region,
        to: Region,
    },
    /// These are the things the randomizer itself considers checks.
    Location(String), //TODO use Location enum?
    /// Used as the context for anonymous events in logic helpers.
    LogicHelper(String), //TODO use LogicHelper enum?
    /// Check whether the given dungeon is MQ or vanilla.
    Mq(Dungeon), //TODO merge with Knowledge check
    Setting(String), //TODO replace with a more generic Knowledge check that checks whether the current knowledge is consistent (subset → true) with the given one, inconsistent (neither subset nor superset → false), or indeterminate (true superset → None)
    TrialActive(Medallion), //TODO merge with Knowledge check
    Trick(String), //TODO merge with Knowledge check
}

impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Check::AnonymousEvent(at_check, id) => write!(f, "requirement {} for {}", id, at_check),
            Check::Event(event) => write!(f, "event: {}", event),
            Check::Exit { from, to } => write!(f, "{} → {}", from, to),
            Check::Location(loc) => loc.fmt(f),
            Check::LogicHelper(fn_name) => write!(f, "logic helper {:?}", fn_name),
            Check::Mq(dungeon) => write!(f, "is {} MQ or vanilla", dungeon),
            Check::Setting(setting) => write!(f, "setting: {}", setting), //TODO show setting's display name
            Check::TrialActive(med) => write!(f, "{} trial active", med.element()),
            Check::Trick(trick) => write!(f, "trick: {}", trick), //TODO show trick's display name
        }
    }
}
