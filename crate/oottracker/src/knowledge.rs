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
        ErrorContext,
        Protocol,
        ReadError,
        ReadErrorKind,
        WriteError,
    },
    collect_mac::collect,
    derivative::Derivative,
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
    crate::websocket::MwItem,
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
    pub string_settings: HashMap<String, HashSet<String>>, //TODO hardcode settings instead? (or only hardcode some settings and fall back to this for unknown settings)
    pub mq: HashMap<Dungeon, Mq>,
    pub dungeon_reward_locations: HashMap<DungeonReward, DungeonRewardLocation>,
    pub progression_mode: ProgressionMode, //TODO automatically determine from remaining model state
    pub songs_as_items: Option<bool>,
    /// Filled by the multiworld plugin, if any, with items sent from song locations in this world.
    ///
    /// Can be used for more accurate song location check tracking when present.
    pub song_locations: Option<HashSet<MwItem>>,
}

impl Knowledge {
    /// We know that everything is vanilla. Used by auto-trackers when the base game, rather than rando, is detected.
    pub fn vanilla() -> Knowledge {
        Knowledge {
            string_settings: collect![
                format!("gerudo_fortress") => collect![format!("normal")],
            ],
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
                Dungeon::GerudoTrainingGround => Mq::Vanilla,
                Dungeon::GanonsCastle => Mq::Vanilla,
            ],
            progression_mode: ProgressionMode::Go,
            songs_as_items: Some(false),
            song_locations: None,
        }
    }
}

pub enum Contradiction {
    StringSetting {
        name: String,
        lhs_values: HashSet<String>,
        rhs_values: HashSet<String>,
    },
    Mq {
        dungeon: Dungeon,
        lhs_mq: Mq,
    },
    DungeonRewardLocation {
        reward: DungeonReward,
        lhs_location: DungeonRewardLocation,
        rhs_location: DungeonRewardLocation,
    },
    SongsAsItems {
        lhs_enabled: bool,
    },
    SongLocation {
        key: u64,
        lhs_kind: u16,
        rhs_kind: u16,
    },
}

impl BitAnd for Knowledge {
    type Output = Result<Knowledge, Contradiction>;

    fn bitand(self, rhs: Knowledge) -> Result<Knowledge, Contradiction> {
        let Knowledge { string_settings, mq, dungeon_reward_locations, progression_mode: _ /*TODO*/, songs_as_items, song_locations } = self;
        Ok(Knowledge {
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
            progression_mode: ProgressionMode::Normal, //TODO this should actually be recalculated from the rest of the knowledge, use a dummy value for now
            songs_as_items: match (songs_as_items, rhs.songs_as_items) {
                (None, None) => None,
                (None, Some(value)) | (Some(value), None) => Some(value),
                (Some(false), Some(false)) => Some(false),
                (Some(true), Some(true)) => Some(true),
                (Some(lhs_enabled), Some(_)) => return Err(Contradiction::SongsAsItems { lhs_enabled }),
            },
            song_locations: if let Some(mut song_locations) = song_locations {
                if let Some(rhs_song_locations) = rhs.song_locations {
                    for MwItem { source, key, kind: rhs_kind } in rhs_song_locations {
                        if let Some(&MwItem { kind: lhs_kind, .. }) = song_locations.iter().find(|loc| loc.key == key) {
                            if lhs_kind != rhs_kind {
                                return Err(Contradiction::SongLocation { key, lhs_kind, rhs_kind })
                            }
                        } else {
                            song_locations.insert(MwItem { source, key, kind: rhs_kind });
                        }
                    }
                }
                Some(song_locations)
            } else {
                rhs.song_locations
            },
        })
    }
}

