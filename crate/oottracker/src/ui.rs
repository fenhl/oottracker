#![allow(unused_qualifications)] // oottracker::ui::TrackerCellKind::SmallKeys vs oottracker::save::SmallKeys

use {
    std::{
        borrow::Cow,
        convert::TryInto as _,
        fmt,
        io,
        sync::Arc,
        vec,
    },
    async_proto::Protocol,
    derivative::Derivative,
    directories::ProjectDirs,
    enum_iterator::IntoEnumIterator,
    horrorshow::{
        html,
        prelude::*,
    },
    iced::keyboard::Modifiers as KeyboardModifiers,
    image::DynamicImage,
    itertools::Itertools as _,
    rocket::{
        http::uri::fmt::{
            Formatter,
            Path,
            UriDisplay,
        },
        request::FromParam,
    },
    serde::{
        Deserialize,
        Serialize,
    },
    tokio::{
        fs::{
            self,
            File,
        },
        io::{
            AsyncReadExt as _,
            AsyncWriteExt as _,
        },
    },
    wheel::FromArc,
    crate::{
        ModelState,
        check::Check,
        checks::CheckExt as _,
        info_tables::*,
        knowledge::ProgressionMode,
        model::{
            Dungeon,
            DungeonReward,
            DungeonRewardLocation,
            MainDungeon,
            Medallion,
            Stone,
        },
        region::Mq,
        save::*,
        settings::GerudoFortressKnowledge,
    },
};

const VERSION: u8 = 0;

#[derive(Debug, FromArc, Clone)]
pub enum Error {
    #[from_arc]
    Io(Arc<io::Error>),
    #[from_arc]
    Json(Arc<serde_json::Error>),
    MissingHomeDir,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Json(e) => e.fmt(f),
            Error::MissingHomeDir => write!(f, "could not find your user folder"),
        }
    }
}

#[derive(Derivative, Debug, Clone, Deserialize, Serialize)]
#[derivative(Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[derivative(Default(value = "ElementOrder::LightShadowSpirit"))]
    #[serde(default = "default_med_order")]
    pub med_order: ElementOrder,
    #[derivative(Default(value = "ElementOrder::SpiritShadowLight"))]
    #[serde(default = "default_warp_song_order")]
    pub warp_song_order: ElementOrder,
    pub auto_update_check: Option<bool>,
    #[derivative(Default(value = "VERSION"))]
    pub version: u8,
}

impl Config {
    /// If the config file doesn't exist, this returns `Ok(None)`, so that the welcome message can be displayed.
    pub async fn new() -> Result<Option<Config>, Error> {
        let dirs = dirs()?;
        let mut file = match File::open(dirs.config_dir().join("config.json")).await {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let mut buf = String::default();
        file.read_to_string(&mut buf).await?;
        Ok(Some(serde_json::from_str(&buf)?)) //TODO use async-json instead
    }

    pub async fn save(&self) -> Result<(), Error> {
        let dirs = dirs()?;
        let buf = serde_json::to_vec(self)?; //TODO use async-json instead
        fs::create_dir_all(dirs.config_dir()).await?;
        let mut file = File::create(dirs.config_dir().join("config.json")).await?;
        file.write_all(&buf).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoEnumIterator, Deserialize, Serialize, Protocol)]
#[serde(rename_all = "camelCase")]
pub enum ElementOrder {
    LightShadowSpirit,
    LightSpiritShadow,
    ShadowSpiritLight,
    SpiritShadowLight,
}

impl IntoIterator for ElementOrder {
    type IntoIter = vec::IntoIter<Medallion>;
    type Item = Medallion;

    fn into_iter(self) -> vec::IntoIter<Medallion> {
        use Medallion::*;

        match self {
            ElementOrder::LightShadowSpirit => vec![Light, Forest, Fire, Water, Shadow, Spirit],
            ElementOrder::LightSpiritShadow => vec![Light, Forest, Fire, Water, Spirit, Shadow],
            ElementOrder::ShadowSpiritLight => vec![Forest, Fire, Water, Shadow, Spirit, Light],
            ElementOrder::SpiritShadowLight => vec![Forest, Fire, Water, Spirit, Shadow, Light],
        }.into_iter()
    }
}

impl fmt::Display for ElementOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElementOrder::LightShadowSpirit => write!(f, "Light first, Shadow before Spirit"),
            ElementOrder::LightSpiritShadow => write!(f, "Light first, Spirit before Shadow"),
            ElementOrder::ShadowSpiritLight => write!(f, "Shadow before Spirit, Light last"),
            ElementOrder::SpiritShadowLight => write!(f, "Spirit before Shadow, Light last"),
        }
    }
}

pub enum TrackerCellKind {
    BigPoeTriforce, // auto-trackers show big Poe count unless at least 1 Triforce piece has been collected, manual mode only shows Triforce pieces
    BossKey {
        active: Box<dyn Fn(&DungeonItems) -> bool>,
        toggle: Box<dyn Fn(&mut DungeonItems)>,
    },
    Composite {
        left_img: ImageInfo,
        right_img: ImageInfo,
        both_img: ImageInfo,
        active: Box<dyn Fn(&ModelState) -> (bool, bool)>,
        toggle_left: Box<dyn Fn(&mut ModelState)>,
        toggle_right: Box<dyn Fn(&mut ModelState)>,
    },
    CompositeKeys {
        small: TrackerCellId,
        boss: TrackerCellId,
    },
    Count {
        dimmed_img: ImageInfo,
        img: ImageInfo,
        get: Box<dyn Fn(&ModelState) -> u8>,
        set: Box<dyn Fn(&mut ModelState, u8)>,
        max: u8,
        step: u8,
    },
    FortressMq, // a cell kind used on Xopar's tracker to show whether Gerudo Fortress has 4 carpenters
    FreeReward,
    GoBk, // a combined go mode/BK mode/finished cell, used on the multiworld restream layout
    MagicLens, // magic meter with a Lens of Truth overlay, but auto-trackers/shift-click also show a different icon for double magic
    Medallion(Medallion),
    MedallionLocation(Medallion),
    Mq(Dungeon),
    OptionalOverlay {
        main_img: ImageInfo,
        overlay_img: ImageInfo,
        active: Box<dyn Fn(&ModelState) -> (bool, bool)>,
        toggle_main: Box<dyn Fn(&mut ModelState)>,
        toggle_overlay: Box<dyn Fn(&mut ModelState)>,
    },
    Overlay {
        main_img: ImageInfo,
        overlay_img: ImageInfo,
        active: Box<dyn Fn(&ModelState) -> (bool, bool)>,
        toggle_main: Box<dyn Fn(&mut ModelState)>,
        toggle_overlay: Box<dyn Fn(&mut ModelState)>,
    },
    Sequence {
        idx: Box<dyn Fn(&ModelState) -> u8>,
        img: Box<dyn Fn(&ModelState) -> (bool, ImageInfo)>,
        increment: Box<dyn Fn(&mut ModelState)>,
        decrement: Box<dyn Fn(&mut ModelState)>,
    },
    Simple {
        img: ImageInfo,
        active: Box<dyn Fn(&ModelState) -> bool>,
        toggle: Box<dyn Fn(&mut ModelState)>,
    },
    SmallKeys {
        get: Box<dyn Fn(&crate::save::SmallKeys) -> u8>,
        set: Box<dyn Fn(&mut crate::save::SmallKeys, u8)>,
        max_vanilla: u8,
        max_mq: u8,
    },
    Song {
        song: QuestItems,
        check: &'static str,
        toggle_overlay: Box<dyn Fn(&mut EventChkInf)>,
    },
    SongCheck {
        check: &'static str,
        toggle_overlay: Box<dyn Fn(&mut EventChkInf)>,
    },
    Spells, // composite Din's Fire & Farore's Wind, but auto-trackers/shift-click also toggle Nayru's Love
    Stone(Stone),
    StoneLocation(Stone),
}

