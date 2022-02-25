#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]

use {
    std::{
        env,
        ffi::CString,
        fmt,
        fs,
        net::{
            Ipv4Addr,
            Ipv6Addr,
            TcpStream,
        },
        process::{
            self,
            Command,
        },
        slice,
        time::Duration,
    },
    async_proto::Protocol as _,
    itertools::Itertools as _,
    libc::c_char,
    semver::Version,
    oottracker::{
        ModelState,
        TrackerCtx,
        github::Repo,
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
            Save,
        },
        ui::{
            CellOverlay,
            CellRender,
            CellStyle,
            Config,
            ImageDirContext,
            LocationStyle,
            TrackerCellId,
            TrackerLayout,
            dirs,
        },
    },
};

#[repr(transparent)]
pub struct FfiBool(u32);

impl From<bool> for FfiBool {
    fn from(b: bool) -> Self {
        Self(b.into())
    }
}

impl From<FfiBool> for bool {
    fn from(FfiBool(b): FfiBool) -> Self {
        b != 0
    }
}

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

pub struct DebugError(String);

impl<E: fmt::Debug> From<E> for DebugError {
    fn from(e: E) -> DebugError {
        DebugError(format!("{:?}", e))
    }
}

impl fmt::Display for DebugError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A result type where the error has been converted to its `Debug` representation.
/// Useful because it somewhat deduplicates boilerplate on the C# side.
pub type DebugResult<T> = Result<T, DebugError>;

trait DebugResultExt {
    type T;

    fn debug_unwrap(self) -> Self::T;
}

impl<T> DebugResultExt for DebugResult<T> {
    type T = T;

    fn debug_unwrap(self) -> T {
        match self {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        }
    }
}

pub fn version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version")
}

#[no_mangle] pub extern "C" fn expected_bizhawk_version_string() -> StringHandle {
    StringHandle::from_string(include_str!(concat!(env!("OUT_DIR"), "/bizhawk-version.txt")))
}

#[no_mangle] pub extern "C" fn running_bizhawk_version_string() -> StringHandle {
    StringHandle::from_string(match winver::get_file_version_info("EmuHawk.exe") {
        Ok([major, minor, patch, _]) => format!("{}.{}.{}", major, minor, patch),
        Err(e) => format!("(error: {})", e),
    })
}

#[no_mangle] pub extern "C" fn version_string() -> StringHandle {
    StringHandle::from_string(version())
}

#[no_mangle] pub extern "C" fn update_available() -> HandleOwned<DebugResult<bool>> {
    let repo = Repo::new("fenhl", "oottracker");
    HandleOwned::new(
        reqwest::blocking::Client::builder()
            .user_agent(concat!("oottracker/", env!("CARGO_PKG_VERSION")))
            .http2_prior_knowledge()
            .use_rustls_tls()
            .https_only(true)
            .build().map_err(DebugError::from)
            .and_then(|client| repo.latest_release_sync(&client).map_err(DebugError::from))
            .and_then(|release| release.ok_or_else(|| DebugError(format!("no releases"))))
            .and_then(|release| Ok(release.version()? > version()))
    )
}

/// # Safety
///
/// `bool_res` must point at a valid `DebugResult<bool>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn bool_result_free(bool_res: HandleOwned<DebugResult<bool>>) {
    let _ = bool_res.into_box();
}

/// # Safety
///
/// `bool_res` must point at a valid `DebugResult<bool>`.
#[no_mangle] pub unsafe extern "C" fn bool_result_is_ok(bool_res: *const DebugResult<bool>) -> FfiBool {
    (&*bool_res).is_ok().into()
}

/// # Safety
///
/// `bool_res` must point at a valid `DebugResult<bool>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn bool_result_unwrap(bool_res: HandleOwned<DebugResult<bool>>) -> FfiBool {
    bool_res.into_box().debug_unwrap().into()
}

/// # Safety
///
/// `bool_res` must point at a valid `DebugResult<bool>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn bool_result_debug_err(bool_res: HandleOwned<DebugResult<bool>>) -> StringHandle {
    StringHandle::from_string(bool_res.into_box().unwrap_err())
}

