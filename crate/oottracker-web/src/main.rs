#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![allow(unused_extern_crates)] // apparently rocket-derive still uses `extern crate`
#![forbid(unsafe_code)]

use {
    std::{
        borrow::Cow,
        collections::HashMap,
        fmt,
        sync::Arc,
    },
    async_proto::{
        Protocol,
        ReadError,
    },
    derive_more::From,
    futures::future::{
        FutureExt as _,
        TryFutureExt as _,
    },
    structopt::StructOpt,
    tokio::sync::Mutex,
    warp::Filter as _,
    ootr::{
        check::Check,
        model::{
            DungeonReward,
            DungeonRewardLocation,
            MainDungeon,
            Stone,
        },
    },
    oottracker::{
        Knowledge,
        ModelState,
        ModelStateView,
        Ram,
        checks::CheckExt as _,
        save::QuestItems,
        ui::{
            DungeonRewardLocationExt as _,
            ImageDirContext,
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

type Restreams = Arc<Mutex<HashMap<String, RestreamState>>>;
type Rooms = Arc<Mutex<HashMap<String, ModelState>>>;

#[derive(Clone, Copy, PartialEq, Eq, Protocol)]
enum CellStyle {
    Normal,
    Dimmed,
    LeftDimmed,
    RightDimmed,
}

#[derive(Clone, PartialEq, Eq, Protocol)]
enum CellOverlay {
    None,
    Count(u8),
    Image {
        overlay_dir: Cow<'static, str>,
        overlay_img: Cow<'static, str>,
    },
    Location {
        dimmed: bool,
        loc_img: Cow<'static, str>,
    },
}

#[derive(Clone, PartialEq, Eq, Protocol)]
struct CellRender {
    img_dir: Cow<'static, str>,
    img_filename: Cow<'static, str>,
    style: CellStyle,
    overlay: CellOverlay,
}

impl CellRender {
    fn to_html(&self) -> String {
        let css_classes = match self.style {
            CellStyle::Normal => "",
            CellStyle::Dimmed => "dimmed",
            CellStyle::LeftDimmed => "left-dimmed",
            CellStyle::RightDimmed => "right-dimmed",
        };
        let overlay = match self.overlay {
            CellOverlay::None => String::default(),
            CellOverlay::Count(count) => format!(r#"<span class="count">{}</span>"#, count),
            CellOverlay::Image { ref overlay_dir, ref overlay_img } => format!(r#"<img src="/static/img/{}/{}.png" />"#, overlay_dir, overlay_img),
            CellOverlay::Location { dimmed, ref loc_img } => format!(r#"<img class="loc{}" src="/static/img/xopar-images/{}.png" />"#, if dimmed { " dimmed" } else { "" }, loc_img),
        };
        format!(r#"<img class="{}" src="/static/img/{}/{}.png" />{}"#, css_classes, self.img_dir, self.img_filename, overlay)
    }
}

trait TrackerCellKindExt {
    fn render(&self, state: &dyn ModelStateView) -> CellRender;
    fn click(&self, state: &mut dyn ModelStateView);
}

impl TrackerCellKindExt for TrackerCellKind {
    fn render(&self, state: &dyn ModelStateView) -> CellRender {
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
            Count { dimmed_img, get, .. } => {
                let count = get(state);
                let (style, overlay) = if count == 0 { (CellStyle::Dimmed, CellOverlay::None) } else { (CellStyle::Normal, CellOverlay::Count(count)) };
                CellRender { img_dir: Cow::Borrowed(dimmed_img.dir.to_string(ImageDirContext::Normal)), img_filename: Cow::Borrowed(dimmed_img.name), style, overlay }
            }
            Medallion(med) => CellRender {
                img_dir: Cow::Borrowed("xopar-images"),
                img_filename: Cow::Owned(format!("{}_medallion", med.element().to_ascii_lowercase())),
                style: if state.ram().save.quest_items.has(*med) { CellStyle::Normal } else { CellStyle::Dimmed },
                overlay: CellOverlay::None,
            },
            MedallionLocation(med) => {
                let location = state.knowledge().dungeon_reward_locations.get(&DungeonReward::Medallion(*med));
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
                style: if state.ram().save.quest_items.contains(*song) { CellStyle::Normal } else { CellStyle::Dimmed },
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
                style: if state.ram().save.quest_items.has(*stone) { CellStyle::Normal } else { CellStyle::Dimmed },
                overlay: CellOverlay::None,
            },
            StoneLocation(stone) => {
                let location = state.knowledge().dungeon_reward_locations.get(&DungeonReward::Stone(*stone));
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
            BigPoeTriforce | BossKey { .. } | FortressMq | Mq(_) | SmallKeys { .. } | SongCheck { .. } => unimplemented!(),
        }
    }

    fn click(&self, state: &mut dyn ModelStateView) {
        match self {
            Composite { active, toggle_left, toggle_right, .. } | Overlay { active, toggle_main: toggle_left, toggle_overlay: toggle_right, .. } => {
                let (left, _) = active(state);
                if left { toggle_right(state) }
                toggle_left(state);
            }
            OptionalOverlay { toggle_main: toggle, .. } | Simple { toggle, .. } => toggle(state),
            Count { get, set, max, .. } => {
                let current = get(state);
                if current == *max { set(state, 0) } else { set(state, current + 1) }
            }
            Medallion(med) => state.ram_mut().save.quest_items.toggle(QuestItems::from(med)),
            MedallionLocation(med) => state.knowledge_mut().dungeon_reward_locations.increment(DungeonReward::Medallion(*med)),
            Sequence { increment, .. } => increment(state),
            Song { song: quest_item, .. } => state.ram_mut().save.quest_items.toggle(*quest_item),
            Stone(stone) => state.ram_mut().save.quest_items.toggle(QuestItems::from(stone)),
            StoneLocation(stone) => state.knowledge_mut().dungeon_reward_locations.increment(DungeonReward::Stone(*stone)),
            BigPoeTriforce | BossKey { .. } | FortressMq | Mq(_) | SmallKeys { .. } | SongCheck { .. } => unimplemented!(),
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
            (Knowledge::default(), vec![(format!("a1"), Ram::default()), (format!("b1"), Ram::default())].into_iter().collect()),
            (Knowledge::default(), vec![(format!("a2"), Ram::default()), (format!("b2"), Ram::default())].into_iter().collect()),
            (Knowledge::default(), vec![(format!("a3"), Ram::default()), (format!("b3"), Ram::default())].into_iter().collect()),
        ];
        map.insert(format!("fenhl"), RestreamState::new(multiworld_3v3));
        Restreams::new(Mutex::new(map))
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