impl TrackerCellKind {
    pub fn render(&self, state: &ModelState) -> CellRender {
        match self {
            BigPoeTriforce => if state.ram.save.triforce_pieces() > 0 {
                CellRender {
                    img: ImageInfo::new("triforce"),
                    style: CellStyle::Normal,
                    overlay: CellOverlay::Count {
                        count: state.ram.save.triforce_pieces(),
                        count_img: ImageInfo::new("force"),
                    },
                }
            } else if state.ram.save.big_poes > 0 { //TODO show dimmed Triforce icon if it's known that it's TH
                CellRender {
                    img: ImageInfo::extra("big_poe"),
                    style: CellStyle::Normal,
                    overlay: CellOverlay::Count {
                        count: state.ram.save.big_poes,
                        count_img: ImageInfo::extra("poes"),
                    },
                }
            } else {
                CellRender {
                    img: ImageInfo::extra("big_poe"),
                    style: CellStyle::Dimmed,
                    overlay: CellOverlay::None,
                }
            },
            BossKey { active, .. } => CellRender {
                img: ImageInfo::extra("boss_key"),
                style: if active(&state.ram.save.dungeon_items) { CellStyle::Normal } else { CellStyle::Dimmed },
                overlay: CellOverlay::None,
            },
            Composite { left_img, right_img, both_img, active, .. } => {
                let is_active = active(state);
                let img = match is_active {
                    (false, false) | (true, true) => both_img,
                    (false, true) => right_img,
                    (true, false) => left_img,
                }.clone();
                CellRender {
                    img,
                    style: if let (false, false) = is_active { CellStyle::Dimmed } else { CellStyle::Normal },
                    overlay: CellOverlay::None,
                }
            }
            CompositeKeys { boss, small } => {
                let (has_boss_key, num_small_keys) = if let (BossKey { active, .. }, TrackerCellKind::SmallKeys { get, .. }) = (boss.kind(), small.kind()) {
                    (active(&state.ram.save.dungeon_items), get(&state.ram.save.small_keys))
                } else {
                    unimplemented!("CompositeKeys that aren't SmallKeys + BossKey")
                };
                CellRender {
                    img: ImageInfo::extra("keys"),
                    style: match (has_boss_key, num_small_keys) {
                        (false, 0) => CellStyle::Dimmed,
                        (false, _) => CellStyle::LeftDimmed,
                        (true, 0) => CellStyle::RightDimmed,
                        (true, _) => CellStyle::Normal,
                    },
                    overlay: if num_small_keys > 0 {
                        CellOverlay::Count {
                            count: num_small_keys,
                            count_img: ImageInfo::new("UNIMPLEMENTED"), //TODO
                        }
                    } else {
                        CellOverlay::None
                    },
                }
            }
            Count { dimmed_img, img, get, .. } => {
                let count = get(state);
                let (style, overlay) = if count == 0 {
                    (CellStyle::Dimmed, CellOverlay::None)
                } else {
                    (CellStyle::Normal, CellOverlay::Count { count, count_img: img.clone() })
                };
                CellRender { img: dimmed_img.clone(), style, overlay }
            }
            FortressMq => {
                CellRender {
                    img: ImageInfo::extra("blank"),
                    style: CellStyle::Normal,
                    overlay: CellOverlay::Location {
                        loc: ImageInfo::extra("fort_text"),
                        style: if state.knowledge.settings.gerudo_fortress == GerudoFortressKnowledge::normal() { LocationStyle::Mq } else { LocationStyle::Normal }, //TODO dim if unknown?
                    },
                }
            }
            FreeReward => {
                //TODO if auto-tracking, use a method on Knowledge to display dungeon rewards that are known under functional equivalence
                let reward = state.knowledge.locations.get("Links Pocket").and_then(|k| k
                    .into_iter()
                    .filter_map(|item| item.try_into().ok())
                    .exactly_one()
                    .ok()
                );
                CellRender {
                    img: ImageInfo { dir: if reward.is_some() { ImageDir::Xopar } else { ImageDir::Extra }, name: match reward {
                        Some(DungeonReward::Medallion(med)) => Cow::Owned(format!("{}_medallion", med.element().to_ascii_lowercase())),
                        Some(DungeonReward::Stone(Stone::KokiriEmerald)) => Cow::Borrowed("kokiri_emerald"),
                        Some(DungeonReward::Stone(Stone::GoronRuby)) => Cow::Borrowed("goron_ruby"),
                        Some(DungeonReward::Stone(Stone::ZoraSapphire)) => Cow::Borrowed("zora_sapphire"),
                        None => Cow::Borrowed("blank"), //TODO “unknown dungeon reward” image?
                    } },
                    style: CellStyle::Normal,
                    overlay: CellOverlay::Location {
                        loc: ImageInfo::new("free_text"),
                        style: LocationStyle::Normal,
                    },
                }
            }
            GoBk => CellRender {
                img: ImageInfo::extra(match state.knowledge.progression_mode {
                    ProgressionMode::Done => "blank",
                    ProgressionMode::Bk => "bk_mode",
                    ProgressionMode::Go | ProgressionMode::Normal => "go_mode",
                }),
                style: if state.knowledge.progression_mode == ProgressionMode::Normal { CellStyle::Dimmed } else { CellStyle::Normal },
                overlay: CellOverlay::None, //TODO overlay with finish time?
            },
            MagicLens => CellRender {
                img: if state.ram.save.magic == MagicCapacity::Large { ImageInfo::new("magic") } else { ImageInfo::extra("small_magic") },
                style: if state.ram.save.magic == MagicCapacity::None { CellStyle::Dimmed } else { CellStyle::Normal },
                overlay: if state.ram.save.inv.lens {
                    CellOverlay::Image(ImageInfo::new("lens"))
                } else {
                    CellOverlay::None
                },
            },
            Medallion(med) => CellRender {
                img: ImageInfo::new(format!("{}_medallion", med.element().to_ascii_lowercase())),
                style: if state.ram.save.quest_items.has(*med) { CellStyle::Normal } else { CellStyle::Dimmed },
                overlay: CellOverlay::None,
            },
            MedallionLocation(med) => {
                let location = state.knowledge.get_dungeon_reward_location(DungeonReward::Medallion(*med));
                CellRender {
                    img: ImageInfo::new(match location {
                        None => "unknown_text",
                        Some(DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)) => "deku_text",
                        Some(DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)) => "dc_text",
                        Some(DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)) => "jabu_text",
                        Some(DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)) => "forest_text",
                        Some(DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)) => "fire_text",
                        Some(DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)) => "water_text",
                        Some(DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)) => "shadow_text",
                        Some(DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)) => "spirit_text",
                        Some(DungeonRewardLocation::LinksPocket) => "free_text",
                    }),
                    style: if location.is_some() { CellStyle::Normal } else { CellStyle::Dimmed },
                    overlay: CellOverlay::None,
                }
            }
            Mq(dungeon) => {
                let reward = if let Dungeon::Main(main_dungeon) = *dungeon {
                    //TODO if auto-tracking, use a method on Knowledge to display dungeon rewards that are known under functional equivalence
                    state.knowledge.locations.get(main_dungeon.reward_location()).and_then(|k| k
                        .into_iter()
                        .filter_map(|item| item.try_into().ok())
                        .exactly_one()
                        .ok()
                    )
                } else {
                    None
                };
                CellRender {
                    img: ImageInfo { dir: if reward.is_some() { ImageDir::Xopar } else { ImageDir::Extra }, name: match reward {
                        Some(DungeonReward::Medallion(med)) => Cow::Owned(format!("{}_medallion", med.element().to_ascii_lowercase())),
                        Some(DungeonReward::Stone(Stone::KokiriEmerald)) => Cow::Borrowed("kokiri_emerald"),
                        Some(DungeonReward::Stone(Stone::GoronRuby)) => Cow::Borrowed("goron_ruby"),
                        Some(DungeonReward::Stone(Stone::ZoraSapphire)) => Cow::Borrowed("zora_sapphire"),
                        None => Cow::Borrowed("blank"), //TODO “unknown dungeon reward” image? (only for dungeons that have rewards)
                    } },
                    style: if reward.map_or(false, |reward| state.ram.save.quest_items.has(reward)) { CellStyle::Normal } else { CellStyle::Dimmed },
                    overlay: CellOverlay::Location {
                        loc: ImageInfo { dir: if let Dungeon::Main(_) = dungeon { ImageDir::Xopar } else { ImageDir::Extra }, name: Cow::Borrowed(match dungeon {
                            Dungeon::Main(MainDungeon::DekuTree) => "deku_text",
                            Dungeon::Main(MainDungeon::DodongosCavern) => "dc_text",
                            Dungeon::Main(MainDungeon::JabuJabu) => "jabu_text",
                            Dungeon::Main(MainDungeon::ForestTemple) => "forest_text",
                            Dungeon::Main(MainDungeon::FireTemple) => "fire_text",
                            Dungeon::Main(MainDungeon::WaterTemple) => "water_text",
                            Dungeon::Main(MainDungeon::ShadowTemple) => "shadow_text",
                            Dungeon::Main(MainDungeon::SpiritTemple) => "spirit_text",
                            Dungeon::IceCavern => "ice_text",
                            Dungeon::BottomOfTheWell => "well_text",
                            Dungeon::GerudoTrainingGround => "gtg_text",
                            Dungeon::GanonsCastle => "ganon_text",
                        }) },
                        style: if state.knowledge.dungeons.get(dungeon) == Some(&Mq::Mq) { LocationStyle::Mq } else { LocationStyle::Normal },
                    },
                }
            }
            OptionalOverlay { main_img, overlay_img, active, .. } | Overlay { main_img, overlay_img, active, .. } => {
                let (main_active, overlay_active) = active(state);
                CellRender {
                    img: main_img.clone(),
                    style: if main_active { CellStyle::Normal } else { CellStyle::Dimmed },
                    overlay: if overlay_active {
                        CellOverlay::Image(overlay_img.clone())
                    } else {
                        CellOverlay::None
                    },
                }
            }
            Sequence { img, .. } => {
                let (is_active, img) = img(state);
                CellRender {
                    img,
                    style: if is_active { CellStyle::Normal } else { CellStyle::Dimmed },
                    overlay: CellOverlay::None,
                }
            }
            Simple { img, active, .. } => CellRender {
                img: img.clone(),
                style: if active(state) { CellStyle::Normal } else { CellStyle::Dimmed },
                overlay: CellOverlay::None,
            },
            TrackerCellKind::SmallKeys { get, .. } => {
                let num_small_keys = get(&state.ram.save.small_keys);
                CellRender {
                    img: ImageInfo::extra("small-key"),
                    style: if num_small_keys > 0 { CellStyle::Normal } else { CellStyle::Dimmed },
                    overlay: if num_small_keys > 0 {
                        CellOverlay::Count {
                            count: num_small_keys,
                            count_img: ImageInfo::new("UNIMPLEMENTED"), //TODO
                        }
                    } else {
                        CellOverlay::None
                    },
                }
            },
            Song { song, check, .. } => CellRender {
                img: ImageInfo::new(match *song {
                    QuestItems::ZELDAS_LULLABY => "lullaby",
                    QuestItems::EPONAS_SONG => "epona",
                    QuestItems::SARIAS_SONG => "saria",
                    QuestItems::SUNS_SONG => "sun",
                    QuestItems::SONG_OF_TIME => "time",
                    QuestItems::SONG_OF_STORMS => "storms",
                    QuestItems::MINUET_OF_FOREST => "minuet",
                    QuestItems::BOLERO_OF_FIRE => "bolero",
                    QuestItems::SERENADE_OF_WATER => "serenade",
                    QuestItems::NOCTURNE_OF_SHADOW => "nocturne",
                    QuestItems::REQUIEM_OF_SPIRIT => "requiem",
                    QuestItems::PRELUDE_OF_LIGHT => "prelude",
                    _ => unreachable!(),
                }),
                style: if state.ram.save.quest_items.contains(*song) { CellStyle::Normal } else { CellStyle::Dimmed },
                overlay: if Check::Location(check.to_string()).checked(state).unwrap_or(false) {
                    CellOverlay::Image(ImageInfo::new("check"))
                } else {
                    CellOverlay::None
                },
            },
            SongCheck { check, .. } => CellRender {
                img: ImageInfo::extra("blank"),
                style: CellStyle::Normal,
                overlay: if Check::Location(check.to_string()).checked(state).unwrap_or(false) {
                    CellOverlay::Image(ImageInfo::new("check"))
                } else {
                    CellOverlay::None
                },
            },
            Spells => CellRender {
                img: match (state.ram.save.inv.dins_fire, state.ram.save.inv.farores_wind, state.ram.save.inv.nayrus_love) {
                    (false, false, false) | (true, true, false) => ImageInfo::new("composite_magic"), //TODO use "spells" for dimmed instead if shift-click is available or auto-tracking?
                    (false, false, true) => ImageInfo::extra("nayrus_love"),
                    (false, true, false) => ImageInfo::new("faores_wind"),
                    (false, true, true) => ImageInfo::extra("farores_nayrus"),
                    (true, false, false) => ImageInfo::new("dins_fire"),
                    (true, false, true) => ImageInfo::extra("dins_nayrus"),
                    (true, true, true) => ImageInfo::extra("spells"),
                },
                style: if !state.ram.save.inv.dins_fire && !state.ram.save.inv.farores_wind && !state.ram.save.inv.nayrus_love { CellStyle::Dimmed } else { CellStyle::Normal },
                overlay: CellOverlay::None,
            },
            Stone(stone) => CellRender {
                img: ImageInfo::new(match *stone {
                    Stone::KokiriEmerald => "kokiri_emerald",
                    Stone::GoronRuby => "goron_ruby",
                    Stone::ZoraSapphire => "zora_sapphire",
                }),
                style: if state.ram.save.quest_items.has(*stone) { CellStyle::Normal } else { CellStyle::Dimmed },
                overlay: CellOverlay::None,
            },
            StoneLocation(stone) => {
                let location = state.knowledge.get_dungeon_reward_location(DungeonReward::Stone(*stone));
                CellRender {
                    img: ImageInfo::new(match location {
                        None => "unknown_text",
                        Some(DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)) => "deku_text",
                        Some(DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)) => "dc_text",
                        Some(DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)) => "jabu_text",
                        Some(DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)) => "forest_text",
                        Some(DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)) => "fire_text",
                        Some(DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)) => "water_text",
                        Some(DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)) => "shadow_text",
                        Some(DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)) => "spirit_text",
                        Some(DungeonRewardLocation::LinksPocket) => "free_text",
                    }),
                    style: if location.is_some() { CellStyle::Normal } else { CellStyle::Dimmed },
                    overlay: CellOverlay::None,
                }
            },
        }
    }

    /// Handle a click action from a frontend that don't distinguish between left and right click.
    pub fn click(&self, state: &mut ModelState) {
        match self {
            Composite { active, toggle_left, toggle_right, .. } | Overlay { active, toggle_main: toggle_left, toggle_overlay: toggle_right, .. } => {
                let (left, _) = active(state);
                if left { toggle_right(state) }
                toggle_left(state);
            }
            OptionalOverlay { toggle_main: toggle, .. } | Simple { toggle, .. } => toggle(state),
            CompositeKeys { boss, small } => {
                let (toggle_boss, get_small, set_small, max_small_vanilla, max_small_mq) = if let (BossKey { toggle, .. }, TrackerCellKind::SmallKeys { get, set, max_vanilla, max_mq }) = (boss.kind(), small.kind()) {
                    (toggle, get, set, max_vanilla, max_mq)
                } else {
                    unimplemented!("CompositeKeys that aren't SmallKeys + BossKey")
                };
                let num_small = get_small(&state.ram.save.small_keys);
                if num_small == max_small_vanilla.max(max_small_mq) { //TODO check MQ knowledge? Does plentiful go to +1?
                    set_small(&mut state.ram.save.small_keys, 0);
                    toggle_boss(&mut state.ram.save.dungeon_items);
                } else {
                    set_small(&mut state.ram.save.small_keys, num_small + 1);
                }
            }
            Count { get, set, max, step, .. } => {
                let current = get(state);
                set(state, if current == *max { 0 } else { current.saturating_add(*step).min(*max) });
            }
            FortressMq => if state.knowledge.settings.gerudo_fortress == GerudoFortressKnowledge::normal() {
                state.knowledge.settings.gerudo_fortress = GerudoFortressKnowledge::default();
            } else {
                state.knowledge.settings.gerudo_fortress = GerudoFortressKnowledge::normal();
            },
            GoBk => state.knowledge.progression_mode = match state.knowledge.progression_mode {
                ProgressionMode::Normal => ProgressionMode::Go,
                ProgressionMode::Go => ProgressionMode::Bk,
                ProgressionMode::Bk => ProgressionMode::Done,
                ProgressionMode::Done => ProgressionMode::Normal,
            },
            MagicLens => {
                if state.ram.save.magic == MagicCapacity::None {
                    state.ram.save.magic = MagicCapacity::Small;
                } else {
                    state.ram.save.magic = MagicCapacity::None;
                    state.ram.save.inv.lens = !state.ram.save.inv.lens;
                }
            }
            Medallion(med) => state.ram.save.quest_items.toggle(QuestItems::from(med)),
            MedallionLocation(med) => state.knowledge.increment_dungeon_reward_location(DungeonReward::Medallion(*med)),
            Mq(dungeon) => if state.knowledge.dungeons.get(dungeon) == Some(&Mq::Mq) {
                state.knowledge.dungeons.remove(dungeon);
            } else {
                state.knowledge.dungeons.insert(*dungeon, Mq::Mq);
            },
            Sequence { increment, .. } => increment(state),
            TrackerCellKind::SmallKeys { get, set, max_vanilla, max_mq } => {
                let num = get(&state.ram.save.small_keys);
                if num == *max_vanilla.max(max_mq) { //TODO check MQ knowledge? Does plentiful go to +1?
                    set(&mut state.ram.save.small_keys, 0);
                } else {
                    set(&mut state.ram.save.small_keys, num + 1);
                }
            }
            Song { song: quest_item, .. } => state.ram.save.quest_items.toggle(*quest_item),
            Spells => {
                if state.ram.save.inv.dins_fire { state.ram.save.inv.farores_wind = !state.ram.save.inv.farores_wind }
                state.ram.save.inv.dins_fire = !state.ram.save.inv.dins_fire;
            }
            Stone(stone) => state.ram.save.quest_items.toggle(QuestItems::from(stone)),
            StoneLocation(stone) => state.knowledge.increment_dungeon_reward_location(DungeonReward::Stone(*stone)),
            FreeReward => {}
            BigPoeTriforce | BossKey { .. } | SongCheck { .. } => unimplemented!(),
        }
    }

    /// Returns `true` if the menu should be opened.
    #[must_use] pub fn left_click(&self, can_change_state: bool, keyboard_modifiers: KeyboardModifiers, state: &mut ModelState) -> bool {
        #[cfg(target_os = "macos")] if keyboard_modifiers.control {
            return self.right_click(can_change_state, keyboard_modifiers, state)
        }
        if can_change_state {
            match self {
                Composite { toggle_left, .. } | Overlay { toggle_main: toggle_left, .. } => toggle_left(state),
                CompositeKeys { boss, .. } => if let BossKey { toggle, .. } = boss.kind() {
                    toggle(&mut state.ram.save.dungeon_items);
                } else {
                    unimplemented!("CompositeKeys that aren't SmallKeys + BossKey")
                },
                Count { get, set, max, step, .. } => {
                    let current = get(state);
                    set(state, if current == *max { 0 } else { current.saturating_add(step * if keyboard_modifiers.shift && *max >= 10 { 10 } else { 1 }).min(*max) });
                }
                GoBk => state.knowledge.progression_mode = match state.knowledge.progression_mode {
                    ProgressionMode::Normal => ProgressionMode::Go,
                    ProgressionMode::Go => ProgressionMode::Normal,
                    ProgressionMode::Bk => ProgressionMode::Done,
                    ProgressionMode::Done => ProgressionMode::Bk,
                },
                MagicLens => state.ram.save.magic = match (keyboard_modifiers.shift, state.ram.save.magic) {
                    (true, MagicCapacity::Large) => MagicCapacity::Small,
                    (true, _) => MagicCapacity::Large,
                    (false, MagicCapacity::None) => MagicCapacity::Small,
                    (false, _) => MagicCapacity::None,
                },
                Spells => if keyboard_modifiers.shift {
                    state.ram.save.inv.nayrus_love = !state.ram.save.inv.nayrus_love;
                } else {
                    state.ram.save.inv.dins_fire = !state.ram.save.inv.dins_fire;
                },
                _ => self.click(state),
            }
        }
        false
    }

    /// Returns `true` if the menu should be opened.
    #[must_use] pub fn right_click(&self, can_change_state: bool, keyboard_modifiers: KeyboardModifiers, state: &mut ModelState) -> bool {
        if let Medallion(_) = self { return true }
        if can_change_state {
            match self {
                Composite { toggle_right, .. } | OptionalOverlay { toggle_overlay: toggle_right, .. } | Overlay { toggle_overlay: toggle_right, .. } => toggle_right(state),
                CompositeKeys { small, .. } => if let TrackerCellKind::SmallKeys { get, set, max_vanilla, max_mq } = small.kind() {
                    let num = get(&state.ram.save.small_keys);
                    if num == max_vanilla.max(max_mq) { //TODO check MQ knowledge? Does plentiful go to +1?
                        set(&mut state.ram.save.small_keys, 0);
                    } else {
                        set(&mut state.ram.save.small_keys, num + 1);
                    }
                } else {
                    unimplemented!("CompositeKeys that aren't SmallKeys + BossKey")
                },
                Count { get, set, max, step, .. } => {
                    let current = get(state);
                    set(state, if current == 0 { *max } else { current.saturating_sub(step * if keyboard_modifiers.shift && *max >= 10 { 10 } else { 1 }) });
                }
                GoBk => state.knowledge.progression_mode = match state.knowledge.progression_mode {
                    ProgressionMode::Normal => ProgressionMode::Bk,
                    ProgressionMode::Bk => ProgressionMode::Normal,
                    ProgressionMode::Go => ProgressionMode::Done,
                    ProgressionMode::Done => ProgressionMode::Go,
                },
                MagicLens => state.ram.save.inv.lens = !state.ram.save.inv.lens,
                Medallion(_) => unreachable!("already handled above"),
                MedallionLocation(med) => state.knowledge.decrement_dungeon_reward_location(DungeonReward::Medallion(*med)),
                Sequence { decrement, .. } => decrement(state),
                TrackerCellKind::SmallKeys { get, set, max_vanilla, max_mq } => {
                    let num = get(&state.ram.save.small_keys);
                    if num == 0 {
                        set(&mut state.ram.save.small_keys, *max_vanilla.max(max_mq)); //TODO check MQ knowledge? Does plentiful go to +1?
                    } else {
                        set(&mut state.ram.save.small_keys, num - 1);
                    }
                }
                Song { toggle_overlay, .. } => toggle_overlay(&mut state.ram.save.event_chk_inf),
                Spells => state.ram.save.inv.farores_wind = !state.ram.save.inv.farores_wind,
                StoneLocation(stone) => state.knowledge.decrement_dungeon_reward_location(DungeonReward::Stone(*stone)),
                FreeReward | FortressMq | Mq(_) | Simple { .. } | Stone(_) => {}
                BigPoeTriforce | BossKey { .. } | SongCheck { .. } => unimplemented!(),
            }
        }
        false
    }
}