#[no_mangle] pub extern "C" fn run_updater() -> HandleOwned<DebugResult<()>> {
    #[cfg(target_os = "windows")] fn inner() -> DebugResult<()> {
        let [major, minor, patch, _] = winver::get_file_version_info("EmuHawk.exe")?;
        let project_dirs = dirs()?;
        let cache_dir = project_dirs.cache_dir();
        fs::create_dir_all(cache_dir)?;
        let updater_path = cache_dir.join("bizhawk-updater.exe");
        #[cfg(target_arch = "x86_64")] let updater_data = include_bytes!("../../../target/x86_64-pc-windows-msvc/release/oottracker-updater-bizhawk.exe");
        fs::write(&updater_path, updater_data)?;
        Command::new(updater_path)
            .arg(env::current_exe()?.canonicalize()?.parent().ok_or(DebugError(format!("current executable at filesystem root")))?)
            .arg(process::id().to_string())
            .arg(format!("{}.{}.{}", major, minor, patch))
            .spawn()?;
        Ok(())
    }

    HandleOwned::new(inner())
}

#[no_mangle] pub extern "C" fn config_default() -> HandleOwned<Config> {
    HandleOwned::new(Config::default())
}

#[no_mangle] pub extern "C" fn config_load() -> HandleOwned<DebugResult<Option<Config>>> {
    HandleOwned::new(Config::new_sync().map_err(DebugError::from))
}

/// # Safety
///
/// `opt_cfg_res` must point at a valid `DebugResult<Option<Config>>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn opt_config_result_free(opt_cfg_res: HandleOwned<DebugResult<Option<Config>>>) {
    let _ = opt_cfg_res.into_box();
}

/// # Safety
///
/// `opt_cfg_res` must point at a valid `DebugResult<Option<Config>>`.
#[no_mangle] pub unsafe extern "C" fn opt_config_result_is_ok(opt_cfg_res: *const DebugResult<Option<Config>>) -> FfiBool {
    (&*opt_cfg_res).is_ok().into()
}

/// # Safety
///
/// `opt_cfg_res` must point at a valid `DebugResult<Option<Config>>`.
#[no_mangle] pub unsafe extern "C" fn opt_config_result_is_ok_some(opt_cfg_res: *const DebugResult<Option<Config>>) -> FfiBool {
    (&*opt_cfg_res).as_ref().map_or(false, |opt_cfg| opt_cfg.is_some()).into()
}

/// # Safety
///
/// `opt_cfg_res` must point at a valid `DebugResult<Option<Config>>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn opt_config_result_unwrap_unwrap_or_default(opt_cfg_res: HandleOwned<DebugResult<Option<Config>>>) -> HandleOwned<Config> {
    HandleOwned::new(opt_cfg_res.into_box().debug_unwrap().unwrap_or_default())
}

/// # Safety
///
/// `opt_cfg_res` must point at a valid `DebugResult<Option<Config>>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn opt_config_result_debug_err(opt_cfg_res: HandleOwned<DebugResult<Option<Config>>>) -> StringHandle {
    StringHandle::from_string(opt_cfg_res.into_box().unwrap_err())
}

/// # Safety
///
/// `cfg` must point at a valid `Config`. This function takes ownership of the `Config`.
#[no_mangle] pub unsafe extern "C" fn config_free(cfg: HandleOwned<Config>) {
    let _ = cfg.into_box();
}

/// # Safety
///
/// `cfg` must point at a valid `Config`.
#[no_mangle] pub unsafe extern "C" fn config_update_check_is_some(cfg: *const Config) -> FfiBool {
    (&*cfg).auto_update_check.is_some().into()
}

/// # Safety
///
/// `cfg` must point at a valid `Config`.
#[no_mangle] pub unsafe extern "C" fn config_update_check(cfg: *const Config) -> FfiBool {
    (&*cfg).auto_update_check.unwrap().into()
}

/// # Safety
///
/// `cfg` must be a unique pointer at a valid `Config`.
#[no_mangle] pub unsafe extern "C" fn config_set_update_check(cfg: *mut Config, auto_update_check: FfiBool) -> HandleOwned<DebugResult<()>> {
    let cfg = &mut *cfg;
    cfg.auto_update_check = Some(auto_update_check.into());
    HandleOwned::new(cfg.save_sync().map_err(DebugError::from))
}

