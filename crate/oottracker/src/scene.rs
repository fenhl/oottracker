//! This module contains types representing the “permanent scene flags” and “gold skulltulas” sections of [save data](crate::save::Save).
//!
//! The entry points are the types [`SceneFlags`] and [`GoldSkulltulas`]. All other types appear in their fields.

use {
    std::fmt,
    oottracker_derive::scene_flags,
    crate::Ram,
};

pub(crate) struct Scene(pub(crate) &'static str);

pub(crate) trait FlagsScene {
    fn set_chests(&mut self, chests: u32);
    fn set_switches(&mut self, switches: u32);
    fn set_room_clear(&mut self, room_clear: u32);
}

scene_flags! {
    pub struct SceneFlags {
        0x48: "Windmill and Dampes Grave" {
            unused: {
                TRIFORCE_PIECES = 0xffff_ffff,
            },
        },
    }
}

impl Scene {
    pub(crate) fn current(ram: &Ram) -> Result<Scene, u8> {
        Scene::from_id(ram.current_scene_id).ok_or(ram.current_scene_id)
    }
}

impl fmt::Display for Scene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
