use byteorder::BigEndian;

use {
    byteorder::ByteOrder as _,
    derive_more::From,
    itertools::EitherOrBoth,
    ootr::Rando,
    crate::{
        save::{
            self,
            Save,
        },
        region::{
            RegionLookup,
            RegionLookupError,
        },
        scene::{
            Scene,
            SceneFlags,
        },
    },
};

pub const SIZE: usize = 0x80_0000;

#[derive(Debug, From)]
pub enum DecodeError {
    Index(u32),
    IndexRange {
        start: u32,
        end: u32,
    },
    #[from]
    Save(save::DecodeError),
    Size(usize),
    UnexpectedValue {
        offset: u32,
        field: &'static str,
        value: u8,
    },
    UnexpectedValueRange {
        start: u32,
        end: u32,
        field: &'static str,
        value: Vec<u8>,
    },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Ram {
    pub save: Save,
    pub current_scene_id: u8,
    pub current_scene_switch_flags: u32,
    pub current_scene_chest_flags: u32,
    pub current_scene_room_clear_flags: u32,
}

impl Ram {
    /// Converts an *Ocarina of Time* RAM dump into a `Ram`.
    ///
    /// # Panics
    ///
    /// This method may panic if `ram_data`'s size is less than `0x80_0000` bytes, or if it doesn't contain a valid OoT RAM dump.
    pub fn from_bytes(ram_data: &[u8]) -> Result<Ram, DecodeError> {
        if ram_data.len() != SIZE { return Err(DecodeError::Size(ram_data.len())) }
        Ok(Ram {
            save: {
                let raw = ram_data.get(0x11a5d0..0x11a5d0 + 0x1450).ok_or(DecodeError::IndexRange { start: 0x11a5d0, end: 0x11a5d0 + 0x1450 })?;
                Save::from_save_data(raw)?
            },
            current_scene_id: *ram_data.get(0x1c8545).ok_or(DecodeError::Index(0x1c8545))?,
            current_scene_switch_flags: {
                let raw = ram_data.get(0x1ca1c8..0x1ca1cc).ok_or(DecodeError::IndexRange { start: 0x1ca1c8, end: 0x1ca1cc })?;
                BigEndian::read_u32(raw)
            },
            current_scene_chest_flags: {
                let raw = ram_data.get(0x1ca1d8..0x1ca1dc).ok_or(DecodeError::IndexRange { start: 0x1ca1d8, end: 0x1ca1dc })?;
                BigEndian::read_u32(raw)
            },
            current_scene_room_clear_flags: {
                let raw = ram_data.get(0x1ca1dc..0x1ca1e0).ok_or(DecodeError::IndexRange { start: 0x1ca1dc, end: 0x1ca1e0 })?;
                BigEndian::read_u32(raw)
            },
        })
    }

    pub(crate) fn current_region<R: Rando>(&self, rando: &R) -> Result<RegionLookup, RegionLookupError<R>> { //TODO disambiguate MQ-ness
        Ok(match Scene::current(self).region(rando, self)? {
            RegionLookup::Dungeon(EitherOrBoth::Both(vanilla, mq)) => {
                //TODO auto-disambiguate
                // visibility of MQ-ness per dungeon
                // immediately upon entering: Deku Tree (torch next to web), Jabu Jabus Belly (boulder and 2 cows), Forest Temple (extra skulltulas and no wolfos), Fire Temple (extra small torches and no hammer blocks), Ganons Castle (extra green bubbles), Spirit Temple (extra switch above and to the right of the exit)
                // not immediately but without checks: Ice Cavern (boulder takes a couple seconds to be visible), Gerudo Training Grounds (the different torches in the first room only become visible after approx. 1 roll forward), Bottom of the Well (the first skulltula being replaced with a ReDead is audible from the entrance)
                // requires checks (exits/locations): Dodongos Cavern (must blow up the first mud block to see that the lobby has an additional boulder)
                // unsure: Water Temple (not sure if the tektite on the ledge of the central pillar is still there in MQ, if not that's the first difference), Shadow Temple (the extra boxes are only visible after going through the first fake wall, not sure if that counts as a check)
                RegionLookup::Dungeon(EitherOrBoth::Both(vanilla, mq))
            }
            lookup => lookup,
        })
    }

    /// Returns the scene flags, with flags for the current scene updated properly.
    pub(crate) fn scene_flags(&self) -> SceneFlags {
        let mut flags = self.save.scene_flags;
        if let Some(flags_scene) = flags.get_mut(Scene::current(self)) {
            flags_scene.set_chests(self.current_scene_chest_flags);
            flags_scene.set_switches(self.current_scene_switch_flags);
            flags_scene.set_room_clear(self.current_scene_room_clear_flags);
            //TODO set collectible flags
            //TODO set unused field? (for triforce pieces; might not be stored separately for current scene at all)
            //TODO set visited rooms (if used)
            //TODO set visited floors (if used)
        }
        flags
    }
}