use TrackerCellKind::*;

macro_rules! cells {
    ($($cell:ident: $kind:expr,)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Protocol)]
        pub enum TrackerCellId {
            $(
                $cell,
            )*
        }

        impl TrackerCellId {
            pub fn kind(&self) -> TrackerCellKind {
                match self {
                    $(TrackerCellId::$cell => $kind,)*
                }
            }
        }
    }
}

cells! {
    GoMode: Simple {
        img: ImageInfo::extra("go_mode"),
        active: Box::new(|state| match state.knowledge.progression_mode {
            ProgressionMode::Go | ProgressionMode::Done => true,
            ProgressionMode::Bk | ProgressionMode::Normal => false,
        }),
        toggle: Box::new(|state| {
            let new_mode = match state.knowledge.progression_mode {
                ProgressionMode::Done => ProgressionMode::Done, // only the racetime integration may toggle .done for now
                ProgressionMode::Go => ProgressionMode::Normal,
                ProgressionMode::Bk | ProgressionMode::Normal => ProgressionMode::Go,
            };
            state.knowledge.progression_mode = new_mode;
        }),
    },
    GoBk: GoBk,
    LightMedallionLocation: MedallionLocation(Medallion::Light),
    ForestMedallionLocation: MedallionLocation(Medallion::Forest),
    FireMedallionLocation: MedallionLocation(Medallion::Fire),
    WaterMedallionLocation: MedallionLocation(Medallion::Water),
    ShadowMedallionLocation: MedallionLocation(Medallion::Shadow),
    SpiritMedallionLocation: MedallionLocation(Medallion::Spirit),
    LightMedallion: Medallion(Medallion::Light),
    ForestMedallion: Medallion(Medallion::Forest),
    FireMedallion: Medallion(Medallion::Fire),
    WaterMedallion: Medallion(Medallion::Water),
    ShadowMedallion: Medallion(Medallion::Shadow),
    SpiritMedallion: Medallion(Medallion::Spirit),
    AdultTrade: Sequence {
        idx: Box::new(|state| match state.ram.save.inv.adult_trade_item {
            AdultTradeItem::None => 0,
            AdultTradeItem::PocketEgg => 1,
            AdultTradeItem::PocketCucco => 2,
            AdultTradeItem::Cojiro => 3,
            AdultTradeItem::OddMushroom => 4,
            AdultTradeItem::OddPotion => 5,
            AdultTradeItem::PoachersSaw => 6,
            AdultTradeItem::BrokenSword => 7,
            AdultTradeItem::Prescription => 8,
            AdultTradeItem::EyeballFrog => 9,
            AdultTradeItem::Eyedrops => 10,
            AdultTradeItem::ClaimCheck => 11,
        }),
        img: Box::new(|state| match state.ram.save.inv.adult_trade_item {
            AdultTradeItem::None => (false, ImageInfo::new("blue_egg")),
            AdultTradeItem::PocketEgg | AdultTradeItem::PocketCucco => (true, ImageInfo::new("blue_egg")),
            AdultTradeItem::Cojiro => (true, ImageInfo::new("cojiro")),
            AdultTradeItem::OddMushroom => (true, ImageInfo::new("odd_mushroom")),
            AdultTradeItem::OddPotion => (true, ImageInfo::new("odd_poultice")),
            AdultTradeItem::PoachersSaw => (true, ImageInfo::new("poachers_saw")),
            AdultTradeItem::BrokenSword => (true, ImageInfo::new("broken_sword")),
            AdultTradeItem::Prescription => (true, ImageInfo::new("prescription")),
            AdultTradeItem::EyeballFrog => (true, ImageInfo::new("eyeball_frog")),
            AdultTradeItem::Eyedrops => (true, ImageInfo::new("eye_drops")),
            AdultTradeItem::ClaimCheck => (true, ImageInfo::new("claim_check")),
        }),
        increment: Box::new(|state| state.ram.save.inv.adult_trade_item = match state.ram.save.inv.adult_trade_item {
            AdultTradeItem::None => AdultTradeItem::PocketEgg,
            AdultTradeItem::PocketEgg => AdultTradeItem::PocketCucco,
            AdultTradeItem::PocketCucco => AdultTradeItem::Cojiro,
            AdultTradeItem::Cojiro => AdultTradeItem::OddMushroom,
            AdultTradeItem::OddMushroom => AdultTradeItem::OddPotion,
            AdultTradeItem::OddPotion => AdultTradeItem::PoachersSaw,
            AdultTradeItem::PoachersSaw => AdultTradeItem::BrokenSword,
            AdultTradeItem::BrokenSword => AdultTradeItem::Prescription,
            AdultTradeItem::Prescription => AdultTradeItem::EyeballFrog,
            AdultTradeItem::EyeballFrog => AdultTradeItem::Eyedrops,
            AdultTradeItem::Eyedrops => AdultTradeItem::ClaimCheck,
            AdultTradeItem::ClaimCheck => AdultTradeItem::None,
        }),
        decrement: Box::new(|state| state.ram.save.inv.adult_trade_item = match state.ram.save.inv.adult_trade_item {
            AdultTradeItem::None => AdultTradeItem::ClaimCheck,
            AdultTradeItem::PocketEgg => AdultTradeItem::None,
            AdultTradeItem::PocketCucco => AdultTradeItem::PocketEgg,
            AdultTradeItem::Cojiro => AdultTradeItem::PocketEgg,
            AdultTradeItem::OddMushroom => AdultTradeItem::Cojiro,
            AdultTradeItem::OddPotion => AdultTradeItem::OddMushroom,
            AdultTradeItem::PoachersSaw => AdultTradeItem::OddPotion,
            AdultTradeItem::BrokenSword => AdultTradeItem::PoachersSaw,
            AdultTradeItem::Prescription => AdultTradeItem::BrokenSword,
            AdultTradeItem::EyeballFrog => AdultTradeItem::Prescription,
            AdultTradeItem::Eyedrops => AdultTradeItem::EyeballFrog,
            AdultTradeItem::ClaimCheck => AdultTradeItem::Eyedrops,
        }),
    },
    AdultTradeNoChicken: Sequence {
        idx: Box::new(|state| match state.ram.save.inv.adult_trade_item {
            AdultTradeItem::None => 0,
            AdultTradeItem::PocketEgg | AdultTradeItem::PocketCucco => 1,
            AdultTradeItem::Cojiro => 2,
            AdultTradeItem::OddMushroom => 3,
            AdultTradeItem::OddPotion => 4,
            AdultTradeItem::PoachersSaw => 5,
            AdultTradeItem::BrokenSword => 6,
            AdultTradeItem::Prescription => 7,
            AdultTradeItem::EyeballFrog => 8,
            AdultTradeItem::Eyedrops => 9,
            AdultTradeItem::ClaimCheck => 10,
        }),
        img: Box::new(|state| match state.ram.save.inv.adult_trade_item {
            AdultTradeItem::None => (false, ImageInfo::new("blue_egg")),
            AdultTradeItem::PocketEgg | AdultTradeItem::PocketCucco => (true, ImageInfo::new("blue_egg")),
            AdultTradeItem::Cojiro => (true, ImageInfo::new("cojiro")),
            AdultTradeItem::OddMushroom => (true, ImageInfo::new("odd_mushroom")),
            AdultTradeItem::OddPotion => (true, ImageInfo::new("odd_poultice")),
            AdultTradeItem::PoachersSaw => (true, ImageInfo::new("poachers_saw")),
            AdultTradeItem::BrokenSword => (true, ImageInfo::new("broken_sword")),
            AdultTradeItem::Prescription => (true, ImageInfo::new("prescription")),
            AdultTradeItem::EyeballFrog => (true, ImageInfo::new("eyeball_frog")),
            AdultTradeItem::Eyedrops => (true, ImageInfo::new("eye_drops")),
            AdultTradeItem::ClaimCheck => (true, ImageInfo::new("claim_check")),
        }),
        increment: Box::new(|state| state.ram.save.inv.adult_trade_item = match state.ram.save.inv.adult_trade_item {
            AdultTradeItem::None => AdultTradeItem::PocketEgg,
            AdultTradeItem::PocketEgg | AdultTradeItem::PocketCucco => AdultTradeItem::Cojiro,
            AdultTradeItem::Cojiro => AdultTradeItem::OddMushroom,
            AdultTradeItem::OddMushroom => AdultTradeItem::OddPotion,
            AdultTradeItem::OddPotion => AdultTradeItem::PoachersSaw,
            AdultTradeItem::PoachersSaw => AdultTradeItem::BrokenSword,
            AdultTradeItem::BrokenSword => AdultTradeItem::Prescription,
            AdultTradeItem::Prescription => AdultTradeItem::EyeballFrog,
            AdultTradeItem::EyeballFrog => AdultTradeItem::Eyedrops,
            AdultTradeItem::Eyedrops => AdultTradeItem::ClaimCheck,
            AdultTradeItem::ClaimCheck => AdultTradeItem::None,
        }),
        decrement: Box::new(|state| state.ram.save.inv.adult_trade_item = match state.ram.save.inv.adult_trade_item {
            AdultTradeItem::None => AdultTradeItem::ClaimCheck,
            AdultTradeItem::PocketEgg | AdultTradeItem::PocketCucco => AdultTradeItem::None,
            AdultTradeItem::Cojiro => AdultTradeItem::PocketEgg,
            AdultTradeItem::OddMushroom => AdultTradeItem::Cojiro,
            AdultTradeItem::OddPotion => AdultTradeItem::OddMushroom,
            AdultTradeItem::PoachersSaw => AdultTradeItem::OddPotion,
            AdultTradeItem::BrokenSword => AdultTradeItem::PoachersSaw,
            AdultTradeItem::Prescription => AdultTradeItem::BrokenSword,
            AdultTradeItem::EyeballFrog => AdultTradeItem::Prescription,
            AdultTradeItem::Eyedrops => AdultTradeItem::EyeballFrog,
            AdultTradeItem::ClaimCheck => AdultTradeItem::Eyedrops,
        }),
    },
    Skulltula: Count {
        dimmed_img: ImageInfo::new("golden_skulltula"),
        img: ImageInfo::new("skulls"),
        get: Box::new(|state| state.ram.save.skull_tokens),
        set: Box::new(|state, value| state.ram.save.skull_tokens = value),
        max: 100,
        step: 1,
    },
    SkulltulaTens: Count {
        dimmed_img: ImageInfo::new("golden_skulltula"),
        img: ImageInfo::new("skulls"),
        get: Box::new(|state| state.ram.save.skull_tokens),
        set: Box::new(|state, value| state.ram.save.skull_tokens = value),
        max: 50,
        step: 10,
    },
    KokiriEmeraldLocation: StoneLocation(Stone::KokiriEmerald),
    KokiriEmerald: Stone(Stone::KokiriEmerald),
    GoronRubyLocation: StoneLocation(Stone::GoronRuby),
    GoronRuby: Stone(Stone::GoronRuby),
    ZoraSapphireLocation: StoneLocation(Stone::ZoraSapphire),
    ZoraSapphire: Stone(Stone::ZoraSapphire),
    Bottle: OptionalOverlay {
        main_img: ImageInfo::new("bottle"),
        overlay_img: ImageInfo::new("letter"),
        active: Box::new(|state| (state.ram.save.inv.emptiable_bottles() > 0, state.ram.save.inv.has_rutos_letter())), //TODO also show Ruto's letter as active if it has been delivered
        toggle_main: Box::new(|state| {
            let new_val = if state.ram.save.inv.emptiable_bottles() > 0 { 0 } else { 1 };
            state.ram.save.inv.set_emptiable_bottles(new_val);
        }),
        toggle_overlay: Box::new(|state| state.ram.save.inv.toggle_rutos_letter()),
    },
    NumBottles: Count {
        dimmed_img: ImageInfo::new("bottle"),
        img: ImageInfo::new("UNIMPLEMENTED"), //TODO make images for 1–4 bottles
        get: Box::new(|state| state.ram.save.inv.emptiable_bottles()),
        set: Box::new(|state, value| state.ram.save.inv.set_emptiable_bottles(value)),
        max: 4,
        step: 1,
    },
    RutosLetter: Simple {
        img: ImageInfo::new("UNIMPLEMENTED"),
        active: Box::new(|state| state.ram.save.inv.has_rutos_letter()), //TODO also show Ruto's letter as active if it has been delivered
        toggle: Box::new(|state| state.ram.save.inv.toggle_rutos_letter()),
    },
    Scale: Sequence {
        idx: Box::new(|state| match state.ram.save.upgrades.scale() {
            Upgrades::SILVER_SCALE => 1,
            Upgrades::GOLD_SCALE => 2,
            _ => 0,
        }),
        img: Box::new(|state| match state.ram.save.upgrades.scale() {
            Upgrades::SILVER_SCALE => (true, ImageInfo::new("silver_scale")),
            Upgrades::GOLD_SCALE => (true, ImageInfo::new("gold_scale")),
            _ => (false, ImageInfo::new("silver_scale")),
        }),
        increment: Box::new(|state| {
            let new_val = match state.ram.save.upgrades.scale() {
                Upgrades::SILVER_SCALE => Upgrades::GOLD_SCALE,
                Upgrades::GOLD_SCALE => Upgrades::NONE,
                _ => Upgrades::SILVER_SCALE,
            };
            state.ram.save.upgrades.set_scale(new_val);
        }),
        decrement: Box::new(|state| {
            let new_val = match state.ram.save.upgrades.scale() {
                Upgrades::SILVER_SCALE => Upgrades::NONE,
                Upgrades::GOLD_SCALE => Upgrades::SILVER_SCALE,
                _ => Upgrades::GOLD_SCALE,
            };
            state.ram.save.upgrades.set_scale(new_val);
        }),
    },
    Slingshot: Simple {
        img: ImageInfo::new("slingshot"),
        active: Box::new(|state| state.ram.save.inv.slingshot),
        toggle: Box::new(|state| {
            state.ram.save.inv.slingshot = !state.ram.save.inv.slingshot;
            let new_bullet_bag = if state.ram.save.inv.slingshot { Upgrades::BULLET_BAG_30 } else { Upgrades::NONE };
            state.ram.save.upgrades.set_bullet_bag(new_bullet_bag);
        }),
    },
    BulletBag: Sequence {
        idx: Box::new(|state| match state.ram.save.upgrades.bullet_bag() {
            Upgrades::BULLET_BAG_30 => 1,
            Upgrades::BULLET_BAG_40 => 2,
            Upgrades::BULLET_BAG_50 => 3,
            _ => 0,
        }),
        img: Box::new(|state| (state.ram.save.inv.slingshot, ImageInfo::new("slingshot"))),
        increment: Box::new(|state| {
            let new_bullet_bag = match state.ram.save.upgrades.bullet_bag() {
                Upgrades::BULLET_BAG_30 => Upgrades::BULLET_BAG_40,
                Upgrades::BULLET_BAG_40 => Upgrades::BULLET_BAG_50,
                Upgrades::BULLET_BAG_50 => Upgrades::NONE,
                _ => Upgrades::BULLET_BAG_30,
            };
            state.ram.save.upgrades.set_bullet_bag(new_bullet_bag);
            state.ram.save.inv.slingshot = state.ram.save.upgrades.bullet_bag() != Upgrades::NONE;
        }),
        decrement: Box::new(|state| {
            let new_bullet_bag = match state.ram.save.upgrades.bullet_bag() {
                Upgrades::BULLET_BAG_30 => Upgrades::NONE,
                Upgrades::BULLET_BAG_40 => Upgrades::BULLET_BAG_30,
                Upgrades::BULLET_BAG_50 => Upgrades::BULLET_BAG_40,
                _ => Upgrades::BULLET_BAG_50,
            };
            state.ram.save.upgrades.set_bullet_bag(new_bullet_bag);
            state.ram.save.inv.slingshot = state.ram.save.upgrades.bullet_bag() != Upgrades::NONE;
        }),
    },
    Bombs: Overlay {
        main_img: ImageInfo::new("bomb_bag"),
        overlay_img: ImageInfo::new("bombchu"),
        active: Box::new(|state| (state.ram.save.upgrades.bomb_bag() != Upgrades::NONE, state.ram.save.inv.bombchus)),
        toggle_main: Box::new(|state| if state.ram.save.upgrades.bomb_bag() == Upgrades::NONE {
            state.ram.save.upgrades.set_bomb_bag(Upgrades::BOMB_BAG_20);
        } else {
            state.ram.save.upgrades.set_bomb_bag(Upgrades::NONE);
        }),
        toggle_overlay: Box::new(|state| state.ram.save.inv.bombchus = !state.ram.save.inv.bombchus),
    },
    BombBag: Sequence {
        idx: Box::new(|state| match state.ram.save.upgrades.bomb_bag() {
            Upgrades::BOMB_BAG_20 => 1,
            Upgrades::BOMB_BAG_30 => 2,
            Upgrades::BOMB_BAG_40 => 3,
            _ => 0,
        }),
        img: Box::new(|state| (state.ram.save.upgrades.bomb_bag() != Upgrades::NONE, ImageInfo::new("bomb_bag"))),
        increment: Box::new(|state| {
            let new_val = match state.ram.save.upgrades.bomb_bag() {
                Upgrades::BOMB_BAG_20 => Upgrades::BOMB_BAG_30,
                Upgrades::BOMB_BAG_30 => Upgrades::BOMB_BAG_40,
                Upgrades::BOMB_BAG_40 => Upgrades::NONE,
                _ => Upgrades::BOMB_BAG_20,
            };
            state.ram.save.upgrades.set_bomb_bag(new_val);
        }),
        decrement: Box::new(|state| {
            let new_val = match state.ram.save.upgrades.bomb_bag() {
                Upgrades::BOMB_BAG_20 => Upgrades::NONE,
                Upgrades::BOMB_BAG_30 => Upgrades::BOMB_BAG_20,
                Upgrades::BOMB_BAG_40 => Upgrades::BOMB_BAG_30,
                _ => Upgrades::BOMB_BAG_40,
            };
            state.ram.save.upgrades.set_bomb_bag(new_val);
        }),
    },
    Bombchus: Simple {
        img: ImageInfo::new("UNIMPLEMENTED"),
        active: Box::new(|state| state.ram.save.inv.bombchus),
        toggle: Box::new(|state| state.ram.save.inv.bombchus = !state.ram.save.inv.bombchus),
    },
    Boomerang: Simple {
        img: ImageInfo::new("boomerang"),
        active: Box::new(|state| state.ram.save.inv.boomerang),
        toggle: Box::new(|state| state.ram.save.inv.boomerang = !state.ram.save.inv.boomerang),
    },
    Strength: Sequence {
        idx: Box::new(|state| match state.ram.save.upgrades.strength() {
            Upgrades::GORON_BRACELET => 1,
            Upgrades::SILVER_GAUNTLETS => 2,
            Upgrades::GOLD_GAUNTLETS => 3,
            _ => 0,
        }),
        img: Box::new(|state| match state.ram.save.upgrades.strength() {
            Upgrades::GORON_BRACELET => (true, ImageInfo::new("goron_bracelet")),
            Upgrades::SILVER_GAUNTLETS => (true, ImageInfo::new("silver_gauntlets")),
            Upgrades::GOLD_GAUNTLETS => (true, ImageInfo::new("gold_gauntlets")),
            _ => (false, ImageInfo::new("goron_bracelet")),
        }),
        increment: Box::new(|state| {
            let new_val = match state.ram.save.upgrades.strength() {
                Upgrades::GORON_BRACELET => Upgrades::SILVER_GAUNTLETS,
                Upgrades::SILVER_GAUNTLETS => Upgrades::GOLD_GAUNTLETS,
                Upgrades::GOLD_GAUNTLETS => Upgrades::NONE,
                _ => Upgrades::GORON_BRACELET,
            };
            state.ram.save.upgrades.set_strength(new_val);
        }),
        decrement: Box::new(|state| {
            let new_val = match state.ram.save.upgrades.strength() {
                Upgrades::GORON_BRACELET => Upgrades::NONE,
                Upgrades::SILVER_GAUNTLETS => Upgrades::GORON_BRACELET,
                Upgrades::GOLD_GAUNTLETS => Upgrades::SILVER_GAUNTLETS,
                _ => Upgrades::GOLD_GAUNTLETS,
            };
            state.ram.save.upgrades.set_strength(new_val);
        }),
    },
    Magic: Simple {
        img: ImageInfo::new("magic"),
        active: Box::new(|state| state.ram.save.magic != MagicCapacity::None),
        toggle: Box::new(|state| if state.ram.save.magic == MagicCapacity::None {
            state.ram.save.magic = MagicCapacity::Small;
        } else {
            state.ram.save.magic = MagicCapacity::None;
        }),
    },
    MagicLens: MagicLens,
    MagicCapacity: Sequence {
        idx: Box::new(|state| match state.ram.save.magic {
            MagicCapacity::None => 0,
            MagicCapacity::Small => 1,
            MagicCapacity::Large => 2,
        }),
        img: Box::new(|state| (state.ram.save.magic != MagicCapacity::None, ImageInfo::new("magic"))),
        increment: Box::new(|state| state.ram.save.magic = match state.ram.save.magic {
            MagicCapacity::None => MagicCapacity::Small,
            MagicCapacity::Small => MagicCapacity::Large,
            MagicCapacity::Large => MagicCapacity::None,
        }),
        decrement: Box::new(|state| state.ram.save.magic = match state.ram.save.magic {
            MagicCapacity::None => MagicCapacity::Large,
            MagicCapacity::Small => MagicCapacity::None,
            MagicCapacity::Large => MagicCapacity::Small,
        }),
    },
    Lens: Simple {
        img: ImageInfo::new("lens"),
        active: Box::new(|state| state.ram.save.inv.lens),
        toggle: Box::new(|state| state.ram.save.inv.lens = !state.ram.save.inv.lens),
    },
    Spells: Spells,
    DinsFire: Simple {
        img: ImageInfo::new("dins_fire"),
        active: Box::new(|state| state.ram.save.inv.dins_fire),
        toggle: Box::new(|state| state.ram.save.inv.dins_fire = !state.ram.save.inv.dins_fire),
    },
    FaroresWind: Simple {
        img: ImageInfo::new("faores_wind"),
        active: Box::new(|state| state.ram.save.inv.farores_wind),
        toggle: Box::new(|state| state.ram.save.inv.farores_wind = !state.ram.save.inv.farores_wind),
    },
    NayrusLove: Simple {
        img: ImageInfo::extra("nayrus_love"),
        active: Box::new(|state| state.ram.save.inv.nayrus_love),
        toggle: Box::new(|state| state.ram.save.inv.nayrus_love = !state.ram.save.inv.nayrus_love),
    },
    Hookshot: Sequence {
        idx: Box::new(|state| match state.ram.save.inv.hookshot {
            Hookshot::None => 0,
            Hookshot::Hookshot => 1,
            Hookshot::Longshot => 2,
        }),
        img: Box::new(|state| match state.ram.save.inv.hookshot {
            Hookshot::None => (false, ImageInfo::new("hookshot")),
            Hookshot::Hookshot => (true, ImageInfo::new("hookshot_accessible")),
            Hookshot::Longshot => (true, ImageInfo::new("longshot_accessible")),
        }),
        increment: Box::new(|state| state.ram.save.inv.hookshot = match state.ram.save.inv.hookshot {
            Hookshot::None => Hookshot::Hookshot,
            Hookshot::Hookshot => Hookshot::Longshot,
            Hookshot::Longshot => Hookshot::None,
        }),
        decrement: Box::new(|state| state.ram.save.inv.hookshot = match state.ram.save.inv.hookshot {
            Hookshot::None => Hookshot::Longshot,
            Hookshot::Hookshot => Hookshot::None,
            Hookshot::Longshot => Hookshot::Hookshot,
        }),
    },
    Bow: OptionalOverlay {
        main_img: ImageInfo::new("bow"),
        overlay_img: ImageInfo::new("ice_arrows"),
        active: Box::new(|state| (state.ram.save.inv.bow, state.ram.save.inv.ice_arrows)),
        toggle_main: Box::new(|state| {
            state.ram.save.inv.bow = !state.ram.save.inv.bow;
            let new_quiver = if state.ram.save.inv.bow { Upgrades::QUIVER_30 } else { Upgrades::NONE };
            state.ram.save.upgrades.set_quiver(new_quiver);
        }),
        toggle_overlay: Box::new(|state| state.ram.save.inv.ice_arrows = !state.ram.save.inv.ice_arrows),
    },
    IceArrows: Simple {
        img: ImageInfo::new("ice_trap"),
        active: Box::new(|state| state.ram.save.inv.ice_arrows),
        toggle: Box::new(|state| state.ram.save.inv.ice_arrows = !state.ram.save.inv.ice_arrows),
    },
    Quiver: Sequence {
        idx: Box::new(|state| match state.ram.save.upgrades.quiver() {
            Upgrades::QUIVER_30 => 1,
            Upgrades::QUIVER_40 => 2,
            Upgrades::QUIVER_50 => 3,
            _ => 0,
        }),
        img: Box::new(|state| (state.ram.save.inv.bow, ImageInfo::new("bow"))),
        increment: Box::new(|state| {
            let new_quiver = match state.ram.save.upgrades.quiver() {
                Upgrades::QUIVER_30 => Upgrades::QUIVER_40,
                Upgrades::QUIVER_40 => Upgrades::QUIVER_50,
                Upgrades::QUIVER_50 => Upgrades::NONE,
                _ => Upgrades::QUIVER_30,
            };
            state.ram.save.upgrades.set_quiver(new_quiver);
            state.ram.save.inv.bow = state.ram.save.upgrades.quiver() != Upgrades::NONE;
        }),
        decrement: Box::new(|state| {
            let new_quiver = match state.ram.save.upgrades.quiver() {
                Upgrades::QUIVER_30 => Upgrades::NONE,
                Upgrades::QUIVER_40 => Upgrades::QUIVER_30,
                Upgrades::QUIVER_50 => Upgrades::QUIVER_40,
                _ => Upgrades::QUIVER_50,
            };
            state.ram.save.upgrades.set_quiver(new_quiver);
            state.ram.save.inv.bow = state.ram.save.upgrades.quiver() != Upgrades::NONE;
        }),
    },
    Arrows: Composite {
        left_img: ImageInfo::new("fire_arrows"),
        right_img: ImageInfo::new("light_arrows"),
        both_img: ImageInfo::new("composite_arrows"),
        active: Box::new(|state| (state.ram.save.inv.fire_arrows, state.ram.save.inv.light_arrows)),
        toggle_left: Box::new(|state| state.ram.save.inv.fire_arrows = !state.ram.save.inv.fire_arrows),
        toggle_right: Box::new(|state| state.ram.save.inv.light_arrows = !state.ram.save.inv.light_arrows),
    },
    FireArrows: Simple {
        img: ImageInfo::new("fire_arrows"),
        active: Box::new(|state| state.ram.save.inv.fire_arrows),
        toggle: Box::new(|state| state.ram.save.inv.fire_arrows = !state.ram.save.inv.fire_arrows),
    },
    LightArrows: Simple {
        img: ImageInfo::new("light_arrows"),
        active: Box::new(|state| state.ram.save.inv.light_arrows),
        toggle: Box::new(|state| state.ram.save.inv.light_arrows = !state.ram.save.inv.light_arrows),
    },
    Hammer: Simple {
        img: ImageInfo::new("hammer"),
        active: Box::new(|state| state.ram.save.inv.hammer),
        toggle: Box::new(|state| state.ram.save.inv.hammer = !state.ram.save.inv.hammer),
    },
    Boots: Composite {
        left_img: ImageInfo::new("iron_boots"),
        right_img: ImageInfo::new("hover_boots"),
        both_img: ImageInfo::new("composite_boots"),
        active: Box::new(|state| (state.ram.save.equipment.contains(Equipment::IRON_BOOTS), state.ram.save.equipment.contains(Equipment::HOVER_BOOTS))),
        toggle_left: Box::new(|state| state.ram.save.equipment.toggle(Equipment::IRON_BOOTS)),
        toggle_right: Box::new(|state| state.ram.save.equipment.toggle(Equipment::HOVER_BOOTS)),
    },
    IronBoots: Simple {
        img: ImageInfo::new("iron_boots"),
        active: Box::new(|state| state.ram.save.equipment.contains(Equipment::IRON_BOOTS)),
        toggle: Box::new(|state| state.ram.save.equipment.toggle(Equipment::IRON_BOOTS)),
    },
    HoverBoots: Simple {
        img: ImageInfo::new("hover_boots"),
        active: Box::new(|state| state.ram.save.equipment.contains(Equipment::HOVER_BOOTS)),
        toggle: Box::new(|state| state.ram.save.equipment.toggle(Equipment::HOVER_BOOTS)),
    },
    MirrorShield: Simple {
        img: ImageInfo::new("mirror_shield"),
        active: Box::new(|state| state.ram.save.equipment.contains(Equipment::MIRROR_SHIELD)),
        toggle: Box::new(|state| state.ram.save.equipment.toggle(Equipment::MIRROR_SHIELD)),
    },
    ChildTrade: Sequence {
        idx: Box::new(|state| match state.ram.save.inv.child_trade_item {
            ChildTradeItem::None => 0,
            ChildTradeItem::WeirdEgg => 1,
            ChildTradeItem::Chicken => 2,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => 3, //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => 4,
            ChildTradeItem::SkullMask => 5,
            ChildTradeItem::SpookyMask => 6,
            ChildTradeItem::BunnyHood => 7,
            ChildTradeItem::MaskOfTruth => 8,
        }),
        img: Box::new(|state| match state.ram.save.inv.child_trade_item {
            ChildTradeItem::None => (false, ImageInfo::new("white_egg")),
            ChildTradeItem::WeirdEgg => (true, ImageInfo::new("white_egg")),
            ChildTradeItem::Chicken => (true, ImageInfo::new("white_chicken")),
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => (true, ImageInfo::new("zelda_letter")), //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => (true, ImageInfo::new("keaton_mask")),
            ChildTradeItem::SkullMask => (true, ImageInfo::new("skull_mask")),
            ChildTradeItem::SpookyMask => (true, ImageInfo::new("spooky_mask")),
            ChildTradeItem::BunnyHood => (true, ImageInfo::new("bunny_hood")),
            ChildTradeItem::MaskOfTruth => (true, ImageInfo::new("mask_of_truth")),
        }),
        increment: Box::new(|state| state.ram.save.inv.child_trade_item = match state.ram.save.inv.child_trade_item {
            ChildTradeItem::None => ChildTradeItem::WeirdEgg,
            ChildTradeItem::WeirdEgg => ChildTradeItem::Chicken,
            ChildTradeItem::Chicken => ChildTradeItem::ZeldasLetter,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => ChildTradeItem::KeatonMask, //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => ChildTradeItem::SkullMask,
            ChildTradeItem::SkullMask => ChildTradeItem::SpookyMask,
            ChildTradeItem::SpookyMask => ChildTradeItem::BunnyHood,
            ChildTradeItem::BunnyHood => ChildTradeItem::MaskOfTruth,
            ChildTradeItem::MaskOfTruth => ChildTradeItem::None,
        }),
        decrement: Box::new(|state| state.ram.save.inv.child_trade_item = match state.ram.save.inv.child_trade_item {
            ChildTradeItem::None => ChildTradeItem::MaskOfTruth,
            ChildTradeItem::WeirdEgg => ChildTradeItem::None,
            ChildTradeItem::Chicken => ChildTradeItem::WeirdEgg,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => ChildTradeItem::Chicken, //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => ChildTradeItem::ZeldasLetter,
            ChildTradeItem::SkullMask => ChildTradeItem::KeatonMask,
            ChildTradeItem::SpookyMask => ChildTradeItem::SkullMask,
            ChildTradeItem::BunnyHood => ChildTradeItem::SpookyMask,
            ChildTradeItem::MaskOfTruth => ChildTradeItem::BunnyHood,
        }),
    },
    ChildTradeNoChicken: Sequence {
        idx: Box::new(|state| match state.ram.save.inv.child_trade_item {
            ChildTradeItem::None => 0,
            ChildTradeItem::WeirdEgg | ChildTradeItem::Chicken => 1,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => 2, //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => 3,
            ChildTradeItem::SkullMask => 4,
            ChildTradeItem::SpookyMask => 5,
            ChildTradeItem::BunnyHood => 6,
            ChildTradeItem::MaskOfTruth => 7,
        }),
        img: Box::new(|state| match state.ram.save.inv.child_trade_item {
            ChildTradeItem::None => (false, ImageInfo::new("white_egg")),
            ChildTradeItem::WeirdEgg | ChildTradeItem::Chicken => (true, ImageInfo::new("white_egg")),
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => (true, ImageInfo::new("zelda_letter")), //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => (true, ImageInfo::new("keaton_mask")),
            ChildTradeItem::SkullMask => (true, ImageInfo::new("skull_mask")),
            ChildTradeItem::SpookyMask => (true, ImageInfo::new("spooky_mask")),
            ChildTradeItem::BunnyHood => (true, ImageInfo::new("bunny_hood")),
            ChildTradeItem::MaskOfTruth => (true, ImageInfo::new("mask_of_truth")),
        }),
        increment: Box::new(|state| state.ram.save.inv.child_trade_item = match state.ram.save.inv.child_trade_item {
            ChildTradeItem::None => ChildTradeItem::WeirdEgg,
            ChildTradeItem::WeirdEgg | ChildTradeItem::Chicken => ChildTradeItem::ZeldasLetter,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => ChildTradeItem::KeatonMask, //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => ChildTradeItem::SkullMask,
            ChildTradeItem::SkullMask => ChildTradeItem::SpookyMask,
            ChildTradeItem::SpookyMask => ChildTradeItem::BunnyHood,
            ChildTradeItem::BunnyHood => ChildTradeItem::MaskOfTruth,
            ChildTradeItem::MaskOfTruth => ChildTradeItem::None,
        }),
        decrement: Box::new(|state| state.ram.save.inv.child_trade_item = match state.ram.save.inv.child_trade_item {
            ChildTradeItem::None => ChildTradeItem::MaskOfTruth,
            ChildTradeItem::WeirdEgg | ChildTradeItem::Chicken => ChildTradeItem::None,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => ChildTradeItem::WeirdEgg, //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => ChildTradeItem::ZeldasLetter,
            ChildTradeItem::SkullMask => ChildTradeItem::KeatonMask,
            ChildTradeItem::SpookyMask => ChildTradeItem::SkullMask,
            ChildTradeItem::BunnyHood => ChildTradeItem::SpookyMask,
            ChildTradeItem::MaskOfTruth => ChildTradeItem::BunnyHood,
        }),
    },
    ChildTradeSoldOut: Sequence {
        idx: Box::new(|state| match state.ram.save.inv.child_trade_item {
            ChildTradeItem::None => 0,
            ChildTradeItem::WeirdEgg => 1,
            ChildTradeItem::Chicken => 2,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => 3, //TODO for SOLD OUT, check trade quest progress
            //TODO Zelda's letter turned in => 4
            ChildTradeItem::KeatonMask => 5,
            //TODO Keaton mask sold => 6
            ChildTradeItem::SkullMask => 7,
            //TODO skull mask sold => 8
            ChildTradeItem::SpookyMask => 9,
            //TODO spooky mask sold => 10
            ChildTradeItem::BunnyHood => 11,
            //TODO bunny hood sold => 12
            ChildTradeItem::MaskOfTruth => 13,
        }),
        img: Box::new(|state| match state.ram.save.inv.child_trade_item {
            ChildTradeItem::None => (false, ImageInfo::new("white_egg")),
            ChildTradeItem::WeirdEgg => (true, ImageInfo::new("white_egg")),
            ChildTradeItem::Chicken => (true, ImageInfo::new("white_chicken")),
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => (true, ImageInfo::new("zelda_letter")), //TODO for SOLD OUT, check trade quest progress
            //TODO Zelda's letter turned in => SOLD OUT
            ChildTradeItem::KeatonMask => (true, ImageInfo::new("keaton_mask")),
            //TODO Keaton mask sold => SOLD OUT
            ChildTradeItem::SkullMask => (true, ImageInfo::new("skull_mask")),
            //TODO skull mask sold => SOLD OUT
            ChildTradeItem::SpookyMask => (true, ImageInfo::new("spooky_mask")),
            //TODO spooky mask sold => SOLD OUT
            ChildTradeItem::BunnyHood => (true, ImageInfo::new("bunny_hood")),
            //TODO bunny hood sold => SOLD OUT
            ChildTradeItem::MaskOfTruth => (true, ImageInfo::new("mask_of_truth")),
        }),
        increment: Box::new(|state| state.ram.save.inv.child_trade_item = match state.ram.save.inv.child_trade_item {
            //TODO consider sold-out states
            ChildTradeItem::None => ChildTradeItem::WeirdEgg,
            ChildTradeItem::WeirdEgg => ChildTradeItem::Chicken,
            ChildTradeItem::Chicken => ChildTradeItem::ZeldasLetter,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => ChildTradeItem::KeatonMask, //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => ChildTradeItem::SkullMask,
            ChildTradeItem::SkullMask => ChildTradeItem::SpookyMask,
            ChildTradeItem::SpookyMask => ChildTradeItem::BunnyHood,
            ChildTradeItem::BunnyHood => ChildTradeItem::MaskOfTruth,
            ChildTradeItem::MaskOfTruth => ChildTradeItem::None,
        }),
        decrement: Box::new(|state| state.ram.save.inv.child_trade_item = match state.ram.save.inv.child_trade_item {
            //TODO consider sold-out states
            ChildTradeItem::None => ChildTradeItem::MaskOfTruth,
            ChildTradeItem::WeirdEgg => ChildTradeItem::None,
            ChildTradeItem::Chicken => ChildTradeItem::WeirdEgg,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => ChildTradeItem::Chicken, //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => ChildTradeItem::ZeldasLetter,
            ChildTradeItem::SkullMask => ChildTradeItem::KeatonMask,
            ChildTradeItem::SpookyMask => ChildTradeItem::SkullMask,
            ChildTradeItem::BunnyHood => ChildTradeItem::SpookyMask,
            ChildTradeItem::MaskOfTruth => ChildTradeItem::BunnyHood,
        }),
    },
    Ocarina: Overlay {
        main_img: ImageInfo::new("ocarina"),
        overlay_img: ImageInfo::new("scarecrow"),
        active: Box::new(|state| (state.ram.save.inv.ocarina, state.ram.save.event_chk_inf.9.contains(EventChkInf9::SCARECROW_SONG))), //TODO only show free Scarecrow's Song once it's known (by settings string input or by check)
        toggle_main: Box::new(|state| state.ram.save.inv.ocarina = !state.ram.save.inv.ocarina),
        toggle_overlay: Box::new(|state| state.ram.save.event_chk_inf.9.toggle(EventChkInf9::SCARECROW_SONG)), //TODO make sure free scarecrow knowledge is toggled off properly
    },
    Beans: Simple { //TODO overlay with number bought if autotracker is on & shuffle beans is off
        img: ImageInfo::new("beans"),
        active: Box::new(|state| state.ram.save.inv.beans),
        toggle: Box::new(|state| state.ram.save.inv.beans = !state.ram.save.inv.beans),
    },
    SwordCard: Composite {
        left_img: ImageInfo::new("kokiri_sword"),
        right_img: ImageInfo::new("gerudo_card"),
        both_img: ImageInfo::extra("composite_ksword_gcard"),
        active: Box::new(|state| (state.ram.save.equipment.contains(Equipment::KOKIRI_SWORD), state.ram.save.quest_items.contains(QuestItems::GERUDO_CARD))),
        toggle_left: Box::new(|state| state.ram.save.equipment.toggle(Equipment::KOKIRI_SWORD)),
        toggle_right: Box::new(|state| state.ram.save.quest_items.toggle(QuestItems::GERUDO_CARD)),
    },
    KokiriSword: Simple {
        img: ImageInfo::new("kokiri_sword"),
        active: Box::new(|state| state.ram.save.equipment.contains(Equipment::KOKIRI_SWORD)),
        toggle: Box::new(|state| state.ram.save.equipment.toggle(Equipment::KOKIRI_SWORD)),
    },
    Tunics: Composite {
        left_img: ImageInfo::new("goron_tunic"),
        right_img: ImageInfo::new("zora_tunic"),
        both_img: ImageInfo::new("composite_tunics"),
        active: Box::new(|state| (state.ram.save.equipment.contains(Equipment::GORON_TUNIC), state.ram.save.equipment.contains(Equipment::ZORA_TUNIC))),
        toggle_left: Box::new(|state| state.ram.save.equipment.toggle(Equipment::GORON_TUNIC)),
        toggle_right: Box::new(|state| state.ram.save.equipment.toggle(Equipment::ZORA_TUNIC)),
    },
    GoronTunic: Simple {
        img: ImageInfo::new("goron_tunic"),
        active: Box::new(|state| state.ram.save.equipment.contains(Equipment::GORON_TUNIC)),
        toggle: Box::new(|state| state.ram.save.equipment.toggle(Equipment::GORON_TUNIC)),
    },
    ZoraTunic: Simple {
        img: ImageInfo::new("zora_tunic"),
        active: Box::new(|state| state.ram.save.equipment.contains(Equipment::ZORA_TUNIC)),
        toggle: Box::new(|state| state.ram.save.equipment.toggle(Equipment::ZORA_TUNIC)),
    },
    Triforce: Count {
        dimmed_img: ImageInfo::new("triforce"),
        img: ImageInfo::new("force"),
        get: Box::new(|state| state.ram.save.triforce_pieces()),
        set: Box::new(|state, value| state.ram.save.set_triforce_pieces(value)),
        max: 100,
        step: 1,
    },
    BigPoeTriforce: BigPoeTriforce,
    TriforceOneAndFives: Sequence {
        idx: Box::new(|state| match state.ram.save.triforce_pieces() {
            0 => 0,
            1..=4 => 1,
            5..=9 => 2,
            10..=14 => 3,
            15..=19 => 4,
            20..=24 => 5,
            25..=29 => 6,
            30..=34 => 7,
            35..=39 => 8,
            40..=44 => 9,
            45..=49 => 10,
            50..=54 => 11,
            55..=59 => 12,
            _ => 13,
        }),
        img: Box::new(|state| (state.ram.save.triforce_pieces() > 0, ImageInfo::new("triforce"))), //TODO images from count?
        increment: Box::new(|state| {
            let new_val = match state.ram.save.triforce_pieces() {
                0 => 1,
                1..=4 => 5,
                5..=9 => 10,
                10..=14 => 15,
                15..=19 => 20,
                20..=24 => 25,
                25..=29 => 30,
                30..=34 => 35,
                35..=39 => 40,
                40..=44 => 45,
                45..=49 => 50,
                50..=54 => 55,
                55..=59 => 60,
                _ => 0,
            };
            state.ram.save.set_triforce_pieces(new_val);
        }),
        decrement: Box::new(|state| {
            let new_val = match state.ram.save.triforce_pieces() {
                0 => 60,
                1..=4 => 0,
                5..=9 => 1,
                10..=14 => 5,
                15..=19 => 10,
                20..=24 => 15,
                25..=29 => 20,
                30..=34 => 25,
                35..=39 => 30,
                40..=44 => 35,
                45..=49 => 40,
                50..=54 => 45,
                55..=59 => 50,
                _ => 55,
            };
            state.ram.save.set_triforce_pieces(new_val);
        }),
    },
    ZeldasLullaby: Song {
        song: QuestItems::ZELDAS_LULLABY,
        check: "Song from Impa",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_IMPA)),
    },
    ZeldasLullabyCheck: SongCheck {
        check: "Song from Impa",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_IMPA)),
    },
    EponasSong: Song {
        song: QuestItems::EPONAS_SONG,
        check: "Song from Malon",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_MALON)),
    },
    EponasSongCheck: SongCheck {
        check: "Song from Malon",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_MALON)),
    },
    SariasSong: Song {
        song: QuestItems::SARIAS_SONG,
        check: "Song from Saria",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_SARIA)),
    },
    SariasSongCheck: SongCheck {
        check: "Song from Saria",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_SARIA)),
    },
    SunsSong: Song {
        song: QuestItems::SUNS_SONG,
        check: "Song from Composers Grave",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_COMPOSERS_GRAVE)),
    },
    SunsSongCheck: SongCheck {
        check: "Song from Composers Grave",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_COMPOSERS_GRAVE)),
    },
    SongOfTime: Song {
        song: QuestItems::SONG_OF_TIME,
        check: "Song from Ocarina of Time",
        toggle_overlay: Box::new(|eci| eci.10.toggle(EventChkInf10::SONG_FROM_OCARINA_OF_TIME)),
    },
    SongOfTimeCheck: SongCheck {
        check: "Song from Ocarina of Time",
        toggle_overlay: Box::new(|eci| eci.10.toggle(EventChkInf10::SONG_FROM_OCARINA_OF_TIME)),
    },
    SongOfStorms: Song {
        song: QuestItems::SONG_OF_STORMS,
        check: "Song from Windmill",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_WINDMILL)),
    },
    SongOfStormsCheck: SongCheck {
        check: "Song from Windmill",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_WINDMILL)),
    },
    Minuet: Song {
        song: QuestItems::MINUET_OF_FOREST,
        check: "Sheik in Forest",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_FOREST)),
    },
    MinuetCheck: SongCheck {
        check: "Sheik in Forest",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_FOREST)),
    },
    Bolero: Song {
        song: QuestItems::BOLERO_OF_FIRE,
        check: "Sheik in Crater",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_CRATER)),
    },
    BoleroCheck: SongCheck {
        check: "Sheik in Crater",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_CRATER)),
    },
    Serenade: Song {
        song: QuestItems::SERENADE_OF_WATER,
        check: "Sheik in Ice Cavern",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_ICE_CAVERN)),
    },
    SerenadeCheck: SongCheck {
        check: "Sheik in Ice Cavern",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_ICE_CAVERN)),
    },
    Requiem: Song {
        song: QuestItems::REQUIEM_OF_SPIRIT,
        check: "Sheik at Colossus",
        toggle_overlay: Box::new(|eci| eci.10.toggle(EventChkInf10::SHEIK_AT_COLOSSUS)),
    },
    RequiemCheck: SongCheck {
        check: "Sheik at Colossus",
        toggle_overlay: Box::new(|eci| eci.10.toggle(EventChkInf10::SHEIK_AT_COLOSSUS)),
    },
    Nocturne: Song {
        song: QuestItems::NOCTURNE_OF_SHADOW,
        check: "Sheik in Kakariko",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_KAKARIKO)),
    },
    NocturneCheck: SongCheck {
        check: "Sheik in Kakariko",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_KAKARIKO)),
    },
    Prelude: Song {
        song: QuestItems::PRELUDE_OF_LIGHT,
        check: "Sheik at Temple",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_AT_TEMPLE)),
    },
    PreludeCheck: SongCheck {
        check: "Sheik at Temple",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_AT_TEMPLE)),
    },
    FreeReward: FreeReward,
    DekuMq: Mq(Dungeon::Main(MainDungeon::DekuTree)),
    DcMq: Mq(Dungeon::Main(MainDungeon::DodongosCavern)),
    JabuMq: Mq(Dungeon::Main(MainDungeon::JabuJabu)),
    ForestMq: Mq(Dungeon::Main(MainDungeon::ForestTemple)),
    ForestSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.forest_temple),
        set: Box::new(|keys, value| keys.forest_temple = value),
        max_vanilla: 5,
        max_mq: 6,
    },
    ForestBossKey: BossKey {
        active: Box::new(|items| items.forest_temple.boss_key),
        toggle: Box::new(|items| items.forest_temple.boss_key = !items.forest_temple.boss_key),
    },
    ForestKeys: CompositeKeys {
        small: TrackerCellId::ForestSmallKeys,
        boss: TrackerCellId::ForestBossKey,
    },
    FireMq: Mq(Dungeon::Main(MainDungeon::FireTemple)),
    FireSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.fire_temple),
        set: Box::new(|keys, value| keys.fire_temple = value),
        max_vanilla: 8,
        max_mq: 5,
    },
    FireBossKey: BossKey {
        active: Box::new(|items| items.fire_temple.boss_key),
        toggle: Box::new(|items| items.fire_temple.boss_key = !items.fire_temple.boss_key),
    },
    FireKeys: CompositeKeys {
        small: TrackerCellId::FireSmallKeys,
        boss: TrackerCellId::FireBossKey,
    },
    WaterMq: Mq(Dungeon::Main(MainDungeon::WaterTemple)),
    WaterSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.water_temple),
        set: Box::new(|keys, value| keys.water_temple = value),
        max_vanilla: 6,
        max_mq: 2,
    },
    WaterBossKey: BossKey {
        active: Box::new(|items| items.water_temple.boss_key),
        toggle: Box::new(|items| items.water_temple.boss_key = !items.water_temple.boss_key),
    },
    WaterKeys: CompositeKeys {
        small: TrackerCellId::WaterSmallKeys,
        boss: TrackerCellId::WaterBossKey,
    },
    ShadowMq: Mq(Dungeon::Main(MainDungeon::ShadowTemple)),
    ShadowSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.shadow_temple),
        set: Box::new(|keys, value| keys.shadow_temple = value),
        max_vanilla: 5,
        max_mq: 6,
    },
    ShadowBossKey: BossKey {
        active: Box::new(|items| items.shadow_temple.boss_key),
        toggle: Box::new(|items| items.shadow_temple.boss_key = !items.shadow_temple.boss_key),
    },
    ShadowKeys: CompositeKeys {
        small: TrackerCellId::ShadowSmallKeys,
        boss: TrackerCellId::ShadowBossKey,
    },
    SpiritMq: Mq(Dungeon::Main(MainDungeon::SpiritTemple)),
    SpiritSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.spirit_temple),
        set: Box::new(|keys, value| keys.spirit_temple = value),
        max_vanilla: 5,
        max_mq: 7,
    },
    SpiritBossKey: BossKey {
        active: Box::new(|items| items.spirit_temple.boss_key),
        toggle: Box::new(|items| items.spirit_temple.boss_key = !items.spirit_temple.boss_key),
    },
    SpiritKeys: CompositeKeys {
        small: TrackerCellId::SpiritSmallKeys,
        boss: TrackerCellId::SpiritBossKey,
    },
    IceMq: Mq(Dungeon::IceCavern),
    WellMq: Mq(Dungeon::BottomOfTheWell),
    WellSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.bottom_of_the_well),
        set: Box::new(|keys, value| keys.bottom_of_the_well = value),
        max_vanilla: 3,
        max_mq: 2,
    },
    FortressMq: FortressMq,
    FortressSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.thieves_hideout),
        set: Box::new(|keys, value| keys.thieves_hideout = value),
        max_vanilla: 4,
        max_mq: 4,
    },
    GtgMq: Mq(Dungeon::GerudoTrainingGround),
    GtgSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.gerudo_training_ground),
        set: Box::new(|keys, value| keys.gerudo_training_ground = value),
        max_vanilla: 9,
        max_mq: 3,
    },
    GanonMq: Mq(Dungeon::GanonsCastle),
    GanonSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.ganons_castle),
        set: Box::new(|keys, value| keys.ganons_castle = value),
        max_vanilla: 2,
        max_mq: 3,
    },
    GanonBossKey: BossKey {
        active: Box::new(|items| items.ganons_castle.boss_key),
        toggle: Box::new(|items| items.ganons_castle.boss_key = !items.ganons_castle.boss_key),
    },
    GanonKeys: CompositeKeys {
        small: TrackerCellId::GanonSmallKeys,
        boss: TrackerCellId::GanonBossKey,
    },
    BiggoronSword: Simple {
        img: ImageInfo::new("UNIMPLEMENTED"),
        active: Box::new(|state| state.ram.save.biggoron_sword && state.ram.save.equipment.contains(Equipment::GIANTS_KNIFE)),
        toggle: Box::new(|state| if state.ram.save.biggoron_sword && state.ram.save.equipment.contains(Equipment::GIANTS_KNIFE) {
            state.ram.save.biggoron_sword = false;
            state.ram.save.equipment.remove(Equipment::GIANTS_KNIFE);
        } else {
            state.ram.save.biggoron_sword = true;
            state.ram.save.equipment.insert(Equipment::GIANTS_KNIFE);
        }),
    },
    WalletNoTycoon: Sequence {
        idx: Box::new(|state| match state.ram.save.upgrades.wallet() {
            Upgrades::ADULTS_WALLET => 1,
            Upgrades::GIANTS_WALLET | Upgrades::TYCOONS_WALLET => 2,
            _ => 0,
        }),
        img: Box::new(|state| (state.ram.save.upgrades.wallet() != Upgrades::NONE, ImageInfo::new("UNIMPLEMENTED"))),
        increment: Box::new(|state| {
            let new_val = match state.ram.save.upgrades.wallet() {
                Upgrades::ADULTS_WALLET => Upgrades::GIANTS_WALLET,
                Upgrades::GIANTS_WALLET | Upgrades::TYCOONS_WALLET => Upgrades::NONE,
                _ => Upgrades::ADULTS_WALLET,
            };
            state.ram.save.upgrades.set_wallet(new_val);
        }),
        decrement: Box::new(|state| {
            let new_val = match state.ram.save.upgrades.wallet() {
                Upgrades::ADULTS_WALLET => Upgrades::NONE,
                Upgrades::GIANTS_WALLET | Upgrades::TYCOONS_WALLET => Upgrades::ADULTS_WALLET,
                _ => Upgrades::GIANTS_WALLET,
            };
            state.ram.save.upgrades.set_wallet(new_val);
        }),
    },
    StoneOfAgony: Simple {
        img: ImageInfo::new("UNIMPLEMENTED"),
        active: Box::new(|state| state.ram.save.quest_items.contains(QuestItems::STONE_OF_AGONY)),
        toggle: Box::new(|state| state.ram.save.quest_items.toggle(QuestItems::STONE_OF_AGONY)),
    },
    Blank: Simple {
        img: ImageInfo::extra("blank"),
        active: Box::new(|_| false),
        toggle: Box::new(|_| ()),
    },
}

