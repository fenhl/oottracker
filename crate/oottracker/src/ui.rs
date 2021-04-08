use {
    std::{
        collections::HashMap,
        fmt,
        io,
        path::{
            Path,
            PathBuf,
        },
        sync::Arc,
        vec,
    },
    async_proto::Protocol,
    directories::ProjectDirs,
    enum_iterator::IntoEnumIterator,
    image::DynamicImage,
    serde::{
        Deserialize,
        Serialize,
    },
    smart_default::SmartDefault,
    tokio::{
        fs::{
            self,
            File,
        },
        io::{
            AsyncReadExt as _,
            AsyncWriteExt as _,
        },
    },
    wheel::FromArc,
    ootr::model::{
        Dungeon,
        DungeonReward,
        DungeonRewardLocation,
        MainDungeon,
        Medallion,
        Stone,
    },
    crate::{
        ModelStateView,
        info_tables::*,
        save::*,
    },
};

const VERSION: u8 = 0;

#[derive(Debug, FromArc, Clone)]
pub enum Error {
    #[from_arc]
    Io(Arc<io::Error>),
    #[from_arc]
    Json(Arc<serde_json::Error>),
    MissingHomeDir,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Json(e) => e.fmt(f),
            Error::MissingHomeDir => write!(f, "could not find your user folder"),
        }
    }
}

#[derive(Debug, SmartDefault, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[default(ElementOrder::LightShadowSpirit)]
    #[serde(default = "default_med_order")]
    pub med_order: ElementOrder,
    #[default(ElementOrder::SpiritShadowLight)]
    #[serde(default = "default_warp_song_order")]
    pub warp_song_order: ElementOrder,
    #[default(VERSION)]
    pub version: u8,
}

impl Config {
    /// If the config file doesn't exist, this returns `Ok(None)`, so that the welcome message can be displayed.
    pub async fn new() -> Result<Option<Config>, Error> {
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

    pub async fn save(&self) -> Result<(), Error> {
        let dirs = dirs()?;
        let buf = serde_json::to_vec(self)?; //TODO use async-json instead
        fs::create_dir_all(dirs.config_dir()).await?;
        let mut file = File::create(dirs.config_dir().join("config.json")).await?;
        file.write_all(&buf).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoEnumIterator, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ElementOrder {
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

pub trait DungeonRewardLocationExt {
    fn increment(&mut self, key: DungeonReward);
    fn decrement(&mut self, key: DungeonReward);
}

impl DungeonRewardLocationExt for HashMap<DungeonReward, DungeonRewardLocation> {
    fn increment(&mut self, key: DungeonReward) {
        match self.get(&key) {
            None => self.insert(key, DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)) => self.insert(key, DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)) => self.insert(key, DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)) => self.insert(key, DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)) => self.insert(key, DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)) => self.insert(key, DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)) => self.insert(key, DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)) => self.insert(key, DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)) => self.insert(key, DungeonRewardLocation::LinksPocket),
            Some(DungeonRewardLocation::LinksPocket) => self.remove(&key),
        };
    }

    fn decrement(&mut self, key: DungeonReward) {
        match self.get(&key) {
            None => self.insert(key, DungeonRewardLocation::LinksPocket),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)) => self.remove(&key),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)) => self.insert(key, DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)) => self.insert(key, DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)) => self.insert(key, DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)) => self.insert(key, DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)) => self.insert(key, DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)) => self.insert(key, DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)),
            Some(DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)) => self.insert(key, DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)),
            Some(DungeonRewardLocation::LinksPocket) => self.insert(key, DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)),
        };
    }
}

pub enum TrackerCellKind {
    BigPoeTriforce,
    BossKey {
        active: Box<dyn Fn(&BossKeys) -> bool>,
        toggle: Box<dyn Fn(&mut BossKeys)>,
    },
    Composite {
        left_img: &'static str,
        right_img: &'static str,
        both_img: &'static str,
        active: Box<dyn Fn(&dyn ModelStateView) -> (bool, bool)>,
        toggle_left: Box<dyn Fn(&mut dyn ModelStateView)>,
        toggle_right: Box<dyn Fn(&mut dyn ModelStateView)>,
    },
    Count {
        dimmed_img: &'static str,
        img: &'static str,
        get: Box<dyn Fn(&dyn ModelStateView) -> u8>,
        set: Box<dyn Fn(&mut dyn ModelStateView, u8)>,
        max: u8,
        step: u8,
    },
    FortressMq, // a cell kind used on Xopar's tracker to show whether Gerudo Fortress has 4 carpenters
    Medallion(Medallion),
    MedallionLocation(Medallion),
    Mq(Dungeon),
    OptionalOverlay {
        main_img: &'static str,
        overlay_img: &'static str,
        active: Box<dyn Fn(&dyn ModelStateView) -> (bool, bool)>,
        toggle_main: Box<dyn Fn(&mut dyn ModelStateView)>,
        toggle_overlay: Box<dyn Fn(&mut dyn ModelStateView)>,
    },
    Overlay {
        main_img: &'static str,
        overlay_img: &'static str,
        active: Box<dyn Fn(&dyn ModelStateView) -> (bool, bool)>,
        toggle_main: Box<dyn Fn(&mut dyn ModelStateView)>,
        toggle_overlay: Box<dyn Fn(&mut dyn ModelStateView)>,
    },
    Sequence {
        idx: Box<dyn Fn(&dyn ModelStateView) -> u8>,
        img: Box<dyn Fn(&dyn ModelStateView) -> (bool, &'static str)>,
        increment: Box<dyn Fn(&mut dyn ModelStateView)>,
        decrement: Box<dyn Fn(&mut dyn ModelStateView)>,
    },
    Simple {
        img: &'static str,
        active: Box<dyn Fn(&dyn ModelStateView) -> bool>,
        toggle: Box<dyn Fn(&mut dyn ModelStateView)>,
    },
    SmallKeys {
        get: Box<dyn Fn(&crate::save::SmallKeys) -> u8>,
        set: Box<dyn Fn(&mut crate::save::SmallKeys, u8)>,
        max_vanilla: u8,
        max_mq: u8,
    },
    Song {
        song: QuestItems,
        check: &'static str,
        toggle_overlay: Box<dyn Fn(&mut EventChkInf)>,
    },
    SongCheck {
        check: &'static str,
        toggle_overlay: Box<dyn Fn(&mut EventChkInf)>,
    },
    Stone(Stone),
    StoneLocation(Stone),
}

use TrackerCellKind::*;

macro_rules! cells {
    ($($cell:ident: $kind:expr,)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Protocol)]
        pub enum TrackerCellId {
            $(
                $cell,
            )*
        }

        impl TrackerCellId {
            pub fn kind(&self) -> TrackerCellKind {
                #[allow(unused_qualifications)]
                match self {
                    $(TrackerCellId::$cell => $kind,)*
                }
            }
        }
    }
}

