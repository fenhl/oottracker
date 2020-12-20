use {
    std::{
        convert::{
            TryFrom,
            TryInto as _,
        },
        fmt,
        io,
        num::TryFromIntError,
        ops::{
            Add,
            Sub,
        },
        sync::Arc,
    },
    bitflags::bitflags,
    byteorder::{
        BigEndian,
        ByteOrder as _,
    },
    derive_more::From,
    smart_default::SmartDefault,
    crate::{
        info_tables::{
            EventChkInf,
            InfTable,
            ItemGetInf,
        },
        item_ids,
        scene::SceneFlags,
    },
};
#[cfg(not(target_arch = "wasm32"))] use {
    std::io::prelude::*,
    async_trait::async_trait,
    tokio::{
        net::TcpStream,
        prelude::*,
    },
    crate::{
        info_tables::EventChkInf3,
        Item,
        proto::Protocol,
    },
};

pub const SIZE: usize = 0x1450;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TimeOfDay(u16);

#[cfg(not(target_arch = "wasm32"))]
impl TimeOfDay {
    pub fn matches(&self, range: crate::access::TimeRange) -> bool {
        match range {
            crate::access::TimeRange::Day => (0x4555..0xc001).contains(&self.0),
            crate::access::TimeRange::Night => (0x0000..0x4555).contains(&self.0) || (0xc001..=0xffff).contains(&self.0),
            crate::access::TimeRange::Dampe => (0xc001..0xe000).contains(&self.0),
        }
    }
}

impl TryFrom<Vec<u8>> for TimeOfDay {
    type Error = Vec<u8>;

    fn try_from(raw_data: Vec<u8>) -> Result<TimeOfDay, Vec<u8>> {
        if raw_data.len() != 2 { return Err(raw_data) }
        Ok(TimeOfDay(BigEndian::read_u16(&raw_data)))
    }
}

impl<'a> From<&'a TimeOfDay> for [u8; 2] {
    fn from(TimeOfDay(repr): &TimeOfDay) -> [u8; 2] {
        repr.to_be_bytes()
    }
}

impl<'a> From<&'a TimeOfDay> for Vec<u8> {
    fn from(time: &TimeOfDay) -> Vec<u8> {
        <[u8; 2]>::from(time).into()
    }
}

#[derive(Debug, SmartDefault, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MagicCapacity {
    #[default]
    None = 0,
    Small = 1,
    Large = 2,
}

impl TryFrom<u8> for MagicCapacity {
    type Error = u8;

    fn try_from(raw_data: u8) -> Result<MagicCapacity, u8> {
        match raw_data {
            0 => Ok(MagicCapacity::None),
            1 => Ok(MagicCapacity::Small),
            2 => Ok(MagicCapacity::Large),
            _ => Err(raw_data),
        }
    }
}

impl<'a> From<&'a MagicCapacity> for u8 {
    fn from(magic: &MagicCapacity) -> u8 {
        match magic {
            MagicCapacity::None => 0,
            MagicCapacity::Small => 1,
            MagicCapacity::Large => 2,
        }
    }
}

#[derive(Debug, SmartDefault, Clone, Copy, PartialEq, Eq)]
pub enum Hookshot {
    #[default]
    None,
    Hookshot,
    Longshot,
}

impl TryFrom<u8> for Hookshot {
    type Error = u8;

    fn try_from(raw_data: u8) -> Result<Hookshot, u8> {
        match raw_data {
            item_ids::NONE => Ok(Hookshot::None),
            item_ids::HOOKSHOT => Ok(Hookshot::Hookshot),
            item_ids::LONGSHOT => Ok(Hookshot::Longshot),
            _ => Err(raw_data),
        }
    }
}

impl From<Hookshot> for u8 {
    fn from(hookshot: Hookshot) -> u8 {
        match hookshot {
            Hookshot::None => item_ids::NONE,
            Hookshot::Hookshot => item_ids::HOOKSHOT,
            Hookshot::Longshot => item_ids::LONGSHOT,
        }
    }
}

#[derive(Debug, SmartDefault, Clone, Copy, PartialEq, Eq)]
pub enum Bottle {
    #[default]
    None,
    Empty,
    RedPotion,
    GreenPotion,
    BluePotion,
    Fairy,
    Fish,
    MilkFull,
    RutosLetter,
    BlueFire,
    Bug,
    BigPoe,
    MilkHalf,
    Poe,
}

impl Bottle {
    fn emptiable(&self) -> bool {
        !matches!(self, Bottle::None | Bottle::RutosLetter | Bottle::BigPoe)
    }
}

impl TryFrom<u8> for Bottle {
    type Error = u8;

    fn try_from(raw_data: u8) -> Result<Bottle, u8> {
        match raw_data {
            item_ids::NONE => Ok(Bottle::None),
            item_ids::EMPTY_BOTTLE => Ok(Bottle::Empty),
            item_ids::RED_POTION => Ok(Bottle::RedPotion),
            item_ids::GREEN_POTION => Ok(Bottle::GreenPotion),
            item_ids::BLUE_POTION => Ok(Bottle::BluePotion),
            item_ids::BOTTLED_FAIRY => Ok(Bottle::Fairy),
            item_ids::FISH => Ok(Bottle::Fish),
            item_ids::LON_LON_MILK_FULL => Ok(Bottle::MilkFull),
            item_ids::RUTOS_LETTER => Ok(Bottle::RutosLetter),
            item_ids::BLUE_FIRE => Ok(Bottle::BlueFire),
            item_ids::BUG => Ok(Bottle::Bug),
            item_ids::BIG_POE => Ok(Bottle::BigPoe),
            item_ids::LON_LON_MILK_HALF => Ok(Bottle::MilkHalf),
            item_ids::POE => Ok(Bottle::Poe),
            _ => Err(raw_data),
        }
    }
}

