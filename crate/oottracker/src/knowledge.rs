use {
    std::{
        collections::{
            HashMap,
            HashSet,
        },
        fmt,
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
    collect_mac::collect,
    derivative::Derivative,
    derive_more::From,
    itertools::Itertools as _,
    serde::{
        Deserialize,
        Serialize,
    },
    serde_json::{
        Value as Json,
        json,
    },
    tokio::io::{
        AsyncRead,
        AsyncWrite,
    },
    ootr::{
        item::Item,
        model::*,
        region::Mq,
    },
};

#[derive(Derivative, Debug, Clone, Copy, PartialEq, Eq, Protocol, Deserialize, Serialize)]
#[derivative(Default)]
#[serde(rename_all = "snake_case")]
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

#[derive(Derivative, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[derivative(Default)]
#[serde(try_from = "KnowledgeJson", into = "KnowledgeJson")]
pub struct Knowledge {
    pub bool_settings: HashMap<String, bool>, //TODO hardcode settings instead? (or only hardcode some settings and fall back to this for unknown settings)
    pub string_settings: HashMap<String, HashSet<String>>, //TODO hardcode settings instead? (or only hardcode some settings and fall back to this for unknown settings)
    #[derivative(Default(value = "Some(HashMap::default())"))]
    pub tricks: Option<HashMap<String, bool>>, //TODO remove option wrapping
    pub mq: HashMap<Dungeon, Mq>,
    pub active_trials: HashMap<Medallion, bool>,
    pub dungeon_reward_locations: HashMap<DungeonReward, DungeonRewardLocation>,
    #[derivative(Default(value = "Some(HashMap::default())"))] //TODO include exits that are never shuffled
    pub exits: Option<HashMap<String, HashMap<String, String>>>, //TODO remove option wrapping
    pub progression_mode: ProgressionMode, //TODO automatically determine from remaining model state
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
            progression_mode: ProgressionMode::Go,
        }
    }

    pub fn get_exit<'a>(&'a self, from: &str, to: &'a str) -> Option<&'a str> {
        self.exits.as_ref().map_or(Some(to), |exits| exits.get(from).and_then(|region_exits| region_exits.get(to)).map(String::as_ref))
    }
}

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

impl BitAnd for Knowledge {
    type Output = Result<Knowledge, Contradiction>;

    fn bitand(self, rhs: Knowledge) -> Result<Knowledge, Contradiction> {
        let Knowledge { bool_settings, string_settings, tricks, mq, active_trials, dungeon_reward_locations, exits: _ /*TODO*/, progression_mode: _ /*TODO*/ } = self;
        Ok(Knowledge {
            bool_settings: {
                let mut bool_settings = bool_settings;
                for (name, rhs_enabled) in rhs.bool_settings {
                    if let Some(&lhs_enabled) = bool_settings.get(&name) {
                        if lhs_enabled != rhs_enabled {
                            return Err(Contradiction::BoolSetting { name, lhs_enabled })
                        }
                    } else {
                        bool_settings.insert(name, rhs_enabled);
                    }
                }
                bool_settings
            },
            string_settings: {
                let mut string_settings = string_settings;
                for (name, rhs_values) in rhs.string_settings {
                    if let Some(lhs_values) = string_settings.get(&name) {
                        let values = lhs_values & &rhs_values;
                        if values.is_empty() {
                            return Err(Contradiction::StringSetting {
                                name, rhs_values,
                                lhs_values: lhs_values.clone(),
                            })
                        }
                        string_settings.insert(name, values);
                    } else {
                        string_settings.insert(name, rhs_values);
                    }
                }
                string_settings
            },
            tricks: if let Some(mut tricks) = tricks {
                if let Some(rhs_tricks) = rhs.tricks {
                    for (name, rhs_enabled) in rhs_tricks {
                        if let Some(&lhs_enabled) = tricks.get(&name) {
                            if lhs_enabled != rhs_enabled {
                                return Err(Contradiction::Trick { name, lhs_enabled })
                            }
                        } else {
                            tricks.insert(name, rhs_enabled);
                        }
                    }
                    Some(tricks)
                } else {
                    // rhs vanilla
                    if let Some((lhs_trick, _)) = tricks.iter().find(|&(_, &enabled)| enabled) {
                        return Err(Contradiction::Trick {
                            name: lhs_trick.clone(),
                            lhs_enabled: true,
                        })
                    } else {
                        None
                    }
                }
            } else {
                // lhs vanilla
                if let Some(rhs_tricks) = rhs.tricks {
                    if let Some((rhs_trick, _)) = rhs_tricks.iter().find(|&(_, &enabled)| enabled) {
                        return Err(Contradiction::Trick {
                            name: rhs_trick.clone(),
                            lhs_enabled: false,
                        })
                    } else {
                        None
                    }
                } else {
                    // rhs vanilla
                    None
                }
            },
            mq: {
                let mut mq = mq;
                for (dungeon, rhs_mq) in rhs.mq {
                    if let Some(&lhs_mq) = mq.get(&dungeon) {
                        if lhs_mq != rhs_mq {
                            return Err(Contradiction::Mq { dungeon, lhs_mq })
                        }
                    } else {
                        mq.insert(dungeon, rhs_mq);
                    }
                }
                mq
            },
            active_trials: {
                let mut active_trials = active_trials;
                for (trial, rhs_active) in rhs.active_trials {
                    if let Some(&lhs_active) = active_trials.get(&trial) {
                        if lhs_active != rhs_active {
                            return Err(Contradiction::Trial { trial, lhs_active })
                        }
                    } else {
                        active_trials.insert(trial, rhs_active);
                    }
                }
                active_trials
            },
            dungeon_reward_locations: {
                let mut dungeon_reward_locations = dungeon_reward_locations;
                for (reward, rhs_location) in rhs.dungeon_reward_locations {
                    if let Some(&lhs_location) = dungeon_reward_locations.get(&reward) {
                        if lhs_location != rhs_location {
                            return Err(Contradiction::DungeonRewardLocation { reward, lhs_location, rhs_location })
                        }
                    } else {
                        dungeon_reward_locations.insert(reward, rhs_location);
                    }
                }
                dungeon_reward_locations
            },
            exits: None, //TODO
            progression_mode: ProgressionMode::Normal, //TODO this should actually be recalculated from the rest of the knowledge, use a dummy value for now
        })
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
                    progression_mode: ProgressionMode::read(stream).await?,
                },
                1 => Knowledge::default(),
                2 => Knowledge::vanilla(),
                n => return Err(ReadError::UnknownVariant8(n)),
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
                self.progression_mode.write(sink).await?;
            }
            Ok(())
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

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct KnowledgeJson { // knowledge in what should eventually be a superset of the plando format. TODO always use this type instead of `Knowledge`
    settings: HashMap<String, Json>,
    dungeons: HashMap<String, Mq>,
    trials: HashMap<Medallion, TrialActive>,
    entrances: HashMap<String, Vec<Entrance>>,
    locations: HashMap<String, Vec<Item>>,
    progression_mode: ProgressionMode,
}

