use {
    std::{
        collections::{
            HashMap,
            hash_set::{
                self,
                HashSet,
            },
        },
        fmt,
        io,
        iter,
        ops::BitAnd,
        sync::Arc,
    },
    async_proto::Protocol,
    enum_iterator::IntoEnumIterator,
    wheel::FromArc,
    crate::{
        ModelState,
        item::Item,
        model::{
            Dungeon,
            MainDungeon,
        },
    },
};

oottracker_derive::region!();

pub type Access = fn(&ModelState) -> bool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Protocol)]
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

#[derive(Debug, Clone, FromArc)]
pub enum RegionLookupError {
    Filename,
    #[from_arc]
    Io(Arc<io::Error>),
    MixedOverworldAndDungeon,
    MultipleFound,
    NotFound,
    UnknownScene(u8),
}

impl fmt::Display for RegionLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Filename => write!(f, "region file name is not valid UTF-8"),
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::MixedOverworldAndDungeon => write!(f, "region found in both the overworld and a dungeon"),
            Self::MultipleFound => write!(f, "found multiple regions with the same name"),
            Self::NotFound => write!(f, "region not found"),
            Self::UnknownScene(id) => write!(f, "unknown scene: 0x{:02x}", id),
        }
    }
}

//TODO review whether this is still required — it might make more sense to include MQ-ness in the Region value itself (merging fully ambiguous regions like Dodongos Cavern Beginning)
/*
#[derive(Debug, Clone)]
pub enum RegionLookup {
    Overworld(Region),
    /// vanilla data on the left, MQ data on the right
    Dungeon(EitherOrBoth<Region, Region>),
}

impl RegionLookup {
    pub fn new(candidates: impl IntoIterator<Item = Region>) -> Result<RegionLookup, RegionLookupError> {
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

    pub fn by_name<N: ?Sized + PartialEq<Region>>(name: &N) -> Result<RegionLookup, RegionLookupError> {
        let candidates = Region::into_enum_iter().iter().filter(|region| region.name == *name).cloned().collect_vec();
        RegionLookup::new(candidates)
    }
}
*/

#[derive(Debug, Clone)]
struct MissingRegionError(pub String);

impl fmt::Display for MissingRegionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "missing region: {}", self.0)
    }
}

impl std::error::Error for MissingRegionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Protocol)]
pub struct Entrance {
    pub from: Region,
    pub to: Region,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Protocol)]
pub struct EntranceKnowledge(/*TODO*/);

impl BitAnd for EntranceKnowledge {
    type Output = Result<Self, ()>;

    fn bitand(self, _: Self) -> Result<Self, ()> {
        Ok(Self(/*TODO*/))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Protocol)]
pub struct LocationKnowledge(HashSet<Item>);

impl LocationKnowledge {
    /// Returns location knowledge with no possible items (a contradiction).
    pub fn empty() -> Self {
        Self(HashSet::default())
    }

    pub fn contains(&self, item: Item) -> bool {
        self.0.contains(&item)
    }

    pub fn insert(&mut self, item: Item) {
        self.0.insert(item);
    }

    pub fn remove(&mut self, item: Item) -> bool {
        self.0.remove(&item)
    }
}

impl BitAnd for LocationKnowledge {
    type Output = Result<Self, ()>;

    fn bitand(self, rhs: Self) -> Result<Self, ()> {
        let intersection = self.0.intersection(&rhs.0).copied().collect::<HashSet<_>>();
        if intersection.is_empty() {
            Err(())
        } else {
            Ok(Self(intersection))
        }
    }
}

impl<'a> IntoIterator for &'a LocationKnowledge {
    type IntoIter = iter::Copied<hash_set::Iter<'a, Item>>;
    type Item = Item;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

pub fn vanilla_entrances() -> HashMap<Entrance, EntranceKnowledge> {
    HashMap::default() //TODO
}

pub fn vanilla_locations() -> HashMap<String, LocationKnowledge> { //TODO Location enum?
    HashMap::default() //TODO
}
