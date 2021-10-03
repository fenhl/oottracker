use {
    std::{
        array::TryFromSliceError,
        borrow::Borrow,
        convert::TryInto as _,
        future::Future,
        io::prelude::*,
        ops::{
            AddAssign,
            Sub,
        },
        pin::Pin,
    },
    async_proto::{
        Protocol,
        ReadError,
        WriteError,
    },
    byteorder::{
        BigEndian,
        ByteOrder as _,
    },
    derive_more::From,
    itertools::{
        EitherOrBoth,
        Itertools as _,
    },
    tokio::io::{
        AsyncRead,
        AsyncReadExt as _,
        AsyncWrite,
        AsyncWriteExt as _,
    },
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
pub const NUM_RANGES: usize = 6;
pub const TEXT_LEN: usize = 0xc0;
pub static RANGES: [u32; NUM_RANGES * 2] = [
    save::ADDR, save::SIZE as u32,
    0x1c8545, 1, // current scene ID
    0x1ca1c8, 4, // current scene's switch flags
    0x1ca1d8, 8, // current scene's chest and room clear flags
    0x1d8870, 2, // current text box ID
    0x1d8328, TEXT_LEN as u32, // current/most recent text box contents
];

#[derive(Debug, From, Clone)]
pub enum DecodeError {
    Index(u32),
    IndexRange {
        start: u32,
        end: u32,
    },
    Ranges,
    #[from]
    Save(save::DecodeError),
    Size(usize),
    #[from]
    TextSize(TryFromSliceError),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ram {
    pub save: Save,
    pub current_scene_id: u8,
    pub current_scene_switch_flags: u32,
    pub current_scene_chest_flags: u32,
    pub current_scene_room_clear_flags: u32,
    pub current_text_box_id: u16,
    pub text_box_contents: [u8; TEXT_LEN],
}

impl Default for Ram {
    fn default() -> Self {
        Self {
            save: Save::default(),
            current_scene_id: 0,
            current_scene_switch_flags: 0,
            current_scene_chest_flags: 0,
            current_scene_room_clear_flags: 0,
            current_text_box_id: 0,
            text_box_contents: [0; TEXT_LEN],
        }
    }
}

impl Ram {
    fn new(
        save: &[u8],
        current_scene_id: u8,
        current_scene_switch_flags: &[u8],
        current_scene_chest_flags: &[u8],
        current_scene_room_clear_flags: &[u8],
        current_text_box_id: &[u8],
        text_box_contents: &[u8],
    ) -> Result<Self, DecodeError> {
        Ok(Self {
            save: Save::from_save_data(save)?,
            current_scene_id,
            current_scene_switch_flags: BigEndian::read_u32(current_scene_switch_flags),
            current_scene_chest_flags: BigEndian::read_u32(current_scene_chest_flags),
            current_scene_room_clear_flags: BigEndian::read_u32(current_scene_room_clear_flags),
            current_text_box_id: BigEndian::read_u16(current_text_box_id),
            text_box_contents: text_box_contents.try_into()?,
        })
    }

    pub fn from_range_bufs(ranges: impl IntoIterator<Item = Vec<u8>>) -> Result<Self, DecodeError> {
        if let Some((
            save,
            current_scene_id,
            current_scene_switch_flags,
            chest_and_room_clear,
            current_text_box_id,
            text_box_contents,
        )) = ranges.into_iter().collect_tuple() {
            let current_scene_id = match current_scene_id[..] {
                [current_scene_id] => current_scene_id,
                _ => return Err(DecodeError::Index(RANGES[2])),
            };
            let (chest_flags, room_clear_flags) = chest_and_room_clear.split_at(4);
            Ok(Self::new(
                &save,
                current_scene_id,
                &current_scene_switch_flags,
                chest_flags,
                room_clear_flags,
                &current_text_box_id,
                &text_box_contents,
            )?)
        } else {
            Err(DecodeError::Ranges)
        }
    }

    pub fn from_ranges<'a, R: Borrow<[u8]> + ?Sized + 'a, I: IntoIterator<Item = &'a R>>(ranges: I) -> Result<Self, DecodeError> {
        if let Some((
            save,
            &[current_scene_id],
            current_scene_switch_flags,
            chest_and_room_clear,
            current_text_box_id,
            text_box_contents,
        )) = ranges.into_iter().map(Borrow::borrow).collect_tuple() {
            let (chest_flags, room_clear_flags) = chest_and_room_clear.split_at(4);
            Ok(Self::new(
                save,
                current_scene_id,
                current_scene_switch_flags,
                chest_flags,
                room_clear_flags,
                current_text_box_id,
                text_box_contents,
            )?)
        } else {
            Err(DecodeError::Ranges)
        }
    }

    /// Converts an *Ocarina of Time* RAM dump into a `Ram`.
    ///
    /// # Panics
    ///
    /// This method may panic if `ram_data` doesn't contain a valid OoT RAM dump.
    pub fn from_bytes(ram_data: &[u8]) -> Result<Self, DecodeError> {
        if ram_data.len() != SIZE { return Err(DecodeError::Size(ram_data.len())) }
        Self::from_ranges(RANGES.iter().tuples().map(|(&start, &len)|
            ram_data.get(start as usize..(start + len) as usize).ok_or(DecodeError::IndexRange { start, end: start + len })
        ).try_collect::<_, Vec<_>, _>()?)
    }

    fn to_ranges(&self) -> Vec<Vec<u8>> {
        let mut chest_and_room_clear = Vec::with_capacity(8);
        chest_and_room_clear.extend_from_slice(&self.current_scene_chest_flags.to_be_bytes());
        chest_and_room_clear.extend_from_slice(&self.current_scene_room_clear_flags.to_be_bytes());
        vec![
            self.save.to_save_data(),
            vec![self.current_scene_id],
            self.current_scene_switch_flags.to_be_bytes().into(),
            chest_and_room_clear,
        ]
    }

    pub(crate) fn current_region<R: Rando>(&self, rando: &R) -> Result<RegionLookup<R>, RegionLookupError<R>> { //TODO disambiguate MQ-ness
        Ok(match Scene::current(self).map_err(RegionLookupError::UnknownScene)?.region(rando, self)? {
            RegionLookup::Dungeon(EitherOrBoth::Both(vanilla, mq)) => {
                //TODO auto-disambiguate
                // visibility of MQ-ness per dungeon
                // immediately upon entering: Deku Tree (torch next to web), Jabu Jabus Belly (boulder and 2 cows), Forest Temple (extra skulltulas and no wolfos), Fire Temple (extra small torches and no hammer blocks), Ganons Castle (extra green bubbles), Spirit Temple (extra boulders)
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
        if let Some(flags_scene) = Scene::current(self).ok().and_then(|current_scene| flags.get_mut(current_scene)) {
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

impl From<Save> for Ram {
    fn from(save: Save) -> Self {
        Self { save, ..Self::default() }
    }
}

impl Protocol for Ram {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let mut ranges = Vec::with_capacity(NUM_RANGES);
            for (_, len) in RANGES.iter().copied().tuples() {
                let mut buf = vec![0; len as usize];
                stream.read_exact(&mut buf).await?;
                ranges.push(buf);
            }
            Ok(Self::from_range_bufs(ranges).map_err(|e| ReadError::Custom(format!("failed to decode RAM data: {:?}", e)))?)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            let ranges = self.to_ranges();
            for range in ranges {
                sink.write_all(&range).await?;
            }
            Ok(())
        })
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        let ranges = self.to_ranges();
        for range in ranges {
            sink.write_all(&range)?;
        }
        Ok(())
    }
}

impl AddAssign<Delta> for Ram {
    fn add_assign(&mut self, rhs: Delta) {
        self.save = &self.save + &rhs.save;
        if let Some((current_scene_id, current_scene_switch_flags, current_scene_chest_flags, current_scene_room_clear_flags)) = rhs.current_scene_data {
            self.current_scene_id = current_scene_id;
            self.current_scene_switch_flags = current_scene_switch_flags;
            self.current_scene_chest_flags = current_scene_chest_flags;
            self.current_scene_room_clear_flags = current_scene_room_clear_flags;
        }
    }
}

impl<'a, 'b> Sub<&'b Ram> for &'a Ram {
    type Output = Delta;

    fn sub(self, rhs: &Ram) -> Delta {
        Delta {
            save: &self.save - &rhs.save,
            current_scene_data: if self.current_scene_id == rhs.current_scene_id
                && self.current_scene_switch_flags == rhs.current_scene_switch_flags
                && self.current_scene_chest_flags == rhs.current_scene_chest_flags
                && self.current_scene_room_clear_flags == rhs.current_scene_room_clear_flags
            { None } else { Some((self.current_scene_id, self.current_scene_switch_flags, self.current_scene_chest_flags, self.current_scene_room_clear_flags)) },
        }
    }
}

/// The difference between two RAM states.
#[derive(Debug, Clone, Protocol)]
pub struct Delta {
    save: save::Delta,
    current_scene_data: Option<(u8, u32, u32, u32)>,
}