cells! {
    GoMode: Simple {
        img: "UNIMPLEMENTED",
        active: Box::new(|_| false), //TODO
        toggle: Box::new(|_| ()), //TODO
    },
    GoBk: Overlay {
        main_img: "UNIMPLEMENTED",
        overlay_img: "UNIMPLEMENTED",
        active: Box::new(|_| (false, false)), //TODO
        toggle_main: Box::new(|_| ()), //TODO
        toggle_overlay: Box::new(|_| ()), //TODO
    },
    LightMedallionLocation: MedallionLocation(Medallion::Light),
    ForestMedallionLocation: MedallionLocation(Medallion::Forest),
    FireMedallionLocation: MedallionLocation(Medallion::Fire),
    WaterMedallionLocation: MedallionLocation(Medallion::Water),
    ShadowMedallionLocation: MedallionLocation(Medallion::Shadow),
    SpiritMedallionLocation: MedallionLocation(Medallion::Spirit),
    LightMedallion: Medallion(Medallion::Light),
    ForestMedallion: Medallion(Medallion::Forest),
    FireMedallion: Medallion(Medallion::Fire),
    WaterMedallion: Medallion(Medallion::Water),
    ShadowMedallion: Medallion(Medallion::Shadow),
    SpiritMedallion: Medallion(Medallion::Spirit),
    AdultTrade: Sequence {
        idx: Box::new(|state| match state.ram().save.inv.adult_trade_item {
            AdultTradeItem::None => 0,
            AdultTradeItem::PocketEgg => 1,
            AdultTradeItem::PocketCucco => 2,
            AdultTradeItem::Cojiro => 3,
            AdultTradeItem::OddMushroom => 4,
            AdultTradeItem::OddPotion => 5,
            AdultTradeItem::PoachersSaw => 6,
            AdultTradeItem::BrokenSword => 7,
            AdultTradeItem::Prescription => 8,
            AdultTradeItem::EyeballFrog => 9,
            AdultTradeItem::Eyedrops => 10,
            AdultTradeItem::ClaimCheck => 11,
        }),
        img: Box::new(|state| match state.ram().save.inv.adult_trade_item {
            AdultTradeItem::None => (false, "blue_egg"),
            AdultTradeItem::PocketEgg | AdultTradeItem::PocketCucco => (true, "blue_egg"),
            AdultTradeItem::Cojiro => (true, "cojiro"),
            AdultTradeItem::OddMushroom => (true, "odd_mushroom"),
            AdultTradeItem::OddPotion => (true, "odd_poultice"),
            AdultTradeItem::PoachersSaw => (true, "poachers_saw"),
            AdultTradeItem::BrokenSword => (true, "broken_sword"),
            AdultTradeItem::Prescription => (true, "prescription"),
            AdultTradeItem::EyeballFrog => (true, "eyeball_frog"),
            AdultTradeItem::Eyedrops => (true, "eye_drops"),
            AdultTradeItem::ClaimCheck => (true, "claim_check"),
        }),
        increment: Box::new(|state| state.ram_mut().save.inv.adult_trade_item = match state.ram().save.inv.adult_trade_item {
            AdultTradeItem::None => AdultTradeItem::PocketEgg,
            AdultTradeItem::PocketEgg => AdultTradeItem::PocketCucco,
            AdultTradeItem::PocketCucco => AdultTradeItem::Cojiro,
            AdultTradeItem::Cojiro => AdultTradeItem::OddMushroom,
            AdultTradeItem::OddMushroom => AdultTradeItem::OddPotion,
            AdultTradeItem::OddPotion => AdultTradeItem::PoachersSaw,
            AdultTradeItem::PoachersSaw => AdultTradeItem::BrokenSword,
            AdultTradeItem::BrokenSword => AdultTradeItem::Prescription,
            AdultTradeItem::Prescription => AdultTradeItem::EyeballFrog,
            AdultTradeItem::EyeballFrog => AdultTradeItem::Eyedrops,
            AdultTradeItem::Eyedrops => AdultTradeItem::ClaimCheck,
            AdultTradeItem::ClaimCheck => AdultTradeItem::None,
        }),
        decrement: Box::new(|state| state.ram_mut().save.inv.adult_trade_item = match state.ram().save.inv.adult_trade_item {
            AdultTradeItem::None => AdultTradeItem::ClaimCheck,
            AdultTradeItem::PocketEgg => AdultTradeItem::None,
            AdultTradeItem::PocketCucco => AdultTradeItem::PocketEgg,
            AdultTradeItem::Cojiro => AdultTradeItem::PocketEgg,
            AdultTradeItem::OddMushroom => AdultTradeItem::Cojiro,
            AdultTradeItem::OddPotion => AdultTradeItem::OddMushroom,
            AdultTradeItem::PoachersSaw => AdultTradeItem::OddPotion,
            AdultTradeItem::BrokenSword => AdultTradeItem::PoachersSaw,
            AdultTradeItem::Prescription => AdultTradeItem::BrokenSword,
            AdultTradeItem::EyeballFrog => AdultTradeItem::Prescription,
            AdultTradeItem::Eyedrops => AdultTradeItem::EyeballFrog,
            AdultTradeItem::ClaimCheck => AdultTradeItem::Eyedrops,
        }),
    },
    AdultTradeNoChicken: Sequence {
        idx: Box::new(|state| match state.ram().save.inv.adult_trade_item {
            AdultTradeItem::None => 0,
            AdultTradeItem::PocketEgg | AdultTradeItem::PocketCucco => 1,
            AdultTradeItem::Cojiro => 2,
            AdultTradeItem::OddMushroom => 3,
            AdultTradeItem::OddPotion => 4,
            AdultTradeItem::PoachersSaw => 5,
            AdultTradeItem::BrokenSword => 6,
            AdultTradeItem::Prescription => 7,
            AdultTradeItem::EyeballFrog => 8,
            AdultTradeItem::Eyedrops => 9,
            AdultTradeItem::ClaimCheck => 10,
        }),
        img: Box::new(|state| match state.ram().save.inv.adult_trade_item {
            AdultTradeItem::None => (false, "blue_egg"),
            AdultTradeItem::PocketEgg | AdultTradeItem::PocketCucco => (true, "blue_egg"),
            AdultTradeItem::Cojiro => (true, "cojiro"),
            AdultTradeItem::OddMushroom => (true, "odd_mushroom"),
            AdultTradeItem::OddPotion => (true, "odd_poultice"),
            AdultTradeItem::PoachersSaw => (true, "poachers_saw"),
            AdultTradeItem::BrokenSword => (true, "broken_sword"),
            AdultTradeItem::Prescription => (true, "prescription"),
            AdultTradeItem::EyeballFrog => (true, "eyeball_frog"),
            AdultTradeItem::Eyedrops => (true, "eye_drops"),
            AdultTradeItem::ClaimCheck => (true, "claim_check"),
        }),
        increment: Box::new(|state| state.ram_mut().save.inv.adult_trade_item = match state.ram().save.inv.adult_trade_item {
            AdultTradeItem::None => AdultTradeItem::PocketEgg,
            AdultTradeItem::PocketEgg | AdultTradeItem::PocketCucco => AdultTradeItem::Cojiro,
            AdultTradeItem::Cojiro => AdultTradeItem::OddMushroom,
            AdultTradeItem::OddMushroom => AdultTradeItem::OddPotion,
            AdultTradeItem::OddPotion => AdultTradeItem::PoachersSaw,
            AdultTradeItem::PoachersSaw => AdultTradeItem::BrokenSword,
            AdultTradeItem::BrokenSword => AdultTradeItem::Prescription,
            AdultTradeItem::Prescription => AdultTradeItem::EyeballFrog,
            AdultTradeItem::EyeballFrog => AdultTradeItem::Eyedrops,
            AdultTradeItem::Eyedrops => AdultTradeItem::ClaimCheck,
            AdultTradeItem::ClaimCheck => AdultTradeItem::None,
        }),
        decrement: Box::new(|state| state.ram_mut().save.inv.adult_trade_item = match state.ram().save.inv.adult_trade_item {
            AdultTradeItem::None => AdultTradeItem::ClaimCheck,
            AdultTradeItem::PocketEgg | AdultTradeItem::PocketCucco => AdultTradeItem::None,
            AdultTradeItem::Cojiro => AdultTradeItem::PocketEgg,
            AdultTradeItem::OddMushroom => AdultTradeItem::Cojiro,
            AdultTradeItem::OddPotion => AdultTradeItem::OddMushroom,
            AdultTradeItem::PoachersSaw => AdultTradeItem::OddPotion,
            AdultTradeItem::BrokenSword => AdultTradeItem::PoachersSaw,
            AdultTradeItem::Prescription => AdultTradeItem::BrokenSword,
            AdultTradeItem::EyeballFrog => AdultTradeItem::Prescription,
            AdultTradeItem::Eyedrops => AdultTradeItem::EyeballFrog,
            AdultTradeItem::ClaimCheck => AdultTradeItem::Eyedrops,
        }),
    },
    Skulltula: Count {
        dimmed_img: "golden_skulltula",
        img: "skulls",
        get: Box::new(|state| state.ram().save.skull_tokens),
        set: Box::new(|state, value| state.ram_mut().save.skull_tokens = value),
        max: 100,
        step: 1,
    },
    SkulltulaTens: Count {
        dimmed_img: "golden_skulltula",
        img: "skulls",
        get: Box::new(|state| state.ram().save.skull_tokens),
        set: Box::new(|state, value| state.ram_mut().save.skull_tokens = value),
        max: 50,
        step: 10,
    },
    KokiriEmeraldLocation: StoneLocation(Stone::KokiriEmerald),
    KokiriEmerald: Stone(Stone::KokiriEmerald),
    GoronRubyLocation: StoneLocation(Stone::GoronRuby),
    GoronRuby: Stone(Stone::GoronRuby),
    ZoraSapphireLocation: StoneLocation(Stone::ZoraSapphire),
    ZoraSapphire: Stone(Stone::ZoraSapphire),
    Bottle: OptionalOverlay {
        main_img: "bottle",
        overlay_img: "letter",
        active: Box::new(|state| (state.ram().save.inv.emptiable_bottles() > 0, state.ram().save.inv.has_rutos_letter())), //TODO also show Ruto's letter as active if it has been delivered
        toggle_main: Box::new(|state| {
            let new_val = if state.ram().save.inv.emptiable_bottles() > 0 { 0 } else { 1 };
            state.ram_mut().save.inv.set_emptiable_bottles(new_val);
        }),
        toggle_overlay: Box::new(|state| state.ram_mut().save.inv.toggle_rutos_letter()),
    },
    NumBottles: Count {
        dimmed_img: "bottle",
        img: "UNIMPLEMENTED",
        get: Box::new(|state| state.ram().save.inv.emptiable_bottles()),
        set: Box::new(|state, value| state.ram_mut().save.inv.set_emptiable_bottles(value)),
        max: 4,
        step: 1,
    },
    RutosLetter: Simple {
        img: "UNIMPLEMENTED",
        active: Box::new(|state| state.ram().save.inv.has_rutos_letter()), //TODO also show Ruto's letter as active if it has been delivered
        toggle: Box::new(|state| state.ram_mut().save.inv.toggle_rutos_letter()),
    },
    Scale: Sequence {
        idx: Box::new(|state| match state.ram().save.upgrades.scale() {
            Upgrades::SILVER_SCALE => 1,
            Upgrades::GOLD_SCALE => 2,
            _ => 0,
        }),
        img: Box::new(|state| match state.ram().save.upgrades.scale() {
            Upgrades::SILVER_SCALE => (true, "silver_scale"),
            Upgrades::GOLD_SCALE => (true, "gold_scale"),
            _ => (false, "silver_scale"),
        }),
        increment: Box::new(|state| {
            let new_val = match state.ram().save.upgrades.scale() {
                Upgrades::SILVER_SCALE => Upgrades::GOLD_SCALE,
                Upgrades::GOLD_SCALE => Upgrades::NONE,
                _ => Upgrades::SILVER_SCALE,
            };
            state.ram_mut().save.upgrades.set_scale(new_val);
        }),
        decrement: Box::new(|state| {
            let new_val = match state.ram().save.upgrades.scale() {
                Upgrades::SILVER_SCALE => Upgrades::NONE,
                Upgrades::GOLD_SCALE => Upgrades::SILVER_SCALE,
                _ => Upgrades::GOLD_SCALE,
            };
            state.ram_mut().save.upgrades.set_scale(new_val);
        }),
    },
    Slingshot: Simple {
        img: "slingshot",
        active: Box::new(|state| state.ram().save.inv.slingshot),
        toggle: Box::new(|state| {
            state.ram_mut().save.inv.slingshot = !state.ram().save.inv.slingshot;
            let new_bullet_bag = if state.ram().save.inv.slingshot { Upgrades::BULLET_BAG_30 } else { Upgrades::NONE };
            state.ram_mut().save.upgrades.set_bullet_bag(new_bullet_bag);
        }),
    },
    BulletBag: Sequence {
        idx: Box::new(|state| match state.ram().save.upgrades.bullet_bag() {
            Upgrades::BULLET_BAG_30 => 1,
            Upgrades::BULLET_BAG_40 => 2,
            Upgrades::BULLET_BAG_50 => 3,
            _ => 0,
        }),
        img: Box::new(|state| (state.ram().save.inv.slingshot, "slingshot")),
        increment: Box::new(|state| {
            let new_bullet_bag = match state.ram().save.upgrades.bullet_bag() {
                Upgrades::BULLET_BAG_30 => Upgrades::BULLET_BAG_40,
                Upgrades::BULLET_BAG_40 => Upgrades::BULLET_BAG_50,
                Upgrades::BULLET_BAG_50 => Upgrades::NONE,
                _ => Upgrades::BULLET_BAG_30,
            };
            state.ram_mut().save.upgrades.set_bullet_bag(new_bullet_bag);
            state.ram_mut().save.inv.slingshot = state.ram().save.upgrades.bullet_bag() != Upgrades::NONE;
        }),
        decrement: Box::new(|state| {
            let new_bullet_bag = match state.ram().save.upgrades.bullet_bag() {
                Upgrades::BULLET_BAG_30 => Upgrades::NONE,
                Upgrades::BULLET_BAG_40 => Upgrades::BULLET_BAG_30,
                Upgrades::BULLET_BAG_50 => Upgrades::BULLET_BAG_40,
                _ => Upgrades::BULLET_BAG_50,
            };
            state.ram_mut().save.upgrades.set_bullet_bag(new_bullet_bag);
            state.ram_mut().save.inv.slingshot = state.ram().save.upgrades.bullet_bag() != Upgrades::NONE;
        }),
    },
    Bombs: Overlay {
        main_img: "bomb_bag",
        overlay_img: "bombchu",
        active: Box::new(|state| (state.ram().save.upgrades.bomb_bag() != Upgrades::NONE, state.ram().save.inv.bombchus)),
        toggle_main: Box::new(|state| if state.ram().save.upgrades.bomb_bag() == Upgrades::NONE {
            state.ram_mut().save.upgrades.set_bomb_bag(Upgrades::BOMB_BAG_20);
        } else {
            state.ram_mut().save.upgrades.set_bomb_bag(Upgrades::NONE);
        }),
        toggle_overlay: Box::new(|state| state.ram_mut().save.inv.bombchus = !state.ram().save.inv.bombchus),
    },
    BombBag: Sequence {
        idx: Box::new(|state| match state.ram().save.upgrades.bomb_bag() {
            Upgrades::BOMB_BAG_20 => 1,
            Upgrades::BOMB_BAG_30 => 2,
            Upgrades::BOMB_BAG_40 => 3,
            _ => 0,
        }),
        img: Box::new(|state| (state.ram().save.upgrades.bomb_bag() != Upgrades::NONE, "bomb_bag")),
        increment: Box::new(|state| {
            let new_val = match state.ram().save.upgrades.bomb_bag() {
                Upgrades::BOMB_BAG_20 => Upgrades::BOMB_BAG_30,
                Upgrades::BOMB_BAG_30 => Upgrades::BOMB_BAG_40,
                Upgrades::BOMB_BAG_40 => Upgrades::NONE,
                _ => Upgrades::BOMB_BAG_20,
            };
            state.ram_mut().save.upgrades.set_bomb_bag(new_val);
        }),
        decrement: Box::new(|state| {
            let new_val = match state.ram().save.upgrades.bomb_bag() {
                Upgrades::BOMB_BAG_20 => Upgrades::NONE,
                Upgrades::BOMB_BAG_30 => Upgrades::BOMB_BAG_20,
                Upgrades::BOMB_BAG_40 => Upgrades::BOMB_BAG_30,
                _ => Upgrades::BOMB_BAG_40,
            };
            state.ram_mut().save.upgrades.set_bomb_bag(new_val);
        }),
    },
    Bombchus: Simple {
        img: "UNIMPLEMENTED",
        active: Box::new(|state| state.ram().save.inv.bombchus),
        toggle: Box::new(|state| state.ram_mut().save.inv.bombchus = !state.ram().save.inv.bombchus),
    },
    Boomerang: Simple {
        img: "boomerang",
        active: Box::new(|state| state.ram().save.inv.boomerang),
        toggle: Box::new(|state| state.ram_mut().save.inv.boomerang = !state.ram().save.inv.boomerang),
    },
    Strength: Sequence {
        idx: Box::new(|state| match state.ram().save.upgrades.strength() {
            Upgrades::GORON_BRACELET => 1,
            Upgrades::SILVER_GAUNTLETS => 2,
            Upgrades::GOLD_GAUNTLETS => 3,
            _ => 0,
        }),
        img: Box::new(|state| match state.ram().save.upgrades.strength() {
            Upgrades::GORON_BRACELET => (true, "goron_bracelet"),
            Upgrades::SILVER_GAUNTLETS => (true, "silver_gauntlets"),
            Upgrades::GOLD_GAUNTLETS => (true, "gold_gauntlets"),
            _ => (false, "goron_bracelet"),
        }),
        increment: Box::new(|state| {
            let new_val = match state.ram().save.upgrades.strength() {
                Upgrades::GORON_BRACELET => Upgrades::SILVER_GAUNTLETS,
                Upgrades::SILVER_GAUNTLETS => Upgrades::GOLD_GAUNTLETS,
                Upgrades::GOLD_GAUNTLETS => Upgrades::NONE,
                _ => Upgrades::GORON_BRACELET,
            };
            state.ram_mut().save.upgrades.set_strength(new_val);
        }),
        decrement: Box::new(|state| {
            let new_val = match state.ram().save.upgrades.strength() {
                Upgrades::GORON_BRACELET => Upgrades::NONE,
                Upgrades::SILVER_GAUNTLETS => Upgrades::GORON_BRACELET,
                Upgrades::GOLD_GAUNTLETS => Upgrades::SILVER_GAUNTLETS,
                _ => Upgrades::GOLD_GAUNTLETS,
            };
            state.ram_mut().save.upgrades.set_strength(new_val);
        }),
    },
    Magic: Overlay {
        main_img: "magic",
        overlay_img: "lens",
        active: Box::new(|state| (state.ram().save.magic != MagicCapacity::None, state.ram().save.inv.lens)),
        toggle_main: Box::new(|state| if state.ram().save.magic == MagicCapacity::None {
            state.ram_mut().save.magic = MagicCapacity::Small;
        } else {
            state.ram_mut().save.magic = MagicCapacity::None;
        }),
        toggle_overlay: Box::new(|state| state.ram_mut().save.inv.lens = !state.ram().save.inv.lens),
    },
    MagicCapacity: Sequence {
        idx: Box::new(|state| match state.ram().save.magic {
            MagicCapacity::None => 0,
            MagicCapacity::Small => 1,
            MagicCapacity::Large => 2,
        }),
        img: Box::new(|state| (state.ram().save.magic != MagicCapacity::None, "magic")),
        increment: Box::new(|state| state.ram_mut().save.magic = match state.ram().save.magic {
            MagicCapacity::None => MagicCapacity::Small,
            MagicCapacity::Small => MagicCapacity::Large,
            MagicCapacity::Large => MagicCapacity::None,
        }),
        decrement: Box::new(|state| state.ram_mut().save.magic = match state.ram().save.magic {
            MagicCapacity::None => MagicCapacity::Large,
            MagicCapacity::Small => MagicCapacity::None,
            MagicCapacity::Large => MagicCapacity::Small,
        }),
    },
    Lens: Simple {
        img: "lens",
        active: Box::new(|state| state.ram().save.inv.lens),
        toggle: Box::new(|state| state.ram_mut().save.inv.lens = !state.ram().save.inv.lens),
    },
    Spells: Composite {
        left_img: "dins_fire",
        right_img: "faores_wind",
        both_img: "composite_magic",
        active: Box::new(|state| (state.ram().save.inv.dins_fire, state.ram().save.inv.farores_wind)),
        toggle_left: Box::new(|state| state.ram_mut().save.inv.dins_fire = !state.ram().save.inv.dins_fire),
        toggle_right: Box::new(|state| state.ram_mut().save.inv.farores_wind = !state.ram().save.inv.farores_wind),
    },
    DinsFire: Simple {
        img: "dins_fire",
        active: Box::new(|state| state.ram().save.inv.dins_fire),
        toggle: Box::new(|state| state.ram_mut().save.inv.dins_fire = !state.ram().save.inv.dins_fire),
    },
    FaroresWind: Simple {
        img: "faores_wind",
        active: Box::new(|state| state.ram().save.inv.farores_wind),
        toggle: Box::new(|state| state.ram_mut().save.inv.farores_wind = !state.ram().save.inv.farores_wind),
    },
    NayrusLove: Simple {
        img: "UNIMPLEMENTED", //TODO
        active: Box::new(|state| state.ram().save.inv.nayrus_love),
        toggle: Box::new(|state| state.ram_mut().save.inv.nayrus_love = !state.ram().save.inv.nayrus_love),
    },
    Hookshot: Sequence {
        idx: Box::new(|state| match state.ram().save.inv.hookshot {
            Hookshot::None => 0,
            Hookshot::Hookshot => 1,
            Hookshot::Longshot => 2,
        }),
        img: Box::new(|state| match state.ram().save.inv.hookshot {
            Hookshot::None => (false, "hookshot"),
            Hookshot::Hookshot => (true, "hookshot_accessible"),
            Hookshot::Longshot => (true, "longshot_accessible"),
        }),
        increment: Box::new(|state| state.ram_mut().save.inv.hookshot = match state.ram().save.inv.hookshot {
            Hookshot::None => Hookshot::Hookshot,
            Hookshot::Hookshot => Hookshot::Longshot,
            Hookshot::Longshot => Hookshot::None,
        }),
        decrement: Box::new(|state| state.ram_mut().save.inv.hookshot = match state.ram().save.inv.hookshot {
            Hookshot::None => Hookshot::Longshot,
            Hookshot::Hookshot => Hookshot::None,
            Hookshot::Longshot => Hookshot::Hookshot,
        }),
    },
    Bow: OptionalOverlay {
        main_img: "bow",
        overlay_img: "ice_arrows",
        active: Box::new(|state| (state.ram().save.inv.bow, state.ram().save.inv.ice_arrows)),
        toggle_main: Box::new(|state| {
            state.ram_mut().save.inv.bow = !state.ram().save.inv.bow;
            let new_quiver = if state.ram().save.inv.bow { Upgrades::QUIVER_30 } else { Upgrades::NONE };
            state.ram_mut().save.upgrades.set_quiver(new_quiver);
        }),
        toggle_overlay: Box::new(|state| state.ram_mut().save.inv.ice_arrows = !state.ram().save.inv.ice_arrows),
    },
    IceArrows: Simple {
        img: "ice_trap",
        active: Box::new(|state| state.ram().save.inv.ice_arrows),
        toggle: Box::new(|state| state.ram_mut().save.inv.ice_arrows = !state.ram().save.inv.ice_arrows),
    },
    Quiver: Sequence {
        idx: Box::new(|state| match state.ram().save.upgrades.quiver() {
            Upgrades::QUIVER_30 => 1,
            Upgrades::QUIVER_40 => 2,
            Upgrades::QUIVER_50 => 3,
            _ => 0,
        }),
        img: Box::new(|state| (state.ram().save.inv.bow, "bow")),
        increment: Box::new(|state| {
            let new_quiver = match state.ram().save.upgrades.quiver() {
                Upgrades::QUIVER_30 => Upgrades::QUIVER_40,
                Upgrades::QUIVER_40 => Upgrades::QUIVER_50,
                Upgrades::QUIVER_50 => Upgrades::NONE,
                _ => Upgrades::QUIVER_30,
            };
            state.ram_mut().save.upgrades.set_quiver(new_quiver);
            state.ram_mut().save.inv.bow = state.ram().save.upgrades.quiver() != Upgrades::NONE;
        }),
        decrement: Box::new(|state| {
            let new_quiver = match state.ram().save.upgrades.quiver() {
                Upgrades::QUIVER_30 => Upgrades::NONE,
                Upgrades::QUIVER_40 => Upgrades::QUIVER_30,
                Upgrades::QUIVER_50 => Upgrades::QUIVER_40,
                _ => Upgrades::QUIVER_50,
            };
            state.ram_mut().save.upgrades.set_quiver(new_quiver);
            state.ram_mut().save.inv.bow = state.ram().save.upgrades.quiver() != Upgrades::NONE;
        }),
    },
    Arrows: Composite {
        left_img: "fire_arrows",
        right_img: "light_arrows",
        both_img: "composite_arrows",
        active: Box::new(|state| (state.ram().save.inv.fire_arrows, state.ram().save.inv.light_arrows)),
        toggle_left: Box::new(|state| state.ram_mut().save.inv.fire_arrows = !state.ram().save.inv.fire_arrows),
        toggle_right: Box::new(|state| state.ram_mut().save.inv.light_arrows = !state.ram().save.inv.light_arrows),
    },
    FireArrows: Simple {
        img: "fire_arrows",
        active: Box::new(|state| state.ram().save.inv.fire_arrows),
        toggle: Box::new(|state| state.ram_mut().save.inv.fire_arrows = !state.ram().save.inv.fire_arrows),
    },
    LightArrows: Simple {
        img: "light_arrows",
        active: Box::new(|state| state.ram().save.inv.light_arrows),
        toggle: Box::new(|state| state.ram_mut().save.inv.light_arrows = !state.ram().save.inv.light_arrows),
    },
    Hammer: Simple {
        img: "hammer",
        active: Box::new(|state| state.ram().save.inv.hammer),
        toggle: Box::new(|state| state.ram_mut().save.inv.hammer = !state.ram().save.inv.hammer),
    },
    Boots: Composite {
        left_img: "iron_boots",
        right_img: "hover_boots",
        both_img: "composite_boots",
        active: Box::new(|state| (state.ram().save.equipment.contains(Equipment::IRON_BOOTS), state.ram().save.equipment.contains(Equipment::HOVER_BOOTS))),
        toggle_left: Box::new(|state| state.ram_mut().save.equipment.toggle(Equipment::IRON_BOOTS)),
        toggle_right: Box::new(|state| state.ram_mut().save.equipment.toggle(Equipment::HOVER_BOOTS)),
    },
    IronBoots: Simple {
        img: "iron_boots",
        active: Box::new(|state| state.ram().save.equipment.contains(Equipment::IRON_BOOTS)),
        toggle: Box::new(|state| state.ram_mut().save.equipment.toggle(Equipment::IRON_BOOTS)),
    },
    HoverBoots: Simple {
        img: "hover_boots",
        active: Box::new(|state| state.ram().save.equipment.contains(Equipment::HOVER_BOOTS)),
        toggle: Box::new(|state| state.ram_mut().save.equipment.toggle(Equipment::HOVER_BOOTS)),
    },
    MirrorShield: Simple {
        img: "mirror_shield",
        active: Box::new(|state| state.ram().save.equipment.contains(Equipment::MIRROR_SHIELD)),
        toggle: Box::new(|state| state.ram_mut().save.equipment.toggle(Equipment::MIRROR_SHIELD)),
    },
    ChildTrade: Sequence {
        idx: Box::new(|state| match state.ram().save.inv.child_trade_item {
            ChildTradeItem::None => 0,
            ChildTradeItem::WeirdEgg => 1,
            ChildTradeItem::Chicken => 2,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => 3, //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => 4,
            ChildTradeItem::SkullMask => 5,
            ChildTradeItem::SpookyMask => 6,
            ChildTradeItem::BunnyHood => 7,
            ChildTradeItem::MaskOfTruth => 8,
        }),
        img: Box::new(|state| match state.ram().save.inv.child_trade_item {
            ChildTradeItem::None => (false, "white_egg"),
            ChildTradeItem::WeirdEgg => (true, "white_egg"),
            ChildTradeItem::Chicken => (true, "white_chicken"),
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => (true, "zelda_letter"), //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => (true, "keaton_mask"),
            ChildTradeItem::SkullMask => (true, "skull_mask"),
            ChildTradeItem::SpookyMask => (true, "spooky_mask"),
            ChildTradeItem::BunnyHood => (true, "bunny_hood"),
            ChildTradeItem::MaskOfTruth => (true, "mask_of_truth"),
        }),
        increment: Box::new(|state| state.ram_mut().save.inv.child_trade_item = match state.ram().save.inv.child_trade_item {
            ChildTradeItem::None => ChildTradeItem::WeirdEgg,
            ChildTradeItem::WeirdEgg => ChildTradeItem::Chicken,
            ChildTradeItem::Chicken => ChildTradeItem::ZeldasLetter,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => ChildTradeItem::KeatonMask, //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => ChildTradeItem::SkullMask,
            ChildTradeItem::SkullMask => ChildTradeItem::SpookyMask,
            ChildTradeItem::SpookyMask => ChildTradeItem::BunnyHood,
            ChildTradeItem::BunnyHood => ChildTradeItem::MaskOfTruth,
            ChildTradeItem::MaskOfTruth => ChildTradeItem::None,
        }),
        decrement: Box::new(|state| state.ram_mut().save.inv.child_trade_item = match state.ram().save.inv.child_trade_item {
            ChildTradeItem::None => ChildTradeItem::MaskOfTruth,
            ChildTradeItem::WeirdEgg => ChildTradeItem::None,
            ChildTradeItem::Chicken => ChildTradeItem::WeirdEgg,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => ChildTradeItem::Chicken, //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => ChildTradeItem::ZeldasLetter,
            ChildTradeItem::SkullMask => ChildTradeItem::KeatonMask,
            ChildTradeItem::SpookyMask => ChildTradeItem::SkullMask,
            ChildTradeItem::BunnyHood => ChildTradeItem::SpookyMask,
            ChildTradeItem::MaskOfTruth => ChildTradeItem::BunnyHood,
        }),
    },
    ChildTradeNoChicken: Sequence {
        idx: Box::new(|state| match state.ram().save.inv.child_trade_item {
            ChildTradeItem::None => 0,
            ChildTradeItem::WeirdEgg | ChildTradeItem::Chicken => 1,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => 2, //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => 3,
            ChildTradeItem::SkullMask => 4,
            ChildTradeItem::SpookyMask => 5,
            ChildTradeItem::BunnyHood => 6,
            ChildTradeItem::MaskOfTruth => 7,
        }),
        img: Box::new(|state| match state.ram().save.inv.child_trade_item {
            ChildTradeItem::None => (false, "white_egg"),
            ChildTradeItem::WeirdEgg | ChildTradeItem::Chicken => (true, "white_egg"),
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => (true, "zelda_letter"), //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => (true, "keaton_mask"),
            ChildTradeItem::SkullMask => (true, "skull_mask"),
            ChildTradeItem::SpookyMask => (true, "spooky_mask"),
            ChildTradeItem::BunnyHood => (true, "bunny_hood"),
            ChildTradeItem::MaskOfTruth => (true, "mask_of_truth"),
        }),
        increment: Box::new(|state| state.ram_mut().save.inv.child_trade_item = match state.ram().save.inv.child_trade_item {
            ChildTradeItem::None => ChildTradeItem::WeirdEgg,
            ChildTradeItem::WeirdEgg | ChildTradeItem::Chicken => ChildTradeItem::ZeldasLetter,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => ChildTradeItem::KeatonMask, //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => ChildTradeItem::SkullMask,
            ChildTradeItem::SkullMask => ChildTradeItem::SpookyMask,
            ChildTradeItem::SpookyMask => ChildTradeItem::BunnyHood,
            ChildTradeItem::BunnyHood => ChildTradeItem::MaskOfTruth,
            ChildTradeItem::MaskOfTruth => ChildTradeItem::None,
        }),
        decrement: Box::new(|state| state.ram_mut().save.inv.child_trade_item = match state.ram().save.inv.child_trade_item {
            ChildTradeItem::None => ChildTradeItem::MaskOfTruth,
            ChildTradeItem::WeirdEgg | ChildTradeItem::Chicken => ChildTradeItem::None,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => ChildTradeItem::WeirdEgg, //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => ChildTradeItem::ZeldasLetter,
            ChildTradeItem::SkullMask => ChildTradeItem::KeatonMask,
            ChildTradeItem::SpookyMask => ChildTradeItem::SkullMask,
            ChildTradeItem::BunnyHood => ChildTradeItem::SpookyMask,
            ChildTradeItem::MaskOfTruth => ChildTradeItem::BunnyHood,
        }),
    },
    ChildTradeSoldOut: Sequence {
        idx: Box::new(|state| match state.ram().save.inv.child_trade_item {
            ChildTradeItem::None => 0,
            ChildTradeItem::WeirdEgg => 1,
            ChildTradeItem::Chicken => 2,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => 3, //TODO for SOLD OUT, check trade quest progress
            //TODO Zelda's letter turned in => 4
            ChildTradeItem::KeatonMask => 5,
            //TODO Keaton mask sold => 6
            ChildTradeItem::SkullMask => 7,
            //TODO skull mask sold => 8
            ChildTradeItem::SpookyMask => 9,
            //TODO spooky mask sold => 10
            ChildTradeItem::BunnyHood => 11,
            //TODO bunny hood sold => 12
            ChildTradeItem::MaskOfTruth => 13,
        }),
        img: Box::new(|state| match state.ram().save.inv.child_trade_item {
            ChildTradeItem::None => (false, "white_egg"),
            ChildTradeItem::WeirdEgg => (true, "white_egg"),
            ChildTradeItem::Chicken => (true, "white_chicken"),
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => (true, "zelda_letter"), //TODO for SOLD OUT, check trade quest progress
            //TODO Zelda's letter turned in => SOLD OUT
            ChildTradeItem::KeatonMask => (true, "keaton_mask"),
            //TODO Keaton mask sold => SOLD OUT
            ChildTradeItem::SkullMask => (true, "skull_mask"),
            //TODO skull mask sold => SOLD OUT
            ChildTradeItem::SpookyMask => (true, "spooky_mask"),
            //TODO spooky mask sold => SOLD OUT
            ChildTradeItem::BunnyHood => (true, "bunny_hood"),
            //TODO bunny hood sold => SOLD OUT
            ChildTradeItem::MaskOfTruth => (true, "mask_of_truth"),
        }),
        increment: Box::new(|state| state.ram_mut().save.inv.child_trade_item = match state.ram().save.inv.child_trade_item {
            //TODO consider sold-out states
            ChildTradeItem::None => ChildTradeItem::WeirdEgg,
            ChildTradeItem::WeirdEgg => ChildTradeItem::Chicken,
            ChildTradeItem::Chicken => ChildTradeItem::ZeldasLetter,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => ChildTradeItem::KeatonMask, //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => ChildTradeItem::SkullMask,
            ChildTradeItem::SkullMask => ChildTradeItem::SpookyMask,
            ChildTradeItem::SpookyMask => ChildTradeItem::BunnyHood,
            ChildTradeItem::BunnyHood => ChildTradeItem::MaskOfTruth,
            ChildTradeItem::MaskOfTruth => ChildTradeItem::None,
        }),
        decrement: Box::new(|state| state.ram_mut().save.inv.child_trade_item = match state.ram().save.inv.child_trade_item {
            //TODO consider sold-out states
            ChildTradeItem::None => ChildTradeItem::MaskOfTruth,
            ChildTradeItem::WeirdEgg => ChildTradeItem::None,
            ChildTradeItem::Chicken => ChildTradeItem::WeirdEgg,
            ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => ChildTradeItem::Chicken, //TODO for SOLD OUT, check trade quest progress
            ChildTradeItem::KeatonMask => ChildTradeItem::ZeldasLetter,
            ChildTradeItem::SkullMask => ChildTradeItem::KeatonMask,
            ChildTradeItem::SpookyMask => ChildTradeItem::SkullMask,
            ChildTradeItem::BunnyHood => ChildTradeItem::SpookyMask,
            ChildTradeItem::MaskOfTruth => ChildTradeItem::BunnyHood,
        }),
    },
    Ocarina: Overlay {
        main_img: "ocarina",
        overlay_img: "scarecrow",
        active: Box::new(|state| (state.ram().save.inv.ocarina, state.ram().save.event_chk_inf.9.contains(EventChkInf9::SCARECROW_SONG))), //TODO only show free Scarecrow's Song once it's known (by settings string input or by check)
        toggle_main: Box::new(|state| state.ram_mut().save.inv.ocarina = !state.ram().save.inv.ocarina),
        toggle_overlay: Box::new(|state| state.ram_mut().save.event_chk_inf.9.toggle(EventChkInf9::SCARECROW_SONG)), //TODO make sure free scarecrow knowledge is toggled off properly
    },
    Beans: Simple { //TODO overlay with number bought if autotracker is on & shuffle beans is off
        img: "beans",
        active: Box::new(|state| state.ram().save.inv.beans),
        toggle: Box::new(|state| state.ram_mut().save.inv.beans = !state.ram().save.inv.beans),
    },
    SwordCard: Composite {
        left_img: "kokiri_sword",
        right_img: "gerudo_card",
        both_img: "composite_ksword_gcard",
        active: Box::new(|state| (state.ram().save.equipment.contains(Equipment::KOKIRI_SWORD), state.ram().save.quest_items.contains(QuestItems::GERUDO_CARD))),
        toggle_left: Box::new(|state| state.ram_mut().save.equipment.toggle(Equipment::KOKIRI_SWORD)),
        toggle_right: Box::new(|state| state.ram_mut().save.quest_items.toggle(QuestItems::GERUDO_CARD)),
    },
    KokiriSword: Simple {
        img: "kokiri_sword",
        active: Box::new(|state| state.ram().save.equipment.contains(Equipment::KOKIRI_SWORD)),
        toggle: Box::new(|state| state.ram_mut().save.equipment.toggle(Equipment::KOKIRI_SWORD)),
    },
    Tunics: Composite {
        left_img: "goron_tunic",
        right_img: "zora_tunic",
        both_img: "composite_tunics",
        active: Box::new(|state| (state.ram().save.equipment.contains(Equipment::GORON_TUNIC), state.ram().save.equipment.contains(Equipment::ZORA_TUNIC))),
        toggle_left: Box::new(|state| state.ram_mut().save.equipment.toggle(Equipment::GORON_TUNIC)),
        toggle_right: Box::new(|state| state.ram_mut().save.equipment.toggle(Equipment::ZORA_TUNIC)),
    },
    GoronTunic: Simple {
        img: "goron_tunic",
        active: Box::new(|state| state.ram().save.equipment.contains(Equipment::GORON_TUNIC)),
        toggle: Box::new(|state| state.ram_mut().save.equipment.toggle(Equipment::GORON_TUNIC)),
    },
    ZoraTunic: Simple {
        img: "zora_tunic",
        active: Box::new(|state| state.ram().save.equipment.contains(Equipment::ZORA_TUNIC)),
        toggle: Box::new(|state| state.ram_mut().save.equipment.toggle(Equipment::ZORA_TUNIC)),
    },
    Triforce: Count {
        dimmed_img: "triforce",
        img: "force",
        get: Box::new(|state| state.ram().save.triforce_pieces()),
        set: Box::new(|state, value| state.ram_mut().save.set_triforce_pieces(value)),
        max: 100,
        step: 1,
    },
    BigPoeTriforce: BigPoeTriforce,
    TriforceOneAndFives: Sequence {
        idx: Box::new(|state| match state.ram().save.triforce_pieces() {
            0 => 0,
            1..=4 => 1,
            5..=9 => 2,
            10..=14 => 3,
            15..=19 => 4,
            20..=24 => 5,
            25..=29 => 6,
            30..=34 => 7,
            35..=39 => 8,
            40..=44 => 9,
            45..=49 => 10,
            50..=54 => 11,
            55..=59 => 12,
            _ => 13,
        }),
        img: Box::new(|state| (state.ram().save.triforce_pieces() > 0, "triforce")), //TODO images from count?
        increment: Box::new(|state| {
            let new_val = match state.ram().save.triforce_pieces() {
                0 => 1,
                1..=4 => 5,
                5..=9 => 10,
                10..=14 => 15,
                15..=19 => 20,
                20..=24 => 25,
                25..=29 => 30,
                30..=34 => 35,
                35..=39 => 40,
                40..=44 => 45,
                45..=49 => 50,
                50..=54 => 55,
                55..=59 => 60,
                _ => 0,
            };
            state.ram_mut().save.set_triforce_pieces(new_val);
        }),
        decrement: Box::new(|state| {
            let new_val = match state.ram().save.triforce_pieces() {
                0 => 60,
                1..=4 => 0,
                5..=9 => 1,
                10..=14 => 5,
                15..=19 => 10,
                20..=24 => 15,
                25..=29 => 20,
                30..=34 => 25,
                35..=39 => 30,
                40..=44 => 35,
                45..=49 => 40,
                50..=54 => 45,
                55..=59 => 50,
                _ => 55,
            };
            state.ram_mut().save.set_triforce_pieces(new_val);
        }),
    },
    ZeldasLullaby: Song {
        song: QuestItems::ZELDAS_LULLABY,
        check: "Song from Impa",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_IMPA)),
    },
    ZeldasLullabyCheck: SongCheck {
        check: "Song from Impa",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_IMPA)),
    },
    EponasSong: Song {
        song: QuestItems::EPONAS_SONG,
        check: "Song from Malon",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_MALON)),
    },
    EponasSongCheck: SongCheck {
        check: "Song from Malon",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_MALON)),
    },
    SariasSong: Song {
        song: QuestItems::SARIAS_SONG,
        check: "Song from Saria",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_SARIA)),
    },
    SariasSongCheck: SongCheck {
        check: "Song from Saria",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_SARIA)),
    },
    SunsSong: Song {
        song: QuestItems::SUNS_SONG,
        check: "Song from Composers Grave",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_COMPOSERS_GRAVE)),
    },
    SunsSongCheck: SongCheck {
        check: "Song from Composers Grave",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_COMPOSERS_GRAVE)),
    },
    SongOfTime: Song {
        song: QuestItems::SONG_OF_TIME,
        check: "Song from Ocarina of Time",
        toggle_overlay: Box::new(|eci| eci.10.toggle(EventChkInf10::SONG_FROM_OCARINA_OF_TIME)),
    },
    SongOfTimeCheck: SongCheck {
        check: "Song from Ocarina of Time",
        toggle_overlay: Box::new(|eci| eci.10.toggle(EventChkInf10::SONG_FROM_OCARINA_OF_TIME)),
    },
    SongOfStorms: Song {
        song: QuestItems::SONG_OF_STORMS,
        check: "Song from Windmill",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_WINDMILL)),
    },
    SongOfStormsCheck: SongCheck {
        check: "Song from Windmill",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_WINDMILL)),
    },
    Minuet: Song {
        song: QuestItems::MINUET_OF_FOREST,
        check: "Sheik in Forest",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_FOREST)),
    },
    MinuetCheck: SongCheck {
        check: "Sheik in Forest",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_FOREST)),
    },
    Bolero: Song {
        song: QuestItems::BOLERO_OF_FIRE,
        check: "Sheik in Crater",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_CRATER)),
    },
    BoleroCheck: SongCheck {
        check: "Sheik in Crater",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_CRATER)),
    },
    Serenade: Song {
        song: QuestItems::SERENADE_OF_WATER,
        check: "Sheik in Ice Cavern",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_ICE_CAVERN)),
    },
    SerenadeCheck: SongCheck {
        check: "Sheik in Ice Cavern",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_ICE_CAVERN)),
    },
    Requiem: Song {
        song: QuestItems::REQUIEM_OF_SPIRIT,
        check: "Sheik at Colossus",
        toggle_overlay: Box::new(|eci| eci.10.toggle(EventChkInf10::SHEIK_AT_COLOSSUS)),
    },
    RequiemCheck: SongCheck {
        check: "Sheik at Colossus",
        toggle_overlay: Box::new(|eci| eci.10.toggle(EventChkInf10::SHEIK_AT_COLOSSUS)),
    },
    Nocturne: Song {
        song: QuestItems::NOCTURNE_OF_SHADOW,
        check: "Sheik in Kakariko",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_KAKARIKO)),
    },
    NocturneCheck: SongCheck {
        check: "Sheik in Kakariko",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_KAKARIKO)),
    },
    Prelude: Song {
        song: QuestItems::PRELUDE_OF_LIGHT,
        check: "Sheik at Temple",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_AT_TEMPLE)),
    },
    PreludeCheck: SongCheck {
        check: "Sheik at Temple",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_AT_TEMPLE)),
    },
    DekuMq: Mq(Dungeon::Main(MainDungeon::DekuTree)),
    DcMq: Mq(Dungeon::Main(MainDungeon::DodongosCavern)),
    JabuMq: Mq(Dungeon::Main(MainDungeon::JabuJabu)),
    ForestMq: Mq(Dungeon::Main(MainDungeon::ForestTemple)),
    ForestSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.forest_temple),
        set: Box::new(|keys, value| keys.forest_temple = value),
        max_vanilla: 5,
        max_mq: 6,
    },
    ForestBossKey: BossKey {
        active: Box::new(|keys| keys.forest_temple),
        toggle: Box::new(|keys| keys.forest_temple = !keys.forest_temple),
    },
    ShadowMq: Mq(Dungeon::Main(MainDungeon::ShadowTemple)),
    ShadowSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.shadow_temple),
        set: Box::new(|keys, value| keys.shadow_temple = value),
        max_vanilla: 5,
        max_mq: 6,
    },
    ShadowBossKey: BossKey {
        active: Box::new(|keys| keys.shadow_temple),
        toggle: Box::new(|keys| keys.shadow_temple = !keys.shadow_temple),
    },
    WellMq: Mq(Dungeon::BottomOfTheWell),
    WellSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.bottom_of_the_well),
        set: Box::new(|keys, value| keys.bottom_of_the_well = value),
        max_vanilla: 3,
        max_mq: 2,
    },
    FireMq: Mq(Dungeon::Main(MainDungeon::FireTemple)),
    FireSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.fire_temple),
        set: Box::new(|keys, value| keys.fire_temple = value),
        max_vanilla: 8,
        max_mq: 5,
    },
    FireBossKey: BossKey {
        active: Box::new(|keys| keys.fire_temple),
        toggle: Box::new(|keys| keys.fire_temple = !keys.fire_temple),
    },
    SpiritMq: Mq(Dungeon::Main(MainDungeon::SpiritTemple)),
    SpiritSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.spirit_temple),
        set: Box::new(|keys, value| keys.spirit_temple = value),
        max_vanilla: 5,
        max_mq: 7,
    },
    SpiritBossKey: BossKey {
        active: Box::new(|keys| keys.spirit_temple),
        toggle: Box::new(|keys| keys.spirit_temple = !keys.spirit_temple),
    },
    FortressMq: FortressMq,
    FortressSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.thieves_hideout),
        set: Box::new(|keys, value| keys.thieves_hideout = value),
        max_vanilla: 4,
        max_mq: 4,
    },
    WaterMq: Mq(Dungeon::Main(MainDungeon::WaterTemple)),
    WaterSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.water_temple),
        set: Box::new(|keys, value| keys.water_temple = value),
        max_vanilla: 6,
        max_mq: 2,
    },
    WaterBossKey: BossKey {
        active: Box::new(|keys| keys.water_temple),
        toggle: Box::new(|keys| keys.water_temple = !keys.water_temple),
    },
    GanonMq: Mq(Dungeon::GanonsCastle),
    GanonSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.ganons_castle),
        set: Box::new(|keys, value| keys.ganons_castle = value),
        max_vanilla: 2,
        max_mq: 3,
    },
    GanonBossKey: BossKey {
        active: Box::new(|keys| keys.ganons_castle),
        toggle: Box::new(|keys| keys.ganons_castle = !keys.ganons_castle),
    },
    GtgMq: Mq(Dungeon::GerudoTrainingGrounds),
    GtgSmallKeys: TrackerCellKind::SmallKeys {
        get: Box::new(|keys| keys.gerudo_training_grounds),
        set: Box::new(|keys, value| keys.gerudo_training_grounds = value),
        max_vanilla: 9,
        max_mq: 3,
    },
    BiggoronSword: Simple {
        img: "UNIMPLEMENTED",
        active: Box::new(|state| state.ram().save.biggoron_sword && state.ram().save.equipment.contains(Equipment::GIANTS_KNIFE)),
        toggle: Box::new(|state| if state.ram().save.biggoron_sword && state.ram().save.equipment.contains(Equipment::GIANTS_KNIFE) {
            state.ram_mut().save.biggoron_sword = false;
            state.ram_mut().save.equipment.remove(Equipment::GIANTS_KNIFE);
        } else {
            state.ram_mut().save.biggoron_sword = true;
            state.ram_mut().save.equipment.insert(Equipment::GIANTS_KNIFE);
        }),
    },
    WalletNoTycoon: Sequence {
        idx: Box::new(|state| match state.ram().save.upgrades.wallet() {
            Upgrades::ADULTS_WALLET => 1,
            Upgrades::GIANTS_WALLET | Upgrades::TYCOONS_WALLET => 2,
            _ => 0,
        }),
        img: Box::new(|state| (state.ram().save.upgrades.wallet() != Upgrades::NONE, "UNIMPLEMENTED")),
        increment: Box::new(|state| {
            let new_val = match state.ram().save.upgrades.wallet() {
                Upgrades::ADULTS_WALLET => Upgrades::GIANTS_WALLET,
                Upgrades::GIANTS_WALLET | Upgrades::TYCOONS_WALLET => Upgrades::NONE,
                _ => Upgrades::ADULTS_WALLET,
            };
            state.ram_mut().save.upgrades.set_wallet(new_val);
        }),
        decrement: Box::new(|state| {
            let new_val = match state.ram().save.upgrades.wallet() {
                Upgrades::ADULTS_WALLET => Upgrades::NONE,
                Upgrades::GIANTS_WALLET | Upgrades::TYCOONS_WALLET => Upgrades::ADULTS_WALLET,
                _ => Upgrades::GIANTS_WALLET,
            };
            state.ram_mut().save.upgrades.set_wallet(new_val);
        }),
    },
    StoneOfAgony: Simple {
        img: "UNIMPLEMENTED",
        active: Box::new(|state| state.ram().save.quest_items.contains(QuestItems::STONE_OF_AGONY)),
        toggle: Box::new(|state| state.ram_mut().save.quest_items.toggle(QuestItems::STONE_OF_AGONY)),
    },
}