impl TrackerCellId {
    pub fn med_location(med: Medallion) -> TrackerCellId {
        match med {
            Medallion::Light => TrackerCellId::LightMedallionLocation,
            Medallion::Forest => TrackerCellId::ForestMedallionLocation,
            Medallion::Fire => TrackerCellId::FireMedallionLocation,
            Medallion::Water => TrackerCellId::WaterMedallionLocation,
            Medallion::Shadow => TrackerCellId::ShadowMedallionLocation,
            Medallion::Spirit => TrackerCellId::SpiritMedallionLocation,
        }
    }

    pub fn warp_song(med: Medallion) -> TrackerCellId {
        match med {
            Medallion::Light => TrackerCellId::Prelude,
            Medallion::Forest => TrackerCellId::Minuet,
            Medallion::Fire => TrackerCellId::Bolero,
            Medallion::Water => TrackerCellId::Serenade,
            Medallion::Shadow => TrackerCellId::Nocturne,
            Medallion::Spirit => TrackerCellId::Requiem,
        }
    }
}

impl From<Medallion> for TrackerCellId {
    fn from(med: Medallion) -> TrackerCellId {
        match med {
            Medallion::Light => TrackerCellId::LightMedallion,
            Medallion::Forest => TrackerCellId::ForestMedallion,
            Medallion::Fire => TrackerCellId::FireMedallion,
            Medallion::Water => TrackerCellId::WaterMedallion,
            Medallion::Shadow => TrackerCellId::ShadowMedallion,
            Medallion::Spirit => TrackerCellId::SpiritMedallion,
        }
    }
}

