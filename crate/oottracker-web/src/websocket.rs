use {
    std::{
        convert::TryInto as _,
        sync::Arc,
        time::Duration,
    },
    async_proto::Protocol,
    futures::stream::{
        SplitSink,
        Stream,
        StreamExt as _,
    },
    iced_core::keyboard::Modifiers as KeyboardModifiers,
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
    oottracker::websocket::{
        ClientMessage,
        ServerMessage,
    },
    crate::{
        Error,
        Restreams,
        Rooms,
        restream::render_double_cell,
    },
};

type WsSink = Arc<Mutex<SplitSink<WebSocket, Message>>>;

async fn client_session(rooms: Rooms, restreams: Restreams, mut stream: impl Stream<Item = Result<Message, warp::Error>> + Unpin + Send, sink: WsSink) -> Result<(), Error> {
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
                        let cells = layout.cells().into_iter()
                            .map(|cell| cell.id.kind().render(&runner))
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
                            layout.cells().into_iter().map(|cell| cell.id.kind().render(&runner)).collect::<Vec<_>>()
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
            ClientMessage::ClickRestream { restream, runner, layout, cell_id, right } => {
                let mut restreams = restreams.write().await;
                let restream = match restreams.get_mut(&restream) {
                    Some(restream) => restream,
                    None => {
                        let _ = ServerMessage::from_error("no such restream").write_warp(&mut *sink.lock().await).await; //TODO better error handling
                        return Ok(())
                    }
                };
                let (tx, runner) = match restream.runner_mut(&runner) {
                    Some((tx, _, runner)) => (tx, runner),
                    None => {
                        let _ = ServerMessage::from_error("no such runner").write_warp(&mut *sink.lock().await).await; //TODO better error handling
                        return Ok(())
                    }
                };
                let cell = match layout.cells().get(usize::from(cell_id)) {
                    Some(cell) => cell.id,
                    None => {
                        let _ = ServerMessage::from_error("no such cell").write_warp(&mut *sink.lock().await).await; //TODO better error handling
                        return Ok(())
                    }
                };
                if right {
                    let _ /* no med right-click menu in web app */ = cell.kind().right_click(true /*TODO verify that the client has access?*/, KeyboardModifiers::default(), runner);
                } else {
                    let _ /* no med right-click menu in web app */ = cell.kind().left_click(true /*TODO verify that the client has access?*/, KeyboardModifiers::default(), runner);
                }
                tx.send(()).expect("failed to notify websockets about state change");
            }
            ClientMessage::SubscribeRaw { room } => {
                let rooms = Rooms::clone(&rooms);
                let sink = WsSink::clone(&sink);
                tokio::spawn(async move {
                    let (mut old_model, mut rx) = {
                        let mut rooms = rooms.lock().await;
                        let room = rooms.entry(room.clone()).or_default();
                        let model = room.model.clone();
                        if ServerMessage::InitRaw(model.clone()).write_warp(&mut *sink.lock().await).await.is_err() { return } //TODO better error handling
                        (model, room.rx.clone())
                    };
                    while let Ok(()) = rx.changed().await { //TODO better error handling
                        let new_model = {
                            let mut rooms = rooms.lock().await;
                            let room = rooms.entry(room.clone()).or_default();
                            room.model.clone()
                        };
                        if old_model != new_model {
                            if (ServerMessage::UpdateRaw(&new_model - &old_model)).write_warp(&mut *sink.lock().await).await.is_err() { return } //TODO better error handling
                        }
                        old_model = new_model;
                    }
                });
            }
            ClientMessage::SubscribeRoom { room, layout } => {
                let rooms = Rooms::clone(&rooms);
                let sink = WsSink::clone(&sink);
                tokio::spawn(async move {
                    let (mut old_cells, mut rx) = {
                        let mut rooms = rooms.lock().await;
                        let room = rooms.entry(room.clone()).or_default();
                        let cells = layout.cells().into_iter()
                            .map(|cell| cell.id.kind().render(&room.model))
                            .collect::<Vec<_>>();
                        if ServerMessage::Init(cells.clone()).write_warp(&mut *sink.lock().await).await.is_err() { return } //TODO better error handling
                        (cells, room.rx.clone())
                    };
                    while let Ok(()) = rx.changed().await { //TODO better error handling
                        let new_cells = {
                            let mut rooms = rooms.lock().await;
                            let room = rooms.entry(room.clone()).or_default();
                            layout.cells().into_iter().map(|cell| cell.id.kind().render(&room.model)).collect::<Vec<_>>()
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
            ClientMessage::SetRaw { room, state } => {
                let mut rooms = rooms.lock().await;
                let room = rooms.entry(room.clone()).or_default();
                room.model = state;
                room.tx.send(()).expect("failed to notify websockets about state change");
            }
            ClientMessage::ClickRoom { room, layout, cell_id, right } => {
                let mut rooms = rooms.lock().await;
                let room = rooms.entry(room.clone()).or_default();
                let cell = match layout.cells().get(usize::from(cell_id)) {
                    Some(cell) => cell.id,
                    None => {
                        let _ = ServerMessage::from_error("no such cell").write_warp(&mut *sink.lock().await).await; //TODO better error handling
                        return Ok(())
                    }
                };
                if right {
                    let _ /* no med right-click menu in web app */ = cell.kind().right_click(true /*TODO verify that the client has access?*/, KeyboardModifiers::default(), &mut room.model);
                } else {
                    let _ /* no med right-click menu in web app */ = cell.kind().left_click(true /*TODO verify that the client has access?*/, KeyboardModifiers::default(), &mut room.model);
                }
                room.tx.send(()).expect("failed to notify websockets about state change");
            }
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
