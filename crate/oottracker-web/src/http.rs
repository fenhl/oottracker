use {
    std::{
        num::NonZeroU8,
        time::Duration,
    },
    ootr_utils::{
        PyModules,
        Version,
    },
    pyo3::{
        prelude::*,
        types::PyDict,
    },
    rocket::{
        FromForm,
        FromFormField,
        Rocket,
        State,
        UriDisplayQuery,
        form::Form,
        fs::{
            FileServer,
            relative,
        },
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
    let (_, _, model, _) = mw_room.world(world)?;
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
        let (tx, _, model, _) = mw_room.world_mut(world).ok_or(NotFound("No such world"))?;
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

fn format_override_key(_modules: PyModules<'_>, key: u32) -> PyResult<String> {
    Ok(format!("0x{key:08x}")) //TODO look up location name using randomizer version
}

fn format_item_kind(modules: PyModules<'_>, kind: u16) -> PyResult<String> {
    let item_list = modules.py().import("ItemList")?;
    for (item_name, entry) in item_list.getattr("item_table")?.downcast::<PyDict>()?.iter() {
        let (_, _, get_item_id, _) = entry.extract::<(&PyAny, &PyAny, Option<u16>, &PyAny)>()?;
        if get_item_id == Some(kind) {
            return item_name.extract()
        }
    }
    Ok(format!("0x{kind:04x}"))
}

#[derive(Debug, thiserror::Error, rocket_util::Error)]
enum NotesError {
    #[error(transparent)] Python(#[from] PyErr),
}

#[rocket::get("/mw-notes/<room>")]
async fn mw_notes(mw_rooms: &State<MwRooms>, room: &str) -> Result<Option<RawHtml<String>>, NotesError> {
    let mw_rooms = mw_rooms.read().await;
    let Some(mw_room) = mw_rooms.get(room) else { return Ok(None) };
    let mw_room = mw_room.read().await;
    let rando_version = Version::from_dev(6, 2, 205); //TODO don't hardcode
    Python::with_gil(|py| {
        let modules = rando_version.py_modules(py)?;
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
                        @for (idx, (_, _, _, queue)) in mw_room.worlds.iter().enumerate() {
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
                                        @for MwItem { source, key, kind } in queue {
                                            tr {
                                                td(class? = world_class(*source)) : source.get();
                                                td(class? = world_class(*source)) : format_override_key(modules, *key)?;
                                                td(class? = world_class(world_id)) : format_item_kind(modules, *kind)?;
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
    })
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

pub(crate) fn rocket(pool: PgPool, rooms: Rooms, restreams: Restreams, mw_rooms: MwRooms) -> Rocket<rocket::Build> {
    rocket::custom(rocket::Config {
        port: 24807,
        ..rocket::Config::default()
    })
    .manage(pool)
    .manage(rooms)
    .manage(restreams)
    .manage(mw_rooms)
    .mount("/static", FileServer::new(relative!("../../assets/web/static"), rocket::fs::Options::None))
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
    ])
}