#[derive(Debug, PartialEq, Protocol)]
pub enum TrackerLayout {
    Default {
        auto: bool,
        meds: ElementOrder,
        warp_songs: ElementOrder,
    },
    MultiworldExpanded,
    MultiworldCollapsed,
    MultiworldEdit,
    RslLeft,
    RslRight,
    RslEdit,
}

pub struct CellLayout {
    pub idx: usize,
    pub id: TrackerCellId,
    pub pos: [u16; 2],
    pub size: [u16; 2],
}

impl TrackerLayout {
    /// The default layout for auto-tracking, which replaces the Triforce piece count cell with a dynamic big Poe count/Triforce piece count cell.
    pub fn default_auto() -> TrackerLayout { TrackerLayout::new_auto(&Config::default()) }

    /// The auto-tracking layout for this config, which replaces the Triforce piece count cell with a dynamic big Poe count/Triforce piece count cell.
    pub fn new_auto(config: &Config) -> TrackerLayout {
        TrackerLayout::Default {
            auto: true,
            meds: config.med_order,
            warp_songs: config.warp_song_order,
        }
    }

    pub fn cells(&self) -> Vec<CellLayout> {
        use TrackerCellId::*;

        macro_rules! columns {
            ($width:expr, [$($id:expr,)*]) => {{
                vec![$($id),*]
                    .into_iter()
                    .enumerate()
                    .map(|(idx, id)| CellLayout { idx, id, pos: [idx as u16 % $width * 60 + 5, idx as u16 / $width * 60 + 5], size: [50, 50] })
                    .collect()
            }};
        }

        match self {
            TrackerLayout::Default { auto, meds, warp_songs } => {
                meds.into_iter().enumerate().map(|(idx, med)| CellLayout { idx, id: TrackerCellId::med_location(med), pos: [idx as u16 * 60 + 5, 5], size: [50, 18] })
                    .chain(meds.into_iter().enumerate().map(|(idx, med)| CellLayout { idx: idx + 6, id: TrackerCellId::from(med), pos: [idx as u16 * 60 + 5, 33], size: [50, 50] }))
                    .chain(vec![
                        CellLayout { idx: 12, id: AdultTradeNoChicken, pos: [5, 93], size: [50, 50] },
                        CellLayout { idx: 13, id: Skulltula, pos: [65, 93], size: [50, 50] },
                        CellLayout { idx: 14, id: KokiriEmeraldLocation, pos: [125, 93], size: [30, 10] },
                        CellLayout { idx: 15, id: GoronRubyLocation, pos: [165, 93], size: [30, 10] },
                        CellLayout { idx: 16, id: ZoraSapphireLocation, pos: [205, 93], size: [30, 10] },
                        CellLayout { idx: 17, id: Bottle, pos: [245, 93], size: [50, 50] },
                        CellLayout { idx: 18, id: Scale, pos: [305, 93], size: [50, 50] },
                        CellLayout { idx: 19, id: KokiriEmerald, pos: [125, 113], size: [30, 30] },
                        CellLayout { idx: 20, id: GoronRuby, pos: [165, 113], size: [30, 30] },
                        CellLayout { idx: 21, id: ZoraSapphire, pos: [205, 113], size: [30, 30] },
                    ]).chain(
                        vec![
                            Slingshot, Bombs, Boomerang, Strength, MagicLens, Spells,
                            Hookshot, Bow, Arrows, Hammer, Boots, MirrorShield,
                            ChildTrade, Ocarina, Beans, SwordCard, Tunics, if *auto { BigPoeTriforce } else { Triforce },
                            ZeldasLullaby, EponasSong, SariasSong, SunsSong, SongOfTime, SongOfStorms,
                        ].into_iter()
                        .chain(warp_songs.into_iter().map(|med| TrackerCellId::warp_song(med)))
                        .enumerate()
                        .map(|(idx, id)| CellLayout { idx: idx + 22, id, pos: [idx as u16 % 6 * 60 + 5, idx as u16 / 6 * 60 + 153], size: [50, 50] })
                    )
                    .collect()
            }
            TrackerLayout::MultiworldExpanded => columns!(4, [
                SwordCard, Slingshot, Skulltula, GoBk,
                Bombs, Bow, ZeldasLullaby, Minuet,
                Boomerang, Hammer, EponasSong, Bolero,
                Hookshot, Spells, SariasSong, Serenade,
                Bottle, Arrows, SunsSong, Requiem,
                MirrorShield, Strength, SongOfTime, Nocturne,
                Boots, Scale, SongOfStorms, Prelude,
            ]),
            TrackerLayout::MultiworldCollapsed => columns!(10, [
                SwordCard, Bottle, Skulltula, Strength, Scale, Spells, Slingshot, Bombs, Boomerang, GoBk,
                ZeldasLullaby, EponasSong, SariasSong, SunsSong, SongOfTime, SongOfStorms, Hookshot, Bow, Hammer, Magic,
                Minuet, Bolero, Serenade, Requiem, Nocturne, Prelude, MirrorShield, Boots, Arrows, Tunics, //TODO replace tunics with wallets once images exist
            ]),
            TrackerLayout::MultiworldEdit => vec![
                KokiriEmeraldLocation, GoronRubyLocation, ZoraSapphireLocation, LightMedallionLocation, ForestMedallionLocation, FireMedallionLocation, WaterMedallionLocation, ShadowMedallionLocation, SpiritMedallionLocation,
            ].into_iter().enumerate().map(|(idx, id)| CellLayout { idx, id, pos: [idx as u16 * 40 + 5, 5], size: [30, 10] }).chain(vec![
                KokiriEmerald, GoronRuby, ZoraSapphire, LightMedallion, ForestMedallion, FireMedallion, WaterMedallion, ShadowMedallion, SpiritMedallion,
            ].into_iter().enumerate().map(|(idx, id)| CellLayout { idx: idx + 9, id, pos: [idx as u16 * 40 + 5, 25], size: [30, 30] })).chain(vec![
                SwordCard, Bottle, Skulltula, Scale, Tunics, GoBk, //TODO replace tunics with wallets once images exist
                Slingshot, Bombs, Boomerang, Strength, Magic, Spells,
                Hookshot, Bow, Arrows, Hammer, Boots, MirrorShield,
                ZeldasLullaby, EponasSong, SariasSong, SunsSong, SongOfTime, SongOfStorms,
                Minuet, Bolero, Serenade, Requiem, Nocturne, Prelude,
            ].into_iter().enumerate().map(|(idx, id)| CellLayout { idx: idx + 18, id, pos: [idx as u16 % 6 * 60 + 5, idx as u16 / 6 * 60 + 65], size: [50, 50] })).collect(),
            TrackerLayout::RslLeft => columns!(9, [
                Slingshot, Bombs, Boomerang, Skulltula, GoMode, GanonMq, GanonKeys, DekuMq, Blank,
                Hookshot, Bow, Hammer, ZeldasLullaby, Minuet, ForestMq, ForestKeys, DcMq, Blank,
                Bottle, Strength, Scale, EponasSong, Bolero, FireMq, FireKeys, JabuMq, Blank,
                ChildTrade, Beans, SwordCard, SariasSong, Serenade, WaterMq, WaterKeys, IceMq, Blank,
                AdultTrade, Tunics, Triforce, SunsSong, Requiem, SpiritMq, SpiritKeys, WellMq, WellSmallKeys,
                MagicLens, Spells, Arrows, SongOfTime, Nocturne, ShadowMq, ShadowKeys, FortressMq, FortressSmallKeys,
                MirrorShield, Boots, Ocarina, SongOfStorms, Prelude, FreeReward, Blank, GtgMq, GtgSmallKeys,
            ]),
            TrackerLayout::RslRight => TrackerLayout::RslLeft.cells()
                .into_iter()
                .chunks(9)
                .into_iter()
                .enumerate()
                .flat_map(|(row_idx, row)| row.collect_vec()
                    .into_iter()
                    .rev()
                    .enumerate()
                    .map(move |(col_idx, CellLayout { id, size, .. })| CellLayout { idx: row_idx * 9 + col_idx, id, pos: [col_idx as u16 * 60 + 5, row_idx as u16 * 60 + 5], size })
                )
                .collect(),
            TrackerLayout::RslEdit => {
                let mut cells = TrackerLayout::MultiworldEdit.cells();
                cells[23].id = GoMode; // unlike multiworld, RSL doesn't track BK mode
                let num_cells_mw = cells.len();
                cells.extend(vec![
                    ForestMq, FireMq, WaterMq, SpiritMq, ShadowMq, GanonMq,
                    ForestKeys, FireKeys, WaterKeys, SpiritKeys, ShadowKeys, GanonKeys,
                    DekuMq, DcMq, JabuMq, WellMq, FortressMq, GtgMq,
                    ChildTrade, Beans, IceMq, WellSmallKeys, FortressSmallKeys, GtgSmallKeys,
                    AdultTrade, Triforce, Ocarina, Blank, Blank, Blank,
                ].into_iter().enumerate().map(|(idx, id)| CellLayout { idx: idx + num_cells_mw, id, pos: [idx as u16 % 6 * 60 + 5, idx as u16 / 6 * 60 + 5], size: [50, 50] }));
                cells
            }
        }
    }
}

