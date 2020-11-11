#![deny(rust_2018_idioms, unused, unused_import_braces, unused_qualifications, warnings)]

use semver::Version;
pub use crate::{
    knowledge::Knowledge,
    save::Save,
};

pub mod checks;
pub mod info_tables;
mod item_ids;
pub mod knowledge;
#[cfg(not(target_arch = "wasm32"))] pub mod proto;
pub mod save;
mod scene_flags;

pub fn version() -> Version {
    let version = Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version");
    assert_eq!(version, oottracker_derive::version!());
    version
}
