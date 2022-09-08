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
        hash::Hash,
        sync::Arc,
    },
    semver::Version,
    crate::{
        item::Item,
        region::Region,
    },
};

pub mod check;
pub mod item;
pub mod model;
pub mod region;

pub trait RandoErr: fmt::Debug + fmt::Display + Clone + Send {
    const ITEM_NOT_FOUND: Self;
}

pub trait Rando: Sized {
    type Err: RandoErr;
    type RegionName: Clone + Eq + Hash + From<&'static str> + AsRef<str> + for<'a> PartialEq<&'a str> + fmt::Debug + fmt::Display + Send;

    fn escaped_items(&self) -> Result<Arc<HashMap<String, Item>>, Self::Err>;
    fn item_table(&self) -> Result<Arc<HashMap<String, Item>>, Self::Err>;
    fn logic_tricks(&self) -> Result<Arc<HashSet<String>>, Self::Err>;
    fn regions(&self) -> Result<Arc<Vec<Arc<Region<Self>>>>, Self::Err>;
    fn root() -> Self::RegionName;
    fn setting_infos(&self) -> Result<Arc<HashSet<String>>, Self::Err>;
}

pub fn version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version")
}
