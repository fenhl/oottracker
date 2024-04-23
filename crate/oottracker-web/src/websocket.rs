use {
    std::{
        sync::Arc,
        time::Duration,
    },
    async_proto::Protocol,
    futures::stream::{
        SplitSink,
        SplitStream,
        StreamExt as _,
    },
    iced_core::keyboard::Modifiers as KeyboardModifiers,
    rocket_ws::Message,
    sqlx::PgPool,
    tokio::{
        sync::Mutex,
        time::sleep,
    },
    tokio_tungstenite::tungstenite,
    oottracker::websocket::{
        ClientMessage,
        MwItem,
        ServerMessage,
    },
    crate::{
        Error,
        MwRooms,
        Restreams,
        Rooms,
        edit_room,
        get_room,
        mw::{
            AutoUpdate,
            MwState,
        },
        restream::render_double_cell,
    },
};

type WsStream = SplitStream<rocket_ws::stream::DuplexStream>;
type WsSink = Arc<Mutex<SplitSink<rocket_ws::stream::DuplexStream, Message>>>;

async fn client_session(pool: &PgPool, rooms: Rooms, restreams: Restreams, mw_rooms: MwRooms, mut stream: WsStream, sink: WsSink) -> Result<(), Error> {
    let ping_sink = WsSink::clone(&sink);
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(30)).await;
            if ServerMessage::Ping.write_ws(&mut *ping_sink.lock().await).await.is_err() { break } //TODO better error handling
        }
    });
    loop {
        match dbg!(ClientMessage::read_ws(&mut stream).await?) {
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
                                let _ = ServerMessage::from_error("no such restream").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                                return
                            }
                        };
                        let (rx, runner) = match restream.runner(&runner) {
                            Some((_, rx, runner)) => (rx, runner),
                            None => {
                                let _ = ServerMessage::from_error("no such runner").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                                return
                            }
                        };
                        let cells = layout.cells().into_iter()
                            .map(|cell| cell.id.kind().render(&runner))
                            .collect::<Vec<_>>();
                        if ServerMessage::Init(cells.clone()).write_ws(&mut *sink.lock().await).await.is_err() { return } //TODO better error handling
                        (cells, rx.clone())
                    };
                    while let Ok(()) = rx.changed().await { //TODO better error handling
                        let new_cells = {
                            let restreams = restreams.read().await;
                            let restream = match restreams.get(&restream) {
                                Some(restream) => restream,
                                None => {
                                    let _ = ServerMessage::from_error("no such restream").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                                    return
                                }
                            };
                            let runner = match restream.runner(&runner) {
                                Some((_, _, runner)) => runner,
                                None => {
                                    let _ = ServerMessage::from_error("no such runner").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                                    return
                                }
                            };
                            layout.cells().into_iter().map(|cell| cell.id.kind().render(&runner)).collect::<Vec<_>>()
                        };
                        for (i, (old_cell, new_cell)) in old_cells.iter().zip(&new_cells).enumerate() {
                            if old_cell != new_cell {
                                if (ServerMessage::Update { cell_id: i.try_into().expect("too many cells"), new_cell: new_cell.clone() }).write_ws(&mut *sink.lock().await).await.is_err() { return } //TODO better error handling
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
                                let _ = ServerMessage::from_error("no such restream").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                                return
                            }
                        };
                        let (rx, runner1) = match restream.runner(&runner1) {
                            Some((_, rx, runner)) => (rx, runner),
                            None => {
                                let _ = ServerMessage::from_error("no such runner").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                                return
                            }
                        };
                        let runner2 = match restream.runner(&runner2) {
                            Some((_, _, runner)) => runner,
                            None => {
                                let _ = ServerMessage::from_error("no such runner").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                                return
                            }
                        };
                        let cells = layout.cells().into_iter()
                            .map(|reward| render_double_cell(runner1, runner2, reward))
                            .collect::<Vec<_>>();
                        if ServerMessage::Init(cells.clone()).write_ws(&mut *sink.lock().await).await.is_err() { return } //TODO better error handling
                        (cells, rx.clone())
                    };
                    while let Ok(()) = rx.changed().await { //TODO better error handling
                        let new_cells = {
                            let restreams = restreams.read().await;
                            let restream = match restreams.get(&restream) {
                                Some(restream) => restream,
                                None => {
                                    let _ = ServerMessage::from_error("no such restream").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                                    return
                                }
                            };
                            let runner1 = match restream.runner(&runner1) {
                                Some((_, _, runner)) => runner,
                                None => {
                                    let _ = ServerMessage::from_error("no such runner").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                                    return
                                }
                            };
                            let runner2 = match restream.runner(&runner2) {
                                Some((_, _, runner)) => runner,
                                None => {
                                    let _ = ServerMessage::from_error("no such runner").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                                    return
                                }
                            };
                            layout.cells().into_iter().map(|reward| render_double_cell(runner1, runner2, reward)).collect::<Vec<_>>()
                        };
                        for (i, (old_cell, new_cell)) in old_cells.iter().zip(&new_cells).enumerate() {
                            if old_cell != new_cell {
                                if (ServerMessage::Update { cell_id: i.try_into().expect("too many cells"), new_cell: new_cell.clone() }).write_ws(&mut *sink.lock().await).await.is_err() { return } //TODO better error handling
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
                        let _ = ServerMessage::from_error("no such restream").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                        return Ok(())
                    }
                };
                let (tx, runner) = match restream.runner_mut(&runner) {
                    Some((tx, _, runner)) => (tx, runner),
                    None => {
                        let _ = ServerMessage::from_error("no such runner").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                        return Ok(())
                    }
                };
                let cell = match layout.cells().get(usize::from(cell_id)) {
                    Some(cell) => cell.id,
                    None => {
                        let _ = ServerMessage::from_error("no such cell").write_ws(&mut *sink.lock().await).await; //TODO better error handling
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
                    let (mut old_model, mut rx) = get_room(&rooms, room.clone(), |room| (room.model.clone(), room.rx.clone())).await?;
                    ServerMessage::InitRaw(old_model.clone()).write_ws(&mut *sink.lock().await).await?;
                    while let Ok(()) = rx.changed().await {
                        let new_model = get_room(&rooms, room.clone(), |room| room.model.clone()).await?;
                        if old_model != new_model {
                            (ServerMessage::UpdateRaw(&new_model - &old_model)).write_ws(&mut *sink.lock().await).await?;
                        }
                        old_model = new_model;
                    }
                    Ok::<_, Error>(())
                }); //TODO send errors from task to client
            }
            ClientMessage::SubscribeRoom { room, layout } => {
                let rooms = Rooms::clone(&rooms);
                let sink = WsSink::clone(&sink);
                tokio::spawn(async move {
                    let (mut old_cells, mut rx) = get_room(&rooms, room.clone(), |room| (
                        layout.cells().into_iter()
                            .map(|cell| cell.id.kind().render(&room.model))
                            .collect::<Vec<_>>(),
                        room.rx.clone(),
                    )).await?;
                    ServerMessage::Init(old_cells.clone()).write_ws(&mut *sink.lock().await).await?;
                    while let Ok(()) = rx.changed().await {
                        let new_cells = get_room(&rooms, room.clone(), |room| layout.cells().into_iter().map(|cell| cell.id.kind().render(&room.model)).collect::<Vec<_>>()).await?;
                        for (i, (old_cell, new_cell)) in old_cells.iter().zip(&new_cells).enumerate() {
                            if old_cell != new_cell {
                                (ServerMessage::Update { cell_id: i.try_into().expect("too many cells"), new_cell: new_cell.clone() }).write_ws(&mut *sink.lock().await).await?;
                            }
                        }
                        old_cells = new_cells;
                    }
                    Ok::<_, Error>(())
                }); //TODO send errors from task to client
            }
            ClientMessage::SetRaw { room, state } => edit_room(pool, &rooms, room, |room| { room.model = state; Ok(()) }).await?,
            ClientMessage::ClickRoom { room, layout, cell_id, right } => {
                let cell = match layout.cells().get(usize::from(cell_id)) {
                    Some(cell) => cell.id,
                    None => {
                        let _ = ServerMessage::from_error("no such cell").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                        return Ok(())
                    }
                };
                edit_room(pool, &rooms, room, |room| {
                    if right {
                        let _ /* no med right-click menu in web app */ = cell.kind().right_click(true /*TODO verify that the client has access?*/, KeyboardModifiers::default(), &mut room.model);
                    } else {
                        let _ /* no med right-click menu in web app */ = cell.kind().left_click(true /*TODO verify that the client has access?*/, KeyboardModifiers::default(), &mut room.model);
                    }
                    Ok(())
                }).await?;
            }
            ClientMessage::MwCreateRoom { room, worlds } => {
                mw_rooms.write().await.insert(room, MwState::new(worlds));
            }
            ClientMessage::MwDeleteRoom { room } => {
                mw_rooms.write().await.remove(&room);
            }
            ClientMessage::MwResetPlayer { room, world, save } => if let Some(room) = mw_rooms.read().await.get(&room) {
                let _ = room.read().await.incoming_queue.send(AutoUpdate::Reset { world, save });
            } else {
                let _ = ServerMessage::from_error("no such multiworld room").write_ws(&mut *sink.lock().await).await; //TODO better error handling
            },
            #[allow(deprecated)]
            ClientMessage::MwGetItem { .. } => {
                let _ = ServerMessage::from_error("MwGetItem command is no longer supported, use MwQueueItem instead").write_ws(&mut *sink.lock().await).await; //TODO better error handling
            }
            ClientMessage::ClickMw { room, world, layout, cell_id, right } => {
                let mw_rooms = mw_rooms.read().await;
                let mw_room = match mw_rooms.get(&room) {
                    Some(mw_room) => mw_room,
                    None => {
                        let _ = ServerMessage::from_error("no such multiworld room").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                        return Ok(())
                    }
                };
                let mut mw_room = mw_room.write().await;
                let (tx, model) = match mw_room.world_mut(world) {
                    Some((tx, _, model, _, _)) => (tx, model),
                    None => {
                        let _ = ServerMessage::from_error("no such world").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                        return Ok(())
                    }
                };
                let cell = match layout.cells().get(usize::from(cell_id)) {
                    Some(cell) => cell.id,
                    None => {
                        let _ = ServerMessage::from_error("no such cell").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                        return Ok(())
                    }
                };
                if right {
                    let _ /* no med right-click menu in web app */ = cell.kind().right_click(true /*TODO verify that the client has access?*/, KeyboardModifiers::default(), model);
                } else {
                    let _ /* no med right-click menu in web app */ = cell.kind().left_click(true /*TODO verify that the client has access?*/, KeyboardModifiers::default(), model);
                }
                tx.send(()).expect("failed to notify websockets about state change");
            }
            ClientMessage::SubscribeMw { room, world, layout } => {
                let mw_rooms = MwRooms::clone(&mw_rooms);
                let sink = WsSink::clone(&sink);
                tokio::spawn(async move {
                    let (mut old_cells, mut rx) = {
                        let mw_rooms = mw_rooms.read().await;
                        let mw_room = match mw_rooms.get(&room) {
                            Some(mw_room) => mw_room,
                            None => {
                                let _ = ServerMessage::from_error("no such multiworld room").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                                return
                            }
                        };
                        let mw_room = mw_room.read().await;
                        let (rx, model) = match mw_room.world(world) {
                            Some((_, rx, model, _, _)) => (rx, model),
                            None => {
                                let _ = ServerMessage::from_error("no such world").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                                return
                            }
                        };
                        let cells = layout.cells().into_iter()
                            .map(|cell| cell.id.kind().render(&model))
                            .collect::<Vec<_>>();
                        if ServerMessage::Init(cells.clone()).write_ws(&mut *sink.lock().await).await.is_err() { return } //TODO better error handling
                        (cells, rx.clone())
                    };
                    while let Ok(()) = rx.changed().await { //TODO better error handling
                        let new_cells = {
                            let mw_rooms = mw_rooms.read().await;
                            let mw_room = match mw_rooms.get(&room) {
                                Some(mw_room) => mw_room,
                                None => {
                                    let _ = ServerMessage::from_error("no such multiworld room").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                                    return
                                }
                            };
                            let mw_room = mw_room.read().await;
                            let model = match mw_room.world(world) {
                                Some((_, _, model, _, _)) => model,
                                None => {
                                    let _ = ServerMessage::from_error("no such world").write_ws(&mut *sink.lock().await).await; //TODO better error handling
                                    return
                                }
                            };
                            layout.cells().into_iter().map(|cell| cell.id.kind().render(&model)).collect::<Vec<_>>()
                        };
                        for (i, (old_cell, new_cell)) in old_cells.iter().zip(&new_cells).enumerate() {
                            if old_cell != new_cell {
                                if (ServerMessage::Update { cell_id: i.try_into().expect("too many cells"), new_cell: new_cell.clone() }).write_ws(&mut *sink.lock().await).await.is_err() { return } //TODO better error handling
                            }
                        }
                        old_cells = new_cells;
                    }
                });
            }
            #[allow(deprecated)]
            ClientMessage::MwGetItemAll { .. } => {
                let _ = ServerMessage::from_error("MwGetItemAll command is no longer supported, use MwQueueItem instead").write_ws(&mut *sink.lock().await).await; //TODO better error handling
            }
            ClientMessage::MwQueueItem { room, source_world, key, kind, target_world } => if let Some(room) = mw_rooms.read().await.get(&room) {
                let _ = room.read().await.incoming_queue.send(AutoUpdate::Queue { item: MwItem { source: source_world, key, kind }, target_world });
            } else {
                let _ = ServerMessage::from_error("no such multiworld room").write_ws(&mut *sink.lock().await).await; //TODO better error handling
            },
            ClientMessage::MwDungeonRewardLocation { room, world, reward, location } => if let Some(room) = mw_rooms.read().await.get(&room) {
                let _ = room.read().await.incoming_queue.send(AutoUpdate::DungeonRewardLocation { world, reward, location });
            } else {
                let _ = ServerMessage::from_error("no such multiworld room").write_ws(&mut *sink.lock().await).await; //TODO better error handling
            },
        }
    }
}