/// # Safety
///
/// `cfg` must point at a valid `Config`.
#[no_mangle] pub unsafe extern "C" fn config_layout(cfg: *const Config) -> HandleOwned<TrackerLayout> {
    HandleOwned::new(TrackerLayout::new_auto(&*cfg))
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
/// If `idx` is outside the range of cells for `layout`.
#[no_mangle] pub unsafe extern "C" fn layout_cell(layout: *const TrackerLayout, idx: u8) -> HandleOwned<TrackerCellId> {
    HandleOwned::new((&*layout).cells()[usize::from(idx)].id)
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
    let CellRender { img, style, overlay } = cell.kind().render(state);
    StringHandle::from_string(match (style, overlay) {
        (CellStyle::Normal, CellOverlay::None) => img.to_string('.', ImageDirContext::Normal),
        (CellStyle::Normal, CellOverlay::Count { count, count_img }) => format!("{}_{}", count_img.to_string('.', ImageDirContext::Count(count)), count),
        (CellStyle::Normal, CellOverlay::Image(overlay)) => img.with_overlay(&overlay).to_string('.', true),
        (CellStyle::Dimmed, CellOverlay::None) => img.to_string('.', ImageDirContext::Dimmed),
        (CellStyle::Dimmed, CellOverlay::Image(overlay)) => img.with_overlay(&overlay).to_string('.', false),
        (_, CellOverlay::Location { loc, style }) => loc.to_string('.', match style {
            LocationStyle::Normal => ImageDirContext::Normal,
            LocationStyle::Dimmed => ImageDirContext::Dimmed,
            LocationStyle::Mq => unimplemented!(),
        }),
        (CellStyle::Dimmed, CellOverlay::Count { .. }) | (CellStyle::LeftDimmed | CellStyle::RightDimmed, _) => unimplemented!(),
    }.replace('-', "_"))
}

/// # Safety
///
/// `addr` must point at the start of a valid slice of length 4 and must not be mutated for the duration of the function call.
#[no_mangle] pub unsafe extern "C" fn connect_ipv4(addr: *const u8) -> HandleOwned<DebugResult<TcpStream>> {
    assert!(!addr.is_null());
    let addr = slice::from_raw_parts(addr, 4);
    let tcp_stream = TcpStream::connect((Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3]), proto::TCP_PORT))
        .map_err(DebugError::from)
        .and_then(|mut tcp_stream| {
            tcp_stream.set_read_timeout(Some(Duration::from_secs(5)))?;
            tcp_stream.set_write_timeout(Some(Duration::from_secs(5)))?;
            proto::handshake_sync(&mut tcp_stream)?;
            Ok(tcp_stream)
        });
    HandleOwned::new(tcp_stream)
}

/// # Safety
///
/// `addr` must point at the start of a valid slice of length 16 and must not be mutated for the duration of the function call.
#[no_mangle] pub unsafe extern "C" fn connect_ipv6(addr: *const u8) -> HandleOwned<DebugResult<TcpStream>> {
    assert!(!addr.is_null());
    let addr = <[u8; 16]>::try_from(slice::from_raw_parts(addr, 16)).unwrap();
    let tcp_stream = TcpStream::connect((Ipv6Addr::from(addr), proto::TCP_PORT))
        .map_err(DebugError::from)
        .and_then(|mut tcp_stream| {
            tcp_stream.set_read_timeout(Some(Duration::from_secs(5)))?;
            tcp_stream.set_write_timeout(Some(Duration::from_secs(5)))?;
            proto::handshake_sync(&mut tcp_stream)?;
            Ok(tcp_stream)
        });
    HandleOwned::new(tcp_stream)
}

/// # Safety
///
/// `tcp_stream_res` must point at a valid `DebugResult<TcpStream>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn tcp_stream_result_free(tcp_stream_res: HandleOwned<DebugResult<TcpStream>>) {
    let _ = tcp_stream_res.into_box();
}

