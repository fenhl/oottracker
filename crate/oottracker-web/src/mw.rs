use {
    std::{
        collections::{
            HashMap,
            HashSet,
            VecDeque,
        },
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
    ootr::model::{
        DungeonReward,
        DungeonRewardLocation,
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
    DungeonRewardLocation {
        world: NonZeroU8,
        reward: DungeonReward,
        location: DungeonRewardLocation,
    },
}

pub(crate) struct MwState {
    pub(crate) worlds: Vec<(watch::Sender<()>, watch::Receiver<()>, ModelState, Vec<MwItem>, HashSet<MwItem>)>,
    pub(crate) autotracker_delay: Duration,
    pub(crate) incoming_queue: mpsc::UnboundedSender<AutoUpdate>,
    pub(crate) location_cache: HashMap<NonZeroU8, HashMap<u64, String>>,
    pub(crate) item_cache: HashMap<u16, String>,
}

impl MwState {
    pub(crate) fn new(worlds: Vec<(ModelState, Vec<MwItem>)>) -> Arc<RwLock<Self>> {
        let (incoming_queue, mut rx) = mpsc::unbounded_channel();
        let this = Arc::new(RwLock::new(Self {
            worlds: worlds.into_iter().map(|(model, queue)| {
                let (tx, rx) = watch::channel(());
                (tx, rx, model, queue, HashSet::default())
            }).collect(),
            autotracker_delay: Duration::default(),
            location_cache: HashMap::default(),
            item_cache: HashMap::default(),
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

    pub(crate) fn world(&self, world: NonZeroU8) -> Option<(&watch::Sender<()>, &watch::Receiver<()>, &ModelState, &[MwItem], &HashSet<MwItem>)> {
        self.worlds.get(usize::from(world.get() - 1)).map(|(tx, rx, model, queue, own_items)| (tx, rx, model, &**queue, own_items))
    }

    pub(crate) fn world_mut(&mut self, world: NonZeroU8) -> Option<(&watch::Sender<()>, &watch::Receiver<()>, &mut ModelState, &mut Vec<MwItem>, &mut HashSet<MwItem>)> {
        self.worlds.get_mut(usize::from(world.get() - 1)).map(|(tx, rx, model, queue, own_items)| (&*tx, &*rx, model, queue, own_items))
    }

    fn handle_auto_update(&mut self, update: AutoUpdate) -> Result<(), ()> {
        match update {
            AutoUpdate::Queue { item, target_world } => if item.kind == TRIFORCE_PIECE {
                for (idx, (tx, _, model, queue, own_items)) in self.worlds.iter_mut().enumerate() {
                    if idx == usize::from(item.source.get()) - 1 {
                        own_items.insert(item);
                    } else {
                        if !queue.iter().any(|iter_item| iter_item.source == item.source && iter_item.key == item.key) {
                            queue.push(item);
                        }
                    }
                    model.ram.save.recv_mw_item(item.kind)?;
                    tx.send(()).expect("failed to notify websockets about state change");
                }
            } else {
                let (tx, _, model, queue, own_items) = self.world_mut(target_world).ok_or(())?;
                if item.source == target_world {
                    own_items.insert(item);
                } else {
                    if !queue.iter().any(|iter_item| iter_item.source == item.source && iter_item.key == item.key) {
                        queue.push(item);
                    }
                }
                model.ram.save.recv_mw_item(item.kind)?;
                tx.send(()).expect("failed to notify websockets about state change");
            },
            AutoUpdate::Reset { world, save } => if let Some((tx, _, model, queue, _)) = self.world_mut(world) {
                model.ram.save = save;
                for &item in &queue[model.ram.save.inv_amounts.num_received_mw_items.into()..] {
                    model.ram.save.recv_mw_item(item.kind)?;
                }
                tx.send(()).expect("failed to notify websockets about state change");
            } else {
                return Err(())
            },
            AutoUpdate::DungeonRewardLocation { world, reward, location } => if let Some((tx, _, model, _, _)) = self.world_mut(world) {
                if model.knowledge.dungeon_reward_locations.insert(reward, location) != Some(location) {
                    tx.send(()).expect("failed to notify websockets about state change");
                }
            } else {
                return Err(())
            },
        }
        Ok(())
    }
}
