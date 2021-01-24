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
        ops::Deref,
    },
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

pub trait RandoErr: fmt::Debug + Clone {
    const ITEM_NOT_FOUND: Self;
}

pub trait Rando {
    type Err: RandoErr;

    fn escaped_items<'a>(&'a self) -> Result<Box<dyn Deref<Target = HashMap<String, Item>> + 'a>, Self::Err>;
    fn item_table<'a>(&'a self) -> Result<Box<dyn Deref<Target = HashMap<String, Item>> + 'a>, Self::Err>;
    fn logic_helpers<'a>(&'a self) -> Result<Box<dyn Deref<Target = HashMap<String, (Vec<String>, access::Expr)>> + 'a>, Self::Err>;
    fn logic_tricks<'a>(&'a self) -> Result<Box<dyn Deref<Target = HashSet<String>> + 'a>, Self::Err>;
    fn regions<'a>(&'a self) -> Result<Box<dyn Deref<Target = Vec<Region>> + 'a>, Self::Err>;
    fn setting_infos<'a>(&'a self) -> Result<Box<dyn Deref<Target = HashSet<String>> + 'a>, Self::Err>;
}
