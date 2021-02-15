#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]

use {
    std::{
        convert::TryFrom as _,
        ffi::CString,
        io,
        net::{
            Ipv4Addr,
            Ipv6Addr,
            TcpStream,
        },
        slice,
        time::Duration,
    },
    async_proto::Protocol as _,
    libc::c_char,
    semver::Version,
    ootr::{
        check::Check,
        model::{
            DungeonReward,
            DungeonRewardLocation,
            MainDungeon,
            Stone,
        },
    },
    oottracker::{
        ModelState,
        checks::CheckExt as _,
        knowledge::*,
        proto::{
            self,
            Packet,
        },
        ram::{
            self,
            Ram,
        },
        save::{
            self,
            GameMode,
            QuestItems,
            Save,
        },
        ui::{
            TrackerCellId,
            TrackerCellKind,
            TrackerLayout,
        },
    },
};

#[repr(transparent)]
pub struct HandleOwned<T: ?Sized>(*mut T); //TODO *mut Fragile<T>

impl<T: ?Sized> HandleOwned<T> {
    fn new(value: T) -> HandleOwned<T>
    where T: Sized {
        HandleOwned(Box::into_raw(Box::new(value)))
    }

    /// # Safety
    ///
    /// `self` must point at a valid `T`. This function takes ownership of the `T`.
    unsafe fn into_box(self) -> Box<T> {
        assert!(!self.0.is_null());
        Box::from_raw(self.0)
    }
}

type StringHandle = HandleOwned<c_char>;

impl StringHandle {
    fn from_string(s: impl ToString) -> StringHandle {
        HandleOwned(CString::new(s.to_string()).unwrap().into_raw())
    }
}

impl<T: Default> Default for HandleOwned<T> {
    fn default() -> HandleOwned<T> {
        HandleOwned(Box::into_raw(Box::default()))
    }
}

pub fn version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version")
}

#[no_mangle] pub extern "C" fn version_string() -> StringHandle {
    StringHandle::from_string(version())
}

#[no_mangle] pub extern "C" fn layout_default() -> HandleOwned<TrackerLayout> {
    HandleOwned::new(TrackerLayout::default())
}

/// # Safety
///
/// `layout` must point at a valid `TrackerLayout`. This function takes ownership of the `TrackerLayout`.
#[no_mangle] pub unsafe extern "C" fn layout_free(layout: HandleOwned<TrackerLayout>) {
    let _ = layout.into_box();
}

/// # Safety
///
/// `layout` must point at a valid `TrackerLayout` and must not be mutated for the duration of the function call.
///
/// # Panics
///
/// If `idx >= 52`.
#[no_mangle] pub unsafe extern "C" fn layout_cell(layout: *const TrackerLayout, idx: u8) -> HandleOwned<TrackerCellId> {
    let layout = &*layout;
    HandleOwned::new(match idx {
        0..=5 => TrackerCellId::med_location(layout.meds.into_iter().nth(usize::from(idx)).expect("ElementOrder has 6 elements")),
        6..=11 => TrackerCellId::from(layout.meds.into_iter().nth(usize::from(idx) - 6).expect("ElementOrder has 6 elements")),
        12 => layout.row2[0],
        13 => layout.row2[1],
        14 => TrackerCellId::KokiriEmeraldLocation,
        15 => TrackerCellId::KokiriEmerald,
        16 => TrackerCellId::GoronRubyLocation,
        17 => TrackerCellId::GoronRuby,
        18 => TrackerCellId::ZoraSapphireLocation,
        19 => TrackerCellId::ZoraSapphire,
        20 => layout.row2[2],
        21 => layout.row2[3],
        22..=45 => layout.rest[(usize::from(idx) - 22) / 6][(usize::from(idx) - 22) % 6],
        46..=51 => TrackerCellId::warp_song(layout.warp_songs.into_iter().nth(usize::from(idx) - 46).expect("ElementOrder has 6 elements")),
        _ => panic!("invalid tracker cell index"),
    })
}

