#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_qualifications, warnings)]
#![allow(unused_extern_crates)] // apparently rocket-derive still uses `extern crate`
#![forbid(unsafe_code)]

use {
    std::{
        collections::HashMap,
        fmt,
        sync::Arc,
    },
    async_proto::ReadError,
    derive_more::From,
    futures::future::{
        FutureExt as _,
        TryFutureExt as _,
    },
    structopt::StructOpt,
    tokio::sync::{
        Mutex,
        RwLock,
        watch::*,
    },
    warp::Filter as _,
    oottracker::ModelState,
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