impl TrackerCellId {
    pub fn med_location(med: Medallion) -> TrackerCellId {
        match med {
            Medallion::Light => TrackerCellId::LightMedallionLocation,
            Medallion::Forest => TrackerCellId::ForestMedallionLocation,
            Medallion::Fire => TrackerCellId::FireMedallionLocation,
            Medallion::Water => TrackerCellId::WaterMedallionLocation,
            Medallion::Shadow => TrackerCellId::ShadowMedallionLocation,
            Medallion::Spirit => TrackerCellId::SpiritMedallionLocation,
        }
    }

    pub fn warp_song(med: Medallion) -> TrackerCellId {
        match med {
            Medallion::Light => TrackerCellId::Prelude,
            Medallion::Forest => TrackerCellId::Minuet,
            Medallion::Fire => TrackerCellId::Bolero,
            Medallion::Water => TrackerCellId::Serenade,
            Medallion::Shadow => TrackerCellId::Nocturne,
            Medallion::Spirit => TrackerCellId::Requiem,
        }
    }
}

impl From<Medallion> for TrackerCellId {
    fn from(med: Medallion) -> TrackerCellId {
        match med {
            Medallion::Light => TrackerCellId::LightMedallion,
            Medallion::Forest => TrackerCellId::ForestMedallion,
            Medallion::Fire => TrackerCellId::FireMedallion,
            Medallion::Water => TrackerCellId::WaterMedallion,
            Medallion::Shadow => TrackerCellId::ShadowMedallion,
            Medallion::Spirit => TrackerCellId::SpiritMedallion,
        }
    }
}

