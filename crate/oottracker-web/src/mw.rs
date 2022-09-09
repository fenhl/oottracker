use {
    std::num::NonZeroU8,
    tokio::sync::watch::*,
    oottracker::Save,
};

pub(crate) struct MwState {
    worlds: Vec<(Sender<()>, Receiver<()>, Save, Vec<u16>)>,
}

impl MwState {
    pub(crate) fn new(worlds: Vec<(Option<Save>, Vec<u16>)>) -> Self {
        Self {
            worlds: worlds.into_iter().map(|(save, queue)| {
                let (tx, rx) = channel(());
                (tx, rx, save.unwrap_or_default(), queue)
            }).collect(),
        }
    }

    pub(crate) fn world(&self, world: NonZeroU8) -> Option<(&Sender<()>, &Receiver<()>, &Save, &[u16])> {
        self.worlds.get(usize::from(world.get() - 1)).map(|(tx, rx, save, queue)| (tx, rx, save, &**queue))
    }

    pub(crate) fn world_mut(&mut self, world: NonZeroU8) -> Option<(&Sender<()>, &Receiver<()>, &mut Save, &mut Vec<u16>)> {
        self.worlds.get_mut(usize::from(world.get() - 1)).map(|(tx, rx, save, queue)| (&*tx, &*rx, save, &mut *queue))
    }

    pub(crate) fn push_all(&mut self, item: u16) -> Result<(), ()> {
        for (tx, _, save, queue) in &mut self.worlds {
            queue.push(item);
            save.recv_mw_item(item)?;
            tx.send(()).expect("failed to notify websockets about state change");
        }
        Ok(())
    }
}