/// # Safety
///
/// `cell` must point at a valid `TrackerCellId`. This function takes ownership of the `TrackerCellId`.
#[no_mangle] pub unsafe extern "C" fn cell_free(cell: HandleOwned<TrackerCellId>) {
    let _ = cell.into_box();
}

/// # Safety
///
/// `state` must point at a valid `ModelState`, and `cell` must point at a valid `TrackerCellId`.
#[no_mangle] pub unsafe extern "C" fn cell_image(model: *const ModelState, cell: *const TrackerCellId) -> StringHandle {
    let state = &*model;
    let cell = &*cell;
    StringHandle::from_string(match cell.kind() {
        TrackerCellKind::Composite { left_img, right_img, both_img, active, .. } => match active(state) {
            (false, false) => format!("xopar_images_dimmed.{}", both_img),
            (false, true) => format!("xopar_images.{}", right_img),
            (true, false) => format!("xopar_images.{}", left_img),
            (true, true) => format!("xopar_images.{}", both_img),
        },
        TrackerCellKind::Count { dimmed_img, img, get, .. } => match get(state) {
            0 => format!("xopar_images_dimmed.{}", dimmed_img),
            n => format!("xopar_images_count.{}_{}", img, n),
        },
        TrackerCellKind::Medallion(med) => format!(
            "xopar_images{}.{}_medallion",
            if state.ram.save.quest_items.has(med) { "" } else { "_dimmed" },
            med.element().to_ascii_lowercase(),
        ),
        TrackerCellKind::MedallionLocation(med) => match state.knowledge.dungeon_reward_locations.get(&DungeonReward::Medallion(med)) {
            None => format!("xopar_images_dimmed.unknown_text"),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)) => format!("xopar_images.deku_text"),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)) => format!("xopar_images.dc_text"),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)) => format!("xopar_images.jabu_text"),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)) => format!("xopar_images.forest_text"),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)) => format!("xopar_images.fire_text"),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)) => format!("xopar_images.water_text"),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)) => format!("xopar_images.shadow_text"),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)) => format!("xopar_images.spirit_text"),
            Some(DungeonRewardLocation::LinksPocket) => format!("xopar_images.free_text"),
        },
        TrackerCellKind::OptionalOverlay { main_img, overlay_img, active, .. } | TrackerCellKind::Overlay { main_img, overlay_img, active, .. } => match active(state) {
            (false, false) => format!("xopar_images_dimmed.{}", main_img),
            (false, true) => format!("xopar_images_overlay_dimmed.{}_{}", main_img, overlay_img),
            (true, false) => format!("xopar_images.{}", main_img),
            (true, true) => format!("xopar_images_overlay.{}_{}", main_img, overlay_img),
        },
        TrackerCellKind::Sequence { img, .. } => {
            let (active, img) = img(state);
            format!("xopar_images{}.{}", if active { "" } else { "_dimmed" }, img)
        }
        TrackerCellKind::Simple { img, active, .. } => format!("xopar_images{}.{}", if active(state) { "" } else { "_dimmed" }, img),
        TrackerCellKind::Song { song, check, .. } => {
            let song_filename = match song {
                QuestItems::ZELDAS_LULLABY => "lullaby",
                QuestItems::EPONAS_SONG => "epona",
                QuestItems::SARIAS_SONG => "saria",
                QuestItems::SUNS_SONG => "sun",
                QuestItems::SONG_OF_TIME => "time",
                QuestItems::SONG_OF_STORMS => "storms",
                QuestItems::MINUET_OF_FOREST => "minuet",
                QuestItems::BOLERO_OF_FIRE => "bolero",
                QuestItems::SERENADE_OF_WATER => "serenade",
                QuestItems::NOCTURNE_OF_SHADOW => "nocturne",
                QuestItems::REQUIEM_OF_SPIRIT => "requiem",
                QuestItems::PRELUDE_OF_LIGHT => "prelude",
                _ => unreachable!(),
            };
            match (state.ram.save.quest_items.contains(song), Check::Location(check.to_string()).checked(state).unwrap_or(false)) {
                (false, false) => format!("xopar_images_dimmed.{}", song_filename),
                (false, true) => format!("xopar_images_overlay_dimmed.{}_check", song_filename),
                (true, false) => format!("xopar_images.{}", song_filename),
                (true, true) => format!("xopar_images_overlay.{}_check", song_filename),
            }
        }
        TrackerCellKind::Stone(stone) => format!(
            "xopar_images{}.{}",
            if state.ram.save.quest_items.has(stone) { "" } else { "_dimmed" },
            match stone {
                Stone::KokiriEmerald => "kokiri_emerald",
                Stone::GoronRuby => "goron_ruby",
                Stone::ZoraSapphire => "zora_sapphire",
            },
        ),
        TrackerCellKind::StoneLocation(stone) => match state.knowledge.dungeon_reward_locations.get(&DungeonReward::Stone(stone)) {
            None => format!("xopar_images_dimmed.unknown_text"),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)) => format!("xopar_images.deku_text"),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)) => format!("xopar_images.dc_text"),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)) => format!("xopar_images.jabu_text"),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)) => format!("xopar_images.forest_text"),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)) => format!("xopar_images.fire_text"),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)) => format!("xopar_images.water_text"),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)) => format!("xopar_images.shadow_text"),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)) => format!("xopar_images.spirit_text"),
            Some(DungeonRewardLocation::LinksPocket) => format!("xopar_images.free_text"),
        },
        TrackerCellKind::BossKey { .. } | TrackerCellKind::FortressMq | TrackerCellKind::Mq(_) | TrackerCellKind::SmallKeys { .. } | TrackerCellKind::SongCheck { .. } => unimplemented!(),
    })
}