impl From<Bottle> for u8 {
    fn from(bottle: Bottle) -> u8 {
        match bottle {
            Bottle::None => item_ids::NONE,
            Bottle::Empty => item_ids::EMPTY_BOTTLE,
            Bottle::RedPotion => item_ids::RED_POTION,
            Bottle::GreenPotion => item_ids::GREEN_POTION,
            Bottle::BluePotion => item_ids::BLUE_POTION,
            Bottle::Fairy => item_ids::BOTTLED_FAIRY,
            Bottle::Fish => item_ids::FISH,
            Bottle::MilkFull => item_ids::LON_LON_MILK_FULL,
            Bottle::RutosLetter => item_ids::RUTOS_LETTER,
            Bottle::BlueFire => item_ids::BLUE_FIRE,
            Bottle::Bug => item_ids::BUG,
            Bottle::BigPoe => item_ids::BIG_POE,
            Bottle::MilkHalf => item_ids::LON_LON_MILK_HALF,
            Bottle::Poe => item_ids::POE,
        }
    }
}

#[derive(Debug, SmartDefault, Clone, Copy, PartialEq, Eq)]
pub enum AdultTradeItem {
    #[default]
    None,
    PocketEgg,
    PocketCucco,
    Cojiro,
    OddMushroom,
    OddPotion,
    PoachersSaw,
    BrokenSword,
    Prescription,
    EyeballFrog,
    Eyedrops,
    ClaimCheck,
}

impl TryFrom<u8> for AdultTradeItem {
    type Error = u8;

    fn try_from(raw_data: u8) -> Result<AdultTradeItem, u8> {
        match raw_data {
            item_ids::NONE => Ok(AdultTradeItem::None),
            item_ids::POCKET_EGG => Ok(AdultTradeItem::PocketEgg),
            item_ids::POCKET_CUCCO => Ok(AdultTradeItem::PocketCucco),
            item_ids::COJIRO => Ok(AdultTradeItem::Cojiro),
            item_ids::ODD_POTION => Ok(AdultTradeItem::OddPotion),
            item_ids::ODD_MUSHROOM => Ok(AdultTradeItem::OddMushroom),
            item_ids::POACHERS_SAW => Ok(AdultTradeItem::PoachersSaw),
            item_ids::GORONS_SWORD_BROKEN => Ok(AdultTradeItem::BrokenSword),
            item_ids::PRESCRIPTION => Ok(AdultTradeItem::Prescription),
            item_ids::EYEBALL_FROG => Ok(AdultTradeItem::EyeballFrog),
            item_ids::EYEDROPS => Ok(AdultTradeItem::Eyedrops),
            item_ids::CLAIM_CHECK => Ok(AdultTradeItem::ClaimCheck),
            _ => Err(raw_data),
        }
    }
}

impl From<AdultTradeItem> for u8 {
    fn from(trade_item: AdultTradeItem) -> u8 {
        match trade_item {
            AdultTradeItem::None => item_ids::NONE,
            AdultTradeItem::PocketEgg => item_ids::POCKET_EGG,
            AdultTradeItem::PocketCucco => item_ids::POCKET_CUCCO,
            AdultTradeItem::Cojiro => item_ids::COJIRO,
            AdultTradeItem::OddPotion => item_ids::ODD_POTION,
            AdultTradeItem::OddMushroom => item_ids::ODD_MUSHROOM,
            AdultTradeItem::PoachersSaw => item_ids::POACHERS_SAW,
            AdultTradeItem::BrokenSword => item_ids::GORONS_SWORD_BROKEN,
            AdultTradeItem::Prescription => item_ids::PRESCRIPTION,
            AdultTradeItem::EyeballFrog => item_ids::EYEBALL_FROG,
            AdultTradeItem::Eyedrops => item_ids::EYEDROPS,
            AdultTradeItem::ClaimCheck => item_ids::CLAIM_CHECK,
        }
    }
}

#[derive(Debug, SmartDefault, Clone, Copy, PartialEq, Eq)]
pub enum ChildTradeItem {
    #[default]
    None,
    WeirdEgg,
    Chicken,
    ZeldasLetter,
    KeatonMask,
    SkullMask,
    SpookyMask,
    BunnyHood,
    GoronMask,
    ZoraMask,
    GerudoMask,
    MaskOfTruth,
    SoldOut,
}

impl TryFrom<u8> for ChildTradeItem {
    type Error = u8;

    fn try_from(raw_data: u8) -> Result<ChildTradeItem, u8> {
        match raw_data {
            item_ids::NONE => Ok(ChildTradeItem::None),
            item_ids::WEIRD_EGG => Ok(ChildTradeItem::WeirdEgg),
            item_ids::CHICKEN => Ok(ChildTradeItem::Chicken),
            item_ids::ZELDAS_LETTER => Ok(ChildTradeItem::ZeldasLetter),
            item_ids::KEATON_MASK => Ok(ChildTradeItem::KeatonMask),
            item_ids::SKULL_MASK => Ok(ChildTradeItem::SkullMask),
            item_ids::SPOOKY_MASK => Ok(ChildTradeItem::SpookyMask),
            item_ids::BUNNY_HOOD => Ok(ChildTradeItem::BunnyHood),
            item_ids::GORON_MASK => Ok(ChildTradeItem::GoronMask),
            item_ids::ZORA_MASK => Ok(ChildTradeItem::ZoraMask),
            item_ids::GERUDO_MASK => Ok(ChildTradeItem::GerudoMask),
            item_ids::MASK_OF_TRUTH => Ok(ChildTradeItem::MaskOfTruth),
            item_ids::SOLD_OUT => Ok(ChildTradeItem::SoldOut),
            _ => Err(raw_data),
        }
    }
}