/// # Safety
///
/// `tcp_stream_res` must point at a valid `DebugResult<TcpStream>`.
#[no_mangle] pub unsafe extern "C" fn tcp_stream_result_is_ok(tcp_stream_res: *const DebugResult<TcpStream>) -> FfiBool {
    (&*tcp_stream_res).is_ok().into()
}

/// # Safety
///
/// `tcp_stream_res` must point at a valid `DebugResult<TcpStream>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn tcp_stream_result_unwrap(tcp_stream_res: HandleOwned<DebugResult<TcpStream>>) -> HandleOwned<TcpStream> {
    HandleOwned::new(tcp_stream_res.into_box().debug_unwrap())
}

/// # Safety
///
/// `tcp_stream` must point at a valid `TcpStream`. This function takes ownership of the `TcpStream`.
#[no_mangle] pub unsafe extern "C" fn tcp_stream_free(tcp_stream: HandleOwned<TcpStream>) {
    let _ = tcp_stream.into_box();
}

/// # Safety
///
/// `tcp_stream_res` must point at a valid `DebugResult<TcpStream>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn tcp_stream_result_debug_err(tcp_stream_res: HandleOwned<DebugResult<TcpStream>>) -> StringHandle {
    StringHandle::from_string(tcp_stream_res.into_box().unwrap_err())
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
#[no_mangle] pub unsafe extern "C" fn tcp_stream_disconnect(tcp_stream: HandleOwned<TcpStream>) -> HandleOwned<DebugResult<()>> {
    let mut tcp_stream = tcp_stream.into_box();
    HandleOwned::new(Packet::Goodbye.write_sync(&mut tcp_stream).map_err(DebugError::from))
}

/// # Safety
///
/// `io_res` must point at a valid `DebugResult<()>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn unit_result_free(unit_res: HandleOwned<DebugResult<()>>) {
    let _ = unit_res.into_box();
}

/// # Safety
///
/// `io_res` must point at a valid `DebugResult<()>`.
#[no_mangle] pub unsafe extern "C" fn unit_result_is_ok(unit_res: *const DebugResult<()>) -> FfiBool {
    (&*unit_res).is_ok().into()
}

/// # Safety
///
/// `io_res` must point at a valid `DebugResult<()>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn unit_result_debug_err(unit_res: HandleOwned<DebugResult<()>>) -> StringHandle {
    StringHandle::from_string(unit_res.into_box().unwrap_err())
}

/// # Safety
///
/// `start` must point at the start of a valid slice of length `0x1450` and must not be mutated for the duration of the function call.
#[no_mangle] pub unsafe extern "C" fn save_from_save_data(start: *const u8) -> HandleOwned<DebugResult<Save>> {
    assert!(!start.is_null());
    let save_data = slice::from_raw_parts(start, save::SIZE);
    HandleOwned::new(Save::from_save_data(save_data).map_err(DebugError::from))
}

/// # Safety
///
/// `save_res` must point at a valid `DebugResult<Save>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn save_result_free(save_res: HandleOwned<DebugResult<Save>>) {
    let _ = save_res.into_box();
}

/// # Safety
///
/// `save_res` must point at a valid `DebugResult<Save>`.
#[no_mangle] pub unsafe extern "C" fn save_result_is_ok(save_res: *const DebugResult<Save>) -> FfiBool {
    (&*save_res).is_ok().into()
}

/// # Safety
///
/// `save_res` must point at a valid `DebugResult<Save>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn save_result_unwrap(save_res: HandleOwned<DebugResult<Save>>) -> HandleOwned<Save> {
    HandleOwned::new(save_res.into_box().debug_unwrap())
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
/// `save_res` must point at a valid `DebugResult<Save>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn save_result_debug_err(save_res: HandleOwned<DebugResult<Save>>) -> StringHandle {
    StringHandle::from_string(save_res.into_box().unwrap_err())
}

/// # Safety
///
/// `tcp_stream` must be a unique pointer at a valid `TcpStream` and `save` must point at a valid `Save`.
#[no_mangle] pub unsafe extern "C" fn save_send(tcp_stream: *mut TcpStream, save: *const Save) -> HandleOwned<DebugResult<()>> {
    HandleOwned::new(Packet::SaveInit((&*save).clone()).write_sync(&mut *tcp_stream).map_err(DebugError::from))
}

