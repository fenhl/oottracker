use {
    std::{
        fmt,
        io,
        sync::Arc,
    },
    derivative::Derivative,
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

#[derive(Derivative)]
#[derivative(Debug(bound = ""), Clone(bound = ""))]
pub enum RegionLookupError<R: Rando> {
    Filename,
    Io(Arc<io::Error>),
    MixedOverworldAndDungeon,
    MultipleFound,
    NotFound,
    Rando(R::Err),
    UnknownScene(u8),
}

impl<R: Rando> From<io::Error> for RegionLookupError<R> { //TODO add support for generics to FromArc derive macro
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
            RegionLookupError::UnknownScene(id) => write!(f, "unknown scene: 0x{:02x}", id),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RegionLookup<R: Rando> {
    Overworld(Arc<Region<R>>),
    /// vanilla data on the left, MQ data on the right
    Dungeon(EitherOrBoth<Arc<Region<R>>, Arc<Region<R>>>),
}

impl<R: Rando> RegionLookup<R> {
    pub fn new(candidates: impl IntoIterator<Item = Arc<Region<R>>>) -> Result<RegionLookup<R>, RegionLookupError<R>> {
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

    pub fn by_name<N: ?Sized>(rando: &R, name: &N) -> Result<RegionLookup<R>, RegionLookupError<R>>
    where R::RegionName: PartialEq<N> {
        let all_regions = rando.regions().map_err(RegionLookupError::Rando)?;
        let candidates = all_regions.iter().filter(|region| region.name == *name).cloned().collect_vec();
        RegionLookup::new(candidates)
    }
}

pub trait RegionExt {
    type R: Rando;

    fn new<'a, N: ?Sized>(rando: &'a Self::R, name: &N) -> Result<RegionLookup<Self::R>, RegionLookupError<Self::R>> where <Self::R as Rando>::RegionName: PartialEq<N>;
    /// A thin wrapper around [`Rando::regions`] with this module's error type.
    fn all<'a>(rando: &'a Self::R) -> Result<Arc<Vec<Arc<Region<Self::R>>>>, RegionLookupError<Self::R>>;
    fn root(rando: &Self::R) -> Result<Arc<Region<Self::R>>, RegionLookupError<Self::R>>; //TODO glitched param
}

impl<R: Rando> RegionExt for Region<R> {
    type R = R;

    fn new<'a, N: ?Sized>(rando: &'a R, name: &N) -> Result<RegionLookup<R>, RegionLookupError<R>>
    where R::RegionName: PartialEq<N> {
        RegionLookup::by_name(rando, name)
    }

    fn all<'a>(rando: &'a R) -> Result<Arc<Vec<Arc<Region<R>>>>, RegionLookupError<R>> {
        rando.regions().map_err(RegionLookupError::Rando)
    }

    fn root(rando: &R) -> Result<Arc<Region<R>>, RegionLookupError<R>> {
        Ok(Arc::clone(Region::all(rando)?.iter().find(|region| region.name == "Root").ok_or(RegionLookupError::NotFound)?))
    }
}
