use {
    std::{
        collections::VecDeque,
        num::NonZeroU8,
        sync::Arc,
        time::Duration,
    },
    futures::future::{
        Either,
        pending,
    },
    tokio::{
        select,
        sync::{
            RwLock,
            mpsc,
            watch,
        },
        time::{
            Instant,
            sleep_until,
        },
    },
    oottracker::{
        ModelState,
        Save,
        websocket::MwItem,
    },
};

const TRIFORCE_PIECE: u16 = 0x00ca;

pub(crate) enum AutoUpdate {
    Queue {
        item: MwItem,
        target_world: NonZeroU8,
    },
    Reset {
        world: NonZeroU8,
        save: Save,
    },
}

pub(crate) struct MwState {
    pub(crate) worlds: Vec<(watch::Sender<()>, watch::Receiver<()>, ModelState, Vec<MwItem>)>,
    pub(crate) autotracker_delay: Duration,
    pub(crate) incoming_queue: mpsc::UnboundedSender<AutoUpdate>,
}

impl MwState {
    pub(crate) fn new(worlds: Vec<(Option<Save>, Vec<MwItem>)>) -> Arc<RwLock<Self>> {
        let (incoming_queue, mut rx) = mpsc::unbounded_channel();
        let this = Arc::new(RwLock::new(Self {
            worlds: worlds.into_iter().map(|(save, queue)| {
                let (tx, rx) = watch::channel(());
                (tx, rx, ModelState { ram: save.unwrap_or_default().into(), knowledge: Default::default(), tracker_ctx: Default::default() }, queue)
            }).collect(),
            autotracker_delay: Duration::default(),
            incoming_queue,
        }));
        let this_clone = Arc::clone(&this);
        tokio::spawn(async move {
            let mut delay_queue = VecDeque::default();
            loop {
                let next_update = if let Some((due, _)) = delay_queue.get(0) {
                    Either::Left(sleep_until(*due))
                } else {
                    Either::Right(pending())
                };
                select! {
                    msg = rx.recv() => if let Some(elt) = msg {
                        delay_queue.push_back((Instant::now() + this_clone.read().await.autotracker_delay, elt));
                    } else {
                        for (due, update) in delay_queue {
                            sleep_until(due).await;
                            this_clone.write().await.handle_auto_update(update).expect("failed to handle delayed room update");
                        }
                        break
                    },
                    () = next_update => this_clone.write().await.handle_auto_update(delay_queue.pop_front().unwrap().1).expect("failed to handle delayed room update"),
                }
            }
        });
        this
    }

    pub(crate) fn world(&self, world: NonZeroU8) -> Option<(&watch::Sender<()>, &watch::Receiver<()>, &ModelState, &[MwItem])> {
        self.worlds.get(usize::from(world.get() - 1)).map(|(tx, rx, model, queue)| (tx, rx, model, &**queue))
    }

    pub(crate) fn world_mut(&mut self, world: NonZeroU8) -> Option<(&watch::Sender<()>, &watch::Receiver<()>, &mut ModelState, &mut Vec<MwItem>)> {
        self.worlds.get_mut(usize::from(world.get() - 1)).map(|(tx, rx, model, queue)| (&*tx, &*rx, model, &mut *queue))
    }

    fn handle_auto_update(&mut self, update: AutoUpdate) -> Result<(), ()> {
        match update {
            AutoUpdate::Queue { item, target_world } => if item.kind == TRIFORCE_PIECE {
                for (idx, (tx, _, model, queue)) in self.worlds.iter_mut().enumerate() {
                    if idx != usize::from(item.source.get()) - 1 {
                        if !queue.iter().any(|iter_item| iter_item.source == item.source && iter_item.key == item.key) {
                            queue.push(item);
                        }
                    }
                    model.ram.save.recv_mw_item(item.kind)?;
                    tx.send(()).expect("failed to notify websockets about state change");
                }
            } else if item.source == target_world {
                let (tx, _, model, _) = self.world_mut(target_world).ok_or(())?;
                model.ram.save.recv_mw_item(item.kind)?;
                tx.send(()).expect("failed to notify websockets about state change");
            } else {
                let (tx, _, model, queue) = self.world_mut(target_world).ok_or(())?;
                if !queue.iter().any(|iter_item| iter_item.source == item.source && iter_item.key == item.key) {
                    queue.push(item);
                }
                model.ram.save.recv_mw_item(item.kind)?;
                tx.send(()).expect("failed to notify websockets about state change");
            },
            AutoUpdate::Reset { world, save } => if let Some((tx, _, model, queue)) = self.world_mut(world) {
                model.ram.save = save;
                for &item in &queue[model.ram.save.inv_amounts.num_received_mw_items.into()..] {
                    model.ram.save.recv_mw_item(item.kind)?;
                }
                tx.send(()).expect("failed to notify websockets about state change");
            } else {
                return Err(())
            }
        }
        Ok(())
    }
}
