use {
    std::{
        collections::{
            HashMap,
            HashSet,
            VecDeque,
        },
        num::NonZero,
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
        MainDungeon,
    },
    oottracker::{
        ModelState,
        save::{
            Bottle,
            Save,
        },
        websocket::MwItem,
    },
};

const TRIFORCE_PIECE: u16 = 0x00ca;

pub(crate) enum AutoUpdate {
    Queue {
        item: MwItem,
        target_world: NonZero<u8>,
    },
    Reset {
        world: NonZero<u8>,
        save: Save,
    },
    DungeonRewardLocation {
        world: NonZero<u8>,
        reward: DungeonReward,
        location: DungeonRewardLocation,
    },
    CurrentScene {
        world: NonZero<u8>,
        scene: u8,
    },
}

pub(crate) struct WorldState {
    tx: watch::Sender<()>,
    rx: watch::Receiver<()>,
    model: ModelState,
    pub(crate) queue: Vec<MwItem>,
    pub(crate) own_items: HashSet<MwItem>,
}

#[allow(unused)] // for consistency with `WorldState` and `WorldStateMut`
pub(crate) struct WorldStateRef<'a> {
    pub(crate) tx: &'a watch::Sender<()>,
    pub(crate) rx: &'a watch::Receiver<()>,
    pub(crate) model: &'a ModelState,
    pub(crate) queue: &'a [MwItem],
    pub(crate) own_items: &'a HashSet<MwItem>,
}

#[allow(unused)] // for consistency with `WorldState` and `WorldStateRef`
pub(crate) struct WorldStateMut<'a> {
    pub(crate) tx: &'a watch::Sender<()>,
    pub(crate) rx: &'a watch::Receiver<()>,
    pub(crate) model: &'a mut ModelState,
    pub(crate) queue: &'a mut Vec<MwItem>,
    pub(crate) own_items: &'a mut HashSet<MwItem>,
}

pub(crate) struct MwState {
    pub(crate) worlds: Vec<WorldState>,
    pub(crate) autotracker_delay: Duration,
    pub(crate) incoming_queue: mpsc::UnboundedSender<AutoUpdate>,
    pub(crate) location_cache: HashMap<NonZero<u8>, HashMap<u64, String>>,
    pub(crate) item_cache: HashMap<u16, String>,
}

impl MwState {
    pub(crate) fn new(worlds: Vec<(ModelState, Vec<MwItem>)>) -> Arc<RwLock<Self>> {
        let (incoming_queue, mut rx) = mpsc::unbounded_channel();
        let this = Arc::new(RwLock::new(Self {
            worlds: worlds.into_iter().map(|(model, queue)| {
                let (tx, rx) = watch::channel(());
                WorldState { tx, rx, model, queue, own_items: HashSet::default() }
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

    pub(crate) fn world(&self, world: NonZero<u8>) -> Option<WorldStateRef<'_>> {
        self.worlds.get(usize::from(world.get() - 1)).map(|WorldState { tx, rx, model, queue, own_items }| WorldStateRef { tx, rx, model, queue: &**queue, own_items })
    }

    pub(crate) fn world_mut(&mut self, world: NonZero<u8>) -> Option<WorldStateMut<'_>> {
        self.worlds.get_mut(usize::from(world.get() - 1)).map(|WorldState { tx, rx, model, queue, own_items }| WorldStateMut { tx: &*tx, rx: &*rx, model, queue, own_items })
    }

    fn handle_auto_update(&mut self, update: AutoUpdate) -> Result<(), ()> {
        match update {
            AutoUpdate::Queue { item, target_world } => {
                if item.key == 0x4d00_0000_0000_000f { // Market 10 Big Poes
                    let source_world = self.world_mut(item.source).ok_or(())?;
                    for bottle in &mut source_world.model.ram.save.inv.bottles {
                        if *bottle == Bottle::BigPoe {
                            *bottle = Bottle::Empty;
                            source_world.tx.send(()).expect("failed to notify websockets about state change");
                            break
                        }
                    }
                }
                if item.kind == TRIFORCE_PIECE {
                    for (idx, world) in self.worlds.iter_mut().enumerate() {
                        if idx == usize::from(item.source.get()) - 1 {
                            world.own_items.insert(item);
                        } else {
                            if !world.queue.iter().any(|iter_item| iter_item.source == item.source && iter_item.key == item.key) {
                                world.queue.push(item);
                            }
                        }
                        world.model.recv_mw_item(item)?;
                        world.tx.send(()).expect("failed to notify websockets about state change");
                    }
                } else if let Some(reward) = DungeonReward::from_get_item_id(item.kind) {
                    if item.source == target_world {
                        if let Some(location) = DungeonRewardLocation::from_override_key(item.key) {
                            let target_world = self.world_mut(target_world).ok_or(())?;
                            if let Some(location) = match location {
                                DungeonRewardLocation::LinksPocket => Some(DungeonRewardLocation::LinksPocket),
                                //HACK: check target world instead of source world since they've been checked to be equal above
                                DungeonRewardLocation::Dungeon(boss_room) => target_world.model.knowledge.boss_entrances.iter().find(|(_, v)| **v == boss_room).map(|(k, _)| DungeonRewardLocation::Dungeon(*k)),
                            } {
                                if target_world.model.knowledge.dungeon_reward_locations.insert(reward, location) != Some(location) {
                                    target_world.tx.send(()).expect("failed to notify websockets about state change");
                                }
                            }
                        }
                    }
                } else {
                    let world = self.world_mut(target_world).ok_or(())?;
                    if item.source == target_world {
                        world.own_items.insert(item);
                    } else {
                        if !world.queue.iter().any(|iter_item| iter_item.source == item.source && iter_item.key == item.key) {
                            world.queue.push(item);
                        }
                    }
                    world.model.recv_mw_item(item)?;
                    world.tx.send(()).expect("failed to notify websockets about state change");
                }
            }
            AutoUpdate::Reset { world, save } => if let Some(world) = self.world_mut(world) {
                world.model.ram.save = save;
                for &item in &world.queue[world.model.ram.save.inv_amounts.num_received_mw_items.into()..] {
                    world.model.recv_mw_item(item)?;
                }
                world.tx.send(()).expect("failed to notify websockets about state change");
            } else {
                return Err(())
            },
            AutoUpdate::DungeonRewardLocation { world, reward, location } => if let Some(world) = self.world_mut(world) {
                if world.model.knowledge.dungeon_reward_locations.insert(reward, location) != Some(location) {
                    world.tx.send(()).expect("failed to notify websockets about state change");
                }
            } else {
                return Err(())
            },
            AutoUpdate::CurrentScene { world, scene } => if let Some(world) = self.world_mut(world) {
                if let Some(boss_room) = MainDungeon::from_boss_room(scene) {
                    if let Some(dungeon) = world.model.knowledge.last_dungeon.take() {
                        world.model.knowledge.boss_entrances.insert(dungeon, boss_room);
                    }
                } else {
                    world.model.knowledge.last_dungeon = MainDungeon::from_scene(scene);
                }
            } else {
                return Err(())
            },
        }
        Ok(())
    }
}
