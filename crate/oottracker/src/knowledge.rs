use {
    std::{
        collections::{
            HashMap,
            HashSet,
        },
        future::Future,
        io::prelude::*,
        pin::Pin,
    },
    async_proto::{
        Protocol,
        ReadError,
        WriteError,
    },
    collect_mac::collect,
    smart_default::SmartDefault,
    tokio::io::{
        AsyncRead,
        AsyncWrite,
    },
    ootr::{
        model::*,
        region::Mq,
    },
};

#[derive(Debug, SmartDefault, Clone, PartialEq, Eq)]
pub struct Knowledge {
    pub bool_settings: HashMap<String, bool>, //TODO hardcode settings instead? (or only hardcode some settings and fall back to this for unknown settings)
    pub string_settings: HashMap<String, HashSet<String>>, //TODO hardcode settings instead? (or only hardcode some settings and fall back to this for unknown settings)
    #[default(Some(Default::default()))]
    pub tricks: Option<HashMap<String, bool>>, //TODO remove option wrapping
    pub mq: HashMap<Dungeon, Mq>,
    pub active_trials: HashMap<Medallion, bool>,
    pub dungeon_reward_locations: HashMap<DungeonReward, DungeonRewardLocation>,
    #[default(Some(Default::default()))] //TODO include exits that are never shuffled
    pub exits: Option<HashMap<String, HashMap<String, String>>>, //TODO remove option wrapping
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
            string_settings: collect![
                format!("open_forest") => collect![format!("closed")],
                format!("open_kakariko") => collect![format!("closed")],
                format!("zora_fountain") => collect![format!("closed")],
                format!("gerudo_fortress") => collect![format!("normal")],
                format!("bridge") => collect![format!("vanilla")],
                format!("logic_rules") => collect![format!("glitchless")],
                format!("shuffle_song_items") => collect![format!("song")],
                format!("shuffle_interior_entrances") => collect![format!("off")],
                format!("mix_entrance_pools") => collect![format!("off")],
                format!("shuffle_scrubs") => collect![format!("off")],
                format!("shopsanity") => collect![format!("off")],
                format!("tokensanity") => collect![format!("off")],
                format!("shuffle_mapcompass") => collect![format!("vanilla")],
                format!("shuffle_smallkeys") => collect![format!("vanilla")],
                format!("shuffle_fortresskeys") => collect![format!("vanilla")],
                format!("shuffle_bosskeys") => collect![format!("vanilla")],
                format!("shuffle_ganon_bosskey") => collect![format!("vanilla")],
                format!("logic_earliest_adult_trade") => collect![format!("pocket_egg")],
                format!("logic_latest_adult_trade") => collect![format!("pocket_egg")],
                format!("hints") => collect![format!("none")],
                format!("hint_dist") => collect![format!("useless")],
                format!("text_shuffle") => collect![format!("none")],
                format!("ice_trap_appearance") => collect![format!("junk_only")],
                format!("junk_ice_traps") => collect![format!("normal")],
                format!("item_pool_value") => collect![format!("balanced")],
                format!("damage_multiplier") => collect![format!("normal")],
                format!("starting_tod") => collect![format!("default")],
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
                Dungeon::Main(MainDungeon::DekuTree) => Mq::Vanilla,
                Dungeon::Main(MainDungeon::DodongosCavern) => Mq::Vanilla,
                Dungeon::Main(MainDungeon::JabuJabu) => Mq::Vanilla,
                Dungeon::Main(MainDungeon::ForestTemple) => Mq::Vanilla,
                Dungeon::Main(MainDungeon::FireTemple) => Mq::Vanilla,
                Dungeon::Main(MainDungeon::WaterTemple) => Mq::Vanilla,
                Dungeon::Main(MainDungeon::ShadowTemple) => Mq::Vanilla,
                Dungeon::Main(MainDungeon::SpiritTemple) => Mq::Vanilla,
                Dungeon::IceCavern => Mq::Vanilla,
                Dungeon::BottomOfTheWell => Mq::Vanilla,
                Dungeon::GerudoTrainingGrounds => Mq::Vanilla,
                Dungeon::GanonsCastle => Mq::Vanilla,
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

impl Protocol for Knowledge {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Knowledge, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(match u8::read(stream).await? {
                0 => Knowledge {
                    bool_settings: HashMap::read(stream).await?,
                    tricks: Some(HashMap::read(stream).await?),
                    dungeon_reward_locations: HashMap::read(stream).await?,
                    mq: HashMap::read(stream).await?,
                    exits: Some(HashMap::read(stream).await?),
                    active_trials: HashMap::read(stream).await?,
                    string_settings: HashMap::read(stream).await?,
                },
                1 => Knowledge::default(),
                2 => Knowledge::vanilla(),
                n => return Err(ReadError::UnknownVariant(n)),
            })
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            if *self == Knowledge::default() {
                1u8.write(sink).await?;
            } else if *self == Knowledge::vanilla() {
                2u8.write(sink).await?;
            } else {
                0u8.write(sink).await?;
                self.bool_settings.write(sink).await?;
                self.tricks.as_ref().expect("non-vanilla Knowledge should have Some in tricks field").write(sink).await?;
                self.dungeon_reward_locations.write(sink).await?;
                self.mq.write(sink).await?;
                self.exits.as_ref().expect("non-vanilla Knowledge should have Some in exits field").write(sink).await?;
                self.active_trials.write(sink).await?;
                self.string_settings.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Knowledge, ReadError> {
        Ok(match u8::read_sync(stream)? {
            0 => Knowledge {
                bool_settings: HashMap::read_sync(stream)?,
                tricks: Some(HashMap::read_sync(stream)?),
                dungeon_reward_locations: HashMap::read_sync(stream)?,
                mq: HashMap::read_sync(stream)?,
                exits: Some(HashMap::read_sync(stream)?),
                active_trials: HashMap::read_sync(stream)?,
                string_settings: HashMap::read_sync(stream)?,
            },
            1 => Knowledge::default(),
            2 => Knowledge::vanilla(),
            n => return Err(ReadError::UnknownVariant(n)),
        })
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        if *self == Knowledge::default() {
            1u8.write_sync(sink)?;
        } else if *self == Knowledge::vanilla() {
            2u8.write_sync(sink)?;
        } else {
            0u8.write_sync(sink)?;
            self.bool_settings.write_sync(sink)?;
            self.tricks.as_ref().expect("non-vanilla Knowledge should have Some in tricks field").write_sync(sink)?;
            self.dungeon_reward_locations.write_sync(sink)?;
            self.mq.write_sync(sink)?;
            self.exits.as_ref().expect("non-vanilla Knowledge should have Some in exits field").write_sync(sink)?;
            self.active_trials.write_sync(sink)?;
            self.string_settings.write_sync(sink)?;
        }
        Ok(())
    }
}