impl Protocol for Knowledge {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Knowledge, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(match u8::read(stream).await? {
                0 => Knowledge {
                    dungeon_reward_locations: HashMap::read(stream).await?,
                    mq: HashMap::read(stream).await?,
                    string_settings: HashMap::read(stream).await?,
                    progression_mode: ProgressionMode::read(stream).await?,
                    songs_as_items: Option::read(stream).await?,
                    song_locations: Option::read(stream).await?,
                },
                1 => Knowledge::default(),
                2 => Knowledge::vanilla(),
                n => return Err(ReadError {
                    context: ErrorContext::Custom(format!("oottracker::knowledge::Knowledge::read_sync")),
                    kind: ReadErrorKind::UnknownVariant8(n),
                }),
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
                self.dungeon_reward_locations.write(sink).await?;
                self.mq.write(sink).await?;
                self.string_settings.write(sink).await?;
                self.progression_mode.write(sink).await?;
                self.songs_as_items.write(sink).await?;
                self.song_locations.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(match u8::read_sync(stream)? {
            0 => Knowledge {
                dungeon_reward_locations: HashMap::read_sync(stream)?,
                mq: HashMap::read_sync(stream)?,
                string_settings: HashMap::read_sync(stream)?,
                progression_mode: ProgressionMode::read_sync(stream)?,
                songs_as_items: Option::read_sync(stream)?,
                song_locations: Option::read_sync(stream)?,
            },
            1 => Knowledge::default(),
            2 => Knowledge::vanilla(),
            n => return Err(ReadError {
                context: ErrorContext::Custom(format!("oottracker::knowledge::Knowledge::read_sync")),
                kind: ReadErrorKind::UnknownVariant8(n),
            })
        })
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        if *self == Knowledge::default() {
            1u8.write_sync(sink)?;
        } else if *self == Knowledge::vanilla() {
            2u8.write_sync(sink)?;
        } else {
            0u8.write_sync(sink)?;
            self.dungeon_reward_locations.write_sync(sink)?;
            self.mq.write_sync(sink)?;
            self.string_settings.write_sync(sink)?;
            self.songs_as_items.write_sync(sink)?;
            self.song_locations.write_sync(sink)?;
        }
        Ok(())
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct KnowledgeJson { // knowledge in what should eventually be a superset of the plando format. TODO always use this type instead of `Knowledge`
    settings: HashMap<String, Json>,
    dungeons: HashMap<String, Mq>,
    entrances: HashMap<String, Vec<Entrance>>,
    locations: HashMap<String, Vec<Item>>,
    progression_mode: ProgressionMode,
}

impl From<Knowledge> for KnowledgeJson {
    fn from(knowledge: Knowledge) -> Self {
        let Knowledge { string_settings, mq, dungeon_reward_locations, progression_mode, songs_as_items: _, song_locations: _ } = knowledge;
        let mut settings = HashMap::default();
        settings.extend(string_settings.into_iter().map(|(setting, values)| (setting, json!(values))));
        let mut locations = HashMap::<_, Vec<Item>>::new();
        for (reward, loc) in dungeon_reward_locations {
            locations.entry(loc.as_str().to_owned()).or_default().push(reward.into());
        }
        Self {
            settings, progression_mode, locations,
            dungeons: mq.into_iter().map(|(dungeon, mq)| (dungeon.rando_name().to_owned(), mq)).collect(),
            entrances: HashMap::default(), //TODO
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum KnowledgeFromJsonError {
    #[error(transparent)] Json(#[from] serde_json::Error),
    #[error("unknown dungeon: {0}")]
    UnknownDungeon(String),
    #[error("unknown item: {}", .0.0)]
    UnknownItem(Item),
    #[error("unknown location: {0}")]
    UnknownLocation(String),
    #[error("unexpected JSON value type for value {0}")]
    ValueType(Json),
}

impl TryFrom<KnowledgeJson> for Knowledge {
    type Error = KnowledgeFromJsonError;

    fn try_from(knowledge: KnowledgeJson) -> Result<Self, KnowledgeFromJsonError> {
        let KnowledgeJson { settings, dungeons, entrances: _, locations, progression_mode } = knowledge;
        let mut string_settings = HashMap::default();
        for (name, value) in settings {
            match value {
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
            string_settings, dungeon_reward_locations, progression_mode,
            mq: dungeons.into_iter().map(|(dungeon, mq)| Ok::<_, KnowledgeFromJsonError>((dungeon.parse().map_err(|()| KnowledgeFromJsonError::UnknownDungeon(dungeon))?, mq))).try_collect()?,
            songs_as_items: None,
            song_locations: None,
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

#[test]
fn test_knowledge_protocol_roundtrip() {
    let mut buf = Vec::default();
    Knowledge::default().write_sync(&mut buf).unwrap();
    assert_eq!(Knowledge::read_sync(&mut &*buf).unwrap(), Knowledge::default());
}
