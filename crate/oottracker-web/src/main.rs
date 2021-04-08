#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![allow(unused_extern_crates)] // apparently rocket-derive still uses `extern crate`
#![forbid(unsafe_code)]

use {
    std::{
        collections::HashMap,
        convert::TryInto as _,
    },
    itertools::Itertools as _,
    rocket::{
        State,
        response::{
            Redirect,
            content::Html,
            status::NotFound,
        },
    },
    rocket_contrib::serve::{
        StaticFiles,
        crate_relative,
    },
    structopt::StructOpt,
    tokio::sync::Mutex,
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
            TrackerCellId,
            TrackerCellKind::{
                self,
                *,
            },
        },
    },
    crate::restream::{
        DoubleTrackerLayout,
        RestreamState,
        TrackerLayout,
    },
};

mod restream;

trait TrackerCellKindExt {
    fn render(&self, state: &dyn ModelStateView) -> String;
    fn click(&self, state: &mut dyn ModelStateView);
}

impl TrackerCellKindExt for TrackerCellKind {
    fn render(&self, state: &dyn ModelStateView) -> String {
        match self {
            Composite { left_img, right_img, both_img, active, .. } => {
                let is_active = active(state);
                let img_filename = match is_active {
                    (false, false) | (true, true) => both_img,
                    (false, true) => right_img,
                    (true, false) => left_img,
                };
                let css_classes = if let (false, false) = is_active { "dimmed" } else { "" };
                format!(r#"<img class="{}" src="/static/img/xopar-images/{}.png" />"#, css_classes, img_filename)
            }
            Count { dimmed_img, get, .. } => {
                let count = get(state);
                if count == 0 {
                    format!(r#"<img class="dimmed" src="/static/img/xopar-images/{}.png" />"#, dimmed_img)
                } else {
                    format!(r#"
                        <img src="/static/img/xopar-images/{}.png" />
                        <span class="count">{}</span>
                    "#, dimmed_img, count)
                }
            }
            Medallion(med) => {
                let img_filename = format!("{}_medallion", med.element().to_ascii_lowercase());
                let css_classes = if state.ram().save.quest_items.has(*med) { "" } else { "dimmed" };
                format!(r#"<img class="{}" src="/static/img/xopar-images/{}.png" />"#, css_classes, img_filename)
            }
            MedallionLocation(med) => {
                let location = state.knowledge().dungeon_reward_locations.get(&DungeonReward::Medallion(*med));
                let img_filename = match location {
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
                };
                let css_classes = if location.is_some() { "" } else { "dimmed" };
                format!(r#"<img class="{}" src="/static/img/xopar-images/{}.png" />"#, css_classes, img_filename)
            }
            OptionalOverlay { main_img, overlay_img, active, .. } | Overlay { main_img, overlay_img, active, .. } => {
                let (main_active, overlay_active) = active(state);
                if overlay_active {
                    let css_classes = if main_active { "" } else { "dimmed" };
                    format!(r#"
                        <img class="{}" src="/static/img/xopar-images/{}.png" />
                        <img src="/static/img/xopar-overlays/{}.png" />
                    "#, css_classes, main_img, overlay_img)
                } else {
                    let css_classes = if main_active { "" } else { "dimmed" };
                    format!(r#"<img class="{}" src="/static/img/xopar-images/{}.png" />"#, css_classes, main_img)
                }
            }
            Sequence { img, .. } => {
                let (is_active, img_filename) = img(state);
                let css_classes = if is_active { "" } else { "dimmed" };
                format!(r#"<img class="{}" src="/static/img/xopar-images/{}.png" />"#, css_classes, img_filename)
            }
            Simple { img, active, .. } => {
                let css_classes = if active(state) { "" } else { "dimmed" };
                format!(r#"<img class="{}" src="/static/img/xopar-images/{}.png" />"#, css_classes, img)
            }
            Song { song, check, .. } => {
                let song_filename = match *song {
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
                };
                if Check::<ootr_static::Rando>::Location(check.to_string()).checked(state).unwrap_or(false) { //TODO allow ootr_dynamic::Rando
                    let css_classes = if state.ram().save.quest_items.contains(*song) { "" } else { "dimmed" };
                    format!(r#"
                        <img class="{}" src="/static/img/xopar-images/{}.png" />
                        <img src="/static/img/xopar-overlays/check.png" />
                    "#, css_classes, song_filename)
                } else {
                    let css_classes = if state.ram().save.quest_items.contains(*song) { "" } else { "dimmed" };
                    format!(r#"<img class="{}" src="/static/img/xopar-images/{}.png" />"#, css_classes, song_filename)
                }
            }
            Stone(stone) => {
                let stone_filename = match *stone {
                    Stone::KokiriEmerald => "kokiri_emerald",
                    Stone::GoronRuby => "goron_ruby",
                    Stone::ZoraSapphire => "zora_sapphire",
                };
                let css_classes = if state.ram().save.quest_items.has(*stone) { "" } else { "dimmed" };
                format!(r#"<img class="{}" src="/static/img/xopar-images/{}.png" />"#, css_classes, stone_filename)
            }
            StoneLocation(stone) => {
                let location = state.knowledge().dungeon_reward_locations.get(&DungeonReward::Stone(*stone));
                let img_filename = match location {
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
                };
                let css_classes = if location.is_some() { "" } else { "dimmed" };
                format!(r#"<img class="{}" src="/static/img/xopar-images/{}.png" />"#, css_classes, img_filename)
            }
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

trait TrackerCellIdExt {
    fn view(&self, room_name: &str, cell_id: u8, state: &dyn ModelStateView, colspan: u8, loc: bool) -> String;
}

impl TrackerCellIdExt for TrackerCellId {
    fn view(&self, room_name: &str, cell_id: u8, state: &dyn ModelStateView, colspan: u8, loc: bool) -> String {
        let content = self.kind().render(state);
        let css_classes = if loc { format!("cols{} loc", colspan) } else { format!("cols{}", colspan) };
        format!(r#"<a href="/{}/click/{}" class="{}">{}</a>"#, room_name, cell_id, css_classes, content) //TODO click action for JS, put link in a noscript tag
    }
}

fn double_view(restream: &mut RestreamState, runner1: &str, runner2: &str, reward: DungeonReward) -> Option<String> {
    let img_filename = match reward {
        DungeonReward::Medallion(med) => format!("{}_medallion", med.element().to_ascii_lowercase()),
        DungeonReward::Stone(Stone::KokiriEmerald) => format!("kokiri_emerald"),
        DungeonReward::Stone(Stone::GoronRuby) => format!("goron_ruby"),
        DungeonReward::Stone(Stone::ZoraSapphire) => format!("zora_sapphire"),
    };
    let runner1_has = restream.runner(runner1)?.ram().save.quest_items.has(reward);
    let runner2 = restream.runner(runner2)?;
    let css_classes = match (runner1_has, runner2.ram().save.quest_items.has(reward)) {
        (false, false) => "dimmed",
        (false, true) => "left-dimmed",
        (true, false) => "right-dimmed",
        (true, true) => "",
    };
    let location = runner2.knowledge().dungeon_reward_locations.get(&reward);
    let loc_img_filename = match location {
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
    };
    let loc_css_classes = if location.is_some() { "" } else { "dimmed" };
    Some(format!(r#"<div class="cols3">
        <img class="{}" src="/static/img/xopar-images/{}.png" />
        <img class="loc {}" src="/static/img/xopar-images/{}.png" />
    </div>"#, css_classes, img_filename, loc_css_classes, loc_img_filename)) //TODO overlay with location knowledge
}

fn tracker_page(layout_name: &str, html_layout: String) -> Html<String> {
    Html(format!(r#"
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="utf-8" />
                <title>OoT Tracker</title>
                <meta name="author" content="Fenhl" /> <!--TODO generate from Cargo.toml? -->
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <!--TODO favicon -->
                <link rel="stylesheet" href="/static/common.css" />
            </head>
            <body>
                <div class="items {}">
                    {}
                </div>
                <footer>
                    <a href="https://fenhl.net/disc">disclaimer / Impressum</a>
                </footer>
            </body>
        </html>
    "#, layout_name, html_layout))
}

#[rocket::get("/")]
fn index() -> Html<String> {
    Html(format!(include_str!("../../../assets/web/index.html"), env!("CARGO_PKG_VERSION")))
}

#[rocket::get("/restream/<restreamer>/<runner>")]
async fn restream_room_input(restreams: State<'_, Mutex<HashMap<String, RestreamState>>>, restreamer: String, runner: String) -> Option<Html<String>> {
    let html_layout = {
        let mut restreams = restreams.lock().await;
        let restream = restreams.get_mut(&restreamer)?;
        let layout = restream.layout();
        let model_state_view = restream.runner(&runner)?;
        let pseudo_name = format!("restream/{}/{}/{}", restreamer, runner, layout);
        layout.cells()
            .enumerate()
            .map(|(cell_id, (cell, colspan, loc))| cell.view(&pseudo_name, cell_id.try_into().expect("too many cells"), &model_state_view, colspan, loc))
            .join("\n")
    };
    Some(tracker_page("default", html_layout))
}

#[rocket::get("/restream/<restreamer>/<runner>/<layout>")]
async fn restream_room_view(restreams: State<'_, Mutex<HashMap<String, RestreamState>>>, restreamer: String, runner: String, layout: TrackerLayout) -> Option<Html<String>> {
    let html_layout = {
        let mut restreams = restreams.lock().await;
        let restream = restreams.get_mut(&restreamer)?;
        let model_state_view = restream.runner(&runner)?;
        let pseudo_name = format!("restream/{}/{}/{}", restreamer, runner, layout);
        layout.cells()
            .enumerate()
            .map(|(cell_id, (cell, colspan, loc))| cell.view(&pseudo_name, cell_id.try_into().expect("too many cells"), &model_state_view, colspan, loc))
            .join("\n")
    };
    Some(tracker_page(&layout.to_string(), html_layout))
}

#[rocket::get("/restream/<restreamer>/<runner>/<layout>/click/<cell_id>")]
async fn restream_click(restreams: State<'_, Mutex<HashMap<String, RestreamState>>>, restreamer: String, runner: String, layout: TrackerLayout, cell_id: u8) -> Result<Redirect, NotFound<&'static str>> {
    {
        let mut restreams = restreams.lock().await;
        let restream = restreams.get_mut(&restreamer).ok_or(NotFound("No such restream"))?;
        let mut model_state_view = restream.runner(&runner).ok_or(NotFound("No such runner"))?;
        layout.cells().nth(cell_id.into()).ok_or(NotFound("No such cell"))?.0.kind().click(&mut model_state_view);
    }
    Ok(Redirect::to(rocket::uri!(restream_room_view: restreamer, runner, layout)))
}

#[rocket::get("/restream/<restreamer>/<runner1>/<layout>/with/<runner2>")]
async fn restream_double_room_layout(restreams: State<'_, Mutex<HashMap<String, RestreamState>>>, restreamer: String, runner1: String, layout: DoubleTrackerLayout, runner2: String) -> Option<Html<String>> {
    let html_layout = {
        let mut restreams = restreams.lock().await;
        let restream = restreams.get_mut(&restreamer)?;
        layout.cells()
            .into_iter()
            .map(|reward| double_view(restream, &runner1, &runner2, reward))
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .join("\n")
    };
    Some(tracker_page(&layout.to_string(), html_layout))
}

#[rocket::get("/room/<name>")]
async fn room(rooms: State<'_, Mutex<HashMap<String, ModelState>>>, name: String) -> Html<String> {
    let html_layout = {
        let mut rooms = rooms.lock().await;
        let room = rooms.entry(name.clone()).or_default();
        let layout = TrackerLayout::default();
        layout.cells()
            .enumerate()
            .map(|(cell_id, (cell, colspan, loc))| cell.view(&name, cell_id.try_into().expect("too many cells"), room, colspan, loc))
            .join("\n")
    };
    tracker_page("default", html_layout)
}

#[rocket::get("/room/<name>/click/<cell_id>")]
async fn click(rooms: State<'_, Mutex<HashMap<String, ModelState>>>, name: String, cell_id: u8) -> Result<Redirect, NotFound<&'static str>> {
    {
        let mut rooms = rooms.lock().await;
        let room = rooms.entry(name.clone()).or_default();
        let layout = TrackerLayout::default();
        layout.cells().nth(cell_id.into()).ok_or(NotFound("No such cell"))?.0.kind().click(room);
    }
    Ok(Redirect::to(rocket::uri!(room: name)))
}

#[derive(StructOpt)]
struct Args {} // for --help/--version support

#[wheel::main]
async fn main(Args {}: Args) -> Result<(), rocket::error::Error> {
    rocket::custom(rocket::Config {
        port: 24807,
        //TODO configure secret_key for release mode
        ..rocket::Config::default()
    })
    .manage(Mutex::<HashMap<String, ModelState>>::default())
    .manage({
        //TODO remove hardcoded restream, allow configuring active restreams somehow
        let mut map = HashMap::new();
        let multiworld_3v3 = vec![
            (Knowledge::default(), vec![(format!("a1"), Ram::default()), (format!("b1"), Ram::default())].into_iter().collect()),
            (Knowledge::default(), vec![(format!("a2"), Ram::default()), (format!("b2"), Ram::default())].into_iter().collect()),
            (Knowledge::default(), vec![(format!("a3"), Ram::default()), (format!("b3"), Ram::default())].into_iter().collect()),
        ];
        map.insert(format!("fenhl"), RestreamState::new(multiworld_3v3));
        Mutex::new(map)
    })
    .mount("/static", StaticFiles::new(crate_relative!("../../assets/web/static"), rocket_contrib::serve::Options::None))
    .mount("/", rocket::routes![
        index,
        restream_room_input,
        restream_room_view,
        restream_click,
        restream_double_room_layout,
        room,
        click,
    ])
    .launch().await
}