impl From<ChildTradeItem> for u8 {
    fn from(trade_item: ChildTradeItem) -> u8 {
        match trade_item {
            ChildTradeItem::None => item_ids::NONE,
            ChildTradeItem::WeirdEgg => item_ids::WEIRD_EGG,
            ChildTradeItem::Chicken => item_ids::CHICKEN,
            ChildTradeItem::ZeldasLetter => item_ids::ZELDAS_LETTER,
            ChildTradeItem::KeatonMask => item_ids::KEATON_MASK,
            ChildTradeItem::SkullMask => item_ids::SKULL_MASK,
            ChildTradeItem::SpookyMask => item_ids::SPOOKY_MASK,
            ChildTradeItem::BunnyHood => item_ids::BUNNY_HOOD,
            ChildTradeItem::GoronMask => item_ids::GORON_MASK,
            ChildTradeItem::ZoraMask => item_ids::ZORA_MASK,
            ChildTradeItem::GerudoMask => item_ids::GERUDO_MASK,
            ChildTradeItem::MaskOfTruth => item_ids::MASK_OF_TRUTH,
            ChildTradeItem::SoldOut => item_ids::SOLD_OUT,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Inventory {
    pub bow: bool,
    pub fire_arrows: bool,
    pub dins_fire: bool,
    pub slingshot: bool,
    pub ocarina: bool,
    pub bombchus: bool,
    pub hookshot: Hookshot,
    pub ice_arrows: bool,
    pub farores_wind: bool,
    pub boomerang: bool,
    pub lens: bool,
    pub beans: bool,
    pub hammer: bool,
    pub light_arrows: bool,
    pub nayrus_love: bool,
    pub bottles: [Bottle; 4],
    pub adult_trade_item: AdultTradeItem,
    pub child_trade_item: ChildTradeItem,
}

impl Inventory {
    pub fn has_emptiable_bottle(&self) -> bool {
        self.bottles.iter().any(Bottle::emptiable)
    }

    pub fn has_rutos_letter(&self) -> bool {
        self.bottles.iter().any(|bottle| *bottle == Bottle::RutosLetter)
    }

    pub fn toggle_emptiable_bottle(&mut self) {
        if self.has_emptiable_bottle() {
            self.bottles.iter_mut().for_each(|bottle| if bottle.emptiable() { *bottle = Bottle::None });
        } else {
            // First, simply try to add an empty bottle.
            for bottle in &mut self.bottles {
                if *bottle == Bottle::None {
                    *bottle = Bottle::Empty;
                    return
                }
            }
            // All 4 bottles have either Ruto's letter or a big poe. Replace one of the 3+ big poes with an empty bottle.
            for bottle in &mut self.bottles {
                if *bottle == Bottle::BigPoe {
                    *bottle = Bottle::Empty;
                    return
                }
            }
        }
    }

    pub fn toggle_rutos_letter(&mut self) {
        if self.has_rutos_letter() {
            self.bottles.iter_mut().for_each(|bottle| if *bottle == Bottle::RutosLetter { *bottle = Bottle::None });
        } else {
            // First, try to put the letter into a new bottle.
            for bottle in &mut self.bottles {
                if *bottle == Bottle::None {
                    *bottle = Bottle::RutosLetter;
                    return
                }
            }
            // All 4 bottles obtained, empty one and put Ruto's letter in it.
            for bottle in &mut self.bottles {
                if bottle.emptiable() {
                    *bottle = Bottle::RutosLetter;
                    return
                }
            }
            // All 4 bottles have big poes in them. Replace one of them with Ruto's letter.
            self.bottles[0] = Bottle::RutosLetter;
        }
    }
}

impl TryFrom<Vec<u8>> for Inventory {
    type Error = Vec<u8>;

    fn try_from(raw_data: Vec<u8>) -> Result<Inventory, Vec<u8>> {
        macro_rules! bool_item {
            ($offset:literal, $($value:pat)|+) => {{
                match *raw_data.get($offset).ok_or_else(|| raw_data.clone())? {
                    item_ids::NONE => false,
                    $($value)|+ => true,
                    _ => return Err(raw_data),
                }
            }};
        }

        if raw_data.len() != 0x18 { return Err(raw_data) }
        Ok(Inventory {
            bow: bool_item!(0x03, item_ids::BOW),
            fire_arrows: bool_item!(0x04, item_ids::FIRE_ARROWS),
            dins_fire: bool_item!(0x05, item_ids::DINS_FIRE),
            slingshot: bool_item!(0x06, item_ids::SLINGSHOT),
            ocarina: bool_item!(0x07, item_ids::FAIRY_OCARINA | item_ids::OCARINA_OF_TIME),
            bombchus: bool_item!(0x08, item_ids::BOMBCHU_10),
            hookshot: Hookshot::try_from(raw_data[0x09]).map_err(|_| raw_data.clone())?,
            ice_arrows: bool_item!(0x0a, item_ids::ICE_ARROWS),
            farores_wind: bool_item!(0x0b, item_ids::FARORES_WIND),
            boomerang: bool_item!(0x0c, item_ids::BOOMERANG),
            lens: bool_item!(0x0d, item_ids::LENS_OF_TRUTH),
            beans: bool_item!(0x0e, item_ids::MAGIC_BEAN),
            hammer: bool_item!(0x0f, item_ids::MEGATON_HAMMER),
            light_arrows: bool_item!(0x10, item_ids::LIGHT_ARROWS),
            nayrus_love: bool_item!(0x11, item_ids::NAYRUS_LOVE),
            bottles: [
                Bottle::try_from(raw_data[0x12]).map_err(|_| raw_data.clone())?,
                Bottle::try_from(raw_data[0x13]).map_err(|_| raw_data.clone())?,
                Bottle::try_from(raw_data[0x14]).map_err(|_| raw_data.clone())?,
                Bottle::try_from(raw_data[0x15]).map_err(|_| raw_data.clone())?,
            ],
            adult_trade_item: AdultTradeItem::try_from(raw_data[0x16]).map_err(|_| raw_data.clone())?,
            child_trade_item: ChildTradeItem::try_from(raw_data[0x17]).map_err(|_| raw_data)?,
        })
    }
}

impl<'a> From<&'a Inventory> for [u8; 0x18] {
    fn from(inv: &Inventory) -> [u8; 0x18] {
        macro_rules! bool_item {
            ($name:ident, $value:expr) => {{
                if inv.$name { $value } else { item_ids::NONE }
            }};
        }

        [
            item_ids::NONE, item_ids::NONE, item_ids::NONE, bool_item!(bow, item_ids::BOW), bool_item!(fire_arrows, item_ids::FIRE_ARROWS), bool_item!(dins_fire, item_ids::DINS_FIRE),
            bool_item!(slingshot, item_ids::SLINGSHOT), bool_item!(ocarina, item_ids::FAIRY_OCARINA), bool_item!(bombchus, item_ids::BOMBCHU_10), inv.hookshot.into(), bool_item!(ice_arrows, item_ids::ICE_ARROWS), bool_item!(farores_wind, item_ids::FARORES_WIND),
            bool_item!(boomerang, item_ids::BOOMERANG), bool_item!(lens, item_ids::LENS_OF_TRUTH), bool_item!(beans, item_ids::MAGIC_BEAN), bool_item!(hammer, item_ids::MEGATON_HAMMER), bool_item!(light_arrows, item_ids::LIGHT_ARROWS), bool_item!(nayrus_love, item_ids::NAYRUS_LOVE),
            inv.bottles[0].into(), inv.bottles[1].into(), inv.bottles[2].into(), inv.bottles[3].into(), inv.adult_trade_item.into(), inv.child_trade_item.into(),
        ]
    }
}

impl<'a> From<&'a Inventory> for Vec<u8> {
    fn from(inv: &Inventory) -> Vec<u8> {
        <[u8; 0x18]>::from(inv).into()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InvAmounts {
    pub deku_sticks: u8,
    pub deku_nuts: u8,
    pub bombchus: u8,
}

impl TryFrom<Vec<u8>> for InvAmounts {
    type Error = Vec<u8>;

    fn try_from(raw_data: Vec<u8>) -> Result<InvAmounts, Vec<u8>> {
        if raw_data.len() != 0xf { return Err(raw_data) }
        Ok(InvAmounts {
            deku_sticks: *raw_data.get(0x00).ok_or_else(|| raw_data.clone())?,
            deku_nuts: *raw_data.get(0x01).ok_or_else(|| raw_data.clone())?,
            bombchus: *raw_data.get(0x08).ok_or_else(|| raw_data.clone())?,
        })
    }
}

impl<'a> From<&'a InvAmounts> for [u8; 0xf] {
    fn from(inv_amounts: &InvAmounts) -> [u8; 0xf] {
        [
            inv_amounts.deku_sticks, inv_amounts.deku_nuts, 0, 0, 0, 0,
            0, 0, inv_amounts.bombchus, 0, 0, 0,
            0, 0, 0,
        ]
    }
}

impl<'a> From<&'a InvAmounts> for Vec<u8> {
    fn from(inv_amounts: &InvAmounts) -> Vec<u8> {
        <[u8; 0xf]>::from(inv_amounts).into()
    }
}

bitflags! {
    #[derive(Default)]
    pub struct Equipment: u16 {
        const HOVER_BOOTS = 0x4000;
        const IRON_BOOTS = 0x2000;
        const ZORA_TUNIC = 0x0400;
        const GORON_TUNIC = 0x0200;
        const MIRROR_SHIELD = 0x0040;
        const HYLIAN_SHIELD = 0x0020;
        const DEKU_SHIELD = 0x0010;
        const GIANTS_KNIFE_BROKEN = 0x0008;
        const GIANTS_KNIFE = 0x0004;
        const MASTER_SWORD = 0x0002;
        const KOKIRI_SWORD = 0x0001;
    }
}

impl TryFrom<Vec<u8>> for Equipment {
    type Error = Vec<u8>;

    fn try_from(raw_data: Vec<u8>) -> Result<Equipment, Vec<u8>> {
        if raw_data.len() != 2 { return Err(raw_data) }
        Ok(Equipment::from_bits_truncate(BigEndian::read_u16(&raw_data)))
    }
}

impl<'a> From<&'a Equipment> for [u8; 2] {
    fn from(equipment: &Equipment) -> [u8; 2] {
        (equipment.bits() as u16).to_be_bytes()
    }
}

impl<'a> From<&'a Equipment> for Vec<u8> {
    fn from(equipment: &Equipment) -> Vec<u8> {
        <[u8; 2]>::from(equipment).into()
    }
}

bitflags! {
    #[derive(Default)]
    pub struct Upgrades: u32 {
        //TODO bullet bag for parity with slingshot
        const SCALE_MASK = 0x0000_0e00;
        const GOLD_SCALE = 0x0000_0400;
        const SILVER_SCALE = 0x0000_0200;
        const STRENGTH_MASK = 0x0000_01c0;
        const GOLD_GAUNTLETS = 0x0000_000c0;
        const SILVER_GAUNTLETS = 0x0000_0080;
        const GORON_BRACELET = 0x0000_0040;
        const BOMB_BAG_MASK = 0x0000_0038;
        const BIGGEST_BOMB_BAG = 0x0000_0018;
        const BIG_BOMB_BAG = 0x0000_0010;
        const BOMB_BAG = 0x0000_0008;
        //TODO quiver for parity with bow
        const NONE = 0x0000_0000;
    }
}

impl Upgrades {
    pub fn scale(&self) -> Upgrades { *self & Upgrades::SCALE_MASK }

    pub fn set_scale(&mut self, scale: Upgrades) {
        self.remove(Upgrades::SCALE_MASK);
        self.insert(scale & Upgrades::SCALE_MASK);
    }

    pub fn strength(&self) -> Upgrades { *self & Upgrades::STRENGTH_MASK }

    pub fn set_strength(&mut self, strength: Upgrades) {
        self.remove(Upgrades::STRENGTH_MASK);
        self.insert(strength & Upgrades::STRENGTH_MASK);
    }

    pub fn bomb_bag(&self) -> Upgrades { *self & Upgrades::BOMB_BAG_MASK }

    pub fn set_bomb_bag(&mut self, bomb_bag: Upgrades) {
        self.remove(Upgrades::BOMB_BAG_MASK);
        self.insert(bomb_bag & Upgrades::BOMB_BAG_MASK);
    }
}

impl TryFrom<Vec<u8>> for Upgrades {
    type Error = Vec<u8>;

    fn try_from(raw_data: Vec<u8>) -> Result<Upgrades, Vec<u8>> {
        if raw_data.len() != 4 { return Err(raw_data) }
        Ok(Upgrades::from_bits_truncate(BigEndian::read_u32(&raw_data)))
    }
}

impl<'a> From<&'a Upgrades> for [u8; 4] {
    fn from(upgrades: &Upgrades) -> [u8; 4] {
        upgrades.bits().to_be_bytes()
    }
}

impl<'a> From<&'a Upgrades> for Vec<u8> {
    fn from(upgrades: &Upgrades) -> Vec<u8> {
        <[u8; 4]>::from(upgrades).into()
    }
}

bitflags! {
    #[derive(Default)]
    pub struct QuestItems: u32 {
        const GERUDO_CARD = 0x0040_0000;
        const ZORA_SAPPHIRE = 0x0010_0000;
        const GORON_RUBY = 0x0008_0000;
        const KOKIRI_EMERALD = 0x0004_0000;
        const SONG_OF_STORMS = 0x0002_0000;
        const SONG_OF_TIME = 0x0001_0000;
        const SUNS_SONG = 0x0000_8000;
        const SARIAS_SONG = 0x0000_4000;
        const EPONAS_SONG = 0x0000_2000;
        const ZELDAS_LULLABY = 0x0000_1000;
        const PRELUDE_OF_LIGHT = 0x0000_0800;
        const NOCTURNE_OF_SHADOW = 0x0000_0400;
        const REQUIEM_OF_SPIRIT = 0x0000_0200;
        const SERENADE_OF_WATER = 0x0000_0100;
        const BOLERO_OF_FIRE = 0x0000_0080;
        const MINUET_OF_FOREST = 0x0000_0040;
        const LIGHT_MEDALLION = 0x0000_0020;
        const SHADOW_MEDALLION = 0x0000_0010;
        const SPIRIT_MEDALLION = 0x0000_0008;
        const WATER_MEDALLION = 0x0000_0004;
        const FIRE_MEDALLION = 0x0000_0002;
        const FOREST_MEDALLION = 0x0000_0001;
    }
}

impl QuestItems {
    pub fn num_stones(&self) -> u8 {
        (if self.contains(QuestItems::KOKIRI_EMERALD) { 1 } else { 0 })
        + if self.contains(QuestItems::GORON_RUBY) { 1 } else { 0 }
        + if self.contains(QuestItems::ZORA_SAPPHIRE) { 1 } else { 0 }
    }
}

impl TryFrom<Vec<u8>> for QuestItems {
    type Error = Vec<u8>;

    fn try_from(raw_data: Vec<u8>) -> Result<QuestItems, Vec<u8>> {
        if raw_data.len() != 4 { return Err(raw_data) }
        Ok(QuestItems::from_bits_truncate(BigEndian::read_u32(&raw_data)))
    }
}

impl<'a> From<&'a QuestItems> for [u8; 4] {
    fn from(quest_items: &QuestItems) -> [u8; 4] {
        quest_items.bits().to_be_bytes()
    }
}

impl<'a> From<&'a QuestItems> for Vec<u8> {
    fn from(quest_items: &QuestItems) -> Vec<u8> {
        <[u8; 4]>::from(quest_items).into()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SmallKeys {
    pub forest_temple: u8,
    pub fire_temple: u8,
    pub water_temple: u8,
    pub spirit_temple: u8,
    pub shadow_temple: u8,
    pub bottom_of_the_well: u8,
    pub gerudo_training_grounds: u8,
    pub thieves_hideout: u8,
    pub ganons_castle: u8,
}

impl TryFrom<Vec<u8>> for SmallKeys {
    type Error = Vec<u8>;

    fn try_from(raw_data: Vec<u8>) -> Result<SmallKeys, Vec<u8>> {
        macro_rules! get {
            ($idx:expr) => {{
                if raw_data[$idx] == 0xff { 0 } else { raw_data[$idx] }
            }};
        }

        if raw_data.len() != 0x13 { return Err(raw_data) }
        Ok(SmallKeys {
            forest_temple: get!(0x03),
            fire_temple: get!(0x04),
            water_temple: get!(0x05),
            spirit_temple: get!(0x06),
            shadow_temple: get!(0x07),
            bottom_of_the_well: get!(0x08),
            gerudo_training_grounds: get!(0x0b),
            thieves_hideout: get!(0x0c),
            ganons_castle: get!(0x0d),
        })
    }
}

impl<'a> From<&'a SmallKeys> for [u8; 0x13] {
    fn from(small_keys: &SmallKeys) -> [u8; 0x13] {
        [
            0, 0, 0, small_keys.forest_temple,
            small_keys.fire_temple, small_keys.water_temple, small_keys.spirit_temple, small_keys.shadow_temple,
            small_keys.bottom_of_the_well, 0, 0, small_keys.gerudo_training_grounds,
            small_keys.thieves_hideout, small_keys.ganons_castle, 0, 0,
            0, 0, 0,
        ]
    }
}

impl<'a> From<&'a SmallKeys> for Vec<u8> {
    fn from(small_keys: &SmallKeys) -> Vec<u8> {
        <[u8; 0x13]>::from(small_keys).into()
    }
}

#[derive(Debug, From, Clone)]
pub enum DecodeError {
    AssertEq {
        offset: u16,
        expected: u8,
        found: u8,
    },
    AssertEqRange {
        start: u16,
        end: u16,
        expected: Vec<u8>,
        found: Vec<u8>,
    },
    Index(u16),
    IndexRange {
        start: u16,
        end: u16,
    },
    Size(usize),
    UnexpectedValue {
        offset: u16,
        field: &'static str,
        value: u8,
    },
    UnexpectedValueRange {
        start: u16,
        end: u16,
        field: &'static str,
        value: Vec<u8>,
    },
    #[from]
    TryFromInt(TryFromIntError),
}

/// The state of a playthrough.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Save {
    pub time_of_day: TimeOfDay,
    pub is_adult: bool,
    pub magic: MagicCapacity,
    pub inv: Inventory,
    pub inv_amounts: InvAmounts,
    pub equipment: Equipment,
    pub upgrades: Upgrades,
    pub quest_items: QuestItems,
    pub small_keys: SmallKeys,
    pub skull_tokens: u8,
    pub scene_flags: SceneFlags,
    pub event_chk_inf: EventChkInf,
    pub item_get_inf: ItemGetInf,
    pub inf_table: InfTable,
}

impl Save {
    /// Converts *Ocarina of Time* save data into a `Save`.
    ///
    /// # Panics
    ///
    /// This method may panic if `save_data`'s size is less than `0x1450` bytes, or if it doesn't contain valid OoT save data.
    pub fn from_save_data(save_data: &[u8]) -> Result<Save, DecodeError> {
        macro_rules! get_offset {
            ($name:expr, $offset:expr) => {{
                *save_data.get($offset).ok_or(DecodeError::Index($offset))?
            }};
            ($name:expr, $offset:expr, $len:expr) => {{
                save_data.get($offset..$offset + $len).ok_or(DecodeError::IndexRange { start: $offset, end: $offset + $len })?
            }};
        }

        macro_rules! try_get_offset {
            ($name:expr, $offset:expr) => {{
                let raw = *save_data.get($offset).ok_or(DecodeError::Index($offset))?;
                raw.try_into().map_err(|value| DecodeError::UnexpectedValue { value, offset: $offset, field: $name })?
            }};
            ($name:expr, $offset:expr, $len:expr) => {{
                let raw = save_data.get($offset..$offset + $len).ok_or(DecodeError::IndexRange { start: $offset, end: $offset + $len })?.to_vec();
                raw.try_into().map_err(|value| DecodeError::UnexpectedValueRange { value, start: $offset, end: $offset + $len, field: $name })?
            }};
        }

        macro_rules! try_eq {
            ($offset:literal, $val:expr) => {{
                let expected = $val;
                let found = *save_data.get($offset).ok_or(DecodeError::Index($offset))?;
                if expected != found { return Err(DecodeError::AssertEq { expected, found, offset: $offset }) }
            }};
            ($start:literal..$end:literal, $val:expr) => {{
                let expected = $val;
                let found = save_data.get($start..$end).ok_or(DecodeError::IndexRange { start: $start, end: $end })?;
                if expected != found { return Err(DecodeError::AssertEqRange { start: $start, end: $end, expected: expected.to_vec(), found: found.to_vec() }) }
            }};
        }

        if save_data.len() != SIZE { return Err(DecodeError::Size(save_data.len())) }
        try_eq!(0x001c..0x0022, b"ZELDAZ");
        Ok(Save {
            is_adult: match BigEndian::read_i32(get_offset!("is_adult", 0x0004, 0x4)) {
                0 => true,
                1 => false,
                n => return Err(DecodeError::UnexpectedValueRange { start: 0x0004, end: 0x0008, field: "is_adult", value: n.to_be_bytes().into() }),
            },
            time_of_day: try_get_offset!("time_of_day", 0x000c, 0x2),
            magic: {
                let magic = try_get_offset!("magic", 0x0032);
                try_eq!(0x003a, match magic {
                    MagicCapacity::None => 0,
                    MagicCapacity::Small | MagicCapacity::Large => 1,
                });
                try_eq!(0x003c, match magic {
                    MagicCapacity::None | MagicCapacity::Small => 0,
                    MagicCapacity::Large => 1,
                });
                magic
            },
            inv: try_get_offset!("inv", 0x0074, 0x18),
            inv_amounts: try_get_offset!("inv_amounts", 0x008c, 0xf),
            equipment: try_get_offset!("equipment", 0x009c, 0x2),
            upgrades: try_get_offset!("upgrades", 0x00a0, 0x4),
            quest_items: try_get_offset!("quest_items", 0x00a4, 0x4),
            small_keys: try_get_offset!("small_keys", 0x00bc, 0x13),
            skull_tokens: BigEndian::read_i16(get_offset!("skull_tokens", 0x00d0, 0x2)).try_into()?,
            scene_flags: try_get_offset!("scene_flags", 0x00d4, 101 * 0x1c),
            event_chk_inf: try_get_offset!("event_chk_inf", 0x0ed4, 0x1c),
            item_get_inf: try_get_offset!("item_get_inf", 0x0ef0, 0x8),
            inf_table: try_get_offset!("inf_table", 0x0ef8, 0x3c),
        })
    }

    fn to_save_data(&self) -> Vec<u8> {
        let mut buf = vec![0; SIZE];
        let Save { is_adult, time_of_day, magic, inv, inv_amounts, equipment, upgrades, quest_items, small_keys, skull_tokens, scene_flags, event_chk_inf, item_get_inf, inf_table } = self;
        buf.splice(0x0004..0x0008, if *is_adult { 0i32 } else { 1 }.to_be_bytes().iter().copied());
        buf.splice(0x000c..0x000e, Vec::from(time_of_day));
        buf.splice(0x001c..0x0022, b"ZELDAZ".into_iter().copied());
        buf[0x0032] = magic.into();
        buf[0x003a] = match magic {
            MagicCapacity::None => 0,
            MagicCapacity::Small | MagicCapacity::Large => 1,
        };
        buf[0x003c] = match magic {
            MagicCapacity::None | MagicCapacity::Small => 0,
            MagicCapacity::Large => 1,
        };
        buf.splice(0x0074..0x008c, Vec::from(inv));
        buf.splice(0x008c..0x009b, Vec::from(inv_amounts));
        buf.splice(0x009c..0x009e, Vec::from(equipment));
        buf.splice(0x00a0..0x00a4, Vec::from(upgrades));
        buf.splice(0x00a4..0x00a8, Vec::from(quest_items));
        buf.splice(0x00bc..0x00cf, Vec::from(small_keys));
        buf.splice(0x00d0..0x00d2, i16::from(*skull_tokens).to_be_bytes().iter().copied());
        buf.splice(0x00d4..0x00d4 + 101 * 0x1c, Vec::from(scene_flags));
        buf.splice(0x0ed4..0x0ef0, Vec::from(event_chk_inf));
        buf.splice(0x0ef0..0x0ef8, Vec::from(item_get_inf));
        buf.splice(0x0ef8..0x0f34, Vec::from(inf_table));
        buf
    }

    pub fn triforce_pieces(&self) -> u8 { //TODO move to Ram depending on how finding a triforce piece in the scene works
        self.scene_flags.windmill_and_dampes_grave.unused.bits().try_into().expect("too many triforce pieces")
    }

    pub fn set_triforce_pieces(&mut self, triforce_pieces: u8) {
        self.scene_flags.windmill_and_dampes_grave.unused = crate::scene::WindmillAndDampesGraveUnused::from_bits_truncate(triforce_pieces.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn amount_of_item(&self, item: &Item) -> u8 {
        match item.name() {
            "Blue Fire" | "Buy Blue Fire" => self.inv.bottles.iter().filter(|&&bottle| bottle == Bottle::BlueFire).count().try_into().expect("more than u8::MAX bottles"),
            "Bomb Bag" => match self.upgrades.bomb_bag() {
                Upgrades::BIGGEST_BOMB_BAG => 3,
                Upgrades::BIG_BOMB_BAG => 2,
                Upgrades::BOMB_BAG => 1,
                _ => 0,
            },
            "Bombchus" | "Bombchu Drop" | "Bombchus (5)" | "Bombchus (10)" | "Bombchus (20)" | "Buy Bombchu (5)" | "Buy Bombchu (10)" | "Buy Bombchu (20)" => self.inv_amounts.bombchus,
            "Boomerang" => if self.inv.boomerang { 1 } else { 0 },
            "Bow" => if self.inv.bow { 1 } else { 0 },
            "Deku Nut Drop" | "Buy Deku Nut (5)" | "Buy Deku Nut (10)" => self.inv_amounts.deku_nuts,
            "Buy Deku Shield" => if self.equipment.contains(Equipment::DEKU_SHIELD) { 1 } else { 0 },
            "Deku Stick Drop" | "Buy Deku Stick (1)" => self.inv_amounts.deku_sticks,
            "Deliver Letter" => if self.event_chk_inf.3.contains(EventChkInf3::DELIVER_RUTOS_LETTER) { 1 } else { 0 }, //TODO only consider when known by settings knowledge or visual confirmation
            "Dins Fire" => if self.inv.dins_fire { 1 } else { 0 },
            "Fish" | "Buy Fish" => self.inv.bottles.iter().filter(|&&bottle| bottle == Bottle::Fish).count().try_into().expect("more than u8::MAX bottles"),
            "Gerudo Membership Card" => if self.quest_items.contains(QuestItems::GERUDO_CARD) { 1 } else { 0 },
            "Hover Boots" => if self.equipment.contains(Equipment::HOVER_BOOTS) { 1 } else { 0 },
            "Buy Hylian Shield" => if self.equipment.contains(Equipment::HYLIAN_SHIELD) { 1 } else { 0 },
            "Kokiri Sword" => if self.equipment.contains(Equipment::KOKIRI_SWORD) { 1 } else { 0 },
            "Lens of Truth" => if self.inv.lens { 1 } else { 0 },
            "Megaton Hammer" => if self.inv.hammer { 1 } else { 0 },
            "Mirror Shield" => if self.equipment.contains(Equipment::MIRROR_SHIELD) { 1 } else { 0 },
            "Nayrus Love" => if self.inv.nayrus_love { 1 } else { 0 },
            "Ocarina" => if self.inv.ocarina { 1 } else { 0 }, //TODO return 2 with Ocarina of Time? (currently unused)
            "Progressive Hookshot" => match self.inv.hookshot {
                Hookshot::None => 0,
                Hookshot::Hookshot => 1,
                Hookshot::Longshot => 2,
            },
            "Progressive Scale" => match self.upgrades.scale() {
                Upgrades::GOLD_SCALE => 2,
                Upgrades::SILVER_SCALE => 1,
                _ => 0,
            },
            "Progressive Strength Upgrade" => match self.upgrades.strength() {
                Upgrades::GOLD_GAUNTLETS => 3,
                Upgrades::SILVER_GAUNTLETS => 2,
                Upgrades::GORON_BRACELET => 1,
                _ => 0,
            },
            "Serenade of Water" => if self.quest_items.contains(QuestItems::SERENADE_OF_WATER) { 1 } else { 0 },
            "Slingshot" => if self.inv.slingshot { 1 } else { 0 },
            //TODO add already opened doors (if Keysy is known or off)
            "Small Key (Fire Temple)" => self.small_keys.fire_temple,
            "Small Key (Forest Temple)" => self.small_keys.forest_temple,
            "Small Key (Gerudo Training Grounds)" => self.small_keys.gerudo_training_grounds,
            "Small Key (Spirit Temple)" => self.small_keys.spirit_temple,
            "Small Key (Water Temple)" => self.small_keys.water_temple,
            "Weird Egg" => if self.inv.child_trade_item == ChildTradeItem::WeirdEgg { 1 } else { 0 },
            name => unimplemented!("check for item {}", name), //TODO (make a list of all items)
        }
    }
}

#[derive(Debug, From, Clone)]
pub enum SaveDataReadError {
    #[from]
    Decode(DecodeError),
    Io(Arc<io::Error>),
}

impl From<io::Error> for SaveDataReadError {
    fn from(e: io::Error) -> SaveDataReadError {
        SaveDataReadError::Io(Arc::new(e))
    }
}

impl fmt::Display for SaveDataReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveDataReadError::Decode(e) => write!(f, "{:?}", e),
            SaveDataReadError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl Protocol for Save {
    type ReadError = SaveDataReadError;

    async fn read(tcp_stream: &mut TcpStream) -> Result<Save, SaveDataReadError> {
        let mut buf = vec![0; SIZE];
        tcp_stream.read_exact(&mut buf).await?;
        Ok(Save::from_save_data(&buf)?)
    }

    async fn write(&self, tcp_stream: &mut TcpStream) -> io::Result<()> {
        let buf = self.to_save_data();
        assert_eq!(buf.len(), SIZE);
        tcp_stream.write_all(&buf).await?;
        Ok(())
    }

    fn write_sync(&self, tcp_stream: &mut std::net::TcpStream) -> io::Result<()> {
        let buf = self.to_save_data();
        assert_eq!(buf.len(), SIZE);
        tcp_stream.write_all(&buf)?;
        Ok(())
    }
}

impl<'a, 'b> Add<&'b Delta> for &'a Save {
    type Output = Save;

    fn add(self, rhs: &Delta) -> Save {
        let mut serialized = self.to_save_data();
        for &(offset, value) in &rhs.0 {
            serialized[offset as usize] = value;
        }
        Save::from_save_data(&serialized).expect("save data patch failed")
    }
}

impl<'a, 'b> Sub<&'b Save> for &'a Save {
    type Output = Delta;

    fn sub(self, rhs: &Save) -> Delta {
        let new = self.to_save_data();
        let old = rhs.to_save_data();
        assert_eq!(old.len(), new.len());
        Delta(
            old.into_iter()
                .zip(new)
                .enumerate()
                .filter(|&(_, (old, new))| old != new)
                .map(|(offset, (_, new))| (offset as u16, new))
                .collect()
        )
    }
}

/// The difference between two save states.
#[derive(Debug, Clone)]
pub struct Delta(Vec<(u16, u8)>);

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl Protocol for Delta {
    type ReadError = io::Error;

    async fn read(tcp_stream: &mut TcpStream) -> io::Result<Delta> {
        let len = u16::read(tcp_stream).await?.into();
        let mut buf = Vec::with_capacity(len);
        for _ in 0..len {
            buf.push((u16::read(tcp_stream).await?, u8::read(tcp_stream).await?));
        }
        Ok(Delta(buf))
    }

    async fn write(&self, tcp_stream: &mut TcpStream) -> io::Result<()> {
        (self.0.len() as u16).write(tcp_stream).await?;
        for &(offset, value) in &self.0 {
            offset.write(tcp_stream).await?;
            value.write(tcp_stream).await?;
        }
        Ok(())
    }

    fn write_sync(&self, tcp_stream: &mut std::net::TcpStream) -> io::Result<()> {
        (self.0.len() as u16).write_sync(tcp_stream)?;
        for &(offset, value) in &self.0 {
            offset.write_sync(tcp_stream)?;
            value.write_sync(tcp_stream)?;
        }
        Ok(())
    }
}
