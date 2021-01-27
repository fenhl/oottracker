use {
    std::{
        collections::HashMap,
        fmt,
        io,
        sync::Arc,
        vec,
    },
    directories::ProjectDirs,
    enum_iterator::IntoEnumIterator,
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
    ootr::model::{
        DungeonReward,
        DungeonRewardLocation,
        MainDungeon,
        Medallion,
        Stone,
    },
    crate::{
        ModelState,
        info_tables::*,
        save::*,
    },
};

const VERSION: u8 = 0;

#[derive(Debug, Clone)]
pub enum Error {
    Io(Arc<io::Error>),
    Json(Arc<serde_json::Error>),
    MissingHomeDir,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(Arc::new(e))
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Json(Arc::new(e))
    }
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
    Composite {
        left_img: &'static str,
        right_img: &'static str,
        both_img: &'static str,
        active: Box<dyn Fn(&ModelState) -> (bool, bool)>,
        toggle_left: Box<dyn Fn(&mut ModelState)>,
        toggle_right: Box<dyn Fn(&mut ModelState)>,
    },
    Count {
        dimmed_img: &'static str,
        img: &'static str,
        get: Box<dyn Fn(&ModelState) -> u8>,
        set: Box<dyn Fn(&mut ModelState, u8)>,
        max: u8,
    },
    Medallion(Medallion),
    MedallionLocation(Medallion),
    OptionalOverlay {
        main_img: &'static str,
        overlay_img: &'static str,
        active: Box<dyn Fn(&ModelState) -> (bool, bool)>,
        toggle_main: Box<dyn Fn(&mut ModelState)>,
        toggle_overlay: Box<dyn Fn(&mut ModelState)>,
    },
    Overlay {
        main_img: &'static str,
        overlay_img: &'static str,
        active: Box<dyn Fn(&ModelState) -> (bool, bool)>,
        toggle_main: Box<dyn Fn(&mut ModelState)>,
        toggle_overlay: Box<dyn Fn(&mut ModelState)>,
    },
    Sequence {
        img: Box<dyn Fn(&ModelState) -> (bool, &'static str)>,
        increment: Box<dyn Fn(&mut ModelState)>,
        decrement: Box<dyn Fn(&mut ModelState)>,
    },
    Simple {
        img: &'static str,
        active: Box<dyn Fn(&ModelState) -> bool>,
        toggle: Box<dyn Fn(&mut ModelState)>,
    },
    Song {
        song: QuestItems,
        check: &'static str,
        toggle_overlay: Box<dyn Fn(&mut EventChkInf)>,
    },
    Stone(Stone),
    StoneLocation(Stone),
}

use TrackerCellKind::*;

macro_rules! cells {
    ($($cell:ident: $kind:expr,)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum TrackerCellId {
            $(
                $cell,
            )*
        }

        impl TrackerCellId {
            pub fn kind(&self) -> TrackerCellKind {
                match self {
                    $(TrackerCellId::$cell => $kind,)*
                }
            }
        }
    }
}

