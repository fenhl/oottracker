use std::{
    fmt,
    hash::Hash,
};
#[cfg(not(target_arch = "wasm32"))] use {
    std::{
        collections::BTreeMap,
        ffi::OsStr,
        hash::Hasher,
        io,
        sync::Arc,
    },
    itertools::{
        EitherOrBoth,
        Itertools as _,
    },
    serde::Deserialize,
    crate::Rando
};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub enum RegionLookupError {
    Filename,
    Io(Arc<io::Error>),
    MixedOverworldAndDungeon,
    MultipleFound,
    NotFound,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<io::Error> for RegionLookupError {
    fn from(e: io::Error) -> RegionLookupError {
        RegionLookupError::Io(Arc::new(e))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl fmt::Display for RegionLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegionLookupError::Filename => write!(f, "region file name is not valid UTF-8"),
            RegionLookupError::Io(e) => write!(f, "I/O error: {}", e),
            RegionLookupError::MixedOverworldAndDungeon => write!(f, "region found in both the overworld and a dungeon"),
            RegionLookupError::MultipleFound => write!(f, "found multiple regions with the same name"),
            RegionLookupError::NotFound => write!(f, "region not found"),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionLookup {
    Overworld(Region),
    /// vanilla data on the left, MQ data on the right
    Dungeon(EitherOrBoth<Region, Region>),
}

#[cfg(not(target_arch = "wasm32"))]
impl RegionLookup {
    pub fn new(candidates: impl IntoIterator<Item = (Mq, Region)>) -> Result<RegionLookup, RegionLookupError> {
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

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
struct MissingRegionError(pub String);

#[cfg(not(target_arch = "wasm32"))]
impl fmt::Display for MissingRegionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "missing region: {}", self.0)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::error::Error for MissingRegionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mq {
    Overworld,
    Vanilla,
    Mq,
}

impl fmt::Display for Mq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mq::Overworld => write!(f, "overworld"),
            Mq::Vanilla => write!(f, "vanilla"),
            Mq::Mq => write!(f, "MQ"),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub region_name: String,
    pub dungeon: Option<String>,
    pub scene: Option<String>,
    hint: Option<String>,
    #[serde(default)]
    time_passes: bool,
    #[serde(default)]
    events: BTreeMap<String, String>,
    #[serde(default)]
    locations: BTreeMap<String, String>,
    #[serde(default)]
    pub exits: BTreeMap<String, String>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Region {
    pub fn new(rando: &Rando, name: &str) -> Result<RegionLookup, RegionLookupError> {
        RegionLookup::new(Region::all(rando)?.into_iter().filter(|(_, region)| region.region_name == name))
    }

    pub fn all(rando: &Rando) -> Result<Vec<(Mq, Region)>, RegionLookupError> {
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

    pub fn root(rando: &Rando) -> io::Result<Region> { //TODO glitched param
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

#[cfg(not(target_arch = "wasm32"))]
impl PartialEq for Region {
    fn eq(&self, rhs: &Region) -> bool {
        self.region_name == rhs.region_name
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Eq for Region {}

#[cfg(not(target_arch = "wasm32"))]
impl Hash for Region {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.region_name.hash(state);
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.hint.as_ref().unwrap_or(&self.region_name).fmt(f)
    }
}
