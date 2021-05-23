#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![allow(unused_extern_crates)] // apparently rocket-derive still uses `extern crate`
#![forbid(unsafe_code)]

use {
    std::{
        borrow::Cow,
        collections::HashMap,
        fmt,
        iter,
        sync::Arc,
    },
    async_proto::ReadError,
    collect_mac::collect,
    derive_more::From,
    futures::future::{
        FutureExt as _,
        TryFutureExt as _,
    },
    itertools::Itertools as _,
    structopt::StructOpt,
    tokio::sync::{
        Mutex,
        RwLock,
        watch::*,
    },
    warp::Filter as _,
    ootr::{
        check::Check,
        model::{
            Dungeon,
            DungeonReward,
            DungeonRewardLocation,
            MainDungeon,
            Stone,
        },
        region::Mq,
    },
    oottracker::{
        ModelState,
        checks::CheckExt as _,
        save::QuestItems,
        ui::{
            CellRender,
            CellOverlay,
            CellStyle,
            DungeonRewardLocationExt as _,
            ImageDirContext,
            LocationStyle,
            TrackerCellKind::{
                self,
                *,
            },
        },
    },
    crate::restream::RestreamState,
};

mod http;
mod restream;
mod websocket;

type Restreams = Arc<RwLock<HashMap<String, RestreamState>>>;
type Rooms = Arc<Mutex<HashMap<String, RoomState>>>;

struct RoomState {
    tx: Sender<()>,
    rx: Receiver<()>,
    model: ModelState,
}

impl Default for RoomState {
    fn default() -> RoomState {
        let (tx, rx) = channel(());
        RoomState {
            tx, rx,
            model: ModelState::default(),
        }
    }
}

trait TrackerCellKindExt {
    fn render(&self, state: &ModelState) -> CellRender;
    fn click(&self, state: &mut ModelState);
    fn left_click(&self, state: &mut ModelState);
    fn right_click(&self, state: &mut ModelState);
}