cells! {
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
        img: Box::new(|state| match state.ram.save.inv.adult_trade_item {
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
        increment: Box::new(|state| state.ram.save.inv.adult_trade_item = match state.ram.save.inv.adult_trade_item {
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
        decrement: Box::new(|state| state.ram.save.inv.adult_trade_item = match state.ram.save.inv.adult_trade_item {
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
        get: Box::new(|state| state.ram.save.skull_tokens),
        set: Box::new(|state, value| state.ram.save.skull_tokens = value),
        max: 100,
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
        active: Box::new(|state| (state.ram.save.inv.has_emptiable_bottle(), state.ram.save.inv.has_rutos_letter())), //TODO also show Ruto's letter as active if it has been delivered
        toggle_main: Box::new(|state| state.ram.save.inv.toggle_emptiable_bottle()),
        toggle_overlay: Box::new(|state| state.ram.save.inv.toggle_rutos_letter()),
    },
    Scale: Sequence {
        img: Box::new(|state| match state.ram.save.upgrades.scale() {
            Upgrades::SILVER_SCALE => (true, "silver_scale"),
            Upgrades::GOLD_SCALE => (true, "gold_scale"),
            _ => (false, "silver_scale"),
        }),
        increment: Box::new(|state| state.ram.save.upgrades.set_scale(match state.ram.save.upgrades.scale() {
            Upgrades::SILVER_SCALE => Upgrades::GOLD_SCALE,
            Upgrades::GOLD_SCALE => Upgrades::NONE,
            _ => Upgrades::SILVER_SCALE,
        })),
        decrement: Box::new(|state| state.ram.save.upgrades.set_scale(match state.ram.save.upgrades.scale() {
            Upgrades::SILVER_SCALE => Upgrades::NONE,
            Upgrades::GOLD_SCALE => Upgrades::SILVER_SCALE,
            _ => Upgrades::GOLD_SCALE,
        })),
    },
    Slingshot: Simple {
        img: "slingshot",
        active: Box::new(|state| state.ram.save.inv.slingshot),
        toggle: Box::new(|state| state.ram.save.inv.slingshot = !state.ram.save.inv.slingshot),
    },
    Bombs: Overlay {
        main_img: "bomb_bag",
        overlay_img: "bombchu",
        active: Box::new(|state| (state.ram.save.upgrades.bomb_bag() != Upgrades::NONE, state.ram.save.inv.bombchus)),
        toggle_main: Box::new(|state| if state.ram.save.upgrades.bomb_bag() == Upgrades::NONE {
            state.ram.save.upgrades.set_bomb_bag(Upgrades::BOMB_BAG);
        } else {
            state.ram.save.upgrades.set_bomb_bag(Upgrades::NONE)
        }),
        toggle_overlay: Box::new(|state| state.ram.save.inv.bombchus = !state.ram.save.inv.bombchus),
    },
    Boomerang: Simple {
        img: "boomerang",
        active: Box::new(|state| state.ram.save.inv.boomerang),
        toggle: Box::new(|state| state.ram.save.inv.boomerang = !state.ram.save.inv.boomerang),
    },
    Strength: Sequence {
        img: Box::new(|state| match state.ram.save.upgrades.strength() {
            Upgrades::GORON_BRACELET => (true, "goron_bracelet"),
            Upgrades::SILVER_GAUNTLETS => (true, "silver_gauntlets"),
            Upgrades::GOLD_GAUNTLETS => (true, "gold_gauntlets"),
            _ => (false, "goron_bracelet"),
        }),
        increment: Box::new(|state| state.ram.save.upgrades.set_strength(match state.ram.save.upgrades.strength() {
            Upgrades::GORON_BRACELET => Upgrades::SILVER_GAUNTLETS,
            Upgrades::SILVER_GAUNTLETS => Upgrades::GOLD_GAUNTLETS,
            Upgrades::GOLD_GAUNTLETS => Upgrades::NONE,
            _ => Upgrades::GORON_BRACELET,
        })),
        decrement: Box::new(|state| state.ram.save.upgrades.set_strength(match state.ram.save.upgrades.strength() {
            Upgrades::GORON_BRACELET => Upgrades::NONE,
            Upgrades::SILVER_GAUNTLETS => Upgrades::GORON_BRACELET,
            Upgrades::GOLD_GAUNTLETS => Upgrades::SILVER_GAUNTLETS,
            _ => Upgrades::GOLD_GAUNTLETS,
        })),
    },
    Magic: Overlay {
        main_img: "magic",
        overlay_img: "lens",
        active: Box::new(|state| (state.ram.save.magic != MagicCapacity::None, state.ram.save.inv.lens)),
        toggle_main: Box::new(|state| if state.ram.save.magic == MagicCapacity::None {
            state.ram.save.magic = MagicCapacity::Small;
        } else {
            state.ram.save.magic = MagicCapacity::None;
        }),
        toggle_overlay: Box::new(|state| state.ram.save.inv.lens = !state.ram.save.inv.lens),
    },
    Spells: Composite {
        left_img: "dins_fire",
        right_img: "faores_wind",
        both_img: "composite_magic",
        active: Box::new(|state| (state.ram.save.inv.dins_fire, state.ram.save.inv.farores_wind)),
        toggle_left: Box::new(|state| state.ram.save.inv.dins_fire = !state.ram.save.inv.dins_fire),
        toggle_right: Box::new(|state| state.ram.save.inv.farores_wind = !state.ram.save.inv.farores_wind),
    },
    Hookshot: Sequence {
        img: Box::new(|state| match state.ram.save.inv.hookshot {
            Hookshot::None => (false, "hookshot"),
            Hookshot::Hookshot => (true, "hookshot_accessible"),
            Hookshot::Longshot => (true, "longshot_accessible"),
        }),
        increment: Box::new(|state| state.ram.save.inv.hookshot = match state.ram.save.inv.hookshot {
            Hookshot::None => Hookshot::Hookshot,
            Hookshot::Hookshot => Hookshot::Longshot,
            Hookshot::Longshot => Hookshot::None,
        }),
        decrement: Box::new(|state| state.ram.save.inv.hookshot = match state.ram.save.inv.hookshot {
            Hookshot::None => Hookshot::Longshot,
            Hookshot::Hookshot => Hookshot::None,
            Hookshot::Longshot => Hookshot::Hookshot,
        }),
    },
    Bow: OptionalOverlay {
        main_img: "bow",
        overlay_img: "ice_arrows",
        active: Box::new(|state| (state.ram.save.inv.bow, state.ram.save.inv.ice_arrows)),
        toggle_main: Box::new(|state| state.ram.save.inv.bow = !state.ram.save.inv.bow),
        toggle_overlay: Box::new(|state| state.ram.save.inv.ice_arrows = !state.ram.save.inv.ice_arrows),
    },
    Arrows: Composite {
        left_img: "fire_arrows",
        right_img: "light_arrows",
        both_img: "composite_arrows",
        active: Box::new(|state| (state.ram.save.inv.fire_arrows, state.ram.save.inv.light_arrows)),
        toggle_left: Box::new(|state| state.ram.save.inv.fire_arrows = !state.ram.save.inv.fire_arrows),
        toggle_right: Box::new(|state| state.ram.save.inv.light_arrows = !state.ram.save.inv.light_arrows),
    },
    Hammer: Simple {
        img: "hammer",
        active: Box::new(|state| state.ram.save.inv.hammer),
        toggle: Box::new(|state| state.ram.save.inv.hammer = !state.ram.save.inv.hammer),
    },
    Boots: Composite {
        left_img: "iron_boots",
        right_img: "hover_boots",
        both_img: "composite_boots",
        active: Box::new(|state| (state.ram.save.equipment.contains(Equipment::IRON_BOOTS), state.ram.save.equipment.contains(Equipment::HOVER_BOOTS))),
        toggle_left: Box::new(|state| state.ram.save.equipment.toggle(Equipment::IRON_BOOTS)),
        toggle_right: Box::new(|state| state.ram.save.equipment.toggle(Equipment::HOVER_BOOTS)),
    },
    MirrorShield: Simple {
        img: "mirror_shield",
        active: Box::new(|state| state.ram.save.equipment.contains(Equipment::MIRROR_SHIELD)),
        toggle: Box::new(|state| state.ram.save.equipment.toggle(Equipment::MIRROR_SHIELD)),
    },
    ChildTrade: Sequence {
        img: Box::new(|state| match state.ram.save.inv.child_trade_item {
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
        increment: Box::new(|state| state.ram.save.inv.child_trade_item = match state.ram.save.inv.child_trade_item {
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
        decrement: Box::new(|state| state.ram.save.inv.child_trade_item = match state.ram.save.inv.child_trade_item {
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
        active: Box::new(|state| (state.ram.save.inv.ocarina, state.ram.save.event_chk_inf.9.contains(EventChkInf9::SCARECROW_SONG))), //TODO only show free Scarecrow's Song once it's known (by settings string input or by check)
        toggle_main: Box::new(|state| state.ram.save.inv.ocarina = !state.ram.save.inv.ocarina),
        toggle_overlay: Box::new(|state| state.ram.save.event_chk_inf.9.toggle(EventChkInf9::SCARECROW_SONG)), //TODO make sure free scarecrow knowledge is toggled off properly
    },
    Beans: Simple { //TODO overlay with number bought if autotracker is on & shuffle beans is off
        img: "beans",
        active: Box::new(|state| state.ram.save.inv.beans),
        toggle: Box::new(|state| state.ram.save.inv.beans = !state.ram.save.inv.beans),
    },
    SwordCard: Composite {
        left_img: "kokiri_sword",
        right_img: "gerudo_card",
        both_img: "composite_ksword_gcard",
        active: Box::new(|state| (state.ram.save.equipment.contains(Equipment::KOKIRI_SWORD), state.ram.save.quest_items.contains(QuestItems::GERUDO_CARD))),
        toggle_left: Box::new(|state| state.ram.save.equipment.toggle(Equipment::KOKIRI_SWORD)),
        toggle_right: Box::new(|state| state.ram.save.quest_items.toggle(QuestItems::GERUDO_CARD)),
    },
    Tunics: Composite {
        left_img: "goron_tunic",
        right_img: "zora_tunic",
        both_img: "composite_tunics",
        active: Box::new(|state| (state.ram.save.equipment.contains(Equipment::GORON_TUNIC), state.ram.save.equipment.contains(Equipment::ZORA_TUNIC))),
        toggle_left: Box::new(|state| state.ram.save.equipment.toggle(Equipment::GORON_TUNIC)),
        toggle_right: Box::new(|state| state.ram.save.equipment.toggle(Equipment::ZORA_TUNIC)),
    },
    Triforce: Count { //TODO if triforce hunt is off and autotracker is on, replace with something else (big poes?)
        dimmed_img: "triforce",
        img: "force",
        get: Box::new(|state| state.ram.save.triforce_pieces()),
        set: Box::new(|state, value| state.ram.save.set_triforce_pieces(value)),
        max: 100,
    },
    ZeldasLullaby: Song {
        song: QuestItems::ZELDAS_LULLABY,
        check: "Song from Impa",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_IMPA)),
    },
    EponasSong: Song {
        song: QuestItems::EPONAS_SONG,
        check: "Song from Malon",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_MALON)),
    },
    SariasSong: Song {
        song: QuestItems::SARIAS_SONG,
        check: "Song from Saria",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_SARIA)),
    },
    SunsSong: Song {
        song: QuestItems::SUNS_SONG,
        check: "Song from Composers Grave",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_COMPOSERS_GRAVE)),
    },
    SongOfTime: Song {
        song: QuestItems::SONG_OF_TIME,
        check: "Song from Ocarina of Time",
        toggle_overlay: Box::new(|eci| eci.10.toggle(EventChkInf10::SONG_FROM_OCARINA_OF_TIME)),
    },
    SongOfStorms: Song {
        song: QuestItems::SONG_OF_STORMS,
        check: "Song from Windmill",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_WINDMILL)),
    },
    Minuet: Song {
        song: QuestItems::MINUET_OF_FOREST,
        check: "Sheik in Forest",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_FOREST)),
    },
    Bolero: Song {
        song: QuestItems::BOLERO_OF_FIRE,
        check: "Sheik in Crater",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_CRATER)),
    },
    Serenade: Song {
        song: QuestItems::SERENADE_OF_WATER,
        check: "Sheik in Ice Cavern",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_ICE_CAVERN)),
    },
    Requiem: Song {
        song: QuestItems::REQUIEM_OF_SPIRIT,
        check: "Sheik at Colossus",
        toggle_overlay: Box::new(|eci| eci.10.toggle(EventChkInf10::SHEIK_AT_COLOSSUS)),
    },
    Nocturne: Song {
        song: QuestItems::NOCTURNE_OF_SHADOW,
        check: "Sheik in Kakariko",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_KAKARIKO)),
    },
    Prelude: Song {
        song: QuestItems::PRELUDE_OF_LIGHT,
        check: "Sheik at Temple",
        toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_AT_TEMPLE)),
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

impl Default for TrackerLayout {
    fn default() -> TrackerLayout { TrackerLayout::from(&Config::default()) }
}

impl<'a> From<&'a Config> for TrackerLayout {
    fn from(config: &Config) -> TrackerLayout {
        use self::TrackerCellId::*;

        TrackerLayout {
            meds: config.med_order,
            row2: [AdultTrade, Skulltula, Bottle, Scale],
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

fn dirs() -> Result<ProjectDirs, Error> {
    ProjectDirs::from("net", "Fenhl", "OoT Tracker").ok_or(Error::MissingHomeDir)
}
