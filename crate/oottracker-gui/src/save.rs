//! Stuff that's saved on disk (config, current tracker state, etc)

use {
    std::{
        fmt,
        vec,
    },
    enum_iterator::IntoEnumIterator,
    smart_default::SmartDefault,
    oottracker::model::Medallion,
};
#[cfg(not(target_arch = "wasm32"))] use {
    std::{
        io,
        sync::Arc,
    },
    directories::ProjectDirs,
    serde::{
        Deserialize,
        Serialize,
    },
    tokio::{
        fs::{
            self,
            File,
        },
        prelude::*,
    },
};

const VERSION: u8 = 0;

#[derive(Debug, SmartDefault, Clone, Copy)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, Serialize))]
#[cfg_attr(not(target_arch = "wasm32"), serde(rename_all = "camelCase"))]
pub(crate) struct Config {
    #[default(ElementOrder::LightShadowSpirit)]
    #[cfg_attr(not(target_arch = "wasm32"), serde(default = "default_med_order"))]
    pub(crate) med_order: ElementOrder,
    #[default(ElementOrder::SpiritShadowLight)]
    #[cfg_attr(not(target_arch = "wasm32"), serde(default = "default_warp_song_order"))]
    pub(crate) warp_song_order: ElementOrder,
    #[default(VERSION)]
    pub(crate) version: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoEnumIterator)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, Serialize))]
#[cfg_attr(not(target_arch = "wasm32"), serde(rename_all = "camelCase"))]
pub(crate) enum ElementOrder {
    LightShadowSpirit,
    LightSpiritShadow,
    ShadowSpiritLight,
    SpiritShadowLight,
}

impl IntoIterator for ElementOrder {
    type IntoIter = vec::IntoIter<Medallion>;
    type Item = Medallion;

    fn into_iter(self) -> vec::IntoIter<Medallion> {
        use Medallion::*;

        match self {
            ElementOrder::LightShadowSpirit => vec![Light, Forest, Fire, Water, Shadow, Spirit],
            ElementOrder::LightSpiritShadow => vec![Light, Forest, Fire, Water, Spirit, Shadow],
            ElementOrder::ShadowSpiritLight => vec![Forest, Fire, Water, Shadow, Spirit, Light],
            ElementOrder::SpiritShadowLight => vec![Forest, Fire, Water, Spirit, Shadow, Light],
        }.into_iter()
    }
}

impl fmt::Display for ElementOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElementOrder::LightShadowSpirit => write!(f, "Light first, Shadow before Spirit"),
            ElementOrder::LightSpiritShadow => write!(f, "Light first, Spirit before Shadow"),
            ElementOrder::ShadowSpiritLight => write!(f, "Shadow before Spirit, Light last"),
            ElementOrder::SpiritShadowLight => write!(f, "Spirit before Shadow, Light last"),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub(crate) enum Error {
    Io(Arc<io::Error>),
    Json(Arc<serde_json::Error>),
    MissingHomeDir,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(Arc::new(e))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Json(Arc::new(e))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Json(e) => e.fmt(f),
            Error::MissingHomeDir => write!(f, "could not find your user folder"),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Config {
    /// If the config file doesn't exist, this returns `Ok(None)`, so that the welcome message can be displayed.
    pub(crate) async fn new() -> Result<Option<Config>, Error> {
        let dirs = dirs()?;
        let mut file = match File::open(dirs.config_dir().join("config.json")).await {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let mut buf = String::default();
        file.read_to_string(&mut buf).await?;
        Ok(Some(serde_json::from_str(&buf)?)) //TODO use async-json instead
    }

    pub(crate) async fn save(&self) -> Result<(), Error> {
        let dirs = dirs()?;
        let buf = serde_json::to_vec(self)?; //TODO use async-json instead
        fs::create_dir_all(dirs.config_dir()).await?;
        let mut file = File::create(dirs.config_dir().join("config.json")).await?;
        file.write_all(&buf).await?;
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn dirs() -> Result<ProjectDirs, Error> {
    ProjectDirs::from("net", "Fenhl", "OoT Tracker").ok_or(Error::MissingHomeDir)
}

#[cfg(not(target_arch = "wasm32"))] fn default_med_order() -> ElementOrder { ElementOrder::LightShadowSpirit }
#[cfg(not(target_arch = "wasm32"))] fn default_warp_song_order() -> ElementOrder { ElementOrder::SpiritShadowLight }
