use {
    std::{
        fmt,
        num::NonZeroU8,
    },
    async_proto::Protocol,
    crate::{
        ModelDelta,
        ModelState,
        Save,
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
        state: ModelState,
    },
    MwCreateRoom {
        room: String,
        worlds: Vec<(Option<Save>, Vec<u16>)>,
    },
    MwDeleteRoom {
        room: String,
    },
    MwResetPlayer {
        room: String,
        world: NonZeroU8,
        save: Save,
    },
    MwGetItem {
        room: String,
        world: NonZeroU8,
        item: u16,
    },
    ClickMw {
        room: String,
        world: NonZeroU8,
        layout: TrackerLayout,
        cell_id: u8,
        right: bool,
    },
    SubscribeMw {
        room: String,
        world: NonZeroU8,
        layout: TrackerLayout,
    },
    MwGetItemAll {
        room: String,
        item: u16,
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
    InitRaw(ModelState),
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