impl Default for TrackerLayout {
    fn default() -> TrackerLayout { TrackerLayout::from(&Config::default()) }
}

impl<'a> From<&'a Config> for TrackerLayout {
    fn from(config: &Config) -> TrackerLayout {
        TrackerLayout::Default {
            auto: false,
            meds: config.med_order,
            warp_songs: config.warp_song_order,
        }
    }
}

impl<'a> From<&'a Option<Config>> for TrackerLayout {
    fn from(config: &Option<Config>) -> TrackerLayout {
        config.as_ref().map(TrackerLayout::from).unwrap_or_default()
    }
}

impl fmt::Display for TrackerLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrackerLayout::Default { .. } if *self == TrackerLayout::default() => write!(f, "default"),
            TrackerLayout::Default { .. } => unimplemented!(), //TODO
            TrackerLayout::MultiworldExpanded => write!(f, "mw-expanded"),
            TrackerLayout::MultiworldCollapsed => write!(f, "mw-collapsed"),
            TrackerLayout::MultiworldEdit => write!(f, "mw-edit"),
            TrackerLayout::RslLeft => write!(f, "rsl-left"),
            TrackerLayout::RslRight => write!(f, "rsl-right"),
            TrackerLayout::RslEdit => write!(f, "rsl-edit"),
        }
    }
}

