#![deny(rust_2018_idioms, unused, unused_import_braces, unused_qualifications, warnings)]

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
    libc::c_char,
    semver::Version,
    oottracker::{
        knowledge::*,
        proto::{
            self,
            Packet,
            Protocol as _,
        },
        save::*,
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

impl<T: Default> Default for HandleOwned<T> {
    fn default() -> HandleOwned<T> {
        HandleOwned(Box::into_raw(Box::default()))
    }
}

pub fn version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version")
}

#[no_mangle] pub extern "C" fn version_string() -> HandleOwned<c_char> {
    HandleOwned(CString::new(version().to_string()).unwrap().into_raw())
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
#[no_mangle] pub unsafe extern "C" fn tcp_stream_result_debug_err(tcp_stream_res: HandleOwned<io::Result<TcpStream>>) -> HandleOwned<c_char> {
    HandleOwned(CString::new(format!("{:?}", tcp_stream_res.into_box().unwrap_err())).unwrap().into_raw())
}

/// # Safety
///
/// `s` must point at a valid string. This function takes ownership of the string.
#[no_mangle] pub unsafe extern "C" fn string_free(s: HandleOwned<c_char>) {
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
#[no_mangle] pub unsafe extern "C" fn io_result_debug_err(io_res: HandleOwned<io::Result<()>>) -> HandleOwned<c_char> {
    HandleOwned(CString::new(format!("{:?}", io_res.into_box().unwrap_err())).unwrap().into_raw())
}

/// # Safety
///
/// `start` must point at the start of a valid slice of length `0x1450` and must not be mutated for the duration of the function call.
#[no_mangle] pub unsafe extern "C" fn save_from_save_data(start: *const u8) -> HandleOwned<Result<Save, DecodeError>> {
    assert!(!start.is_null());
    let save_data = slice::from_raw_parts(start, SIZE);
    HandleOwned::new(Save::from_save_data(save_data))
}

/// # Safety
///
/// `save_res` must point at a valid `Result<Save, save::DecodeError>`. This function takes ownership of the `Result`.
#[no_mangle] pub unsafe extern "C" fn save_result_free(save_res: HandleOwned<Result<Save, DecodeError>>) {
    let _ = save_res.into_box();
}

/// # Safety
///
/// `save_res` must point at a valid `Result<Save, save::DecodeError>`.
#[no_mangle] pub unsafe extern "C" fn save_result_is_ok(save_res: *const Result<Save, DecodeError>) -> bool {
    (&*save_res).is_ok()
}

/// # Safety
///
/// `save_res` must point at a valid `Result<Save, save::DecodeError>`. This function takes ownership of the `Result`.
#[no_mangle] pub unsafe extern "C" fn save_result_unwrap(save_res: HandleOwned<Result<Save, DecodeError>>) -> HandleOwned<Save> {
    HandleOwned::new(save_res.into_box().unwrap())
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
#[no_mangle] pub unsafe extern "C" fn save_debug(save: *const Save) -> HandleOwned<c_char> {
    HandleOwned(CString::new(format!("{:?}", *save)).unwrap().into_raw())
}

/// # Safety
///
/// `save_res` must point at a valid `Result<Save, save::DecodeError>`. This function takes ownership of the `Result`.
#[no_mangle] pub unsafe extern "C" fn save_result_debug_err(save_res: HandleOwned<Result<Save, DecodeError>>) -> HandleOwned<c_char> {
    HandleOwned(CString::new(format!("{:?}", save_res.into_box().unwrap_err())).unwrap().into_raw())
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
#[no_mangle] pub unsafe extern "C" fn saves_diff(old_save: *const Save, new_save: *const Save) -> HandleOwned<Delta> {
    HandleOwned::new(&*new_save - &*old_save)
}

/// # Safety
///
/// `diff` must point at a valid `Delta`. This function takes ownership of the `Delta`.
#[no_mangle] pub unsafe extern "C" fn saves_diff_free(diff: HandleOwned<Delta>) {
    let _ = diff.into_box();
}

/// # Safety
///
/// `tcp_stream` must be a unique pointer at a valid `TcpStream`.
///
/// `diff` must point at a valid `Delta`. This function takes ownership of the `Delta`.
#[no_mangle] pub unsafe extern "C" fn saves_diff_send(tcp_stream: *mut TcpStream, diff: HandleOwned<Delta>) -> HandleOwned<io::Result<()>> {
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
