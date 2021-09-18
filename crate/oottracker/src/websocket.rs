use {
    std::fmt,
    async_proto::Protocol,
    crate::{
        ModelDelta,
        ModelState,
        ui::{
            CellRender,
            TrackerLayout,
            DoubleTrackerLayout,
        },
    },
};

#[derive(Protocol)]
pub enum ClientMessage {
    Pong,
    SubscribeRestream {
        restream: String,
        runner: String,
        layout: TrackerLayout,
    },
    SubscribeDoubleRestream {
        restream: String,
        runner1: String,
        runner2: String,
        layout: DoubleTrackerLayout,
    },
    ClickRestream {
        restream: String,
        runner: String,
        layout: TrackerLayout,
        cell_id: u8,
        right: bool,
    },
    SubscribeRoom {
        room: String,
        layout: TrackerLayout,
    },
    ClickRoom {
        room: String,
        layout: TrackerLayout,
        cell_id: u8,
        right: bool,
    },
    SubscribeRaw {
        room: String,
    },
    SetRaw {
        room: String,
        state: ModelState<ootr_static::Rando>, //TODO support other Rando impls?
    },
}

#[derive(Protocol)]
pub enum ServerMessage {
    Ping,
    Error {
        debug: String,
        display: String,
    },
    Init(Vec<CellRender>),
    Update {
        cell_id: u8,
        new_cell: CellRender,
    },
    InitRaw(ModelState<ootr_static::Rando>), //TODO support other Rando impls?
    UpdateRaw(ModelDelta),
}

impl ServerMessage {
    pub fn from_error(e: impl fmt::Debug + fmt::Display) -> ServerMessage {
        ServerMessage::Error {
            debug: format!("{:?}", e),
            display: e.to_string(),
        }
    }
}