#[derive(Debug)]
pub struct TrackerLayout {
    pub meds: ElementOrder,
    pub row2: [TrackerCellId; 4],
    pub rest: [[TrackerCellId; 6]; 4],
    pub warp_songs: ElementOrder,
}

impl TrackerLayout {
    /// The default layout for auto-tracking, which replaces the Triforce piece count cell with a dynamic big Poe count/Triforce piece count cell.
    pub fn default_auto() -> TrackerLayout { TrackerLayout::new_auto(&Config::default()) }

    /// The auto-tracking layout for this config, which replaces the Triforce piece count cell with a dynamic big Poe count/Triforce piece count cell.
    pub fn new_auto(config: &Config) -> TrackerLayout {
        let mut layout = TrackerLayout::from(config);
        layout.rest[2][5] = TrackerCellId::BigPoeTriforce;
        layout
    }
}

impl Default for TrackerLayout {
    fn default() -> TrackerLayout { TrackerLayout::from(&Config::default()) }
}

impl<'a> From<&'a Config> for TrackerLayout {
    fn from(config: &Config) -> TrackerLayout {
        use self::TrackerCellId::*;

        TrackerLayout {
            meds: config.med_order,
            row2: [AdultTradeNoChicken, Skulltula, Bottle, Scale],
            rest: [
                [Slingshot, Bombs, Boomerang, Strength, Magic, Spells],
                [Hookshot, Bow, Arrows, Hammer, Boots, MirrorShield],
                [ChildTrade, Ocarina, Beans, SwordCard, Tunics, Triforce],
                [ZeldasLullaby, EponasSong, SariasSong, SunsSong, SongOfTime, SongOfStorms],
            ],
            warp_songs: config.warp_song_order,
        }
    }
}