impl From<Knowledge> for KnowledgeJson {
    fn from(knowledge: Knowledge) -> Self {
        let Knowledge { bool_settings, string_settings, tricks, mq, active_trials, dungeon_reward_locations, exits: _, progression_mode } = knowledge;
        let mut settings = bool_settings.into_iter().map(|(setting, enabled)| (setting, json!(enabled))).collect::<HashMap<_, _>>();
        settings.extend(string_settings.into_iter().map(|(setting, values)| (setting, json!(values))));
        settings.insert(format!("allowed_tricks"), json!(tricks));
        let mut locations = HashMap::<_, Vec<Item>>::new();
        for (reward, loc) in dungeon_reward_locations {
            locations.entry(loc.as_str().to_owned()).or_default().push(reward.into());
        }
        Self {
            settings, progression_mode, locations,
            dungeons: mq.into_iter().map(|(dungeon, mq)| (dungeon.rando_name().to_owned(), mq)).collect(),
            trials: active_trials.into_iter().map(|(trial, active)| (trial, active.into())).collect(),
            entrances: HashMap::default(), //TODO
        }
    }
}

#[derive(From)]
enum KnowledgeFromJsonError {
    #[from]
    Json(serde_json::Error),
    UnknownDungeon(String),
    UnknownItem(Item),
    UnknownLocation(String),
    ValueType(Json),
}

impl fmt::Display for KnowledgeFromJsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => e.fmt(f),
            Self::UnknownDungeon(name) => write!(f, "unknown dungeon: {}", name),
            Self::UnknownItem(item) => write!(f, "unknown item: {}", item.0),
            Self::UnknownLocation(name) => write!(f, "unknown location: {}", name),
            Self::ValueType(value) => write!(f, "unexpected JSON value type for value {}", value),
        }
    }
}

impl TryFrom<KnowledgeJson> for Knowledge {
    type Error = KnowledgeFromJsonError;

    fn try_from(knowledge: KnowledgeJson) -> Result<Self, KnowledgeFromJsonError> {
        let KnowledgeJson { settings, dungeons, trials, entrances: _, locations, progression_mode } = knowledge;
        let mut bool_settings = HashMap::default();
        let mut string_settings = HashMap::default();
        let mut tricks = HashMap::default();
        for (name, value) in settings {
            if name == "allowed_tricks" {
                tricks = serde_json::from_value(value)?;
                continue
            }
            match value {
                Json::Bool(enabled) => { bool_settings.insert(name, enabled); }
                Json::Array(values) => { string_settings.insert(name, values.into_iter().map(|value| serde_json::from_value(value)).try_collect()?); }
                _ => return Err(KnowledgeFromJsonError::ValueType(value)),
            }
        }
        let mut dungeon_reward_locations = HashMap::default();
        for (loc, items) in locations {
            let loc = loc.parse().map_err(|()| KnowledgeFromJsonError::UnknownLocation(loc))?;
            for item in items {
                let item = item.clone().try_into().map_err(|()| KnowledgeFromJsonError::UnknownItem(item))?;
                dungeon_reward_locations.insert(item, loc);
            }
        }
        Ok(Self {
            bool_settings, string_settings, dungeon_reward_locations, progression_mode,
            mq: dungeons.into_iter().map(|(dungeon, mq)| Ok::<_, KnowledgeFromJsonError>((dungeon.parse().map_err(|()| KnowledgeFromJsonError::UnknownDungeon(dungeon))?, mq))).try_collect()?,
            active_trials: trials.into_iter().map(|(trial, active)| (trial, active.into())).collect(),
            exits: Some(HashMap::default()), //TODO
            tricks: Some(tricks),
        })
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum TrialActive {
    Inactive,
    Active,
}

impl From<bool> for TrialActive {
    fn from(active: bool) -> Self {
        if active { Self::Active } else { Self::Inactive }
    }
}

impl From<TrialActive> for bool {
    fn from(active: TrialActive) -> Self {
        match active {
            TrialActive::Active => true,
            TrialActive::Inactive => false,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct Entrance {
    region: String,
    from: String,
}
