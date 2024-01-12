use {
    std::{
        array::TryFromSliceError,
        borrow::Borrow,
        fmt,
        future::Future,
        io::prelude::*,
        ops::{
            AddAssign,
            Sub,
        },
        pin::Pin,
    },
    async_proto::{
        ErrorContext,
        Protocol,
        ReadError,
        ReadErrorKind,
        WriteError,
    },
    bitflags::bitflags,
    byteorder::{
        BigEndian,
        ByteOrder as _,
    },
    derive_more::From,
    itertools::Itertools as _,
    serde::{
        Deserialize,
        Serialize,
    },
    tokio::io::{
        AsyncRead,
        AsyncReadExt as _,
        AsyncWrite,
        AsyncWriteExt as _,
    },
    crate::{
        save::{
            self,
            Save,
        },
        scene::{
            Scene,
            SceneFlags,
        },
    },
};

pub const SIZE: usize = 0x80_0000;
pub const NUM_RANGES: usize = 8;
pub const TEXT_LEN: usize = 0xc0;
pub const PAUSE_CTX_LEN: usize = 0x16;
pub static RANGES: [u32; NUM_RANGES * 2] = [
    save::ADDR, save::SIZE as u32,
    0x1c84b4, 2, // buttons currently pressed on controller 1
    0x1c8545, 1, // current scene ID
    0x1ca1c8, 4, // current scene's switch flags
    0x1ca1d8, 8, // current scene's chest and room clear flags
    0x1d8870, 2, // current text box ID
    0x1d887e, TEXT_LEN as u32, // current/most recent text box contents
    0x1d8dd4, PAUSE_CTX_LEN as u32, // relevant parts of z64_game.pause_ctxt
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

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error decoding RAM: {:?}", self)
    }
}

impl std::error::Error for DecodeError {} //TODO use thiserror?

bitflags! {
    #[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct Pad: u16 {
        const A = 0x8000;
        const B = 0x4000;
        const Z = 0x2000;
        const START = 0x1000;
        const D_UP = 0x0800;
        const D_DOWN = 0x0400;
        const D_LEFT = 0x0200;
        const D_RIGHT = 0x0100;
        const L = 0x0020;
        const R = 0x0010;
        const C_UP = 0x0008;
        const C_DOWN = 0x0004;
        const C_LEFT = 0x0002;
        const C_RIGHT = 0x0001;
    }
}

async_proto::bitflags!(Pad: u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(try_from = "Vec<Vec<u8>>", into = "Vec<Vec<u8>>")]
pub struct Ram {
    pub save: Save,
    pub input_p1_raw_pad: Pad,
    pub current_scene_id: u8,
    pub current_scene_switch_flags: u32,
    pub current_scene_chest_flags: u32,
    pub current_scene_room_clear_flags: u32,
    pub current_text_box_id: u16,
    pub text_box_contents: [u8; TEXT_LEN],
    pub pause_state: u16,
    pub pause_changing: bool,
    pub pause_screen_idx: u16,
}

impl Default for Ram {
    fn default() -> Self {
        Self {
            save: Save::default(),
            input_p1_raw_pad: Pad::default(),
            current_scene_id: 0,
            current_scene_switch_flags: 0,
            current_scene_chest_flags: 0,
            current_scene_room_clear_flags: 0,
            current_text_box_id: 0,
            text_box_contents: [0; TEXT_LEN],
            pause_state: 0,
            pause_changing: false,
            pause_screen_idx: 0,
        }
    }
}

impl Ram {
    fn new(
        save: &[u8],
        input_p1_raw_pad: &[u8],
        current_scene_id: u8,
        current_scene_switch_flags: &[u8],
        current_scene_chest_flags: &[u8],
        current_scene_room_clear_flags: &[u8],
        current_text_box_id: &[u8],
        text_box_contents: &[u8],
        pause_state: &[u8],
        pause_changing: &[u8],
        pause_screen_idx: &[u8],
    ) -> Result<Self, DecodeError> {
        Ok(Self {
            save: Save::from_save_data(save)?,
            input_p1_raw_pad: Pad::from_bits_truncate(BigEndian::read_u16(input_p1_raw_pad)),
            current_scene_id,
            current_scene_switch_flags: BigEndian::read_u32(current_scene_switch_flags),
            current_scene_chest_flags: BigEndian::read_u32(current_scene_chest_flags),
            current_scene_room_clear_flags: BigEndian::read_u32(current_scene_room_clear_flags),
            current_text_box_id: BigEndian::read_u16(current_text_box_id),
            text_box_contents: text_box_contents.try_into()?,
            pause_state: BigEndian::read_u16(pause_state),
            pause_changing: BigEndian::read_u16(pause_changing) != 0,
            pause_screen_idx: BigEndian::read_u16(pause_screen_idx),
        })
    }