/// # Safety
///
/// `addr` must point at the start of a valid slice of length 4 and must not be mutated for the duration of the function call.
#[no_mangle] pub unsafe extern "C" fn connect_ipv4(addr: *const u8) -> HandleOwned<io::Result<TcpStream>> {
    assert!(!addr.is_null());
    let addr = slice::from_raw_parts(addr, 4);
    let tcp_stream = TcpStream::connect((Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3]), proto::TCP_PORT))
        .and_then(|mut tcp_stream| {
            proto::VERSION.write_sync(&mut tcp_stream)?;
            Ok(tcp_stream)
        });
    HandleOwned::new(tcp_stream)
}

/// # Safety
///
/// `addr` must point at the start of a valid slice of length 16 and must not be mutated for the duration of the function call.
#[no_mangle] pub unsafe extern "C" fn connect_ipv6(addr: *const u8) -> HandleOwned<io::Result<TcpStream>> {
    assert!(!addr.is_null());
    let addr = <[u8; 16]>::try_from(slice::from_raw_parts(addr, 16)).unwrap();
    let tcp_stream = TcpStream::connect((Ipv6Addr::from(addr), proto::TCP_PORT))
        .and_then(|mut tcp_stream| {
            tcp_stream.set_read_timeout(Some(Duration::from_secs(5)))?;
            tcp_stream.set_write_timeout(Some(Duration::from_secs(5)))?;
            proto::VERSION.write_sync(&mut tcp_stream)?;
            Ok(tcp_stream)
        });
    HandleOwned::new(tcp_stream)
}

/// # Safety
///
/// `tcp_stream_res` must point at a valid `io::Result<TcpStream>`. This function takes ownership of the `Result`.
#[no_mangle] pub unsafe extern "C" fn tcp_stream_result_free(tcp_stream_res: HandleOwned<io::Result<TcpStream>>) {
    let _ = tcp_stream_res.into_box();
}

/// # Safety
///
/// `tcp_stream_res` must point at a valid `io::Result<TcpStream>`.
#[no_mangle] pub unsafe extern "C" fn tcp_stream_result_is_ok(tcp_stream_res: *const io::Result<TcpStream>) -> bool {
    (&*tcp_stream_res).is_ok()
}

/// # Safety
///
/// `tcp_stream_res` must point at a valid `io::Result<TcpStream>`. This function takes ownership of the `Result`.
#[no_mangle] pub unsafe extern "C" fn tcp_stream_result_unwrap(tcp_stream_res: HandleOwned<io::Result<TcpStream>>) -> HandleOwned<TcpStream> {
    HandleOwned::new(tcp_stream_res.into_box().unwrap())
}

