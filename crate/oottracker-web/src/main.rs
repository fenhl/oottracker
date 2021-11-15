#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_qualifications, warnings)]
#![allow(unused_extern_crates)] // apparently rocket-derive still uses `extern crate`
#![forbid(unsafe_code)]

use {
    std::{
        collections::hash_map::{
            self,
            HashMap,
        },
        fmt,
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
    derive_more::From,
    futures::{
        future::{
            FutureExt as _,
            TryFutureExt as _,
        },
        stream::TryStreamExt as _,
    },
    lazy_regex::regex_is_match,
    rocket::http::Status,
    sqlx::{
        PgPool,
        postgres::PgConnectOptions,
        types::Json,
    },
    structopt::StructOpt,
    tokio::sync::{
        Mutex,
        RwLock,
        watch::*,
    },
    warp::Filter as _,
    oottracker::{
        Knowledge,
        ModelState,
        Ram,
    },
    crate::restream::RestreamState,
};

mod http;
mod restream;
mod websocket;

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
        let ModelState { ref knowledge, ref ram } = self.model;
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

#[derive(Debug, From)]
enum Error {
    CellId,
    Horrorshow(horrorshow::Error),
    Json(serde_json::Error),
    RamDecode(oottracker::ram::DecodeError),
    Read(ReadError),
    Rocket(rocket::error::Error),
    RoomName,
    Sql(sqlx::Error),
    Task(tokio::task::JoinError),
    Write(WriteError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CellId => write!(f, "no such cell"),
            Self::Horrorshow(e) => write!(f, "error rendering HTML: {}", e),
            Self::Json(e) => write!(f, "JSON error: {}", e),
            Self::RamDecode(e) => write!(f, "error decoding RAM: {}", e),
            Self::Read(e) => write!(f, "read error: {}", e),
            Self::Rocket(e) => write!(f, "rocket error: {}", e),
            Self::RoomName => write!(f, "invalid room name"),
            Self::Sql(e) => write!(f, "database error: {}", e),
            Self::Task(e) => write!(f, "task error: {}", e),
            Self::Write(e) => write!(f, "write error: {}", e),
        }
    }
}

impl<'r> rocket::response::Responder<'r, 'static> for Error {
    fn respond_to(self, _: &rocket::Request<'_>) -> rocket::response::Result<'static> {
        match self {
            Self::CellId => Err(Status::NotFound),
            Self::Horrorshow(_) => Err(Status::InternalServerError),
            Self::Json(_) => Err(Status::InternalServerError),
            Self::RamDecode(_) => Err(Status::InternalServerError),
            Self::Read(_) => Err(Status::InternalServerError),
            Self::Rocket(_) => Err(Status::InternalServerError),
            Self::RoomName => Err(Status::NotFound),
            Self::Sql(_) => Err(Status::InternalServerError),
            Self::Task(_) => Err(Status::InternalServerError),
            Self::Write(_) => Err(Status::InternalServerError),
        }
    }
}

#[derive(StructOpt)]
struct Args {} // for --help/--version support

#[wheel::main]
async fn main(Args {}: Args) -> Result<(), Error> {
    let pool = PgPool::connect_with(PgConnectOptions::default().database("oottracker").application_name("oottracker-web")).await?;
    let rooms = {
        let mut rooms = HashMap::default();
        let mut query = sqlx::query!(r#"SELECT name, knowledge as "knowledge: Json<Knowledge>", ram FROM rooms"#).fetch(&pool);
        while let Some(room) = query.try_next().await? {
            let state = RoomState::from_model(&room.name, ModelState { knowledge: room.knowledge.0, ram: Ram::from_range_bufs(room.ram)? });
            rooms.insert(room.name, state);
        }
        Rooms::new(Mutex::new(rooms))
    };
    //TODO force-save all rooms on stop
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
        let pool = pool.clone();
        let rooms = Rooms::clone(&rooms);
        let restreams = Restreams::clone(&restreams);
        let handler = warp::ws().and_then(move |ws| websocket::ws_handler(pool.clone(), Rooms::clone(&rooms), Restreams::clone(&restreams), ws));
        tokio::spawn(warp::serve(handler).run(([127, 0, 0, 1], 24808))).err_into()
    };
    let rocket_task = tokio::spawn(http::rocket(pool, rooms, restreams).launch()).map(|res| match res {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(Error::from(e)),
        Err(e) => Err(Error::from(e)),
    });
    let ((), ()) = tokio::try_join!(websocket_task, rocket_task)?;
    Ok(())
}