    pub fn from_range_bufs(ranges: impl IntoIterator<Item = Vec<u8>>) -> Result<Self, DecodeError> {
        if let Some((
            save,
            input_p1_raw_pad,
            current_scene_id,
            current_scene_switch_flags,
            chest_and_room_clear,
            current_text_box_id,
            text_box_contents,
            pause_ctx,
        )) = ranges.into_iter().collect_tuple() {
            let current_scene_id = match current_scene_id[..] {
                [current_scene_id] => current_scene_id,
                _ => return Err(DecodeError::Index(RANGES[2])),
            };
            let (chest_flags, room_clear_flags) = chest_and_room_clear.split_at(4);
            Ok(Self::new(
                &save,
                &input_p1_raw_pad,
                current_scene_id,
                &current_scene_switch_flags,
                chest_flags,
                room_clear_flags,
                &current_text_box_id,
                &text_box_contents,
                pause_ctx.get(0x00..0x02).ok_or(DecodeError::Index(RANGES[12]))?,
                pause_ctx.get(0x10..0x12).ok_or(DecodeError::Index(RANGES[12]))?,
                pause_ctx.get(0x14..0x16).ok_or(DecodeError::Index(RANGES[12]))?,
            )?)
        } else {
            Err(DecodeError::Ranges)
        }
    }

