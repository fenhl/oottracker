use {
    std::{
        ffi::OsStr,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionLookup {
    Overworld(Region),
    /// vanilla data on the left, MQ data on the right
    Dungeon(EitherOrBoth<Region, Region>),
}

impl RegionLookup {
    pub fn new<R: Rando>(candidates: impl IntoIterator<Item = (Mq, Region)>) -> Result<RegionLookup, RegionLookupError<R>> {
        let mut candidates = candidates.into_iter().collect_vec();
        Ok(if candidates.len() == 0 {
            return Err(RegionLookupError::NotFound)
        } else if candidates.len() == 1 && candidates[0].0 == Mq::Overworld {
            RegionLookup::Overworld(candidates.pop().expect("just checked").1.clone())
        } else if candidates.iter().all(|(mq_info, _)| *mq_info != Mq::Overworld) {
            let mut vanilla = None;
            let mut mq = None;
            for (mq_info, region) in candidates {
                let item = match mq_info {
                    Mq::Overworld => unreachable!("just checked that no candidates are overworld"),
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
    fn new<R: Rando>(rando: &R, name: &str) -> Result<RegionLookup, RegionLookupError<R>>;
    fn all<R: Rando>(rando: &R) -> Result<Vec<(Mq, Region)>, RegionLookupError<R>>;
    fn root<R: Rando>(rando: &R) -> io::Result<Region>; //TODO glitched param
}

impl RegionExt for Region {
    fn new<R: Rando>(rando: &R, name: &str) -> Result<RegionLookup, RegionLookupError<R>> {
        RegionLookup::new(Region::all(rando)?.into_iter().filter(|(_, region)| region.region_name == name))
    }

    fn all<R: Rando>(rando: &R) -> Result<Vec<(Mq, Region)>, RegionLookupError<R>> {
        let mut buf = Vec::default();
        let region_files = rando.regions()?;
        for (filename, regions) in region_files.iter() {
            let filename = filename.to_str().ok_or(RegionLookupError::Filename)?;
            for region in regions {
                buf.push((
                    if filename == "Overworld.json" { Mq::Overworld } else if filename.ends_with(" MQ.json") { Mq::Mq } else { Mq::Vanilla },
                    region.clone(),
                ));
            }
        }
        Ok(buf)
    }

    fn root<R: Rando>(rando: &R) -> io::Result<Region> {
        Ok(
            rando.regions()?
                .get(OsStr::new("Overworld.json"))
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, MissingRegionError(format!("Root"))))?
                .iter()
                .find(|Region { region_name, .. }| region_name == "Root")
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, MissingRegionError(format!("Root"))))?
                .clone()
        )
    }
}
