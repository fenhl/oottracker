#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![allow(unused_extern_crates)] // apparently rocket-derive still uses `extern crate`
#![forbid(unsafe_code)]

use {
    std::{
        collections::HashMap,
        convert::TryInto as _,
        io,
    },
    itertools::Itertools as _,
    rocket::{
        State,
        response::{
            Debug,
            NamedFile,
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
        ModelState,
        checks::CheckExt as _,
        save::QuestItems,
        ui::{
            DungeonRewardLocationExt as _,
            TrackerCellId,
            TrackerCellKind::{
                self,
                *,
            },
            TrackerLayout,
        },
    },
};

#[rocket::get("/")]
async fn index() -> Result<NamedFile, Debug<io::Error>> {
    Ok(NamedFile::open("assets/web/index.html").await?)
}

trait TrackerCellKindExt {
    fn render(&self, state: &ModelState) -> String;
    fn click(&self, state: &mut ModelState);
}

impl TrackerCellKindExt for TrackerCellKind {
    fn render(&self, state: &ModelState) -> String {
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
                let css_classes = if state.ram.save.quest_items.has(*med) { "" } else { "dimmed" };
                format!(r#"<img class="{}" src="/static/img/xopar-images/{}.png" />"#, css_classes, img_filename)
            }
            MedallionLocation(med) => {
                let location = state.knowledge.dungeon_reward_locations.get(&DungeonReward::Medallion(*med));
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
                if Check::Location(check.to_string()).checked(state).unwrap_or(false) {
                    let css_classes = if state.ram.save.quest_items.contains(*song) { "" } else { "dimmed" };
                    format!(r#"
                        <img class="{}" src="/static/img/xopar-images/{}.png" />
                        <img src="/static/img/xopar-overlays/check.png" />
                    "#, css_classes, song_filename)
                } else {
                    let css_classes = if state.ram.save.quest_items.contains(*song) { "" } else { "dimmed" };
                    format!(r#"<img class="{}" src="/static/img/xopar-images/{}.png" />"#, css_classes, song_filename)
                }
            }
            Stone(stone) => {
                let stone_filename = match *stone {
                    Stone::KokiriEmerald => "kokiri_emerald",
                    Stone::GoronRuby => "goron_ruby",
                    Stone::ZoraSapphire => "zora_sapphire",
                };
                let css_classes = if state.ram.save.quest_items.has(*stone) { "" } else { "dimmed" };
                format!(r#"<img class="{}" src="/static/img/xopar-images/{}.png" />"#, css_classes, stone_filename)
            }
            StoneLocation(stone) => {
                let location = state.knowledge.dungeon_reward_locations.get(&DungeonReward::Stone(*stone));
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
            Count { get, set, max, .. } => {
                let current = get(state);
                if current == *max { set(state, 0) } else { set(state, current + 1) }
            }
            Medallion(med) => state.ram.save.quest_items.toggle(QuestItems::from(med)),
            MedallionLocation(med) => state.knowledge.dungeon_reward_locations.increment(DungeonReward::Medallion(*med)),
            Sequence { increment, .. } => increment(state),
            Song { song: quest_item, .. } => state.ram.save.quest_items.toggle(*quest_item),
            Stone(stone) => state.ram.save.quest_items.toggle(QuestItems::from(stone)),
            StoneLocation(stone) => state.knowledge.dungeon_reward_locations.increment(DungeonReward::Stone(*stone)),
        }
    }
}

trait TrackerCellIdExt {
    fn view(&self, room_name: &str, cell_id: u8, state: &ModelState, colspan: u8, loc: bool) -> String;
}

impl TrackerCellIdExt for TrackerCellId {
    fn view(&self, room_name: &str, cell_id: u8, state: &ModelState, colspan: u8, loc: bool) -> String {
        let content = self.kind().render(state);
        let css_classes = if loc { format!("cols{} loc", colspan) } else { format!("cols{}", colspan) };
        format!(r#"<a href="/{}/click/{}" class="{}">{}</a>"#, room_name, cell_id, css_classes, content) //TODO click action
    }
}

fn cells(layout: TrackerLayout) -> impl Iterator<Item = (TrackerCellId, u8, bool)> {
    layout.meds.into_iter().map(|med| (TrackerCellId::med_location(med), 3, true))
        .chain(layout.meds.into_iter().map(|med| (TrackerCellId::from(med), 3, false)))
        .chain(vec![
            (layout.row2[0], 3, false),
            (layout.row2[1], 3, false),
            (TrackerCellId::KokiriEmeraldLocation, 2, true),
            (TrackerCellId::GoronRubyLocation, 2, true),
            (TrackerCellId::ZoraSapphireLocation, 2, true),
            (layout.row2[2], 3, false),
            (layout.row2[3], 3, false),
            (TrackerCellId::KokiriEmerald, 2, false),
            (TrackerCellId::GoronRuby, 2, false),
            (TrackerCellId::ZoraSapphire, 2, false),
        ])
        .chain(layout.rest.iter().flat_map(|row|
            row.iter().map(|&cell| (cell, 3, false))
        ).collect_vec())
        .chain(layout.warp_songs.into_iter().map(|med| (TrackerCellId::warp_song(med), 3, false)))
}

#[rocket::get("/<name>")]
async fn room(rooms: State<'_, Mutex<HashMap<String, ModelState>>>, name: String) -> Html<String> {
    let html_layout = {
        let mut rooms = rooms.lock().await;
        let room = rooms.entry(name.clone()).or_default();
        let layout = TrackerLayout::default();
        cells(layout)
            .enumerate()
            .map(|(cell_id, (cell, colspan, loc))| cell.view(&name, cell_id.try_into().expect("too many cells"), room, colspan, loc))
            .join("\n")
    };
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
                <div class="items">
                    {}
                </div>
                <footer>
                    <a href="https://fenhl.net/disc">disclaimer / Impressum</a>
                </footer>
            </body>
        </html>
    "#, html_layout))
}

#[rocket::get("/<name>/click/<cell_id>")]
async fn click(rooms: State<'_, Mutex<HashMap<String, ModelState>>>, name: String, cell_id: u8) -> Result<Redirect, NotFound<&'static str>> {
    {
        let mut rooms = rooms.lock().await;
        let room = rooms.entry(name.clone()).or_default();
        let layout = TrackerLayout::default();
        cells(layout).nth(cell_id.into()).ok_or(NotFound("No such cell"))?.0.kind().click(room);
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
    .mount("/static", StaticFiles::new(crate_relative!("../../assets/web/static"), rocket_contrib::serve::Options::None))
    .mount("/", rocket::routes![
        index,
        room,
        click,
    ])
    .launch().await
}
