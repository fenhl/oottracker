#![deny(rust_2018_idioms, unused, unused_import_braces, unused_qualifications, warnings)]

use semver::Version;
pub use crate::{
    knowledge::Knowledge,
    save::Save,
};

pub mod event_chk_inf;
mod item_ids;
pub mod knowledge;
pub mod proto;
pub mod save;

pub fn version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version")
}
