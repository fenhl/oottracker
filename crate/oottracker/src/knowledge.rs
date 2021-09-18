use {
    std::{
        collections::{
            HashMap,
            HashSet,
        },
        future::Future,
        io::prelude::*,
        ops::BitAnd,
        pin::Pin,
    },
    async_proto::{
        Protocol,
        ReadError,
        WriteError,
    },
    derivative::Derivative,
    enum_iterator::IntoEnumIterator as _,
    tokio::io::{
        AsyncRead,
        AsyncWrite,
    },
    ootr::{
        Rando,
        model::*,
        region::Mq,
        settings,
    },
};

#[derive(Derivative, Debug, Clone, Copy, PartialEq, Eq, Protocol)]
#[derivative(Default)]
pub enum ProgressionMode {
    /// No progression available. Should only occur in multiworld and no-logic seeds.
    Bk,
    /// The player is neither done nor in go mode nor in BK mode.
    #[derivative(Default)]
    Normal,
    /// The player either has or knows the location of every item required to beat the game.
    ///
    /// See <https://github.com/fenhl/oottracker/issues/9#issuecomment-783503311> for a more detailed definition.
    Go,
    /// Game beaten.
    Done,
}

/// Represents information the player has about the game, whether from external sources (settings string, spoiler log, RSL weights) or from things seen in the game itself.
///
/// The format of this type is a superset of the plando format.
#[derive(Debug)]
pub struct Knowledge<R: Rando> {
    pub settings: R::SettingsKnowledge,
    pub dungeons: HashMap<Dungeon, Mq>,
    pub trials: HashMap<Medallion, bool>, //TODO use "active"/"inactive" in ser/de
    pub entrances: HashMap<Entrance<R>, EntranceKnowledge<R>>,
    pub locations: HashMap<String, LocationKnowledge>,
    pub progression_mode: ProgressionMode, //TODO automatically determine from remaining model state
}

impl<R: Rando> Knowledge<R> {
    /// We know that everything is vanilla. Used by auto-trackers when the base game, rather than rando, is detected.
    pub fn vanilla(rando: &R) -> Self {
        Self {
            settings: settings::Knowledge::vanilla(rando),
            dungeons: Dungeon::into_enum_iter().map(|dungeon| (dungeon, Mq::Vanilla)).collect(),
            trials: Medallion::into_enum_iter().map(|trial| (trial, true)).collect(),
            entrances: rando.vanilla_entrances(),
            locations: rando.vanilla_locations(),
            progression_mode: ProgressionMode::Go,
        }
    }

    pub fn get_dungeon_reward_location(&self, reward: DungeonReward) -> Option<DungeonRewardLocation> {
        //TODO show equivalent rewards in order, e.g. show stone n in nth unknown location if all meds are assigned
        DungeonRewardLocation::into_enum_iter()
            .filter(|iter_loc| self.locations.get(iter_loc.as_str()).map_or(false, |loc_info| loc_info.contains(&reward)))
            .exactly_one()
            .ok()
    }

    pub fn set_dungeon_reward_location(&mut self, reward: DungeonReward, loc: DungeonRewardLocation) {
        for iter_loc in DungeonRewardLocation::into_enum_iter() {
            if iter_loc == loc {
                self.locations.entry(iter_loc.to_string()).or_default().add(reward);
            } else {
                if let Some(loc_info) = self.locations.get_mut(iter_loc.as_str()) {
                    loc_info.remove(&reward);
                }
            }
        }
    }

    pub fn remove_dungeon_reward_location(&mut self, reward: DungeonReward) {
        for iter_loc in DungeonRewardLocation::into_enum_iter() {
            if let Some(loc_info) = self.locations.get_mut(iter_loc.as_str()) {
                loc_info.remove(&reward);
            }
        }
    }
}

/*
impl Knowledge {
    /// We know that everything is vanilla. Used by auto-trackers when the base game, rather than rando, is detected.
    pub fn vanilla() -> Knowledge {
        Knowledge {
            dungeon_reward_locations: collect![
                DungeonReward::Stone(Stone::KokiriEmerald) => DungeonRewardLocation::Dungeon(MainDungeon::DekuTree),
                DungeonReward::Stone(Stone::GoronRuby) => DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern),
                DungeonReward::Stone(Stone::ZoraSapphire) => DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu),
                DungeonReward::Medallion(Medallion::Forest) => DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple),
                DungeonReward::Medallion(Medallion::Fire) => DungeonRewardLocation::Dungeon(MainDungeon::FireTemple),
                DungeonReward::Medallion(Medallion::Water) => DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple),
                DungeonReward::Medallion(Medallion::Shadow) => DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple),
                DungeonReward::Medallion(Medallion::Spirit) => DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple),
                DungeonReward::Medallion(Medallion::Light) => DungeonRewardLocation::LinksPocket,
            ],
            exits: None, //TODO properly initialize with all exits
        }
    }
}
*/ //TODO delete (replaced with new definition above)

