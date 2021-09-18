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
        str::FromStr,
        sync::Arc,
    },
    async_proto::Protocol,
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
pub mod settings;

pub trait RandoErr: fmt::Debug + fmt::Display + Clone + Send {
    const ITEM_NOT_FOUND: Self;
}

pub trait Rando: fmt::Debug + Sized {
    type Err: RandoErr;
    type RegionName: Clone + Eq + Hash + FromStr + AsRef<str> + for<'a> PartialEq<&'a str> + fmt::Debug + fmt::Display + Protocol + Send;
    type SettingsKnowledge: settings::Knowledge<Self>;

    fn escaped_items(&self) -> Result<Arc<HashMap<String, Item>>, Self::Err>;
    fn item_table(&self) -> Result<Arc<HashMap<String, Item>>, Self::Err>;
    fn logic_helpers(&self) -> Result<Arc<HashMap<String, (Vec<String>, access::Expr<Self>)>>, Self::Err>;
    fn logic_tricks(&self) -> Result<Arc<HashSet<String>>, Self::Err>;
    fn regions(&self) -> Result<Arc<Vec<Arc<Region<Self>>>>, Self::Err>;
    fn root() -> Self::RegionName;
    fn setting_names(&self) -> Result<Arc<HashMap<String, String>>, Self::Err>;
}

pub fn version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version")
}