fn default_med_order() -> ElementOrder { ElementOrder::LightShadowSpirit }
fn default_warp_song_order() -> ElementOrder { ElementOrder::SpiritShadowLight }

pub fn dirs() -> Result<ProjectDirs, Error> {
    ProjectDirs::from("net", "Fenhl", "OoT Tracker").ok_or(Error::MissingHomeDir)
}

pub struct EmbeddedImage {
    pub path: PathBuf,
    pub contents: &'static [u8],
}

pub trait FromEmbeddedImage {
    fn from_embedded_image(name: &Path, contents: &'static [u8]) -> Self;
}

impl FromEmbeddedImage for EmbeddedImage {
    fn from_embedded_image(name: &Path, contents: &'static [u8]) -> EmbeddedImage {
        EmbeddedImage { path: name.to_owned(), contents }
    }
}

impl FromEmbeddedImage for iced::widget::Image {
    fn from_embedded_image(_: &Path, contents: &'static [u8]) -> iced::widget::Image {
        iced::widget::Image::new(iced::image::Handle::from_memory(contents.to_vec()))
    }
}

impl FromEmbeddedImage for DynamicImage {
    fn from_embedded_image(_: &Path, contents: &'static [u8]) -> DynamicImage {
        image::load_from_memory(contents).expect("failed to load embedded DynamicImage")
    }
}

pub mod images {
    use super::FromEmbeddedImage;

    oottracker_derive::embed_images!("assets/img/extra-images");
    oottracker_derive::embed_images!("assets/img/extra-images-count");
    oottracker_derive::embed_images!("assets/img/extra-images-dimmed");
    oottracker_derive::embed_images!("assets/img/xopar-images");
    oottracker_derive::embed_images!("assets/img/xopar-images-count");
    oottracker_derive::embed_images!("assets/img/xopar-images-dimmed");
    oottracker_derive::embed_images!("assets/img/xopar-images-overlay");
    oottracker_derive::embed_images!("assets/img/xopar-images-overlay-dimmed");
    oottracker_derive::embed_image!("assets/icon.ico");
}
