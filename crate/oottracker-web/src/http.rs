use {
    std::{
        collections::hash_map::{
            self,
            HashMap,
        },
        num::NonZeroU8,
        time::Duration,
    },
    itertools::Itertools as _,
    ootr_utils::{
        PyJsonError,
        PyModules,
        Version,
    },
    rocket::{
        FromForm,
        FromFormField,
        Rocket,
        State,
        UriDisplayQuery,
        form::Form,
        fs::FileServer,
        http::uri::Origin,
        response::{
            Redirect,
            content::RawHtml,
            status::NotFound,
        },
        uri,
    },
    rocket_util::{
        Doctype,
        ToHtml,
        html,
    },
    rocket_ws::WebSocket,
    sqlx::PgPool,
    oottracker::{
        ModelState,
        ui::{
            DoubleTrackerLayout,
            TrackerCellId,
            TrackerLayout,
        },
        websocket::MwItem,
    },
    crate::{
        Error,
        MwRooms,
        Restreams,
        Rooms,
        edit_room,
        get_room,
        restream::render_double_cell,
    },
};

//TODO don't hardcode
const RANDO_VERSION: Version = Version::from_dev(7, 1, 199);

//HACK assume all child trade items are shuffled for the purpose of override key generation, not sure if this breaks anything
const SHUFFLE_CHILD_TRADE: [&str; 11] = [
    "Weird Egg",
    "Chicken",
    "Zeldas Letter",
    "Keaton Mask",
    "Skull Mask",
    "Spooky Mask",
    "Bunny Hood",
    "Goron Mask",
    "Zora Mask",
    "Gerudo Mask",
    "Mask of Truth",
];

trait TrackerCellIdExt {
    fn view<'a>(&self, click_uri: Origin<'_>, cell_id: u8, state: &ModelState, colspan: u8, loc: bool) -> RawHtml<String>;
}

impl TrackerCellIdExt for TrackerCellId {
    fn view<'a>(&self, click_uri: Origin<'_>, cell_id: u8, state: &ModelState, colspan: u8, loc: bool) -> RawHtml<String> {
        let kind = self.kind();
        let content = kind.render(state);
        let css_classes = if loc { format!("cols{colspan} loc") } else { format!("cols{colspan}") };
        html! {
            a(id = format!("cell{cell_id}"), href = click_uri.to_string(), class = css_classes) : content; //TODO impl ToHtml for rocket::uri
        }
    }
}

#[derive(FromFormField, UriDisplayQuery)]
enum Theme {
    Light,
    Dark,
}

fn tracker_page<'a>(layout_name: &'a str, theme: Option<Theme>, items: impl ToHtml) -> RawHtml<String> {
    html! {
        : Doctype;
        html {
            head {
                meta(charset = "utf-8");
                title : "OoT Tracker";
                meta(name = "author", content = "Fenhl");
                meta(name = "viewport", content = "width=device-width, initial-scale=1");
                link(rel = "icon", sizes = "512x512", type = "image/png", href = "/static/img/favicon.png");
                link(rel = "stylesheet", href = "/static/common.css");
                @match theme {
                    Some(Theme::Light) => link(rel = "stylesheet", href = "/static/light.css");
                    None => link(rel = "stylesheet", href = "/static/light.css", media = "(prefers-color-scheme: light)");
                    Some(Theme::Dark) => {}
                }
            }
            body {
                div(class = format!("items {layout_name}")) : items;
                noscript {
                    p : "live update disabled (requires JavaScript)";
                }
                footer {
                    a(href = "https://fenhl.net/disc") : "disclaimer / Impressum";
                }
                script(src = "/static/proto.js");
            }
        }
    }
}

#[rocket::get("/")]
fn index() -> RawHtml<String> {
    RawHtml(format!(include_str!("../../../assets/web/index.html"), env!("CARGO_PKG_VERSION")))
}

#[derive(FromForm)]
struct GoRoomForm<'r> {
    #[field(validate = len(1..))]
    room: &'r str,
}

#[rocket::post("/", data = "<form>")]
fn post_index(form: Form<GoRoomForm<'_>>) -> Redirect {
    Redirect::to(rocket::uri!(room(form.room.to_owned(), _)))
}

#[rocket::get("/mw/<room>/<world>?<theme>&<delay>")]
async fn mw_room_input(room: &str, world: NonZeroU8, theme: Option<Theme>, delay: Option<f64>) -> Redirect {
    Redirect::permanent(uri!(mw_room_view(room, world, TrackerLayout::default(), theme, delay)))
}

