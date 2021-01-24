use {
    std::collections::HashMap,
    collect_mac::collect,
    smart_default::SmartDefault,
    ootr::model::*,
};
#[cfg(not(target_arch = "wasm32"))] use {
    std::{
        fmt,
        future::Future,
        io::{
            self,
            prelude::*,
        },
        pin::Pin,
        sync::Arc,
    },
    async_proto::{
        Protocol,
        impls::MapReadError,
    },
    tokio::io::{
        AsyncRead,
        AsyncWrite,
    },
};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub enum KnowledgeReadError {
    ActiveTrials(Arc<MapReadError<Medallion, bool>>),
    BoolSettings(Arc<MapReadError<String, bool>>),
    DungeonRewardLocations(Arc<MapReadError<DungeonReward, DungeonRewardLocation>>),
    Exits(Arc<MapReadError<String, HashMap<String, String>>>),
    Io(Arc<io::Error>),
    Mq(Arc<MapReadError<Dungeon, bool>>),
    UnknownPreset(u8),
}

#[cfg(not(target_arch = "wasm32"))]
impl From<MapReadError<Medallion, bool>> for KnowledgeReadError {
    fn from(e: MapReadError<Medallion, bool>) -> KnowledgeReadError {
        KnowledgeReadError::ActiveTrials(Arc::new(e))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<MapReadError<String, bool>> for KnowledgeReadError {
    fn from(e: MapReadError<String, bool>) -> KnowledgeReadError {
        KnowledgeReadError::BoolSettings(Arc::new(e))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<MapReadError<DungeonReward, DungeonRewardLocation>> for KnowledgeReadError {
    fn from(e: MapReadError<DungeonReward, DungeonRewardLocation>) -> KnowledgeReadError {
        KnowledgeReadError::DungeonRewardLocations(Arc::new(e))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<MapReadError<String, HashMap<String, String>>> for KnowledgeReadError {
    fn from(e: MapReadError<String, HashMap<String, String>>) -> KnowledgeReadError {
        KnowledgeReadError::Exits(Arc::new(e))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<io::Error> for KnowledgeReadError {
    fn from(e: io::Error) -> KnowledgeReadError {
        KnowledgeReadError::Io(Arc::new(e))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<MapReadError<Dungeon, bool>> for KnowledgeReadError {
    fn from(e: MapReadError<Dungeon, bool>) -> KnowledgeReadError {
        KnowledgeReadError::Mq(Arc::new(e))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl fmt::Display for KnowledgeReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KnowledgeReadError::ActiveTrials(e) => write!(f, "failed to decode trials knowledge: {}", e),
            KnowledgeReadError::BoolSettings(e) => write!(f, "failed to decode settings knowledge: {}", e),
            KnowledgeReadError::DungeonRewardLocations(e) => write!(f, "failed to decode dungeon reward locations: {}", e),
            KnowledgeReadError::Exits(e) => write!(f, "failed to decode entrance knowledge: {}", e),
            KnowledgeReadError::Io(e) => write!(f, "I/O error: {}", e),
            KnowledgeReadError::Mq(e) => write!(f, "failed to decode MQ knowledge: {}", e),
            KnowledgeReadError::UnknownPreset(id) => write!(f, "unknown knowledge preset: {}", id),
        }
    }
}

#[derive(Debug, SmartDefault, Clone, PartialEq, Eq)]
pub struct Knowledge {
    pub bool_settings: HashMap<String, bool>, //TODO hardcode settings instead? (or only hardcode some settings and fall back to this for unknown settings)
    #[default(Some(Default::default()))]
    pub tricks: Option<HashMap<String, bool>>, //TODO remove option wrapping
    pub dungeon_reward_locations: HashMap<DungeonReward, DungeonRewardLocation>,
    pub mq: HashMap<Dungeon, bool>,
    #[default(Some(Default::default()))] //TODO include exits that are never shuffled
    pub exits: Option<HashMap<String, HashMap<String, String>>>, //TODO remove option wrapping
    pub active_trials: HashMap<Medallion, bool>,
}

impl Knowledge {
    /// We know that everything is vanilla. Used by auto-trackers when the base game, rather than rando, is detected.
    pub fn vanilla() -> Knowledge {
        Knowledge {
            bool_settings: collect![
                format!("open_door_of_time") => false,
                format!("triforce_hunt") => false,
                format!("all_reachable") => true,
                format!("bombchus_in_logic") => false,
                format!("one_item_per_dungeon") => false,
                format!("trials_random") => false,
                format!("skip_child_zelda") => false,
                format!("no_escape_sequence") => false,
                format!("no_guard_stealth") => false,
                format!("no_epona_race") => false,
                format!("skip_some_minigame_phases") => false,
                format!("useful_cutscenes") => true,
                format!("complete_mask_quest") => false,
                format!("fast_chests") => false,
                format!("logic_no_night_tokens_without_suns_song") => false,
                format!("free_scarecrow") => false,
                format!("fast_bunny_hood") => false,
                format!("start_with_rupees") => false,
                format!("start_with_consumables") => false,
                format!("chicken_count_random") => false,
                format!("big_poe_count_random") => false,
                format!("shuffle_kokiri_sword") => false,
                format!("shuffle_ocarinas") => false,
                format!("shuffle_weird_egg") => false,
                format!("shuffle_gerudo_card") => false,
                format!("shuffle_cows") => false,
                format!("shuffle_beans") => false,
                format!("shuffle_medigoron_carpet_salesman") => false,
                format!("shuffle_grotto_entrances") => false,
                format!("shuffle_dungeon_entrances") => false,
                format!("shuffle_overworld_entrances") => false,
                format!("decouple_entrances") => false,
                format!("owl_drops") => false,
                format!("warp_songs") => false,
                format!("spawn_positions") => false,
                format!("enhance_map_compass") => false,
                format!("mq_dungeons_random") => false,
                format!("ocarina_songs") => false,
                format!("correct_chest_sizes") => false,
                format!("no_collectible_hearts") => false,
            ],
            tricks: None, //TODO properly initialize with all tricks set to false
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
            mq: collect![
                Dungeon::Main(MainDungeon::DekuTree) => false,
                Dungeon::Main(MainDungeon::DodongosCavern) => false,
                Dungeon::Main(MainDungeon::JabuJabu) => false,
                Dungeon::Main(MainDungeon::ForestTemple) => false,
                Dungeon::Main(MainDungeon::FireTemple) => false,
                Dungeon::Main(MainDungeon::WaterTemple) => false,
                Dungeon::Main(MainDungeon::ShadowTemple) => false,
                Dungeon::Main(MainDungeon::SpiritTemple) => false,
                Dungeon::IceCavern => false,
                Dungeon::BottomOfTheWell => false,
                Dungeon::GerudoTrainingGrounds => false,
                Dungeon::GanonsCastle => false,
            ],
            exits: None, //TODO properly initialize with all exits
            active_trials: collect![
                Medallion::Light => true,
                Medallion::Forest => true,
                Medallion::Fire => true,
                Medallion::Water => true,
                Medallion::Shadow => true,
                Medallion::Spirit => true,
            ],
        }
    }

    pub fn get_exit<'a>(&'a self, from: &str, to: &'a str) -> Option<&'a str> {
        self.exits.as_ref().map_or(Some(to), |exits| exits.get(from).and_then(|region_exits| region_exits.get(to)).map(String::as_ref))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Protocol for Knowledge {
    type ReadError = KnowledgeReadError;

    fn read<'a, R: AsyncRead + 'a>(mut stream: R) -> Pin<Box<dyn Future<Output = Result<Knowledge, KnowledgeReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(match u8::read(&mut stream).await? {
                0 => Knowledge {
                    bool_settings: HashMap::read(&mut stream).await?,
                    tricks: Some(HashMap::read(&mut stream).await?),
                    dungeon_reward_locations: HashMap::read(&mut stream).await?,
                    mq: HashMap::read(&mut stream).await?,
                    exits: Some(HashMap::read(&mut stream).await?),
                    active_trials: HashMap::read(stream).await?,
                },
                1 => Knowledge::default(),
                2 => Knowledge::vanilla(),
                n => return Err(KnowledgeReadError::UnknownPreset(n)),
            })
        })
    }

    fn write<'a, W: AsyncWrite + 'a>(&'a self, mut sink: W) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            if *self == Knowledge::default() {
                1u8.write(sink).await?;
            } else if *self == Knowledge::vanilla() {
                2u8.write(sink).await?;
            } else {
                0u8.write(&mut sink).await?;
                self.bool_settings.write(&mut sink).await?;
                self.tricks.as_ref().expect("non-vanilla Knowledge should have Some in tricks field").write(&mut sink).await?;
                self.dungeon_reward_locations.write(&mut sink).await?;
                self.mq.write(&mut sink).await?;
                self.exits.as_ref().expect("non-vanilla Knowledge should have Some in exits field").write(&mut sink).await?;
                self.active_trials.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_sync<'a>(mut stream: impl Read + 'a) -> Result<Knowledge, KnowledgeReadError> {
        Ok(match u8::read_sync(&mut stream)? {
            0 => Knowledge {
                bool_settings: HashMap::read_sync(&mut stream)?,
                tricks: Some(HashMap::read_sync(&mut stream)?),
                dungeon_reward_locations: HashMap::read_sync(&mut stream)?,
                mq: HashMap::read_sync(&mut stream)?,
                exits: Some(HashMap::read_sync(&mut stream)?),
                active_trials: HashMap::read_sync(stream)?,
            },
            1 => Knowledge::default(),
            2 => Knowledge::vanilla(),
            n => return Err(KnowledgeReadError::UnknownPreset(n)),
        })
    }

    fn write_sync<'a>(&self, mut sink: impl Write + 'a) -> io::Result<()> {
        if *self == Knowledge::default() {
            1u8.write_sync(sink)?;
        } else if *self == Knowledge::vanilla() {
            2u8.write_sync(sink)?;
        } else {
            0u8.write_sync(&mut sink)?;
            self.bool_settings.write_sync(&mut sink)?;
            self.tricks.as_ref().expect("non-vanilla Knowledge should have Some in tricks field").write_sync(&mut sink)?;
            self.dungeon_reward_locations.write_sync(&mut sink)?;
            self.mq.write_sync(&mut sink)?;
            self.exits.as_ref().expect("non-vanilla Knowledge should have Some in exits field").write_sync(&mut sink)?;
            self.active_trials.write_sync(sink)?;
        }
        Ok(())
    }
}
