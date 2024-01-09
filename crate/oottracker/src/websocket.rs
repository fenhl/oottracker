#![allow(deprecated)] // avoid deprecation errors in the Protocol derivation for ClientMessage

use {
    std::{
        fmt,
        num::NonZeroU8,
    },
    async_proto::Protocol,
    ootr::model::{
        DungeonReward,
        DungeonRewardLocation,
    },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Protocol)]
pub struct MwItem {
    pub source: NonZeroU8,
    pub key: u64,
    pub kind: u16,
}

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
        worlds: Vec<(Option<Save>, Vec<MwItem>)>,
    },
    MwDeleteRoom {
        room: String,
    },
    MwResetPlayer {
        room: String,
        world: NonZeroU8,
        save: Save,
    },
    /// No longer supported. Use `MwQueueItem` instead.
    #[deprecated]
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
    /// No longer supported. Use `MwQueueItem` instead.
    #[deprecated]
    MwGetItemAll {
        room: String,
        item: u16,
    },
    MwQueueItem {
        room: String,
        source_world: NonZeroU8,
        key: u64,
        kind: u16,
        target_world: NonZeroU8,
    },
    MwDungeonRewardLocation {
        room: String,
        world: NonZeroU8,
        reward: DungeonReward,
        location: DungeonRewardLocation,
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
            debug: format!("{e:?}"),
            display: e.to_string(),
        }
    }
}