#[rocket::get("/mw/<room>/<world>/<layout>?<theme>&<delay>")]
async fn mw_room_view(mw_rooms: &State<MwRooms>, room: &str, world: NonZeroU8, layout: TrackerLayout, theme: Option<Theme>, delay: Option<f64>) -> Option<RawHtml<String>> {
    let mw_rooms = mw_rooms.read().await;
    let mw_room = mw_rooms.get(room)?;
    if let Some(delay) = delay {
        mw_room.write().await.autotracker_delay = Duration::try_from_secs_f64(delay).ok()?;
    }
    let mw_room = mw_room.read().await;
    let (_, _, model, _, _) = mw_room.world(world)?;
    Some(tracker_page(&layout.to_string(), theme, html! {
        @for cell in layout.cells() {
            @let cell_id = cell.idx.try_into().expect("too many cells");
            : cell.id.view(rocket::uri!(mw_click(room, world, layout, cell_id)), cell_id, model, (cell.size[0] / 20 + 1) as u8, cell.size[1] < 30);
        }
    }))
}

#[rocket::get("/mw/<room>/<world>/<layout>/click/<cell_id>")]
async fn mw_click(mw_rooms: &State<MwRooms>, room: &str, world: NonZeroU8, layout: TrackerLayout, cell_id: u8) -> Result<Redirect, NotFound<&'static str>> {
    {
        let mw_rooms = mw_rooms.read().await;
        let mw_room = mw_rooms.get(room).ok_or(NotFound("No such multiworld room"))?;
        let mut mw_room = mw_room.write().await;
        let (tx, _, model, _, _) = mw_room.world_mut(world).ok_or(NotFound("No such world"))?;
        layout.cells().get(usize::from(cell_id)).ok_or(NotFound("No such cell"))?.id.kind().click(model);
        tx.send(()).expect("failed to notify websockets about state change");
    }
    Ok(Redirect::to(rocket::uri!(mw_room_view(room, world, layout, _, _))))
}

fn world_class(world_id: NonZeroU8) -> Option<&'static str> {
    match world_id.get() {
        0 => unreachable!(),
        1 => Some("power"),
        2 => Some("wisdom"),
        3 => Some("courage"),
        _ => None,
    }
}

async fn format_override_key<'a>(modules: &PyModules, cache: &'a mut HashMap<NonZeroU8, HashMap<u64, String>>, shuffle_child_trade: &[&str], source_world: NonZeroU8, key: u64, target_world: NonZeroU8, item: &str) -> Result<&'a str, PyJsonError> {
    Ok(match cache.entry(source_world) {
        hash_map::Entry::Occupied(entry) => entry.into_mut(),
        hash_map::Entry::Vacant(entry) => {
            let entries = modules.py_json::<HashMap<String, [u8; 16]>>(&format!("
import json, Item, Location, LocationList, Patches

class Settings:
    def __init__(self):
        self.shuffle_child_trade = {shuffle_child_trade:?}

class World:
    def __init__(self, id):
        self.id = id
        self.settings = Settings()

entries = {{}}
for loc_name in LocationList.location_table:
    loc = Location.LocationFactory(loc_name)
    loc.world = World({source_world})
    loc.item = Item.ItemFactory({item:?}, World({target_world}))
    entry = Patches.get_override_entry(loc)
    if entry is not None:
        entries[loc_name] = list(Patches.override_struct.pack(*entry))
print(json.dumps(entries))
            ")).await?;
            entry.insert(entries.into_iter().map(|(name, [k0, k1, k2, k3, k4, k5, k6, k7, _, _, _, _, _, _, _, _])| (u64::from_be_bytes([k0, k1, k2, k3, k4, k5, k6, k7]), name)).collect())
        }
    }.entry(key).or_insert_with(|| format!("0x{key:016x}")))
}

async fn format_item_kind<'a>(modules: &PyModules, cache: &'a mut HashMap<u16, String>, kind: u16) -> Result<&'a str, PyJsonError> {
    Ok(match cache.entry(kind) {
        hash_map::Entry::Occupied(entry) => entry.into_mut(),
        hash_map::Entry::Vacant(entry) => {
            let mut entries = modules.py_json::<HashMap<u16, String>>("
import json
import ItemList

print(json.dumps({
    str(get_item_id): name
    for name, (kind, _, get_item_id, _) in ItemList.item_table.items()
    if get_item_id is not None
    and kind != 'Shop'
}))
            ").await?;
            entry.insert(entries.remove(&kind).unwrap_or_else(|| format!("0x{kind:04x}"))) //TODO generate entire table in 1 Python call
        }
    })
}

#[derive(Debug, thiserror::Error, rocket_util::Error)]
enum NotesError {
    #[error(transparent)] Clone(#[from] ootr_utils::CloneError),
    #[error(transparent)] Dir(#[from] ootr_utils::DirError),
    #[error(transparent)] PyJson(#[from] PyJsonError),
}

