#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        borrow::Cow,
        collections::{
            HashMap,
            HashSet,
        },
        convert::TryFrom,
        fmt,
        future::Future,
        io::prelude::*,
        ops::RangeInclusive,
        pin::Pin,
        str::FromStr,
        sync::Arc,
    },
    async_proto::{
        Protocol,
        ReadError,
        WriteError,
    },
    once_cell::sync::Lazy,
    semver::Version,
    tokio::io::{
        AsyncRead,
        AsyncWrite,
    },
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

impl fmt::Display for RandoErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RandoErr::ItemNotFound => write!(f, "no such item"),
        }
    }
}

impl ootr::RandoErr for RandoErr {
    const ITEM_NOT_FOUND: RandoErr = RandoErr::ItemNotFound;
}

#[derive(Debug, Clone, Copy, ootr_static_derive::Rando)]
pub struct Rando;

pub fn version() -> Version {
    let version = Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version");
    assert_eq!(version, ootr_static_derive::version!());
    version
}