pub enum Contradiction {
    BoolSetting {
        name: String,
        lhs_enabled: bool,
    },
    StringSetting {
        name: String,
        lhs_values: HashSet<String>,
        rhs_values: HashSet<String>,
    },
    Trick {
        name: String,
        lhs_enabled: bool,
    },
    Mq {
        dungeon: Dungeon,
        lhs_mq: Mq,
    },
    Trial {
        trial: Medallion,
        lhs_active: bool,
    },
    DungeonRewardLocation {
        reward: DungeonReward,
        lhs_location: DungeonRewardLocation,
        rhs_location: DungeonRewardLocation,
    },
}

impl<R: Rando> BitAnd for Knowledge<R> {
    type Output = Result<Self, Contradiction>;

    fn bitand(self, rhs: Self) -> Result<Self, Contradiction> {
        let Self { settings, dungeons, trials, entrances, locations, progression_mode } = self;
        Ok(Self {
            settings: settings & rhs.settings,
            dungeons: {
                let mut dungeons = dungeons;
                for (dungeon, rhs_mq) in rhs.dungeons {
                    if let Some(&lhs_mq) = dungeons.get(&dungeon) {
                        if lhs_mq != rhs_mq {
                            return Err(Contradiction::Mq { dungeon, lhs_mq })
                        }
                    } else {
                        dungeons.insert(dungeon, rhs_mq);
                    }
                }
                dungeons
            },
            trials: {
                let mut trials = trials;
                for (trial, rhs_active) in rhs.trials {
                    if let Some(&lhs_active) = trials.get(&trial) {
                        if lhs_active != rhs_active {
                            return Err(Contradiction::Trial { trial, lhs_active })
                        }
                    } else {
                        trials.insert(trial, rhs_active);
                    }
                }
                trials
            },
            entrances: {
                let mut entrances = entrances;
                for (exit, rhs_exit) in rhs.entrances {
                    if let Some(&lhs_exit) = entrances.get_mut(&exit) {
                        *lhs_exit = lhs_exit & rhs_exit?;
                    } else {
                        entrances.insert(exit, rhs_exit);
                    }
                }
                entrances
            },
            locations: {
                let mut locations = locations;
                for (loc_name, rhs_loc) in rhs.locations {
                    if let Some(&lhs_loc) = locations.get_mut(&loc_name) {
                        *lhs_loc = lhs_loc & rhs_loc?;
                    } else {
                        locations.insert(loc_name, rhs_loc);
                    }
                }
                locations
            },
            progression_mode: if progression_mode == rhs.progression_mode { progression_mode } else { ProgressionMode::Normal }, //TODO this should actually be recalculated from the rest of the knowledge, using dummy values for now
        })
    }
}

impl<R: Rando> Protocol for Knowledge<R> {
    fn read<'a, Rd: AsyncRead + Unpin + Send + 'a>(stream: &'a mut Rd) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(match u8::read(stream).await? {
                0 => Self {
                    settings: R::SettingsKnowledge::read(stream).await?,
                    dungeons: HashMap::read(stream).await?,
                    trials: HashMap::read(stream).await?,
                    entrances: HashMap::read(stream).await?,
                    locations: HashMap::read(stream).await?,
                    progression_mode: ProgressionMode::read(stream).await?,
                },
                1 => Self::default(),
                //2 => Knowledge::<ootr_static::Rando>::vanilla_static().into(), //TODO reenable?
                n => return Err(ReadError::UnknownVariant(n)),
            })
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            if *self == Self::default() {
                1u8.write(sink).await?;
            /*
            } else if *self == Self::vanilla() {
                2u8.write(sink).await?;
            */ //TODO reenable functionality somehow? (e.g. :is_vanilla entry)
            } else {
                0u8.write(sink).await?;
                self.settings.write(sink).await?;
                self.dungeons.write(sink).await?;
                self.trials.write(sink).await?;
                self.entrances.write(sink).await?;
                self.locations.write(sink).await?;
                self.progression_mode.write(sink).await?;
            }
            Ok(())
        })
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        if *self == Self::default() {
            1u8.write_sync(sink)?;
        /*
        } else if *self == Self::vanilla() {
            2u8.write_sync(sink)?;
        */ //TODO reenable functionality somehow? (e.g. :is_vanilla entry)
        } else {
            0u8.write_sync(sink)?;
            self.settings.write_sync(sink)?;
            self.dungeons.write_sync(sink)?;
            self.trials.write_sync(sink)?;
            self.entrances.write_sync(sink)?;
            self.locations.write_sync(sink)?;
            self.progression_mode.write_sync(sink)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct LocationKnowledge(/*TODO*/);

#[derive(Derivative, Debug)]
#[derivative(PartialEq(bound = ""), Eq(bound = ""), Hash(bound = ""))]
pub struct Entrance<R: Rando> {
    pub from: R::RegionName,
    pub to: R::RegionName,
}

impl<R: Rando> Protocol for Entrance<R> { //TODO derive
    fn read<'a, Rd: AsyncRead + Unpin + Send + 'a>(stream: &'a mut Rd) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(Self {
                from: R::RegionName::read(stream).await?,
                to: R::RegionName::read(stream).await?,
            })
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            self.from.write(sink).await?;
            self.to.write(sink).await?;
            Ok(())
        })
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.from.write_sync(sink)?;
        self.to.write_sync(sink)?;
        Ok(())
    }
}

use std::marker::PhantomData; //TODO
#[derive(Debug)]
pub struct EntranceKnowledge<R: Rando>(PhantomData<R> /*TODO*/);