pub(crate) async fn client_connection(pool: PgPool, rooms: Rooms, restreams: Restreams, mw_rooms: MwRooms, ws: rocket_ws::stream::DuplexStream) {
    let (ws_sink, ws_stream) = ws.split();
    let ws_sink = WsSink::new(Mutex::new(ws_sink));
    match client_session(&pool, rooms, restreams, mw_rooms, ws_stream, WsSink::clone(&ws_sink)).await {
        Ok(()) => {}
        Err(Error::Read(async_proto::ReadError { kind: async_proto::ReadErrorKind::MessageKind(tungstenite::Message::Close(_)), .. })) => {} // client disconnected normally
        Err(Error::Read(async_proto::ReadError { kind: async_proto::ReadErrorKind::Tungstenite(tungstenite::Error::Protocol(tungstenite::error::ProtocolError::ResetWithoutClosingHandshake)), .. })) => {} // this happens when a player force quits their tracker app (or normally quits on macOS, see https://github.com/iced-rs/iced/issues/1941)
        Err(e) => {
            eprintln!("error in WebSocket handler: {e}");
            eprintln!("debug info: {e:?}");
            let _ = wheel::night_report("/games/zelda/oot/tracker/error", Some(&format!("error in WebSocket handler: {e}\ndebug info: {e:?}"))).await;
            let _ = ServerMessage::from_error(e).write_ws(&mut *ws_sink.lock().await).await;
        }
    }
}
