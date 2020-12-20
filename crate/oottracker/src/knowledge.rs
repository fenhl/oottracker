use {
    std::collections::HashMap,
    collect_mac::collect,
    smart_default::SmartDefault,
    crate::model::*,
};
#[cfg(not(target_arch = "wasm32"))] use {
    std::{
        fmt,
        io,
        sync::Arc,
    },
    async_trait::async_trait,
    derive_more::From,
    tokio::net::TcpStream,
    crate::proto::Protocol,
};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, From, Clone)]
pub enum KnowledgeReadError {
    ActiveTrials(Arc<<HashMap<Medallion, bool> as Protocol>::ReadError>),
    BoolSettings(Arc<<HashMap<String, bool> as Protocol>::ReadError>),
    DungeonRewardLocations(Arc<<HashMap<DungeonReward, DungeonRewardLocation> as Protocol>::ReadError>),
    Exits(Arc<<HashMap<String, HashMap<String, String>> as Protocol>::ReadError>),
    Io(Arc<io::Error>),
    Mq(Arc<<HashMap<Dungeon, bool> as Protocol>::ReadError>),
    UnknownPreset(u8),
}

#[cfg(not(target_arch = "wasm32"))]
impl From<<HashMap<Medallion, bool> as Protocol>::ReadError> for KnowledgeReadError {
    fn from(e: <HashMap<Medallion, bool> as Protocol>::ReadError) -> KnowledgeReadError {
        KnowledgeReadError::ActiveTrials(Arc::new(e))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<<HashMap<String, bool> as Protocol>::ReadError> for KnowledgeReadError {
    fn from(e: <HashMap<String, bool> as Protocol>::ReadError) -> KnowledgeReadError {
        KnowledgeReadError::BoolSettings(Arc::new(e))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<<HashMap<DungeonReward, DungeonRewardLocation> as Protocol>::ReadError> for KnowledgeReadError {
    fn from(e: <HashMap<DungeonReward, DungeonRewardLocation> as Protocol>::ReadError) -> KnowledgeReadError {
        KnowledgeReadError::DungeonRewardLocations(Arc::new(e))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<<HashMap<String, HashMap<String, String>> as Protocol>::ReadError> for KnowledgeReadError {
    fn from(e: <HashMap<String, HashMap<String, String>> as Protocol>::ReadError) -> KnowledgeReadError {
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
impl From<<HashMap<Dungeon, bool> as Protocol>::ReadError> for KnowledgeReadError {
    fn from(e: <HashMap<Dungeon, bool> as Protocol>::ReadError) -> KnowledgeReadError {
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
#[async_trait]
impl Protocol for Knowledge {
    type ReadError = KnowledgeReadError;

    async fn read(tcp_stream: &mut TcpStream) -> Result<Knowledge, KnowledgeReadError> {
        Ok(match u8::read(tcp_stream).await? {
            0 => Knowledge {
                bool_settings: HashMap::read(tcp_stream).await?,
                tricks: Some(HashMap::read(tcp_stream).await?),
                dungeon_reward_locations: HashMap::read(tcp_stream).await?,
                mq: HashMap::read(tcp_stream).await?,
                exits: Some(HashMap::read(tcp_stream).await?),
                active_trials: HashMap::read(tcp_stream).await?,
            },
            1 => Knowledge::default(),
            2 => Knowledge::vanilla(),
            n => return Err(KnowledgeReadError::UnknownPreset(n)),
        })
    }

    async fn write(&self, tcp_stream: &mut TcpStream) -> io::Result<()> {
        if *self == Knowledge::default() {
            1u8.write(tcp_stream).await?;
        } else if *self == Knowledge::vanilla() {
            2u8.write(tcp_stream).await?;
        } else {
            0u8.write(tcp_stream).await?;
            self.bool_settings.write(tcp_stream).await?;
            self.tricks.as_ref().expect("non-vanilla Knowledge should have Some in tricks field").write(tcp_stream).await?;
            self.dungeon_reward_locations.write(tcp_stream).await?;
            self.mq.write(tcp_stream).await?;
            self.exits.as_ref().expect("non-vanilla Knowledge should have Some in exits field").write(tcp_stream).await?;
            self.active_trials.write(tcp_stream).await?;
        }
        Ok(())
    }

    fn write_sync(&self, tcp_stream: &mut std::net::TcpStream) -> io::Result<()> {
        if *self == Knowledge::default() {
            1u8.write_sync(tcp_stream)?;
        } else if *self == Knowledge::vanilla() {
            2u8.write_sync(tcp_stream)?;
        } else {
            0u8.write_sync(tcp_stream)?;
            self.bool_settings.write_sync(tcp_stream)?;
            self.tricks.as_ref().expect("non-vanilla Knowledge should have Some in tricks field").write_sync(tcp_stream)?;
            self.dungeon_reward_locations.write_sync(tcp_stream)?;
            self.mq.write_sync(tcp_stream)?;
            self.exits.as_ref().expect("non-vanilla Knowledge should have Some in exits field").write_sync(tcp_stream)?;
            self.active_trials.write_sync(tcp_stream)?;
        }
        Ok(())
    }
}
