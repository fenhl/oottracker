//! This crate includes the `Rando` trait which provides access to the OoT randomizer's code and data.
//!
//! It is implemented by the crates `ootr-static`, which accesses the randomizer at compile time, and `ootr-dynamic`, which does so at runtime.

#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        collections::{
            HashMap,
            HashSet,
        },
        fmt,
        sync::Arc,
    },
    semver::Version,
    crate::{
        item::Item,
        region::Region,
    },
};

pub mod access;
pub mod check;
pub mod item;
pub mod model;
pub mod region;

pub trait RandoErr: fmt::Debug + fmt::Display + Clone {
    const ITEM_NOT_FOUND: Self;
}

pub trait Rando {
    type Err: RandoErr;

    fn escaped_items(&self) -> Result<Arc<HashMap<String, Item>>, Self::Err>;
    fn item_table(&self) -> Result<Arc<HashMap<String, Item>>, Self::Err>;
    fn logic_helpers(&self) -> Result<Arc<HashMap<String, (Vec<String>, access::Expr)>>, Self::Err>;
    fn logic_tricks(&self) -> Result<Arc<HashSet<String>>, Self::Err>;
    fn regions(&self) -> Result<Arc<Vec<Arc<Region>>>, Self::Err>;
    fn setting_infos(&self) -> Result<Arc<HashSet<String>>, Self::Err>;
}

pub fn version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version")
}
