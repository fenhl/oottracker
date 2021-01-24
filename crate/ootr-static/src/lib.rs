#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        collections::{
            BTreeMap,
            HashMap,
            HashSet,
        },
        ops::Deref,
    },
    lazy_static::lazy_static,
    ootr::{
        access::{
            Expr,
            ForAge,
        },
        check::Check,
        item::Item,
        model::{
            Dungeon,
            MainDungeon,
            Medallion,
            TimeRange,
        },
        region::{
            Mq,
            Region,
        },
    },
};

#[derive(Debug, Clone)]
pub enum RandoErr {
    ItemNotFound,
}

impl ootr::RandoErr for RandoErr {
    const ITEM_NOT_FOUND: RandoErr = RandoErr::ItemNotFound;
}

#[derive(ootr_static_derive::Rando)]
pub struct Rando;
