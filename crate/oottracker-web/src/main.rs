#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_qualifications, warnings)]
#![allow(unused_extern_crates)] // apparently rocket-derive still uses `extern crate`
#![forbid(unsafe_code)]

use {
    std::{
        collections::hash_map::{
            self,
            HashMap,
        },
        sync::Arc,
        time::{
            Duration,
            Instant,
        },
    },
    async_proto::{
        ReadError,
        WriteError,
    },
    futures::stream::TryStreamExt as _,
    lazy_regex::regex_is_match,
    rocket::{
        Rocket,
        http::Status,
    },
    sqlx::{
        PgPool,
        postgres::PgConnectOptions,
        types::Json,
    },
    tokio::sync::{
        Mutex,
        RwLock,
        watch::*,
    },
    wheel::traits::IsNetworkError,
    oottracker::{
        Knowledge,
        ModelState,
        Ram,
        TrackerCtx,
    },
    crate::{
        mw::MwState,
        restream::RestreamState,
    },
};

mod http;
mod mw;
mod restream;
mod websocket;

type MwRooms = Arc<RwLock<HashMap<String, Arc<RwLock<MwState>>>>>;
type Restreams = Arc<RwLock<HashMap<String, RestreamState>>>;
type Rooms = Arc<Mutex<HashMap<String, RoomState>>>;

struct RoomState {
    name: String,
    tx: Sender<()>,
    rx: Receiver<()>,
    last_saved: Instant,
    model: ModelState,
}

impl RoomState {
    pub(crate) fn new(name: &str) -> Result<Self, Error> {
        if regex_is_match!("^[0-9a-z]+(?:-[0-9a-z]+)*$", name) {
            Ok(Self::from_model(name, ModelState::default()))
        } else {
            Err(Error::RoomName)
        }
    }

    fn from_model(name: &str, model: ModelState) -> Self {
        let (tx, rx) = channel(());
        Self {
            tx, rx, model,
            name: name.to_owned(),
            last_saved: Instant::now(),
        }
    }

    pub(crate) async fn save(&mut self, pool: &PgPool) -> Result<(), Error> {
        if self.last_saved.elapsed() >= Duration::from_secs(60) {
            self.force_save(pool).await?;
        }
        Ok(())
    }

    pub(crate) async fn force_save(&mut self, pool: &PgPool) -> Result<(), Error> {
        let ModelState { ref knowledge, ref ram, .. } = self.model; //TODO include tracker context
        //TODO versioning (e.g. to recover RAM from previous versions)
        sqlx::query!("INSERT INTO rooms (name, knowledge, ram) VALUES ($1, $2, $3) ON CONFLICT (name) DO UPDATE SET knowledge = EXCLUDED.knowledge, ram = EXCLUDED.ram", self.name, serde_json::to_value(knowledge)?, &ram.to_ranges()[..]).execute(pool).await?;
        self.last_saved = Instant::now();
        Ok(())
    }
}

async fn get_room<T>(rooms: &Rooms, name: String, f: impl FnOnce(&RoomState) -> T) -> Result<T, Error> {
    let mut rooms = rooms.lock().await;
    Ok(f(match rooms.entry(name.clone()) {
        hash_map::Entry::Occupied(entry) => entry.into_mut(),
        hash_map::Entry::Vacant(entry) => entry.insert(RoomState::new(&name)?),
    }))
}

async fn edit_room(pool: &PgPool, rooms: &Rooms, name: String, f: impl FnOnce(&mut RoomState) -> Result<(), Error>) -> Result<(), Error> {
    let mut rooms = rooms.lock().await;
    let room = match rooms.entry(name.clone()) {
        hash_map::Entry::Occupied(entry) => entry.into_mut(),
        hash_map::Entry::Vacant(entry) => entry.insert(RoomState::new(&name)?),
    };
    f(room)?;
    room.tx.send(()).expect("failed to notify websockets about state change");
    room.save(pool).await?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("error decoding RAM: {0}")]
    RamDecode(#[from] oottracker::ram::DecodeError),
    #[error("read error: {0}")]
    Read(#[from] ReadError),
    #[error("rocket error: {0}")]
    Rocket(#[from] rocket::error::Error),
    #[error("database error: {0}")]
    Sql(#[from] sqlx::Error),
    #[error("task error: {0}")]
    Task(#[from] tokio::task::JoinError),
    #[error("WebSocket error: {0}")]
    Tungstenite(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("write error: {0}")]
    Write(#[from] WriteError),
    #[error("no such cell")]
    CellId,
    #[error("invalid room name")]
    RoomName,
}

impl<'r> rocket::response::Responder<'r, 'static> for Error {
    fn respond_to(self, _: &rocket::Request<'_>) -> rocket::response::Result<'static> {
        match self {
            Self::Json(_) => Err(Status::InternalServerError),
            Self::RamDecode(_) => Err(Status::InternalServerError),
            Self::Read(_) => Err(Status::InternalServerError),
            Self::Rocket(_) => Err(Status::InternalServerError),
            Self::Sql(_) => Err(Status::InternalServerError),
            Self::Task(_) => Err(Status::InternalServerError),
            Self::Tungstenite(_) => Err(Status::InternalServerError),
            Self::Write(_) => Err(Status::InternalServerError),
            Self::CellId => Err(Status::NotFound),
            Self::RoomName => Err(Status::NotFound),
        }
    }
}

impl IsNetworkError for Error {
    fn is_network_error(&self) -> bool {
        match self {
            Self::Json(_) => false,
            Self::RamDecode(_) => false,
            Self::Read(e) => e.is_network_error(),
            Self::Rocket(e) => match e.kind() {
                rocket::error::ErrorKind::Bind(e) | rocket::error::ErrorKind::Io(e) => e.is_network_error(),
                _ => false,
            },
            Self::Sql(_) => false,
            Self::Task(_) => false,
            Self::Tungstenite(e) => e.is_network_error(),
            Self::Write(e) => e.is_network_error(),
            Self::CellId => false,
            Self::RoomName => false,
        }
    }
}

#[wheel::main(rocket)]
async fn main() -> Result<(), Error> {
    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = wheel::night_report_sync("/games/zelda/oot/tracker/error", Some("thread panic"));
        default_panic_hook(info)
    }));
    let pool = PgPool::connect_with(PgConnectOptions::default().database("oottracker").application_name("oottracker-web")).await?;
    let rooms = {
        let mut rooms = HashMap::default();
        let mut query = sqlx::query!(r#"SELECT name, knowledge AS "knowledge: Json<Knowledge>", ram FROM rooms"#).fetch(&pool);
        while let Some(room) = query.try_next().await? {
            let state = RoomState::from_model(&room.name, ModelState { knowledge: room.knowledge.0, ram: Ram::from_range_bufs(room.ram)?, tracker_ctx: TrackerCtx::default() });
            rooms.insert(room.name, state);
        }
        Rooms::new(Mutex::new(rooms))
    };
    //TODO force-save all rooms on stop
    let restreams = {
        //TODO remove hardcoded restream, allow configuring active restreams somehow
        let mut map = HashMap::default();
        for restreamer in ["fenhl", "utz"] {
            let multiworld_3v3 = vec![
                vec!["a1", "b1"],
                vec!["a2", "b2"],
                vec!["a3", "b3"],
            ];
            map.insert(restreamer.to_owned(), RestreamState::new(multiworld_3v3));
        }
        Restreams::new(RwLock::new(map))
    };
    let mw_rooms = MwRooms::default();
    let Rocket { .. } = http::rocket(pool, rooms, restreams, mw_rooms).launch().await?;
    Ok(())
}