impl<'a> FromParam<'a> for TrackerLayout {
    type Error = ();

    fn from_param(param: &'a str) -> Result<TrackerLayout, ()> {
        Ok(match param {
            "default" => TrackerLayout::default(),
            //TODO parse Default variant with custom fields
            "mw-expanded" => TrackerLayout::MultiworldExpanded,
            "mw-collapsed" => TrackerLayout::MultiworldCollapsed,
            "mw-edit" => TrackerLayout::MultiworldEdit,
            "rsl-left" => TrackerLayout::RslLeft,
            "rsl-right" => TrackerLayout::RslRight,
            "rsl-edit" => TrackerLayout::RslEdit,
            _ => return Err(()),
        })
    }
}

rocket::http::impl_from_uri_param_identity!([Path] TrackerLayout);

impl UriDisplay<Path> for TrackerLayout {
    fn fmt(&self, f: &mut Formatter<'_, Path>) -> fmt::Result {
        f.write_raw(format!("{}", self))
    }
}

/// A layout for a tracker displaying data from two players at once.
///
/// Used in the web app for more compact dungeon reward layouts on restreams.
#[derive(Protocol)]
pub enum DoubleTrackerLayout {
    DungeonRewards,
}

impl DoubleTrackerLayout {
    pub fn cells(&self) -> Vec<DungeonReward> {
        match self {
            DoubleTrackerLayout::DungeonRewards => vec![
                DungeonReward::Stone(Stone::KokiriEmerald),
                DungeonReward::Stone(Stone::GoronRuby),
                DungeonReward::Stone(Stone::ZoraSapphire),
                DungeonReward::Medallion(Medallion::Forest),
                DungeonReward::Medallion(Medallion::Fire),
                DungeonReward::Medallion(Medallion::Water),
                DungeonReward::Medallion(Medallion::Shadow),
                DungeonReward::Medallion(Medallion::Spirit),
                DungeonReward::Medallion(Medallion::Light),
            ],
        }
    }
}

