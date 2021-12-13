#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]

use {
    std::ffi::CString,
    libc::c_char,
    semver::Version,
};

#[repr(transparent)]
pub struct HandleOwned<T: ?Sized>(*mut T);

type StringHandle = HandleOwned<c_char>;

impl StringHandle {
    fn from_string(s: impl ToString) -> StringHandle {
        HandleOwned(CString::new(s.to_string()).unwrap().into_raw())
    }
}

pub fn version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version")
}

#[no_mangle] pub extern "C" fn version_string() -> StringHandle {
    StringHandle::from_string(version())
}

/// # Safety
///
/// `s` must point at a valid string. This function takes ownership of the string.
#[no_mangle] pub unsafe extern "C" fn string_free(s: StringHandle) {
    let _ = CString::from_raw(s.0);
}
