use {
    std::num::NonZeroU8,
    tokio::sync::watch::*,
    oottracker::{
        ModelState,
        Save,
        websocket::MwItem,
    },
};

const TRIFORCE_PIECE: u16 = 0x00ca;

pub(crate) struct MwState {
    pub(crate) worlds: Vec<(Sender<()>, Receiver<()>, ModelState, Vec<MwItem>)>,
}

impl MwState {
    pub(crate) fn new(worlds: Vec<(Option<Save>, Vec<MwItem>)>) -> Self {
        Self {
            worlds: worlds.into_iter().map(|(save, queue)| {
                let (tx, rx) = channel(());
                (tx, rx, ModelState { ram: save.unwrap_or_default().into(), knowledge: Default::default(), tracker_ctx: Default::default() }, queue)
            }).collect(),
        }
    }

    pub(crate) fn world(&self, world: NonZeroU8) -> Option<(&Sender<()>, &Receiver<()>, &ModelState, &[MwItem])> {
        self.worlds.get(usize::from(world.get() - 1)).map(|(tx, rx, model, queue)| (tx, rx, model, &**queue))
    }

    pub(crate) fn world_mut(&mut self, world: NonZeroU8) -> Option<(&Sender<()>, &Receiver<()>, &mut ModelState, &mut Vec<MwItem>)> {
        self.worlds.get_mut(usize::from(world.get() - 1)).map(|(tx, rx, model, queue)| (&*tx, &*rx, model, &mut *queue))
    }

    pub(crate) fn queue_item(&mut self, source_world: NonZeroU8, key: u32, kind: u16, target_world: NonZeroU8) -> Result<(), ()> {
        let item = MwItem { source: source_world, key, kind };
        if kind == TRIFORCE_PIECE {
            for (idx, (tx, _, model, queue)) in self.worlds.iter_mut().enumerate() {
                if idx != usize::from(source_world.get()) - 1 {
                    if !queue.iter().any(|item| item.source == source_world && item.key == key) {
                        queue.push(item);
                    }
                }
                model.ram.save.recv_mw_item(kind)?;
                tx.send(()).expect("failed to notify websockets about state change");
            }
        } else if source_world == target_world {
            let (tx, _, model, _) = self.world_mut(target_world).ok_or(())?;
            model.ram.save.recv_mw_item(kind)?;
            tx.send(()).expect("failed to notify websockets about state change");
        } else {
            let (tx, _, model, queue) = self.world_mut(target_world).ok_or(())?;
            if !queue.iter().any(|item| item.source == source_world && item.key == key) {
                queue.push(item);
            }
            model.ram.save.recv_mw_item(kind)?;
            tx.send(()).expect("failed to notify websockets about state change");
        }
        Ok(())
    }
}