#[tokio::test]
async fn test_mw_notes() {
    let python = {
        #[cfg(windows)] { r"C:\Users\fenhl\scoop\apps\python\current\python.exe" }
        #[cfg(unix)] { "/usr/bin/python3" }
    };
    RANDO_VERSION.clone_repo().await.unwrap();
    let modules = RANDO_VERSION.py_modules(python).unwrap();
    let mw_room = crate::mw::MwState::new(Vec::default());
    let source_world = NonZeroU8::new(1).unwrap();
    let key = 0x2801_0000_0000_0000;
    let kind = 0x000D;
    let target_world = NonZeroU8::new(2).unwrap();
    let mut mw_room = mw_room.write().await;
    let item_name = format_item_kind(&modules, &mut mw_room.item_cache, kind).await.unwrap().to_owned();
    assert_eq!(item_name, "Megaton Hammer");
    let location_name = format_override_key(&modules, &mut mw_room.location_cache, &SHUFFLE_CHILD_TRADE, source_world, key, target_world, &item_name).await.unwrap();
    assert_eq!(location_name, "KF Midos Top Left Chest");
}

#[rocket::get("/mw-notes/<room>")]
async fn mw_notes(mw_rooms: &State<MwRooms>, room: &str) -> Result<Option<RawHtml<String>>, NotesError> {
    let mw_rooms = mw_rooms.read().await;
    let Some(mw_room) = mw_rooms.get(room) else { return Ok(None) };
    let mut mw_room = mw_room.write().await;
    let mw_room = &mut *mw_room;
    RANDO_VERSION.clone_repo().await?;
    let modules = RANDO_VERSION.py_modules("/usr/bin/python3")?;
    Ok(Some(html! {
        : Doctype;
        html {
            head {
                meta(charset = "utf-8");
                title : "OoT Tracker";
                meta(name = "author", content = "Fenhl");
                meta(name = "viewport", content = "width=device-width, initial-scale=1");
                link(rel = "icon", sizes = "512x512", type = "image/png", href = "/static/img/favicon.png");
                link(rel = "stylesheet", href = "/static/common.css");
                link(rel = "stylesheet", href = "/static/light.css", media = "(prefers-color-scheme: light)");
            }
            body {
                div(class = "table-wrapper") {
                    @for (idx, (_, _, _, queue, own_items)) in mw_room.worlds.iter().enumerate() {
                        @let world_id = NonZeroU8::new((idx + 1).try_into().unwrap()).unwrap();
                        div {
                            h1(class? = world_class(world_id)) {
                                : "For player ";
                                : world_id.get();
                            };
                            table {
                                thead {
                                    tr {
                                        th : "From world";
                                        th : "From location";
                                        th : "Item";
                                    }
                                }
                                tbody {
                                    @for MwItem { source, key, kind } in own_items.iter().sorted().chain(queue) {
                                        tr {
                                            @let item_name = format_item_kind(&modules, &mut mw_room.item_cache, *kind).await?;
                                            td(class? = world_class(*source)) : source.get();
                                            td(class? = world_class(*source)) : format_override_key(&modules, &mut mw_room.location_cache, &SHUFFLE_CHILD_TRADE, *source, *key, world_id, item_name).await?;
                                            td(class? = world_class(world_id)) : item_name;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                p : "live update not yet implemented (refresh to update)"; //TODO
                footer {
                    a(href = "https://fenhl.net/disc") : "disclaimer / Impressum";
                }
            }
        }
    }))
}

#[rocket::get("/restream/<restreamer>/<runner>?<theme>")]
async fn restream_room_input(restreamer: &str, runner: &str, theme: Option<Theme>) -> Redirect {
    Redirect::permanent(uri!(restream_room_view(restreamer, runner, TrackerLayout::default(), theme)))
}

#[rocket::get("/restream/<restreamer>/<runner>/<layout>?<theme>")]
async fn restream_room_view(restreams: &State<Restreams>, restreamer: &str, runner: &str, layout: TrackerLayout, theme: Option<Theme>) -> Option<RawHtml<String>> {
    let restreams = restreams.read().await;
    let restream = restreams.get(restreamer)?;
    let (_, _, model_state_view) = restream.runner(runner)?;
    Some(tracker_page(&layout.to_string(), theme, html! {
        @for cell in layout.cells() {
            @let cell_id = cell.idx.try_into().expect("too many cells");
            : cell.id.view(rocket::uri!(restream_click(restreamer, runner, layout, cell_id)), cell_id, &model_state_view, (cell.size[0] / 20 + 1) as u8, cell.size[1] < 30);
        }
    }))
}

#[rocket::get("/restream/<restreamer>/<runner>/<layout>/click/<cell_id>")]
async fn restream_click(restreams: &State<Restreams>, restreamer: &str, runner: &str, layout: TrackerLayout, cell_id: u8) -> Result<Redirect, NotFound<&'static str>> {
    {
        let mut restreams = restreams.write().await;
        let restream = restreams.get_mut(restreamer).ok_or(NotFound("No such restream"))?;
        let (tx, _, model_state_view) = restream.runner_mut(runner).ok_or(NotFound("No such runner"))?;
        layout.cells().get(usize::from(cell_id)).ok_or(NotFound("No such cell"))?.id.kind().click(model_state_view);
        tx.send(()).expect("failed to notify websockets about state change");
    }
    Ok(Redirect::to(rocket::uri!(restream_room_view(restreamer, runner, layout, _))))
}

#[rocket::get("/restream/<restreamer>/<runner1>/<layout>/with/<runner2>?<theme>")]
async fn restream_double_room_layout(restreams: &State<Restreams>, restreamer: &str, runner1: &str, layout: DoubleTrackerLayout, runner2: &str, theme: Option<Theme>) -> Option<RawHtml<String>> {
    let restreams = restreams.read().await;
    let restream = restreams.get(restreamer)?;
    let cells = layout.cells()
        .into_iter()
        .map(|reward| Some(render_double_cell(restream.runner(runner1)?.2, restream.runner(runner2)?.2, reward)))
        .collect::<Option<Vec<_>>>()?;
    Some(tracker_page(&layout.to_string(), theme, html! {
        @for (cell_id, render) in cells.into_iter().enumerate() {
            div(id = format!("cell{cell_id}"), class = "cols3") : render;
        }
    }))
}

#[rocket::get("/room/<name>?<theme>")]
async fn room(rooms: &State<Rooms>, name: &str, theme: Option<Theme>) -> Result<RawHtml<String>, Error> {
    Ok(get_room(rooms, name.to_owned(), |room| {
        let layout = TrackerLayout::default();
        tracker_page(&layout.to_string(), theme, html! {
            @for cell in layout.cells() {
                @let cell_id = cell.idx.try_into().expect("too many cells");
                : cell.id.view(rocket::uri!(click(name, cell_id)), cell_id, &room.model, (cell.size[0] / 20 + 1) as u8, cell.size[1] < 30);
            }
        })
    }).await?)
}

#[rocket::get("/room/<name>/click/<cell_id>")]
async fn click(pool: &State<PgPool>, rooms: &State<Rooms>, name: &str, cell_id: u8) -> Result<Redirect, Error> {
    edit_room(pool, rooms, name.to_owned(), |room| {
        let layout = TrackerLayout::default();
        layout.cells().get(usize::from(cell_id)).ok_or(Error::CellId)?.id.kind().click(&mut room.model);
        Ok(())
    }).await?;
    Ok(Redirect::to(rocket::uri!(room(name, _))))
}

#[rocket::get("/websocket")]
fn websocket(db_pool: &State<PgPool>, rooms: &State<Rooms>, restreams: &State<Restreams>, mw_rooms: &State<MwRooms>, ws: WebSocket) -> rocket_ws::Channel<'static> {
    let db_pool = (*db_pool).clone();
    let rooms = (*rooms).clone();
    let restreams = (*restreams).clone();
    let mw_rooms = (*mw_rooms).clone();
    ws.channel(move |stream| Box::pin(async move {
        let () = crate::websocket::client_connection(db_pool, rooms, restreams, mw_rooms, stream).await;
        Ok(())
    }))
}

#[rocket::catch(404)]
fn not_found() -> RawHtml<String> {
    RawHtml(format!(include_str!("../../../assets/web/404.html"), env!("CARGO_PKG_VERSION")))
}

#[rocket::catch(500)]
async fn internal_server_error() -> Result<RawHtml<String>, rocket_util::Error<wheel::Error>> {
    wheel::night_report("/games/zelda/oot/tracker/error", Some("internal server error")).await?;
    Ok(RawHtml(format!(include_str!("../../../assets/web/500.html"), env!("CARGO_PKG_VERSION"))))
}

pub(crate) fn rocket(pool: PgPool, rooms: Rooms, restreams: Restreams, mw_rooms: MwRooms) -> Rocket<rocket::Build> {
    rocket::custom(rocket::Config {
        port: 24807,
        ..rocket::Config::default()
    })
    .mount("/static", FileServer::new("assets/web/static", rocket::fs::Options::None))
    .mount("/", rocket::routes![
        index,
        post_index,
        mw_room_input,
        mw_room_view,
        mw_click,
        mw_notes,
        restream_room_input,
        restream_room_view,
        restream_click,
        restream_double_room_layout,
        room,
        click,
        websocket,
    ])
    .register("/", rocket::catchers![
        not_found,
        internal_server_error,
    ])
    .manage(pool)
    .manage(rooms)
    .manage(restreams)
    .manage(mw_rooms)
}
