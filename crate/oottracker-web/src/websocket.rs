use {
    std::{
        convert::TryInto as _,
        fmt,
        sync::Arc,
        time::Duration,
    },
    async_proto::Protocol,
    futures::stream::{
        SplitSink,
        Stream,
        StreamExt as _,
    },
    tokio::{
        sync::Mutex,
        time::sleep,
    },
    warp::{
        reject::Rejection,
        reply::Reply,
        ws::{
            Message,
            WebSocket,
        },
    },
    crate::{
        CellRender,
        Error,
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

type WsSink = Arc<Mutex<SplitSink<WebSocket, Message>>>;

#[derive(Protocol)]
enum ServerMessage {
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
}

impl ServerMessage {
    fn from_error(e: impl fmt::Debug + fmt::Display) -> ServerMessage {
        ServerMessage::Error {
            debug: format!("{:?}", e),
            display: e.to_string(),
        }
    }
}

#[derive(Protocol)]
enum ClientMessage {
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
}

async fn client_session(_ /*rooms*/: Rooms, restreams: Restreams, mut stream: impl Stream<Item = Result<Message, warp::Error>> + Unpin + Send, sink: WsSink) -> Result<(), Error> {
    let ping_sink = WsSink::clone(&sink);
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(30)).await;
            if ServerMessage::Ping.write_warp(&mut *ping_sink.lock().await).await.is_err() { break } //TODO better error handling
        }
    });
    loop {
        match ClientMessage::read_warp(&mut stream).await? {
            ClientMessage::Pong => {}
            ClientMessage::SubscribeRestream { restream, runner, layout } => {
                let restreams = Restreams::clone(&restreams);
                let sink = WsSink::clone(&sink);
                tokio::spawn(async move {
                    let (mut old_cells, mut rx) = {
                        let restreams = restreams.read().await;
                        let restream = match restreams.get(&restream) {
                            Some(restream) => restream,
                            None => {
                                let _ = ServerMessage::from_error("no such restream").write_warp(&mut *sink.lock().await).await; //TODO better error handling
                                return
                            }
                        };
                        let (rx, runner) = match restream.runner(&runner) {
                            Some((_, rx, runner)) => (rx, runner),
                            None => {
                                let _ = ServerMessage::from_error("no such runner").write_warp(&mut *sink.lock().await).await; //TODO better error handling
                                return
                            }
                        };
                        let cells = layout.cells()
                            .map(|(cell, _, _)| cell.kind().render(&runner))
                            .collect::<Vec<_>>();
                        if ServerMessage::Init(cells.clone()).write_warp(&mut *sink.lock().await).await.is_err() { return } //TODO better error handling
                        (cells, rx.clone())
                    };
                    while let Ok(()) = rx.changed().await { //TODO better error handling
                        let new_cells = {
                            let restreams = restreams.read().await;
                            let restream = match restreams.get(&restream) {
                                Some(restream) => restream,
                                None => {
                                    let _ = ServerMessage::from_error("no such restream").write_warp(&mut *sink.lock().await).await; //TODO better error handling
                                    return
                                }
                            };
                            let runner = match restream.runner(&runner) {
                                Some((_, _, runner)) => runner,
                                None => {
                                    let _ = ServerMessage::from_error("no such runner").write_warp(&mut *sink.lock().await).await; //TODO better error handling
                                    return
                                }
                            };
                            layout.cells().map(|(cell, _, _)| cell.kind().render(&runner)).collect::<Vec<_>>()
                        };
                        for (i, (old_cell, new_cell)) in old_cells.iter().zip(&new_cells).enumerate() {
                            if old_cell != new_cell {
                                if (ServerMessage::Update { cell_id: i.try_into().expect("too many cells"), new_cell: new_cell.clone() }).write_warp(&mut *sink.lock().await).await.is_err() { return } //TODO better error handling
                            }
                        }
                        old_cells = new_cells;
                    }
                });
            }
            ClientMessage::SubscribeDoubleRestream { restream, runner1, runner2, layout } => {
                let restreams = Restreams::clone(&restreams);
                let sink = WsSink::clone(&sink);
                tokio::spawn(async move {
                    let (mut old_cells, mut rx) = {
                        let restreams = restreams.read().await;
                        let restream = match restreams.get(&restream) {
                            Some(restream) => restream,
                            None => {
                                let _ = ServerMessage::from_error("no such restream").write_warp(&mut *sink.lock().await).await; //TODO better error handling
                                return
                            }
                        };
                        let (rx, runner1) = match restream.runner(&runner1) {
                            Some((_, rx, runner)) => (rx, runner),
                            None => {
                                let _ = ServerMessage::from_error("no such runner").write_warp(&mut *sink.lock().await).await; //TODO better error handling
                                return
                            }
                        };
                        let runner2 = match restream.runner(&runner2) {
                            Some((_, _, runner)) => runner,
                            None => {
                                let _ = ServerMessage::from_error("no such runner").write_warp(&mut *sink.lock().await).await; //TODO better error handling
                                return
                            }
                        };
                        let cells = layout.cells().into_iter()
                            .map(|reward| render_double_cell(runner1, runner2, reward))
                            .collect::<Vec<_>>();
                        if ServerMessage::Init(cells.clone()).write_warp(&mut *sink.lock().await).await.is_err() { return } //TODO better error handling
                        (cells, rx.clone())
                    };
                    while let Ok(()) = rx.changed().await { //TODO better error handling
                        let new_cells = {
                            let restreams = restreams.read().await;
                            let restream = match restreams.get(&restream) {
                                Some(restream) => restream,
                                None => {
                                    let _ = ServerMessage::from_error("no such restream").write_warp(&mut *sink.lock().await).await; //TODO better error handling
                                    return
                                }
                            };
                            let runner1 = match restream.runner(&runner1) {
                                Some((_, _, runner)) => runner,
                                None => {
                                    let _ = ServerMessage::from_error("no such runner").write_warp(&mut *sink.lock().await).await; //TODO better error handling
                                    return
                                }
                            };
                            let runner2 = match restream.runner(&runner2) {
                                Some((_, _, runner)) => runner,
                                None => {
                                    let _ = ServerMessage::from_error("no such runner").write_warp(&mut *sink.lock().await).await; //TODO better error handling
                                    return
                                }
                            };
                            layout.cells().into_iter().map(|reward| render_double_cell(runner1, runner2, reward)).collect::<Vec<_>>()
                        };
                        for (i, (old_cell, new_cell)) in old_cells.iter().zip(&new_cells).enumerate() {
                            if old_cell != new_cell {
                                if (ServerMessage::Update { cell_id: i.try_into().expect("too many cells"), new_cell: new_cell.clone() }).write_warp(&mut *sink.lock().await).await.is_err() { return } //TODO better error handling
                            }
                        }
                        old_cells = new_cells;
                    }
                });
            }
            //TODO allow subscriptions for regular rooms
            //TODO accept client messages to update the room
        }
    }
}

async fn client_connection(rooms: Rooms, restreams: Restreams, ws: WebSocket) {
    let (ws_sink, ws_stream) = ws.split();
    let ws_sink = WsSink::new(Mutex::new(ws_sink));
    if let Err(e) = client_session(rooms, restreams, ws_stream, WsSink::clone(&ws_sink)).await {
        let _ = ServerMessage::from_error(e).write_warp(&mut *ws_sink.lock().await).await;
    }
}

pub(crate) async fn ws_handler(rooms: Rooms, restreams: Restreams, ws: warp::ws::Ws) -> Result<impl Reply, Rejection> {
    Ok(ws.on_upgrade(|ws| client_connection(rooms, restreams, ws)))
}
