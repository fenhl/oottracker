use {
    std::num::NonZeroU8,
    oottracker::Save,
};

pub(crate) struct MwState {
    worlds: Vec<(Save, Vec<u16>)>,
}

impl MwState {
    pub(crate) fn new(worlds: Vec<(Option<Save>, Vec<u16>)>) -> Self {
        Self {
            worlds: worlds.into_iter().map(|(save, queue)| (save.unwrap_or_default(), queue)).collect(),
        }
    }

    pub(crate) fn world_mut(&mut self, world: NonZeroU8) -> Option<&mut (Save, Vec<u16>)> {
        self.worlds.get_mut(usize::from(world.get() - 1))
    }
}
