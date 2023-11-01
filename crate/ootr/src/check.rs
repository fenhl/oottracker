use {
    std::fmt,
    quote_value::QuoteValue,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, QuoteValue)]
pub enum Check {
    /// These are the things the randomizer itself considers checks.
    Location(String),
}

impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Check::Location(loc) => loc.fmt(f),
        }
    }
}
