use {
    std::convert::TryInto as _,
    itertools::Itertools as _,
    rocket::{
        Rocket,
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
    oottracker::{
        ModelStateView,
        ui::TrackerCellId,
    },
    crate::{
        Restreams,
        Rooms,
        TrackerCellKindExt as _,
        restream::{
            DoubleTrackerLayout,
            TrackerLayout,
            render_double_cell,
        },
    },
};

trait TrackerCellIdExt {
    fn view(&self, room_name: &str, cell_id: u8, state: &dyn ModelStateView, colspan: u8, loc: bool) -> String;
}

impl TrackerCellIdExt for TrackerCellId {
    fn view(&self, room_name: &str, cell_id: u8, state: &dyn ModelStateView, colspan: u8, loc: bool) -> String {
        let kind = self.kind();
        let content = kind.render(state);
        let css_classes = if loc { format!("cols{} loc", colspan) } else { format!("cols{}", colspan) };
        format!(r#"<a id="cell{}" href="/{}/click/{}" class="{}">{}</a>"#, cell_id, room_name, cell_id, css_classes, content.to_html()) //TODO click action for JS, put link in a noscript tag
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
                <script src="/static/proto.js"></script>
            </body>
        </html>
    "#, layout_name, html_layout))
}

#[rocket::get("/")]
fn index() -> Html<String> {
    Html(format!(include_str!("../../../assets/web/index.html"), env!("CARGO_PKG_VERSION")))
}

#[rocket::get("/restream/<restreamer>/<runner>")]
async fn restream_room_input(restreams: State<'_, Restreams>, restreamer: String, runner: String) -> Option<Html<String>> {
    let html_layout = {
        let mut restreams = restreams.lock().await;
        let restream = restreams.get_mut(&restreamer)?;
        let layout = restream.layout();
        let (_, _, model_state_view) = restream.runner(&runner)?;
        let pseudo_name = format!("restream/{}/{}/{}", restreamer, runner, layout);
        layout.cells()
            .enumerate()
            .map(|(cell_id, (cell, colspan, loc))| cell.view(&pseudo_name, cell_id.try_into().expect("too many cells"), &model_state_view, colspan, loc))
            .join("\n")
    };
    Some(tracker_page("default", html_layout))
}

#[rocket::get("/restream/<restreamer>/<runner>/<layout>")]
async fn restream_room_view(restreams: State<'_, Restreams>, restreamer: String, runner: String, layout: TrackerLayout) -> Option<Html<String>> {
    let html_layout = {
        let mut restreams = restreams.lock().await;
        let restream = restreams.get_mut(&restreamer)?;
        let (_, _, model_state_view) = restream.runner(&runner)?;
        let pseudo_name = format!("restream/{}/{}/{}", restreamer, runner, layout);
        layout.cells()
            .enumerate()
            .map(|(cell_id, (cell, colspan, loc))| cell.view(&pseudo_name, cell_id.try_into().expect("too many cells"), &model_state_view, colspan, loc))
            .join("\n")
    };
    Some(tracker_page(&layout.to_string(), html_layout))
}

#[rocket::get("/restream/<restreamer>/<runner>/<layout>/click/<cell_id>")]
async fn restream_click(restreams: State<'_, Restreams>, restreamer: String, runner: String, layout: TrackerLayout, cell_id: u8) -> Result<Redirect, NotFound<&'static str>> {
    {
        let mut restreams = restreams.lock().await;
        let restream = restreams.get_mut(&restreamer).ok_or(NotFound("No such restream"))?;
        let (tx, _, mut model_state_view) = restream.runner(&runner).ok_or(NotFound("No such runner"))?;
        layout.cells().nth(cell_id.into()).ok_or(NotFound("No such cell"))?.0.kind().click(&mut model_state_view);
        tx.send(()).expect("failed to notify websockets about state change");
    }
    Ok(Redirect::to(rocket::uri!(restream_room_view: restreamer, runner, layout)))
}

#[rocket::get("/restream/<restreamer>/<runner1>/<layout>/with/<runner2>")]
async fn restream_double_room_layout(restreams: State<'_, Restreams>, restreamer: String, runner1: String, layout: DoubleTrackerLayout, runner2: String) -> Option<Html<String>> {
    let html_layout = {
        let mut restreams = restreams.lock().await;
        let restream = restreams.get_mut(&restreamer)?;
        layout.cells()
            .into_iter()
            .map(|reward| render_double_cell(restream, &runner1, &runner2, reward))
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .enumerate()
            .map(|(cell_id, render)| format!(r#"<div id="cell{}" class="cols3">{}</div>"#, cell_id, render.to_html()))
            .join("\n")
    };
    Some(tracker_page(&layout.to_string(), html_layout))
}

#[rocket::get("/room/<name>")]
async fn room(rooms: State<'_, Rooms>, name: String) -> Html<String> {
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
async fn click(rooms: State<'_, Rooms>, name: String, cell_id: u8) -> Result<Redirect, NotFound<&'static str>> {
    {
        let mut rooms = rooms.lock().await;
        let room = rooms.entry(name.clone()).or_default();
        let layout = TrackerLayout::default();
        layout.cells().nth(cell_id.into()).ok_or(NotFound("No such cell"))?.0.kind().click(room);
    }
    Ok(Redirect::to(rocket::uri!(room: name)))
}

pub(crate) fn rocket(rooms: Rooms, restreams: Restreams) -> Rocket {
    rocket::custom(rocket::Config {
        port: 24807,
        //TODO configure secret_key for release mode
        ..rocket::Config::default()
    })
    .manage(rooms)
    .manage(restreams)
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
}