impl TrackerCellKindExt for TrackerCellKind {
    fn render(&self, state: &ModelState) -> CellRender {
        match self {
            Composite { left_img, right_img, both_img, active, .. } => {
                let is_active = active(state);
                let img = match is_active {
                    (false, false) | (true, true) => both_img,
                    (false, true) => right_img,
                    (true, false) => left_img,
                };
                CellRender {
                    img_dir: Cow::Borrowed(img.dir.to_string(ImageDirContext::Normal)),
                    img_filename: Cow::Borrowed(img.name),
                    style: if let (false, false) = is_active { CellStyle::Dimmed } else { CellStyle::Normal },
                    overlay: CellOverlay::None,
                }
            }
            CompositeKeys { boss, small } => {
                let (has_boss_key, num_small_keys) = if let (BossKey { active, .. }, SmallKeys { get, .. }) = (boss.kind(), small.kind()) {
                    (active(&state.ram.save.boss_keys), get(&state.ram.save.small_keys))
                } else {
                    unimplemented!("CompositeKeys that aren't SmallKeys + BossKey")
                };
                CellRender {
                    img_dir: Cow::Borrowed("extra-images"),
                    img_filename: Cow::Borrowed("keys"),
                    style: match (has_boss_key, num_small_keys) {
                        (false, 0) => CellStyle::Dimmed,
                        (false, _) => CellStyle::LeftDimmed,
                        (true, 0) => CellStyle::RightDimmed,
                        (true, _) => CellStyle::Normal,
                    },
                    overlay: if num_small_keys > 0 { CellOverlay::Count(num_small_keys) } else { CellOverlay::None },
                }
            }
            Count { dimmed_img, get, .. } => {
                let count = get(state);
                let (style, overlay) = if count == 0 { (CellStyle::Dimmed, CellOverlay::None) } else { (CellStyle::Normal, CellOverlay::Count(count)) };
                CellRender { img_dir: Cow::Borrowed(dimmed_img.dir.to_string(ImageDirContext::Normal)), img_filename: Cow::Borrowed(dimmed_img.name), style, overlay }
            }
            FortressMq => {
                CellRender {
                    img_dir: Cow::Borrowed("extra-images"),
                    img_filename: Cow::Borrowed("blank"),
                    style: CellStyle::Normal,
                    overlay: CellOverlay::Location {
                        loc_dir: Cow::Borrowed("extra-images"),
                        loc_img: Cow::Borrowed("fort_text"),
                        style: if state.knowledge.string_settings.get("gerudo_fortress").map_or(false, |values| values.iter().eq(iter::once("normal"))) { LocationStyle::Mq } else { LocationStyle::Normal }, //TODO dim if unknown?
                    },
                }
            }
            FreeReward => {
                let reward = state.knowledge.dungeon_reward_locations.iter()
                    .filter_map(|(reward, &loc)| if loc == DungeonRewardLocation::LinksPocket { Some(reward) } else { None })
                    .exactly_one()
                    .ok();
                CellRender {
                    img_dir: Cow::Borrowed(if reward.is_some() { "xopar-images" } else { "extra-images" }),
                    img_filename: match reward {
                        Some(DungeonReward::Medallion(med)) => Cow::Owned(format!("{}_medallion", med.element().to_ascii_lowercase())),
                        Some(DungeonReward::Stone(Stone::KokiriEmerald)) => Cow::Borrowed("kokiri_emerald"),
                        Some(DungeonReward::Stone(Stone::GoronRuby)) => Cow::Borrowed("goron_ruby"),
                        Some(DungeonReward::Stone(Stone::ZoraSapphire)) => Cow::Borrowed("zora_sapphire"),
                        None => Cow::Borrowed("blank"), //TODO “unknown dungeon reward” image?
                    },
                    style: CellStyle::Normal,
                    overlay: CellOverlay::Location {
                        loc_dir: Cow::Borrowed("xopar-images"),
                        loc_img: Cow::Borrowed("free_text"),
                        style: LocationStyle::Normal,
                    },
                }
            }
            Medallion(med) => CellRender {
                img_dir: Cow::Borrowed("xopar-images"),
                img_filename: Cow::Owned(format!("{}_medallion", med.element().to_ascii_lowercase())),
                style: if state.ram.save.quest_items.has(*med) { CellStyle::Normal } else { CellStyle::Dimmed },
                overlay: CellOverlay::None,
            },
            MedallionLocation(med) => {
                let location = state.knowledge.dungeon_reward_locations.get(&DungeonReward::Medallion(*med));
                CellRender {
                    img_dir: Cow::Borrowed("xopar-images"),
                    img_filename: Cow::Borrowed(match location {
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
                    state.knowledge.dungeon_reward_locations.iter()
                        .filter_map(|(reward, &loc)| if loc == DungeonRewardLocation::Dungeon(main_dungeon) { Some(reward) } else { None })
                        .exactly_one()
                        .ok()
                } else {
                    None
                };
                CellRender {
                    img_dir: Cow::Borrowed(if reward.is_some() { "xopar-images" } else { "extra-images" }),
                    img_filename: match reward {
                        Some(DungeonReward::Medallion(med)) => Cow::Owned(format!("{}_medallion", med.element().to_ascii_lowercase())),
                        Some(DungeonReward::Stone(Stone::KokiriEmerald)) => Cow::Borrowed("kokiri_emerald"),
                        Some(DungeonReward::Stone(Stone::GoronRuby)) => Cow::Borrowed("goron_ruby"),
                        Some(DungeonReward::Stone(Stone::ZoraSapphire)) => Cow::Borrowed("zora_sapphire"),
                        None => Cow::Borrowed("blank"), //TODO “unknown dungeon reward” image? (only for dungeons that have rewards)
                    },
                    style: if reward.map_or(false, |&reward| state.ram.save.quest_items.has(reward)) { CellStyle::Normal } else { CellStyle::Dimmed },
                    overlay: CellOverlay::Location {
                        loc_dir: Cow::Borrowed(if let Dungeon::Main(_) = dungeon { "xopar-images" } else { "extra-images" }),
                        loc_img: Cow::Borrowed(match dungeon {
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
                            Dungeon::GerudoTrainingGrounds => "gtg_text",
                            Dungeon::GanonsCastle => "ganon_text",
                        }),
                        style: if state.knowledge.mq.get(dungeon) == Some(&Mq::Mq) { LocationStyle::Mq } else { LocationStyle::Normal },
                    },
                }
            }
            OptionalOverlay { main_img, overlay_img, active, .. } | Overlay { main_img, overlay_img, active, .. } => {
                let (main_active, overlay_active) = active(state);
                CellRender {
                    img_dir: Cow::Borrowed(main_img.dir.to_string(ImageDirContext::Normal)),
                    img_filename: Cow::Borrowed(main_img.name),
                    style: if main_active { CellStyle::Normal } else { CellStyle::Dimmed },
                    overlay: if overlay_active {
                        CellOverlay::Image {
                            overlay_dir: Cow::Borrowed(overlay_img.dir.to_string(ImageDirContext::OverlayOnly)),
                            overlay_img: Cow::Borrowed(overlay_img.name),
                        }
                    } else {
                        CellOverlay::None
                    },
                }
            }
            Sequence { img, .. } => {
                let (is_active, img_filename) = img(state);
                CellRender {
                    img_dir: Cow::Borrowed(img_filename.dir.to_string(ImageDirContext::Normal)),
                    img_filename: Cow::Borrowed(img_filename.name),
                    style: if is_active { CellStyle::Normal } else { CellStyle::Dimmed },
                    overlay: CellOverlay::None,
                }
            }
            Simple { img, active, .. } => CellRender {
                img_dir: Cow::Borrowed(img.dir.to_string(ImageDirContext::Normal)),
                img_filename: Cow::Borrowed(img.name),
                style: if active(state) { CellStyle::Normal } else { CellStyle::Dimmed },
                overlay: CellOverlay::None,
            },
            SmallKeys { get, .. } => {
                let num_small_keys = get(&state.ram.save.small_keys);
                CellRender {
                    img_dir: Cow::Borrowed("extra-images"),
                    img_filename: Cow::Borrowed("small_key"),
                    style: if num_small_keys > 0 { CellStyle::Normal } else { CellStyle::Dimmed },
                    overlay: if num_small_keys > 0 { CellOverlay::Count(num_small_keys) } else { CellOverlay::None },
                }
            },
            Song { song, check, .. } => CellRender {
                img_dir: Cow::Borrowed("xopar-images"),
                img_filename: Cow::Borrowed(match *song {
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
                overlay: if Check::<ootr_static::Rando>::Location(check.to_string()).checked(state).unwrap_or(false) { //TODO allow ootr_dynamic::Rando
                    CellOverlay::Image {
                        overlay_dir: Cow::Borrowed("xopar-overlays"),
                        overlay_img: Cow::Borrowed("check"),
                    }
                } else {
                    CellOverlay::None
                },
            },
            Stone(stone) => CellRender {
                img_dir: Cow::Borrowed("xopar-images"),
                img_filename: Cow::Borrowed(match *stone {
                    Stone::KokiriEmerald => "kokiri_emerald",
                    Stone::GoronRuby => "goron_ruby",
                    Stone::ZoraSapphire => "zora_sapphire",
                }),
                style: if state.ram.save.quest_items.has(*stone) { CellStyle::Normal } else { CellStyle::Dimmed },
                overlay: CellOverlay::None,
            },
            StoneLocation(stone) => {
                let location = state.knowledge.dungeon_reward_locations.get(&DungeonReward::Stone(*stone));
                CellRender {
                    img_dir: Cow::Borrowed("xopar-images"),
                    img_filename: Cow::Borrowed(match location {
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
            BigPoeTriforce | BossKey { .. } | SongCheck { .. } => unimplemented!(),
        }
    }

    fn click(&self, state: &mut ModelState) {
        match self {
            Composite { active, toggle_left, toggle_right, .. } | Overlay { active, toggle_main: toggle_left, toggle_overlay: toggle_right, .. } => {
                let (left, _) = active(state);
                if left { toggle_right(state) }
                toggle_left(state);
            }
            OptionalOverlay { toggle_main: toggle, .. } | Simple { toggle, .. } => toggle(state),
            CompositeKeys { boss, small } => {
                let (toggle_boss, get_small, set_small, max_small_vanilla, max_small_mq) = if let (BossKey { toggle, .. }, SmallKeys { get, set, max_vanilla, max_mq }) = (boss.kind(), small.kind()) {
                    (toggle, get, set, max_vanilla, max_mq)
                } else {
                    unimplemented!("CompositeKeys that aren't SmallKeys + BossKey")
                };
                let num_small = get_small(&state.ram.save.small_keys);
                if num_small == max_small_vanilla.max(max_small_mq) { //TODO check MQ knowledge? Does plentiful go to +1?
                    set_small(&mut state.ram.save.small_keys, 0);
                    toggle_boss(&mut state.ram.save.boss_keys);
                } else {
                    set_small(&mut state.ram.save.small_keys, num_small + 1);
                }
            }
            Count { get, set, max, .. } => {
                let current = get(state);
                set(state, if current == *max { 0 } else { current + 1 });
            }
            FortressMq => if state.knowledge.string_settings.get("gerudo_fortress").map_or(false, |fort| fort.iter().eq(iter::once("normal"))) {
                state.knowledge.string_settings.remove("gerudo_fortress");
            } else {
                state.knowledge.string_settings.insert(format!("gerudo_fortress"), collect![format!("normal")]);
            },
            Medallion(med) => state.ram.save.quest_items.toggle(QuestItems::from(med)),
            MedallionLocation(med) => state.knowledge.dungeon_reward_locations.increment(DungeonReward::Medallion(*med)),
            Mq(dungeon) => if state.knowledge.mq.get(dungeon) == Some(&Mq::Mq) {
                state.knowledge.mq.remove(dungeon);
            } else {
                state.knowledge.mq.insert(*dungeon, Mq::Mq);
            },
            Sequence { increment, .. } => increment(state),
            SmallKeys { get, set, max_vanilla, max_mq } => {
                let num = get(&state.ram.save.small_keys);
                if num == *max_vanilla.max(max_mq) { //TODO check MQ knowledge? Does plentiful go to +1?
                    set(&mut state.ram.save.small_keys, 0);
                } else {
                    set(&mut state.ram.save.small_keys, num + 1);
                }
            }
            Song { song: quest_item, .. } => state.ram.save.quest_items.toggle(*quest_item),
            Stone(stone) => state.ram.save.quest_items.toggle(QuestItems::from(stone)),
            StoneLocation(stone) => state.knowledge.dungeon_reward_locations.increment(DungeonReward::Stone(*stone)),
            FreeReward => {}
            BigPoeTriforce | BossKey { .. } | SongCheck { .. } => unimplemented!(),
        }
    }

    fn left_click(&self, state: &mut ModelState) {
        match self {
            Composite { toggle_left, .. } | Overlay { toggle_main: toggle_left, .. } => toggle_left(state),
            CompositeKeys { boss, .. } => if let BossKey { toggle, .. } = boss.kind() {
                toggle(&mut state.ram.save.boss_keys);
            } else {
                unimplemented!("CompositeKeys that aren't SmallKeys + BossKey")
            },
            _ => self.click(state),
        }
    }

    fn right_click(&self, state: &mut ModelState) {
        match self {
            Composite { toggle_right, .. } | OptionalOverlay { toggle_overlay: toggle_right, .. } | Overlay { toggle_overlay: toggle_right, .. } => toggle_right(state),
            CompositeKeys { small, .. } => if let SmallKeys { get, set, max_vanilla, max_mq } = small.kind() {
                let num = get(&state.ram.save.small_keys);
                if num == max_vanilla.max(max_mq) { //TODO check MQ knowledge? Does plentiful go to +1?
                    set(&mut state.ram.save.small_keys, 0);
                } else {
                    set(&mut state.ram.save.small_keys, num + 1);
                }
            } else {
                unimplemented!("CompositeKeys that aren't SmallKeys + BossKey")
            },
            Count { get, set, max, .. } => {
                let current = get(state);
                set(state, if current == 0 { *max } else { current - 1 });
            }
            MedallionLocation(med) => state.knowledge.dungeon_reward_locations.decrement(DungeonReward::Medallion(*med)),
            Sequence { decrement, .. } => decrement(state),
            SmallKeys { get, set, max_vanilla, max_mq } => {
                let num = get(&state.ram.save.small_keys);
                if num == 0 {
                    set(&mut state.ram.save.small_keys, *max_vanilla.max(max_mq)); //TODO check MQ knowledge? Does plentiful go to +1?
                } else {
                    set(&mut state.ram.save.small_keys, num - 1);
                }
            }
            Song { toggle_overlay, .. } => toggle_overlay(&mut state.ram.save.event_chk_inf),
            StoneLocation(stone) => state.knowledge.dungeon_reward_locations.decrement(DungeonReward::Stone(*stone)),
            FreeReward | FortressMq | Medallion(_) | Mq(_) | Simple { .. } | Stone(_) => {}
            BigPoeTriforce | BossKey { .. } | SongCheck { .. } => unimplemented!(),
        }
    }
}

#[derive(Debug, From)]
enum Error {
    Read(ReadError),
    Rocket(rocket::error::Error),
    Task(tokio::task::JoinError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Read(e) => write!(f, "read error: {}", e),
            Error::Rocket(e) => write!(f, "rocket error: {}", e),
            Error::Task(e) => write!(f, "task error: {}", e),
        }
    }
}

#[derive(StructOpt)]
struct Args {} // for --help/--version support

#[wheel::main]
async fn main(Args {}: Args) -> Result<(), Error> {
    let rooms = Rooms::default();
    let restreams = {
        //TODO remove hardcoded restream, allow configuring active restreams somehow
        let mut map = HashMap::default();
        let multiworld_3v3 = vec![
            vec!["a1", "b1"],
            vec!["a2", "b2"],
            vec!["a3", "b3"],
        ];
        map.insert(format!("fenhl"), RestreamState::new(multiworld_3v3));
        Restreams::new(RwLock::new(map))
    };
    let websocket_task = {
        let rooms = Rooms::clone(&rooms);
        let restreams = Restreams::clone(&restreams);
        let handler = warp::ws().and_then(move |ws| websocket::ws_handler(Rooms::clone(&rooms), Restreams::clone(&restreams), ws));
        tokio::spawn(warp::serve(handler).run(([127, 0, 0, 1], 24808))).err_into()
    };
    let rocket_task = tokio::spawn(http::rocket(rooms, restreams).launch()).map(|res| match res {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(Error::from(e)),
        Err(e) => Err(Error::from(e)),
    });
    let ((), ()) = tokio::try_join!(websocket_task, rocket_task)?;
    Ok(())
}
