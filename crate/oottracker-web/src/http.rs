use {
    std::convert::TryInto as _,
    itertools::Itertools as _,
    rocket::{
        FromForm,
        Rocket,
        State,
        form::Form,
        fs::{
            FileServer,
            relative,
        },
        response::{
            Redirect,
            content::Html,
            status::NotFound,
        },
    },
    oottracker::{
        ModelState,
        ui::{
            DoubleTrackerLayout,
            TrackerCellId,
            TrackerLayout,
        },
    },
    crate::{
        Restreams,
        Rooms,
        restream::render_double_cell,
    },
};

trait TrackerCellIdExt {
    fn view(&self, room_name: &str, cell_id: u8, state: &ModelState, colspan: u8, loc: bool) -> String;
}

impl TrackerCellIdExt for TrackerCellId {
    fn view(&self, room_name: &str, cell_id: u8, state: &ModelState, colspan: u8, loc: bool) -> String {
        let kind = self.kind();
        let content = kind.render(state);
        let css_classes = if loc { format!("cols{} loc", colspan) } else { format!("cols{}", colspan) };
        format!(r#"<a id="cell{}" href="{}" class="{}">{}</a>"#, cell_id, rocket::uri!(click(room_name, cell_id)), css_classes, content.to_html())
    }
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
                <link rel="icon" type="image/vnd.microsoft.icon" href="/static/img/icon.ico" />
                <link rel="stylesheet" href="/static/common.css" />
            </head>
            <body>
                <div class="items {}">
                    {}
                </div>
                <noscript>
                    <p>live update disabled (requires JavaScript)</p>
                </noscript>
                <footer>
                    <a href="https://fenhl.net/disc">disclaimer / Impressum</a>
                </footer>
                <script src="/static/proto.js"></script>
            </body>
        </html>
    "#, layout_name, html_layout))
}

#[rocket::get("/")]
fn index() -> Html<String> {
    Html(format!(include_str!("../../../assets/web/index.html"), env!("CARGO_PKG_VERSION")))
}

#[derive(FromForm)]
struct GoRoomForm<'r> {
    #[field(validate = len(1..))]
    room: &'r str,
}

#[rocket::post("/", data = "<form>")]
fn post_index(form: Form<GoRoomForm<'_>>) -> Redirect {
    Redirect::to(rocket::uri!(room(form.room.to_owned())))
}

#[rocket::get("/restream/<restreamer>/<runner>")]
async fn restream_room_input(restreams: &State<Restreams>, restreamer: String, runner: String) -> Option<Html<String>> {
    let html_layout = {
        let restreams = restreams.read().await;
        let restream = restreams.get(&restreamer)?;
        let layout = restream.layout();
        let (_, _, model_state_view) = restream.runner(&runner)?;
        let pseudo_name = format!("restream/{}/{}/{}", restreamer, runner, layout);
        layout.cells().into_iter()
            .map(|cell| cell.id.view(&pseudo_name, cell.idx.try_into().expect("too many cells"), &model_state_view, (cell.size[0] / 20 + 1) as u8, cell.size[1] < 30))
            .join("\n")
    };
    Some(tracker_page("default", html_layout))
}

#[rocket::get("/restream/<restreamer>/<runner>/<layout>")]
async fn restream_room_view(restreams: &State<Restreams>, restreamer: String, runner: String, layout: TrackerLayout) -> Option<Html<String>> {
    let html_layout = {
        let restreams = restreams.read().await;
        let restream = restreams.get(&restreamer)?;
        let (_, _, model_state_view) = restream.runner(&runner)?;
        let pseudo_name = format!("restream/{}/{}/{}", restreamer, runner, layout);
        layout.cells().into_iter()
            .map(|cell| cell.id.view(&pseudo_name, cell.idx.try_into().expect("too many cells"), &model_state_view, (cell.size[0] / 20 + 1) as u8, cell.size[1] < 30))
            .join("\n")
    };
    Some(tracker_page(&layout.to_string(), html_layout))
}

#[rocket::get("/restream/<restreamer>/<runner>/<layout>/click/<cell_id>")]
async fn restream_click(restreams: &State<Restreams>, restreamer: String, runner: String, layout: TrackerLayout, cell_id: u8) -> Result<Redirect, NotFound<&'static str>> {
    {
        let mut restreams = restreams.write().await;
        let restream = restreams.get_mut(&restreamer).ok_or(NotFound("No such restream"))?;
        let (tx, _, model_state_view) = restream.runner_mut(&runner).ok_or(NotFound("No such runner"))?;
        layout.cells().get(usize::from(cell_id)).ok_or(NotFound("No such cell"))?.id.kind().click(model_state_view);
        tx.send(()).expect("failed to notify websockets about state change");
    }
    Ok(Redirect::to(if layout == TrackerLayout::default() {
        rocket::uri!(restream_room_input(restreamer, runner))
    } else {
        rocket::uri!(restream_room_view(restreamer, runner, layout))
    }))
}

#[rocket::get("/restream/<restreamer>/<runner1>/<layout>/with/<runner2>")]
async fn restream_double_room_layout(restreams: &State<Restreams>, restreamer: String, runner1: String, layout: DoubleTrackerLayout, runner2: String) -> Option<Html<String>> {
    let html_layout = {
        let restreams = restreams.read().await;
        let restream = restreams.get(&restreamer)?;
        layout.cells()
            .into_iter()
            .map(|reward| Some(render_double_cell(restream.runner(&runner1)?.2, restream.runner(&runner2)?.2, reward)))
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .enumerate()
            .map(|(cell_id, render)| format!(r#"<div id="cell{}" class="cols3">{}</div>"#, cell_id, render.to_html()))
            .join("\n")
    };
    Some(tracker_page(&layout.to_string(), html_layout))
}

#[rocket::get("/room/<name>")]
async fn room(rooms: &State<Rooms>, name: String) -> Html<String> {
    let html_layout = {
        let mut rooms = rooms.lock().await;
        let room = rooms.entry(name.clone()).or_default();
        let layout = TrackerLayout::default();
        layout.cells().into_iter()
            .map(|cell| cell.id.view(&name, cell.idx.try_into().expect("too many cells"), &room.model, (cell.size[0] / 20 + 1) as u8, cell.size[1] < 30))
            .join("\n")
    };
    tracker_page("default", html_layout)
}

#[rocket::get("/room/<name>/click/<cell_id>")]
async fn click(rooms: &State<Rooms>, name: String, cell_id: u8) -> Result<Redirect, NotFound<&'static str>> {
    {
        let mut rooms = rooms.lock().await;
        let room = rooms.entry(name.clone()).or_default();
        let layout = TrackerLayout::default();
        layout.cells().get(usize::from(cell_id)).ok_or(NotFound("No such cell"))?.id.kind().click(&mut room.model);
    }
    Ok(Redirect::to(rocket::uri!(room(name))))
}

pub(crate) fn rocket(rooms: Rooms, restreams: Restreams) -> Rocket<rocket::Build> {
    rocket::custom(rocket::Config {
        port: 24807,
        //TODO configure secret_key for release mode
        ..rocket::Config::default()
    })
    .manage(rooms)
    .manage(restreams)
    .mount("/static", FileServer::new(relative!("../../assets/web/static"), rocket::fs::Options::None))
    .mount("/", rocket::routes![
        index,
        post_index,
        restream_room_input,
        restream_room_view,
        restream_click,
        restream_double_room_layout,
        room,
        click,
    ])
}