/// # Safety
///
/// `tcp_stream` must point at a valid `TcpStream`. This function takes ownership of the `TcpStream`.
#[no_mangle] pub unsafe extern "C" fn tcp_stream_free(tcp_stream: HandleOwned<TcpStream>) {
    let _ = tcp_stream.into_box();
}

/// # Safety
///
/// `tcp_stream_res` must point at a valid `io::Result<TcpStream>`. This function takes ownership of the `Result`.
#[no_mangle] pub unsafe extern "C" fn tcp_stream_result_debug_err(tcp_stream_res: HandleOwned<io::Result<TcpStream>>) -> StringHandle {
    StringHandle::from_string(format!("{:?}", tcp_stream_res.into_box().unwrap_err()))
}

/// # Safety
///
/// `s` must point at a valid string. This function takes ownership of the string.
#[no_mangle] pub unsafe extern "C" fn string_free(s: StringHandle) {
    let _ = CString::from_raw(s.0);
}

/// # Safety
///
/// `tcp_stream` must point at a valid `TcpStream`. This function takes ownership of the `TcpStream`.
#[no_mangle] pub unsafe extern "C" fn tcp_stream_disconnect(tcp_stream: HandleOwned<TcpStream>) -> HandleOwned<io::Result<()>> {
    let mut tcp_stream = tcp_stream.into_box();
    HandleOwned::new(Packet::Goodbye.write_sync(&mut tcp_stream))
}

/// # Safety
///
/// `io_res` must point at a valid `io::Result<()>`. This function takes ownership of the `Result`.
#[no_mangle] pub unsafe extern "C" fn io_result_free(io_res: HandleOwned<io::Result<()>>) {
    let _ = io_res.into_box();
}

/// # Safety
///
/// `io_res` must point at a valid `io::Result<()>`.
#[no_mangle] pub unsafe extern "C" fn io_result_is_ok(io_res: *const io::Result<()>) -> bool {
    (&*io_res).is_ok()
}

/// # Safety
///
/// `io_res` must point at a valid `io::Result<()>`. This function takes ownership of the `Result`.
#[no_mangle] pub unsafe extern "C" fn io_result_debug_err(io_res: HandleOwned<io::Result<()>>) -> StringHandle {
    StringHandle::from_string(format!("{:?}", io_res.into_box().unwrap_err()))
}

/// # Safety
///
/// `start` must point at the start of a valid slice of length `0x1450` and must not be mutated for the duration of the function call.
#[no_mangle] pub unsafe extern "C" fn save_from_save_data(start: *const u8) -> HandleOwned<Result<Save, save::DecodeError>> {
    assert!(!start.is_null());
    let save_data = slice::from_raw_parts(start, save::SIZE);
    HandleOwned::new(Save::from_save_data(save_data))
}

/// # Safety
///
/// `save_res` must point at a valid `Result<Save, save::DecodeError>`. This function takes ownership of the `Result`.
#[no_mangle] pub unsafe extern "C" fn save_result_free(save_res: HandleOwned<Result<Save, save::DecodeError>>) {
    let _ = save_res.into_box();
}

/// # Safety
///
/// `save_res` must point at a valid `Result<Save, save::DecodeError>`.
#[no_mangle] pub unsafe extern "C" fn save_result_is_ok(save_res: *const Result<Save, save::DecodeError>) -> bool {
    (&*save_res).is_ok()
}

/// # Safety
///
/// `save_res` must point at a valid `Result<Save, save::DecodeError>`. This function takes ownership of the `Result`.
#[no_mangle] pub unsafe extern "C" fn save_result_unwrap(save_res: HandleOwned<Result<Save, save::DecodeError>>) -> HandleOwned<Save> {
    HandleOwned::new(save_res.into_box().unwrap())
}

#[no_mangle] pub extern "C" fn save_default() -> HandleOwned<Save> {
    HandleOwned::default()
}

/// # Safety
///
/// `save` must point at a valid `Save`. This function takes ownership of the `Save`.
#[no_mangle] pub unsafe extern "C" fn save_free(save: HandleOwned<Save>) {
    let _ = save.into_box();
}