/// # Safety
///
/// `save1` and `save2` must point at valid `Save`s.
#[no_mangle] pub unsafe extern "C" fn saves_equal(save1: *const Save, save2: *const Save) -> FfiBool {
    (&*save1 == &*save2).into()
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
#[no_mangle] pub unsafe extern "C" fn saves_diff_send(tcp_stream: *mut TcpStream, diff: HandleOwned<save::Delta>) -> HandleOwned<DebugResult<()>> {
    HandleOwned::new(Packet::SaveDelta(*diff.into_box()).write_sync(&mut *tcp_stream).map_err(DebugError::from))
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
#[no_mangle] pub unsafe extern "C" fn knowledge_send(tcp_stream: *mut TcpStream, knowledge: *const Knowledge) -> HandleOwned<DebugResult<()>> {
    HandleOwned::new(Packet::KnowledgeInit((&*knowledge).clone()).write_sync(&mut *tcp_stream).map_err(DebugError::from))
}

/// # Safety
///
/// `save` must point at a valid `Save`, and `knowledge` must point at a valid `Knowledge`. This function takes ownership of both arguments.
#[no_mangle] pub unsafe extern "C" fn model_new(save: HandleOwned<Save>, knowledge: HandleOwned<Knowledge>) -> HandleOwned<ModelState> {
    HandleOwned::new(ModelState {
        knowledge: *knowledge.into_box(),
        ram: (*save.into_box()).into(),
        tracker_ctx: TrackerCtx::default(),
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
#[no_mangle] pub unsafe extern "C" fn ram_from_ranges(ranges: *const *const u8) -> HandleOwned<DebugResult<Ram>> {
    assert!(!ranges.is_null());
    let ranges = slice::from_raw_parts(ranges, ram::NUM_RANGES);
    let ranges = ranges.iter().zip(ram::RANGES.iter().tuples()).map(|(&range, (_, &len))| slice::from_raw_parts(range, len as usize));
    HandleOwned::new(Ram::from_ranges(ranges).map_err(DebugError::from))
}

/// # Safety
///
/// `ram_res` must point at a valid `DebugResult<Ram>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn ram_result_free(ram_res: HandleOwned<DebugResult<Ram>>) {
    let _ = ram_res.into_box();
}

/// # Safety
///
/// `ram_res` must point at a valid `DebugResult<Ram>`.
#[no_mangle] pub unsafe extern "C" fn ram_result_is_ok(ram_res: *const DebugResult<Ram>) -> FfiBool {
    (&*ram_res).is_ok().into()
}

/// # Safety
///
/// `ram_res` must point at a valid `DebugResult<Ram>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn ram_result_unwrap(ram_res: HandleOwned<DebugResult<Ram>>) -> HandleOwned<Ram> {
    HandleOwned::new(ram_res.into_box().debug_unwrap())
}

/// # Safety
///
/// `ram_res` must point at a valid `DebugResult<Ram>`. This function takes ownership of the `DebugResult`.
#[no_mangle] pub unsafe extern "C" fn ram_result_debug_err(ram_res: HandleOwned<DebugResult<Ram>>) -> StringHandle {
    StringHandle::from_string(ram_res.into_box().unwrap_err())
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
#[no_mangle] pub unsafe extern "C" fn ram_equal(ram1: *const Ram, ram2: *const Ram) -> FfiBool {
    (&*ram1 == &*ram2).into()
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

/// # Safety
///
/// `model` must point at a valid `ModelState` and mut not be read or mutated during the function call.
///
/// `data` must point at the start of a valid slice with the given `length` and must not be mutated for the duration of the function call.
#[no_mangle] pub unsafe extern "C" fn model_set_tracker_ctx(model: *mut ModelState, length: i32, data: *const u8) {
    assert!(!data.is_null());
    let model = &mut *model;
    let data = slice::from_raw_parts(data, length.try_into().expect("negative or excessive length"));
    model.tracker_ctx = TrackerCtx::new(data);
    model.update_knowledge();
}