impl<'a> FromParam<'a> for DoubleTrackerLayout {
    type Error = ();

    fn from_param(param: &'a str) -> Result<DoubleTrackerLayout, ()> {
        Ok(match param {
            "dungeon-rewards" => DoubleTrackerLayout::DungeonRewards,
            _ => return Err(()),
        })
    }
}

impl fmt::Display for DoubleTrackerLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DoubleTrackerLayout::DungeonRewards => write!(f, "dungeon-rewards"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Protocol)]
pub enum CellStyle {
    Normal,
    Dimmed,
    LeftDimmed,
    RightDimmed,
}

impl CellStyle {
    fn css_class(&self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Dimmed => "dimmed",
            Self::LeftDimmed => "left-dimmed",
            Self::RightDimmed => "right-dimmed",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Protocol)]
pub enum CellOverlay {
    None,
    Count {
        count: u8,
        count_img: ImageInfo,
    },
    Image(ImageInfo),
    Location {
        loc: ImageInfo,
        style: LocationStyle,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Protocol)]
pub enum LocationStyle {
    Normal,
    Dimmed,
    Mq,
}

impl LocationStyle {
    fn css_classes(&self) -> &'static str {
        match self {
            Self::Normal => "loc",
            Self::Dimmed => "loc dimmed",
            Self::Mq => "loc mq",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Protocol)]
pub struct CellRender {
    pub img: ImageInfo,
    pub style: CellStyle,
    pub overlay: CellOverlay,
}

impl RenderOnce for CellRender {
    fn render_once<'a>(self, tmpl: &mut horrorshow::TemplateBuffer<'a>) {
        self.render(tmpl);
    }
}

impl RenderMut for CellRender {
    fn render_mut<'a>(&mut self, tmpl: &mut horrorshow::TemplateBuffer<'a>) {
        self.render(tmpl);
    }
}

impl Render for CellRender {
    fn render<'a>(&self, tmpl: &mut horrorshow::TemplateBuffer<'a>) {
        (&mut *tmpl) << html! {
            img(class = self.style.css_class(), src = format_args!("/static/img/{}.png", self.img.to_string('/', ImageDirContext::Normal)));
        };
        match self.overlay {
            CellOverlay::None => {}
            CellOverlay::Count { count, .. } => tmpl << html! {
                span(class = "count") : count;
            },
            CellOverlay::Image(ref overlay) => tmpl << html! {
                img(src = format_args!("/static/img/{}.png", overlay.to_string('/', ImageDirContext::OverlayOnly)));
            },
            CellOverlay::Location { ref loc, style } => tmpl << html! {
                img(class = style.css_classes(), src = format_args!("/static/img/{}.png", loc.to_string('/', ImageDirContext::Normal)));
            },
        }
    }
}

fn default_med_order() -> ElementOrder { ElementOrder::LightShadowSpirit }
fn default_warp_song_order() -> ElementOrder { ElementOrder::SpiritShadowLight }

pub fn dirs() -> Result<ProjectDirs, Error> {
    ProjectDirs::from("net", "Fenhl", "OoT Tracker").ok_or(Error::MissingHomeDir)
}

pub enum ImageDirContext {
    Normal,
    Count(u8),
    Dimmed,
    OverlayOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Protocol)]
pub enum ImageDir {
    Xopar,
    Extra,
}

impl ImageDir {
    pub fn to_string(&self, ctx: ImageDirContext) -> &'static str {
        match (self, ctx) {
            (ImageDir::Xopar, ImageDirContext::Normal) => "xopar-images",
            (ImageDir::Extra, ImageDirContext::Normal) => "extra-images",
            (ImageDir::Xopar, ImageDirContext::Count(_)) => "xopar-images-count",
            (ImageDir::Extra, ImageDirContext::Count(_)) => "extra-images-count",
            (ImageDir::Xopar, ImageDirContext::Dimmed) => "xopar-images-dimmed",
            (ImageDir::Extra, ImageDirContext::Dimmed) => "extra-images-dimmed",
            (ImageDir::Xopar, ImageDirContext::OverlayOnly) => "xopar-overlays",
            (ImageDir::Extra, ImageDirContext::OverlayOnly) => "extra-overlays",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Protocol)]
pub struct ImageInfo {
    pub dir: ImageDir,
    pub name: Cow<'static, str>,
}

impl ImageInfo {
    pub fn new(name: impl Into<Cow<'static, str>>) -> ImageInfo {
        ImageInfo { dir: ImageDir::Xopar, name: name.into() }
    }

    pub fn extra(name: impl Into<Cow<'static, str>>) -> ImageInfo {
        ImageInfo { dir: ImageDir::Extra, name: name.into() }
    }

    pub fn embedded<T: FromEmbeddedImage>(&self, ctx: ImageDirContext) -> T {
        match (self.dir, ctx) {
            (ImageDir::Xopar, ImageDirContext::Normal) => images::xopar_images(&self.name),
            (ImageDir::Extra, ImageDirContext::Normal) => images::extra_images(&self.name),
            (ImageDir::Xopar, ImageDirContext::Count(count)) => images::xopar_images_count(&format!("{}_{}", self.name, count)),
            (ImageDir::Extra, ImageDirContext::Count(count)) => images::extra_images_count(&format!("{}_{}", self.name, count)),
            (ImageDir::Xopar, ImageDirContext::Dimmed) => images::xopar_images_dimmed(&self.name),
            (ImageDir::Extra, ImageDirContext::Dimmed) => images::extra_images_dimmed(&self.name),
            (ImageDir::Xopar, ImageDirContext::OverlayOnly) => images::xopar_overlays(&self.name),
            (ImageDir::Extra, ImageDirContext::OverlayOnly) => images::extra_overlays(&self.name),
        }
    }

    pub fn to_string(&self, sep: char, ctx: ImageDirContext) -> String {
        format!("{}{}{}", self.dir.to_string(ctx), sep, self.name)
    }

    pub fn with_overlay(&self, overlay: &ImageInfo) -> OverlayImageInfo {
        OverlayImageInfo {
            dir: if self.dir == ImageDir::Xopar && overlay.dir == ImageDir::Xopar { ImageDir::Xopar } else { ImageDir::Extra },
            main: self.name.clone(),
            overlay: overlay.name.clone(),
        }
    }
}

pub struct OverlayImageInfo {
    dir: ImageDir,
    main: Cow<'static, str>,
    overlay: Cow<'static, str>,
}

impl OverlayImageInfo {
    pub fn embedded<T: FromEmbeddedImage>(&self, main_active: bool) -> T {
        (match (self.dir, main_active) {
            (ImageDir::Xopar, false) => images::xopar_images_overlay_dimmed,
            (ImageDir::Xopar, true) => images::xopar_images_overlay,
            (ImageDir::Extra, false) => images::extra_images_overlay_dimmed,
            (ImageDir::Extra, true) => images::extra_images_overlay,
        })(&format!("{}_{}", self.main, self.overlay))
    }

    pub fn to_string(&self, sep: char, main_active: bool) -> String {
        format!(
            "{}-images-overlay{}{}{}_{}",
            match self.dir { ImageDir::Xopar => "xopar", ImageDir::Extra => "extra" },
            if main_active { "" } else { "-dimmed" },
            sep,
            self.main,
            self.overlay,
        )
    }
}

pub trait FromEmbeddedImage {
    fn from_embedded_image(contents: &'static [u8]) -> Self;
}

impl FromEmbeddedImage for iced::widget::Image {
    fn from_embedded_image(contents: &'static [u8]) -> iced::widget::Image {
        iced::widget::Image::new(iced::image::Handle::from_memory(contents.to_vec()))
    }
}

impl FromEmbeddedImage for DynamicImage {
    fn from_embedded_image(contents: &'static [u8]) -> DynamicImage {
        image::load_from_memory(contents).expect("failed to load embedded DynamicImage")
    }
}

pub mod images {
    use super::FromEmbeddedImage;

    oottracker_derive::embed_images!("assets/img/extra-images");
    oottracker_derive::embed_images!("assets/img/extra-images-count");
    oottracker_derive::embed_images!("assets/img/extra-images-dimmed");
    oottracker_derive::embed_images!("assets/img/extra-images-overlay");
    oottracker_derive::embed_images!("assets/img/extra-images-overlay-dimmed");
    oottracker_derive::embed_images!("assets/img/extra-overlays");
    oottracker_derive::embed_images!("assets/img/xopar-images");
    oottracker_derive::embed_images!("assets/img/xopar-images-count");
    oottracker_derive::embed_images!("assets/img/xopar-images-dimmed");
    oottracker_derive::embed_images!("assets/img/xopar-images-overlay");
    oottracker_derive::embed_images!("assets/img/xopar-images-overlay-dimmed");
    oottracker_derive::embed_images!("assets/img/xopar-overlays");
    oottracker_derive::embed_image!("assets/icon.ico");
}