/// # Safety
///
/// `save` must point at a valid `Save`.
#[no_mangle] pub unsafe extern "C" fn save_debug(save: *const Save) -> StringHandle {
    StringHandle::from_string(format!("{:?}", *save))
}

/// # Safety
///
/// `save_res` must point at a valid `Result<Save, save::DecodeError>`. This function takes ownership of the `Result`.
#[no_mangle] pub unsafe extern "C" fn save_result_debug_err(save_res: HandleOwned<Result<Save, save::DecodeError>>) -> StringHandle {
    StringHandle::from_string(format!("{:?}", save_res.into_box().unwrap_err()))
}

/// # Safety
///
/// `tcp_stream` must be a unique pointer at a valid `TcpStream` and `save` must point at a valid `Save`.
#[no_mangle] pub unsafe extern "C" fn save_send(tcp_stream: *mut TcpStream, save: *const Save) -> HandleOwned<io::Result<()>> {
    HandleOwned::new(Packet::SaveInit((&*save).clone()).write_sync(&mut *tcp_stream))
}

/// # Safety
///
/// `save1` and `save2` must point at valid `Save`s.
#[no_mangle] pub unsafe extern "C" fn saves_equal(save1: *const Save, save2: *const Save) -> bool {
    &*save1 == &*save2
}

/// # Safety
///
/// `old_save` and `new_save` must point at valid `Save`s.
#[no_mangle] pub unsafe extern "C" fn saves_diff(old_save: *const Save, new_save: *const Save) -> HandleOwned<save::Delta> {
    HandleOwned::new(&*new_save - &*old_save)
}

/// # Safety
///
/// `diff` must point at a valid `Delta`. This function takes ownership of the `Delta`.
#[no_mangle] pub unsafe extern "C" fn saves_diff_free(diff: HandleOwned<save::Delta>) {
    let _ = diff.into_box();
}

/// # Safety
///
/// `tcp_stream` must be a unique pointer at a valid `TcpStream`.
///
/// `diff` must point at a valid `Delta`. This function takes ownership of the `Delta`.
#[no_mangle] pub unsafe extern "C" fn saves_diff_send(tcp_stream: *mut TcpStream, diff: HandleOwned<save::Delta>) -> HandleOwned<io::Result<()>> {
    HandleOwned::new(Packet::SaveDelta(*diff.into_box()).write_sync(&mut *tcp_stream))
}

#[no_mangle] pub extern "C" fn knowledge_none() -> HandleOwned<Knowledge> {
    HandleOwned::default()
}

#[no_mangle] pub extern "C" fn knowledge_vanilla() -> HandleOwned<Knowledge> {
    HandleOwned::new(Knowledge::vanilla())
}

/// # Safety
///
/// `knowledge` must point at a valid `Knowledge`. This function takes ownership of the `Knowledge`.
#[no_mangle] pub unsafe extern "C" fn knowledge_free(knowledge: HandleOwned<Knowledge>) {
    let _ = knowledge.into_box();
}

/// # Safety
///
/// `tcp_stream` must be a unique pointer at a valid `TcpStream`.
///
/// `knowledge` must point at a valid `Knowledge`.
#[no_mangle] pub unsafe extern "C" fn knowledge_send(tcp_stream: *mut TcpStream, knowledge: *const Knowledge) -> HandleOwned<io::Result<()>> {
    HandleOwned::new(Packet::KnowledgeInit((&*knowledge).clone()).write_sync(&mut *tcp_stream))
}

/// # Safety
///
/// `save` must point at a valid `Save`, and `knowledge` must point at a valid `Knowledge`. This function takes ownership of both arguments.
#[no_mangle] pub unsafe extern "C" fn model_new(save: HandleOwned<Save>, knowledge: HandleOwned<Knowledge>) -> HandleOwned<ModelState> {
    HandleOwned::new(ModelState {
        knowledge: *knowledge.into_box(),
        ram: (*save.into_box()).into(),
    })
}

