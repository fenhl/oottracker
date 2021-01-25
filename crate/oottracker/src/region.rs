use {
    std::{
        fmt,
        io,
        sync::Arc,
    },
    itertools::{
        EitherOrBoth,
        Itertools as _,
    },
    ootr::{
        Rando,
        region::{
            Mq,
            Region,
        },
    },
};

#[derive(Debug, Clone)]
pub enum RegionLookupError<R: Rando> {
    Filename,
    Io(Arc<io::Error>),
    MixedOverworldAndDungeon,
    MultipleFound,
    NotFound,
    Rando(R::Err),
}

impl<R: Rando> From<io::Error> for RegionLookupError<R> {
    fn from(e: io::Error) -> RegionLookupError<R> {
        RegionLookupError::Io(Arc::new(e))
    }
}

impl<R: Rando> fmt::Display for RegionLookupError<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegionLookupError::Filename => write!(f, "region file name is not valid UTF-8"),
            RegionLookupError::Io(e) => write!(f, "I/O error: {}", e),
            RegionLookupError::MixedOverworldAndDungeon => write!(f, "region found in both the overworld and a dungeon"),
            RegionLookupError::MultipleFound => write!(f, "found multiple regions with the same name"),
            RegionLookupError::NotFound => write!(f, "region not found"),
            RegionLookupError::Rando(e) => e.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RegionLookup {
    Overworld(Arc<Region>),
    /// vanilla data on the left, MQ data on the right
    Dungeon(EitherOrBoth<Arc<Region>, Arc<Region>>),
}

impl RegionLookup {
    pub fn new<R: Rando>(candidates: impl IntoIterator<Item = Arc<Region>>) -> Result<RegionLookup, RegionLookupError<R>> {
        let mut candidates = candidates.into_iter().collect_vec();
        Ok(if candidates.len() == 0 {
            return Err(RegionLookupError::NotFound)
        } else if candidates.len() == 1 && candidates[0].dungeon.is_none() {
            RegionLookup::Overworld(candidates.pop().expect("just checked"))
        } else if candidates.iter().all(|region| region.dungeon.is_some()) {
            let mut vanilla = None;
            let mut mq = None;
            for region in candidates {
                let item = match region.dungeon.expect("just_checked").1 {
                    Mq::Vanilla => &mut vanilla,
                    Mq::Mq => &mut mq,
                };
                if item.is_some() { return Err(RegionLookupError::MultipleFound) }
                *item = Some(region);
            }
            RegionLookup::Dungeon(match (vanilla, mq) {
                (None, None) => return Err(RegionLookupError::NotFound),
                (None, Some(mq)) => EitherOrBoth::Right(mq),
                (Some(vanilla), None) => EitherOrBoth::Left(vanilla),
                (Some(vanilla), Some(mq)) => EitherOrBoth::Both(vanilla, mq),
            })
        } else {
            return Err(RegionLookupError::MixedOverworldAndDungeon)
        })
    }

    pub fn by_name<R: Rando>(rando: &R, name: &str) -> Result<RegionLookup, RegionLookupError<R>> {
        let all_regions = rando.regions().map_err(RegionLookupError::Rando)?;
        let candidates = all_regions.iter().filter(|region| region.name == name).cloned().collect_vec();
        RegionLookup::new(candidates)
    }
}

#[derive(Debug, Clone)]
struct MissingRegionError(pub String);

impl fmt::Display for MissingRegionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "missing region: {}", self.0)
    }
}

impl std::error::Error for MissingRegionError {}

pub trait RegionExt {
    fn new<'a, R: Rando>(rando: &'a R, name: &str) -> Result<RegionLookup, RegionLookupError<R>>;
    /// A thin wrapper around [`Rando::regions`] with this module's error type.
    fn all<'a, R: Rando>(rando: &'a R) -> Result<Arc<Vec<Arc<Region>>>, RegionLookupError<R>>;
    fn root<R: Rando>(rando: &R) -> Result<Arc<Region>, RegionLookupError<R>>; //TODO glitched param
}

impl RegionExt for Region {
    fn new<'a, R: Rando>(rando: &'a R, name: &str) -> Result<RegionLookup, RegionLookupError<R>> {
        RegionLookup::by_name(rando, name)
    }

    fn all<'a, R: Rando>(rando: &'a R) -> Result<Arc<Vec<Arc<Region>>>, RegionLookupError<R>> {
        rando.regions().map_err(RegionLookupError::Rando)
    }

    fn root<R: Rando>(rando: &R) -> Result<Arc<Region>, RegionLookupError<R>> {
        Ok(Arc::clone(Region::all(rando)?.iter().find(|region| region.name == "Root").ok_or(RegionLookupError::NotFound)?))
    }
}
