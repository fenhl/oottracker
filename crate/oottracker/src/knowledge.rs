use {
    std::{
        collections::HashMap,
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
    derive_more::From,
    enum_iterator::IntoEnumIterator as _,
    itertools::Itertools as _,
    tokio::io::{
        AsyncRead,
        AsyncWrite,
    },
    crate::{
        model::*,
        region::{
            Entrance,
            EntranceKnowledge,
            LocationKnowledge,
            Mq,
        },
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Knowledge {
    pub settings: settings::Knowledge,
    pub dungeons: HashMap<Dungeon, Mq>,
    pub trials: HashMap<Medallion, bool>, //TODO use "active"/"inactive" in ser/de
    pub entrances: HashMap<Entrance, EntranceKnowledge>,
    pub locations: HashMap<String, LocationKnowledge>,
    pub progression_mode: ProgressionMode, //TODO automatically determine from remaining model state
}

impl Knowledge {
    /// We know that everything is vanilla. Used by auto-trackers when the base game, rather than rando, is detected.
    pub fn vanilla() -> Self {
        Self {
            settings: settings::Knowledge::vanilla(),
            dungeons: Dungeon::into_enum_iter().map(|dungeon| (dungeon, Mq::Vanilla)).collect(),
            trials: Medallion::into_enum_iter().map(|trial| (trial, true)).collect(),
            entrances: crate::region::vanilla_entrances(), //TODO const/static?
            locations: crate::region::vanilla_locations(), //TODO const/static?
            progression_mode: ProgressionMode::Go,
        }
    }

    // some convenience methods for working with dungeon reward locations, since those are the only purely knowledge-based cells on the default layout

    pub fn get_dungeon_reward_location(&self, reward: DungeonReward) -> Option<DungeonRewardLocation> {
        //TODO bool parameter determining whether to show equivalent rewards in order, e.g. show stone n in nth unknown location if all meds are assigned
        DungeonRewardLocation::into_enum_iter()
            .filter(|iter_loc| self.locations.get(iter_loc.as_str()).map_or(false, |loc_info| loc_info.contains(reward.into())))
            .exactly_one()
            .ok()
    }

    pub fn increment_dungeon_reward_location(&mut self, reward: DungeonReward) {
        match self.get_dungeon_reward_location(reward) {
            None => self.set_dungeon_reward_location(reward, DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)) => self.set_dungeon_reward_location(reward, DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)) => self.set_dungeon_reward_location(reward, DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)) => self.set_dungeon_reward_location(reward, DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)) => self.set_dungeon_reward_location(reward, DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)) => self.set_dungeon_reward_location(reward, DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)) => self.set_dungeon_reward_location(reward, DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)) => self.set_dungeon_reward_location(reward, DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)) => self.set_dungeon_reward_location(reward, DungeonRewardLocation::LinksPocket),
            Some(DungeonRewardLocation::LinksPocket) => self.remove_dungeon_reward_location(reward),
        }
    }

    pub fn decrement_dungeon_reward_location(&mut self, reward: DungeonReward) {
        match self.get_dungeon_reward_location(reward) {
            None => self.set_dungeon_reward_location(reward, DungeonRewardLocation::LinksPocket),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)) => self.remove_dungeon_reward_location(reward),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)) => self.set_dungeon_reward_location(reward, DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)) => self.set_dungeon_reward_location(reward, DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)) => self.set_dungeon_reward_location(reward, DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)) => self.set_dungeon_reward_location(reward, DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)) => self.set_dungeon_reward_location(reward, DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)) => self.set_dungeon_reward_location(reward, DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)) => self.set_dungeon_reward_location(reward, DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)),
            Some(DungeonRewardLocation::LinksPocket) => self.set_dungeon_reward_location(reward, DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)),
        }
    }

    pub fn set_dungeon_reward_location(&mut self, reward: DungeonReward, loc: DungeonRewardLocation) {
        for iter_loc in DungeonRewardLocation::into_enum_iter() {
            if iter_loc == loc {
                self.locations.entry(iter_loc.to_string()).or_insert_with(LocationKnowledge::empty).insert(reward.into());
            } else {
                if let Some(loc_info) = self.locations.get_mut(iter_loc.as_str()) {
                    loc_info.remove(reward.into());
                }
            }
        }
    }

    pub fn remove_dungeon_reward_location(&mut self, reward: DungeonReward) {
        for iter_loc in DungeonRewardLocation::into_enum_iter() {
            if let Some(loc_info) = self.locations.get_mut(iter_loc.as_str()) {
                loc_info.remove(reward.into());
            }
        }
    }
}

impl Default for Knowledge {
    /// We don't know anything about the seed.
    fn default() -> Self {
        Self {
            settings: settings::Knowledge::default(),
            dungeons: HashMap::default(),
            trials: HashMap::default(),
            entrances: HashMap::default(), //TODO fill in always-unshuffled entrances
            locations: HashMap::default(), //TODO fill in always-unshuffled locations
            progression_mode: ProgressionMode::Normal,
        }
    }
}

#[derive(From)]
pub enum Contradiction {
    Settings(settings::Contradiction),
    Mq {
        dungeon: Dungeon,
        lhs_mq: Mq,
    },
    Trial {
        trial: Medallion,
        lhs_active: bool,
    },
    Entrance,
    Location,
}

impl BitAnd for Knowledge {
    type Output = Result<Self, Contradiction>;

    fn bitand(self, rhs: Self) -> Result<Self, Contradiction> {
        let Self { settings, dungeons, trials, entrances, locations, progression_mode } = self;
        Ok(Self {
            settings: (settings & rhs.settings)?,
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
                    if let Some(lhs_exit) = entrances.get_mut(&exit) {
                        *lhs_exit = (*lhs_exit & rhs_exit).map_err(|()| Contradiction::Entrance)?;
                    } else {
                        entrances.insert(exit, rhs_exit);
                    }
                }
                entrances
            },
            locations: {
                let mut locations = locations;
                for (loc_name, rhs_loc) in rhs.locations {
                    if let Some(lhs_loc) = locations.get_mut(&loc_name) {
                        *lhs_loc = (lhs_loc.clone() & rhs_loc).map_err(|()| Contradiction::Location)?;
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

impl Protocol for Knowledge {
    fn read<'a, Rd: AsyncRead + Unpin + Send + 'a>(stream: &'a mut Rd) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(match u8::read(stream).await? {
                0 => Self {
                    settings: settings::Knowledge::read(stream).await?,
                    dungeons: HashMap::read(stream).await?,
                    trials: HashMap::read(stream).await?,
                    entrances: HashMap::read(stream).await?,
                    locations: HashMap::read(stream).await?,
                    progression_mode: ProgressionMode::read(stream).await?,
                },
                1 => Self::default(),
                2 => Knowledge::vanilla(),
                n => return Err(ReadError::UnknownVariant8(n)),
            })
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            if *self == Self::default() {
                1u8.write(sink).await?;
            } else if *self == Self::vanilla() {
                2u8.write(sink).await?;
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
        } else if *self == Self::vanilla() {
            2u8.write_sync(sink)?;
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