/// # Safety
///
/// `model` must point at a valid `ModelState`. This function takes ownership of the `ModelState`.
#[no_mangle] pub unsafe extern "C" fn model_free(model: HandleOwned<ModelState>) {
    let _ = model.into_box();
}

#[no_mangle] pub extern "C" fn ram_num_ranges() -> u8 { ram::NUM_RANGES as u8 }
#[no_mangle] pub extern "C" fn ram_ranges() -> *const u32 { &ram::RANGES[0] }

/// # Safety
///
/// `ranges` must point at the start of a valid slice of `ram::NUM_RANGES` slices with the lengths specified in `ram::RANGES` and must not be mutated for the duration of the function call.
#[no_mangle] pub unsafe extern "C" fn ram_from_ranges(ranges: *const *const u8) -> HandleOwned<Result<Ram, ram::DecodeError>> {
    assert!(!ranges.is_null());
    let ranges = slice::from_raw_parts(ranges, ram::NUM_RANGES);
    let chest_and_room_clear = slice::from_raw_parts(ranges[3], 8);
    let (chest_flags, room_clear_flags) = chest_and_room_clear.split_at(4);
    HandleOwned::new(Ram::from_ranges(
        slice::from_raw_parts(ranges[0], save::SIZE),
        *ranges[1],
        slice::from_raw_parts(ranges[2], 4),
        chest_flags,
        room_clear_flags,
    ))
}

/// # Safety
///
/// `ram_res` must point at a valid `Result<Ram, ram::DecodeError>`. This function takes ownership of the `Result`.
#[no_mangle] pub unsafe extern "C" fn ram_result_free(ram_res: HandleOwned<Result<Ram, ram::DecodeError>>) {
    let _ = ram_res.into_box();
}

/// # Safety
///
/// `ram_res` must point at a valid `Result<Ram, ram::DecodeError>`.
#[no_mangle] pub unsafe extern "C" fn ram_result_is_ok(ram_res: *const Result<Ram, ram::DecodeError>) -> bool {
    (&*ram_res).is_ok()
}

/// # Safety
///
/// `ram_res` must point at a valid `Result<Ram, ram::DecodeError>`. This function takes ownership of the `Result`.
#[no_mangle] pub unsafe extern "C" fn ram_result_unwrap(ram_res: HandleOwned<Result<Ram, ram::DecodeError>>) -> HandleOwned<Ram> {
    HandleOwned::new(ram_res.into_box().unwrap())
}

/// # Safety
///
/// `ram_res` must point at a valid `Result<Ram, ram::DecodeError>`. This function takes ownership of the `Result`.
#[no_mangle] pub unsafe extern "C" fn ram_result_debug_err(ram_res: HandleOwned<Result<Ram, ram::DecodeError>>) -> StringHandle {
    StringHandle::from_string(format!("{:?}", ram_res.into_box().unwrap_err()))
}

/// # Safety
///
/// `ram` must point at a valid `Ram`. This function takes ownership of the `Ram`.
#[no_mangle] pub unsafe extern "C" fn ram_free(ram: HandleOwned<Ram>) {
    let _ = ram.into_box();
}

/// # Safety
///
/// `ram1` and `ram2` must point at valid `Ram` values.
#[no_mangle] pub unsafe extern "C" fn ram_equal(ram1: *const Ram, ram2: *const Ram) -> bool {
    &*ram1 == &*ram2
}

/// # Safety
///
/// `model` must point at a valid `ModelState` and must not be read or mutated during the function call.
///
/// `ram` must point at a valid `Ram` and must not be mutated during the function call.
#[no_mangle] pub unsafe extern "C" fn model_set_ram(model: *mut ModelState, ram: *const Ram) {
    let model = &mut *model;
    let ram = &*ram;
    if ram.save.game_mode == GameMode::Gameplay { model.ram = *ram }
    model.update_knowledge();
}

/// # Safety
///
/// `ram` must point at a valid `Ram` and must not be mutated during the function call.
#[no_mangle] pub unsafe extern "C" fn ram_clone_save(ram: *const Ram) -> HandleOwned<Save> {
    HandleOwned::new((&*ram).save.clone())
}