    pub fn from_ranges<'a, R: Borrow<[u8]> + ?Sized + 'a, I: IntoIterator<Item = &'a R>>(ranges: I) -> Result<Self, DecodeError> {
        if let Some((
            save,
            input_p1_raw_pad,
            &[current_scene_id],
            current_scene_switch_flags,
            chest_and_room_clear,
            current_text_box_id,
            text_box_contents,
            pause_ctx,
        )) = ranges.into_iter().map(Borrow::borrow).collect_tuple() {
            let (chest_flags, room_clear_flags) = chest_and_room_clear.split_at(4);
            Ok(Self::new(
                save,
                input_p1_raw_pad,
                current_scene_id,
                current_scene_switch_flags,
                chest_flags,
                room_clear_flags,
                current_text_box_id,
                text_box_contents,
                pause_ctx.get(0x00..0x02).ok_or(DecodeError::Index(RANGES[12]))?,
                pause_ctx.get(0x10..0x12).ok_or(DecodeError::Index(RANGES[12]))?,
                pause_ctx.get(0x14..0x16).ok_or(DecodeError::Index(RANGES[12]))?,
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

    pub fn to_ranges(&self) -> [Vec<u8>; NUM_RANGES] {
        let mut chest_and_room_clear = Vec::with_capacity(8);
        chest_and_room_clear.extend_from_slice(&self.current_scene_chest_flags.to_be_bytes());
        chest_and_room_clear.extend_from_slice(&self.current_scene_room_clear_flags.to_be_bytes());
        let mut pause_ctx = vec![0; PAUSE_CTX_LEN];
        pause_ctx.splice(0x00..0x02, self.pause_state.to_be_bytes().into_iter());
        pause_ctx.splice(0x10..0x12, if self.pause_changing { 1u16 } else { 0 }.to_be_bytes().into_iter());
        pause_ctx.splice(0x14..0x16, self.pause_screen_idx.to_be_bytes().into_iter());
        [
            self.save.to_save_data(),
            self.input_p1_raw_pad.bits().to_be_bytes().into(),
            vec![self.current_scene_id],
            self.current_scene_switch_flags.to_be_bytes().into(),
            chest_and_room_clear,
            self.current_text_box_id.to_be_bytes().into(),
            self.text_box_contents.into(),
            pause_ctx,
        ]
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
                stream.read_exact(&mut buf).await.map_err(|e| ReadError {
                    context: ErrorContext::Custom(format!("oottracker::ram::Ram::read")),
                    kind: e.into(),
                })?;
                ranges.push(buf);
            }
            Ok(Self::from_range_bufs(ranges).map_err(|e| ReadError {
                context: ErrorContext::Custom(format!("oottracker::ram::Ram::read")),
                kind: ReadErrorKind::Custom(format!("failed to decode RAM data: {e:?}")),
            })?)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            for range in self.to_ranges() {
                sink.write_all(&range).await.map_err(|e| WriteError {
                    context: ErrorContext::Custom(format!("oottracker::ram::Ram::write")),
                    kind: e.into(),
                })?;
            }
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let mut ranges = Vec::with_capacity(NUM_RANGES);
        for (_, len) in RANGES.iter().copied().tuples() {
            let mut buf = vec![0; len as usize];
            stream.read_exact(&mut buf).map_err(|e| ReadError {
                context: ErrorContext::Custom(format!("oottracker::ram::Ram::read_sync")),
                kind: e.into(),
            })?;
            ranges.push(buf);
        }
        Ok(Self::from_range_bufs(ranges).map_err(|e| ReadError {
            context: ErrorContext::Custom(format!("oottracker::ram::Ram::read_sync")),
            kind: ReadErrorKind::Custom(format!("failed to decode RAM data: {e:?}")),
        })?)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        for range in self.to_ranges() {
            sink.write_all(&range).map_err(|e| WriteError {
                context: ErrorContext::Custom(format!("oottracker::ram::Ram::write_sync")),
                kind: e.into(),
            })?;
        }
        Ok(())
    }
}

impl AddAssign<Delta> for Ram {
    fn add_assign(&mut self, rhs: Delta) {
        let Delta { save, input_p1_raw_pad, current_scene_data, text_box_data, pause_data } = rhs;
        self.save = &self.save + &save;
        self.input_p1_raw_pad = input_p1_raw_pad;
        if let Some((current_scene_id, current_scene_switch_flags, current_scene_chest_flags, current_scene_room_clear_flags)) = current_scene_data {
            self.current_scene_id = current_scene_id;
            self.current_scene_switch_flags = current_scene_switch_flags;
            self.current_scene_chest_flags = current_scene_chest_flags;
            self.current_scene_room_clear_flags = current_scene_room_clear_flags;
        }
        if let Some((current_text_box_id, text_box_contents)) = text_box_data {
            self.current_text_box_id = current_text_box_id;
            self.text_box_contents = text_box_contents;
        }
        if let Some((pause_state, pause_changing, pause_screen_idx)) = pause_data {
            self.pause_state = pause_state;
            self.pause_changing = pause_changing;
            self.pause_screen_idx = pause_screen_idx;
        }
    }
}

impl<'a, 'b> Sub<&'b Ram> for &'a Ram {
    type Output = Delta;

    fn sub(self, rhs: &Ram) -> Delta {
        let Ram { ref save, input_p1_raw_pad, current_scene_id, current_scene_switch_flags, current_scene_chest_flags, current_scene_room_clear_flags, current_text_box_id, text_box_contents, pause_state, pause_changing, pause_screen_idx } = *self;
        Delta {
            save: save - &rhs.save,
            input_p1_raw_pad,
            current_scene_data: if current_scene_id == rhs.current_scene_id
                && current_scene_switch_flags == rhs.current_scene_switch_flags
                && current_scene_chest_flags == rhs.current_scene_chest_flags
                && current_scene_room_clear_flags == rhs.current_scene_room_clear_flags
            { None } else { Some((current_scene_id, current_scene_switch_flags, current_scene_chest_flags, current_scene_room_clear_flags)) },
            text_box_data: if current_text_box_id == rhs.current_text_box_id
                && text_box_contents == rhs.text_box_contents
            { None } else { Some((current_text_box_id, text_box_contents)) },
            pause_data: if pause_state == rhs.pause_state
                && pause_changing == rhs.pause_changing
                && pause_screen_idx == rhs.pause_screen_idx
            { None } else { Some((pause_state, pause_changing, pause_screen_idx)) },
        }
    }
}

/// The difference between two RAM states.
#[derive(Debug, Clone, Protocol)]
pub struct Delta {
    save: save::Delta,
    input_p1_raw_pad: Pad,
    current_scene_data: Option<(u8, u32, u32, u32)>,
    text_box_data: Option<(u16, [u8; TEXT_LEN])>,
    pause_data: Option<(u16, bool, u16)>,
}

impl From<Ram> for Vec<Vec<u8>> {
    fn from(ram: Ram) -> Self {
        ram.to_ranges().into()
    }
}

impl TryFrom<Vec<Vec<u8>>> for Ram {
    type Error = DecodeError;

    fn try_from(ranges: Vec<Vec<u8>>) -> Result<Self, DecodeError> {
        Self::from_range_bufs(ranges)
    }
}
