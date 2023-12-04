use {
    std::{
        future::Future,
        io::prelude::*,
        num::TryFromIntError,
        ops::{
            Add,
            Sub,
        },
        pin::Pin,
    },
    async_proto::{
        Protocol,
        ReadError,
        WriteError,
    },
    bitflags::bitflags,
    byteorder::{
        BigEndian,
        ByteOrder as _,
    },
    derivative::Derivative,
    derive_more::From,
    tokio::io::{
        AsyncRead,
        AsyncReadExt as _,
        AsyncWrite,
        AsyncWriteExt as _,
    },
    ootr::model::{
        Dungeon,
        DungeonReward,
        MainDungeon,
        Medallion,
        Stone,
        TimeRange,
    },
    crate::{
        info_tables::{
            EventChkInf,
            InfTable,
            ItemGetInf,
        },
        item_ids,
        scene::{
            GoldSkulltulas,
            SceneFlags,
        },
    },
};

pub const ADDR: u32 = 0x11a5d0;
pub const SIZE: usize = 0x1450;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TimeOfDay(u16);

impl TimeOfDay {
    pub fn matches(&self, range: TimeRange) -> bool {
        match range {
            TimeRange::Day => (0x4555..0xc001).contains(&self.0),
            TimeRange::Night => (0x0000..0x4555).contains(&self.0) || (0xc001..=0xffff).contains(&self.0),
            TimeRange::Dampe => (0xc001..0xe000).contains(&self.0),
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

#[derive(Derivative, Debug, Clone, Copy, PartialEq, Eq)]
#[derivative(Default)]
#[repr(u8)]
pub enum MagicCapacity {
    #[derivative(Default)]
    None = 0,
    Small = 1,
    Large = 2,
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

#[derive(Derivative, Debug, Clone, Copy, PartialEq, Eq)]
#[derivative(Default)]
pub enum Ocarina {
    #[derivative(Default)]
    None,
    FairyOcarina,
    OcarinaOfTime,
}

impl TryFrom<u8> for Ocarina {
    type Error = u8;

    fn try_from(raw_data: u8) -> Result<Ocarina, u8> {
        match raw_data {
            item_ids::NONE => Ok(Ocarina::None),
            item_ids::FAIRY_OCARINA=> Ok(Ocarina::FairyOcarina),
            item_ids::OCARINA_OF_TIME => Ok(Ocarina::OcarinaOfTime),
            _ => Err(raw_data),
        }
    }
}

impl From<Ocarina> for u8 {
    fn from(ocarina: Ocarina) -> u8 {
        match ocarina {
            Ocarina::None => item_ids::NONE,
            Ocarina::FairyOcarina => item_ids::FAIRY_OCARINA,
            Ocarina::OcarinaOfTime => item_ids::OCARINA_OF_TIME,
        }
    }
}

#[derive(Derivative, Debug, Clone, Copy, PartialEq, Eq)]
#[derivative(Default)]
pub enum Hookshot {
    #[derivative(Default)]
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

#[derive(Derivative, Debug, Clone, Copy, PartialEq, Eq)]
#[derivative(Default)]
pub enum Bottle {
    #[derivative(Default)]
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

#[derive(Derivative, Debug, Clone, Copy, PartialEq, Eq)]
#[derivative(Default)]
pub enum AdultTradeItem {
    #[derivative(Default)]
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

#[derive(Derivative, Debug, Clone, Copy, PartialEq, Eq)]
#[derivative(Default)]
pub enum ChildTradeItem {
    #[derivative(Default)]
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
    pub ocarina: Ocarina,
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
    fn add_bottle(&mut self, mut new_bottle: Bottle) -> bool {
        for bottle in &mut self.bottles {
            if *bottle == Bottle::None {
                *bottle = new_bottle;
                return true
            } else if *bottle == Bottle::RutosLetter && new_bottle == Bottle::RutosLetter {
                new_bottle = Bottle::Empty;
            }
        }
        false
    }

    pub fn emptiable_bottles(&self) -> u8 {
        self.bottles.iter().filter(|bottle| bottle.emptiable()).count().try_into().expect("there are only 4 bottles")
    }

    pub fn has_rutos_letter(&self) -> bool {
        self.bottles.iter().any(|bottle| *bottle == Bottle::RutosLetter)
    }

    pub fn set_emptiable_bottles(&mut self, amount: u8) {
        assert!(amount <= 4);
        'increment: while self.emptiable_bottles() < amount {
            for bottle in &mut self.bottles {
                if *bottle == Bottle::None {
                    *bottle = Bottle::Empty;
                    continue 'increment
                }
            }
            for bottle in &mut self.bottles {
                if *bottle == Bottle::BigPoe {
                    *bottle = Bottle::Empty;
                    continue 'increment
                }
            }
            for bottle in &mut self.bottles {
                if *bottle == Bottle::RutosLetter {
                    *bottle = Bottle::Empty;
                    continue 'increment
                }
            }
            unreachable!("could not increment emptiable bottles")
        }
        'decrement: while self.emptiable_bottles() > amount {
            for bottle in &mut self.bottles {
                if bottle.emptiable() {
                    *bottle = Bottle::None;
                    continue 'decrement
                }
            }
            unreachable!("could not decrement emptiable bottles")
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
            ($offset:literal, $value:pat) => {{
                match *raw_data.get($offset).ok_or_else(|| raw_data.clone())? {
                    item_ids::NONE => false,
                    $value => true,
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
            ocarina: Ocarina::try_from(raw_data[0x07]).map_err(|_| raw_data.clone())?,
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
            bool_item!(slingshot, item_ids::SLINGSHOT), inv.ocarina.into(), bool_item!(bombchus, item_ids::BOMBCHU_10), inv.hookshot.into(), bool_item!(ice_arrows, item_ids::ICE_ARROWS), bool_item!(farores_wind, item_ids::FARORES_WIND),
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
    pub num_received_mw_items: u16,
    pub bombchus: u8,
}

impl TryFrom<Vec<u8>> for InvAmounts {
    type Error = Vec<u8>;

    fn try_from(raw_data: Vec<u8>) -> Result<InvAmounts, Vec<u8>> {
        if raw_data.len() != 0xf { return Err(raw_data) }
        Ok(InvAmounts {
            deku_sticks: *raw_data.get(0x00).ok_or_else(|| raw_data.clone())?,
            deku_nuts: *raw_data.get(0x01).ok_or_else(|| raw_data.clone())?,
            num_received_mw_items: match raw_data.get(0x04..0x06) {
                Some(&[hi, lo]) => u16::from_be_bytes([hi, lo]),
                _ => unreachable!(),
            },
            bombchus: *raw_data.get(0x08).ok_or_else(|| raw_data.clone())?,
        })
    }
}

impl<'a> From<&'a InvAmounts> for [u8; 0xf] {
    fn from(inv_amounts: &InvAmounts) -> [u8; 0xf] {
        let [hi, lo] = inv_amounts.num_received_mw_items.to_be_bytes();
        [
            inv_amounts.deku_sticks, inv_amounts.deku_nuts, 0, 0, hi, lo,
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
        const DEKU_NUT_CAPACITY_MASK = 0x0070_0000;
        const DEKU_NUT_CAPACITY_40 = 0x0030_0000;
        const DEKU_NUT_CAPACITY_30 = 0x0020_0000;
        const DEKU_NUT_CAPACITY_20 = 0x0010_0000;
        const DEKU_STICK_CAPACITY_MASK = 0x000E_0000;
        const DEKU_STICK_CAPACITY_30 = 0x0006_0000;
        const DEKU_STICK_CAPACITY_20 = 0x0004_0000;
        const DEKU_STICK_CAPACITY_10 = 0x0002_0000;
        const BULLET_BAG_MASK = 0x0001_c000;
        const BULLET_BAG_50 = 0x0000_c000;
        const BULLET_BAG_40 = 0x0000_8000;
        const BULLET_BAG_30 = 0x0000_4000; //TODO check for parity with slingshot
        const WALLET_MASK = 0x0000_3000;
        const ADULTS_WALLET = 0x0000_1000;
        const GIANTS_WALLET = 0x0000_2000;
        const TYCOONS_WALLET = 0x0000_3000;
        const SCALE_MASK = 0x0000_0e00;
        const GOLD_SCALE = 0x0000_0400;
        const SILVER_SCALE = 0x0000_0200;
        const STRENGTH_MASK = 0x0000_01c0;
        const GOLD_GAUNTLETS = 0x0000_000c0;
        const SILVER_GAUNTLETS = 0x0000_0080;
        const GORON_BRACELET = 0x0000_0040;
        const BOMB_BAG_MASK = 0x0000_0038;
        const BOMB_BAG_40 = 0x0000_0018;
        const BOMB_BAG_30 = 0x0000_0010;
        const BOMB_BAG_20 = 0x0000_0008;
        const QUIVER_MASK = 0x0000_0007;
        const QUIVER_50 = 0x0000_0003;
        const QUIVER_40 = 0x0000_0002;
        const QUIVER_30 = 0x0000_0001; //TODO check for parity with bow
        const NONE = 0x0000_0000;
    }
}

impl Upgrades {
    pub fn nut_capacity(&self) -> Upgrades { *self & Upgrades::DEKU_NUT_CAPACITY_MASK }

    pub fn set_nut_capacity(&mut self, nut_capacity: Upgrades) {
        self.remove(Upgrades::DEKU_NUT_CAPACITY_MASK);
        self.insert(nut_capacity & Upgrades::DEKU_NUT_CAPACITY_MASK);
    }

    pub fn stick_capacity(&self) -> Upgrades { *self & Upgrades::DEKU_STICK_CAPACITY_MASK }

    pub fn set_stick_capacity(&mut self, stick_capacity: Upgrades) {
        self.remove(Upgrades::DEKU_STICK_CAPACITY_MASK);
        self.insert(stick_capacity & Upgrades::DEKU_STICK_CAPACITY_MASK);
    }

    pub fn bullet_bag(&self) -> Upgrades { *self & Upgrades::BULLET_BAG_MASK }

    pub fn set_bullet_bag(&mut self, bullet_bag: Upgrades) {
        self.remove(Upgrades::BULLET_BAG_MASK);
        self.insert(bullet_bag & Upgrades::BULLET_BAG_MASK);
    }

    pub fn wallet(&self) -> Upgrades { *self & Upgrades::WALLET_MASK }

    pub fn set_wallet(&mut self, wallet: Upgrades) {
        self.remove(Upgrades::WALLET_MASK);
        self.insert(wallet & Upgrades::WALLET_MASK);
    }

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

    pub fn quiver(&self) -> Upgrades { *self & Upgrades::QUIVER_MASK }

    pub fn set_quiver(&mut self, quiver: Upgrades) {
        self.remove(Upgrades::QUIVER_MASK);
        self.insert(quiver & Upgrades::QUIVER_MASK);
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
        const STONE_OF_AGONY = 0x0020_0000;
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
    pub fn has(&self, items: impl Into<QuestItems>) -> bool {
        self.contains(items.into())
    }

    pub fn num_stones(&self) -> u8 {
        (if self.contains(QuestItems::KOKIRI_EMERALD) { 1 } else { 0 })
        + if self.contains(QuestItems::GORON_RUBY) { 1 } else { 0 }
        + if self.contains(QuestItems::ZORA_SAPPHIRE) { 1 } else { 0 }
    }
}

impl From<Medallion> for QuestItems {
    fn from(med: Medallion) -> QuestItems {
        match med {
            Medallion::Light => QuestItems::LIGHT_MEDALLION,
            Medallion::Forest => QuestItems::FOREST_MEDALLION,
            Medallion::Fire => QuestItems::FIRE_MEDALLION,
            Medallion::Water => QuestItems::WATER_MEDALLION,
            Medallion::Shadow => QuestItems::SHADOW_MEDALLION,
            Medallion::Spirit => QuestItems::SPIRIT_MEDALLION,
        }
    }
}

impl From<Stone> for QuestItems {
    fn from(stone: Stone) -> QuestItems {
        match stone {
            Stone::KokiriEmerald => QuestItems::KOKIRI_EMERALD,
            Stone::GoronRuby => QuestItems::GORON_RUBY,
            Stone::ZoraSapphire => QuestItems::ZORA_SAPPHIRE,
        }
    }
}

impl From<DungeonReward> for QuestItems {
    fn from(reward: DungeonReward) -> QuestItems {
        match reward {
            DungeonReward::Medallion(med) => med.into(),
            DungeonReward::Stone(stone) => stone.into(),
        }
    }
}

impl<'a, T: Into<QuestItems> + Clone> From<&'a T> for QuestItems {
    fn from(x: &T) -> QuestItems { x.clone().into() }
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

bitflags! {
    #[derive(Default)]
    pub struct DungeonItems: u8 {
        const MAP = 0x04;
        const COMPASS = 0x02;
        const BOSS_KEY = 0x01;
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AllDungeonItems {
    pub deku_tree: DungeonItems,
    pub dodongos_cavern: DungeonItems,
    pub jabu_jabu: DungeonItems,
    pub forest_temple: DungeonItems,
    pub fire_temple: DungeonItems,
    pub water_temple: DungeonItems,
    pub spirit_temple: DungeonItems,
    pub shadow_temple: DungeonItems,
    pub bottom_of_the_well: DungeonItems,
    pub ice_cavern: DungeonItems,
    pub ganons_castle: DungeonItems,
}

impl AllDungeonItems {
    pub fn get(&self, dungeon: Dungeon) -> DungeonItems {
        match dungeon {
            Dungeon::Main(MainDungeon::DekuTree) => self.deku_tree,
            Dungeon::Main(MainDungeon::DodongosCavern) => self.dodongos_cavern,
            Dungeon::Main(MainDungeon::JabuJabu) => self.jabu_jabu,
            Dungeon::Main(MainDungeon::ForestTemple) => self.forest_temple,
            Dungeon::Main(MainDungeon::FireTemple) => self.fire_temple,
            Dungeon::Main(MainDungeon::WaterTemple) => self.water_temple,
            Dungeon::Main(MainDungeon::ShadowTemple) => self.shadow_temple,
            Dungeon::Main(MainDungeon::SpiritTemple) => self.spirit_temple,
            Dungeon::IceCavern => self.ice_cavern,
            Dungeon::BottomOfTheWell => self.bottom_of_the_well,
            Dungeon::GerudoTrainingGround => DungeonItems::default(),
            Dungeon::GanonsCastle => self.ganons_castle,
        }
    }
}

impl TryFrom<Vec<u8>> for AllDungeonItems {
    type Error = Vec<u8>;

    fn try_from(raw_data: Vec<u8>) -> Result<Self, Vec<u8>> {
        macro_rules! get {
            ($idx:expr) => {{
                DungeonItems::from_bits_truncate(raw_data[$idx])
            }};
        }

        if raw_data.len() != 0x14 { return Err(raw_data) }
        Ok(Self {
            deku_tree: get!(0x00),
            dodongos_cavern: get!(0x01),
            jabu_jabu: get!(0x02),
            forest_temple: get!(0x03),
            fire_temple: get!(0x04),
            water_temple: get!(0x05),
            spirit_temple: get!(0x06),
            shadow_temple: get!(0x07),
            bottom_of_the_well: get!(0x08),
            ice_cavern: get!(0x09),
            ganons_castle: get!(0x0a),
        })
    }
}

impl<'a> From<&'a AllDungeonItems> for [u8; 0x14] {
    fn from(items: &AllDungeonItems) -> [u8; 0x14] {
        [
            items.deku_tree.bits(), items.dodongos_cavern.bits(), items.jabu_jabu.bits(), items.forest_temple.bits(),
            items.fire_temple.bits(), items.water_temple.bits(), items.spirit_temple.bits(), items.shadow_temple.bits(),
            items.bottom_of_the_well.bits(), items.ice_cavern.bits(), items.ganons_castle.bits(), 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]
    }
}

impl<'a> From<&'a AllDungeonItems> for Vec<u8> {
    fn from(items: &AllDungeonItems) -> Vec<u8> {
        <[u8; 0x14]>::from(items).into()
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
    pub gerudo_training_ground: u8,
    pub thieves_hideout: u8,
    pub ganons_castle: u8,
    pub treasure_chest_game: u8,
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
            gerudo_training_ground: get!(0x0b),
            thieves_hideout: get!(0x0c),
            ganons_castle: get!(0x0d),
            treasure_chest_game: get!(0x10),
        })
    }
}

impl<'a> From<&'a SmallKeys> for [u8; 0x13] {
    fn from(small_keys: &SmallKeys) -> [u8; 0x13] {
        [
            0, 0, 0, small_keys.forest_temple,
            small_keys.fire_temple, small_keys.water_temple, small_keys.spirit_temple, small_keys.shadow_temple,
            small_keys.bottom_of_the_well, 0, 0, small_keys.gerudo_training_ground,
            small_keys.thieves_hideout, small_keys.ganons_castle, 0, 0,
            small_keys.treasure_chest_game, 0, 0,
        ]
    }
}

impl<'a> From<&'a SmallKeys> for Vec<u8> {
    fn from(small_keys: &SmallKeys) -> Vec<u8> {
        <[u8; 0x13]>::from(small_keys).into()
    }
}

bitflags! {
    #[derive(Default)]
    pub struct FishingContext: u32 {
        const ADULT_PRIZE_OBTAINED = 0x0000_0800;
        const CHILD_PRIZE_OBTAINED = 0x0000_0400;
    }
}

impl TryFrom<Vec<u8>> for FishingContext {
    type Error = Vec<u8>;

    fn try_from(raw_data: Vec<u8>) -> Result<FishingContext, Vec<u8>> {
        if raw_data.len() != 4 { return Err(raw_data) }
        Ok(FishingContext::from_bits_truncate(BigEndian::read_u32(&raw_data)))
    }
}

impl<'a> From<&'a FishingContext> for [u8; 4] {
    fn from(fishing_context: &FishingContext) -> [u8; 4] {
        fishing_context.bits().to_be_bytes()
    }
}

impl<'a> From<&'a FishingContext> for Vec<u8> {
    fn from(fishing_context: &FishingContext) -> Vec<u8> {
        <[u8; 4]>::from(fishing_context).into()
    }
}

#[derive(Derivative, Debug, Clone, Copy, PartialEq, Eq)]
#[derivative(Default)]
pub enum GameMode {
    #[derivative(Default)] // represented as 0x0000_0000
    Gameplay,
    TitleScreen,
    FileSelect,
}

impl TryFrom<Vec<u8>> for GameMode {
    type Error = Vec<u8>;

    fn try_from(raw_data: Vec<u8>) -> Result<GameMode, Vec<u8>> {
        Ok(match raw_data[..] {
            [0, 0, 0, 0] => GameMode::Gameplay,
            [0, 0, 0, 1] => GameMode::TitleScreen,
            [0, 0, 0, 2] => GameMode::FileSelect,
            _ => return Err(raw_data),
        })
    }
}

impl<'a> From<&'a GameMode> for [u8; 4] {
    fn from(game_mode: &GameMode) -> [u8; 4] {
        match game_mode {
            GameMode::Gameplay => [0, 0, 0, 0],
            GameMode::TitleScreen => [0, 0, 0, 1],
            GameMode::FileSelect => [0, 0, 0, 2],
        }
    }
}

impl<'a> From<&'a GameMode> for Vec<u8> {
    fn from(game_mode: &GameMode) -> Vec<u8> {
        <[u8; 4]>::from(game_mode).into()
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
    pub biggoron_sword: bool,
    pub dmt_biggoron_checked: bool,
    pub inv: Inventory,
    pub inv_amounts: InvAmounts,
    pub equipment: Equipment,
    pub upgrades: Upgrades,
    pub quest_items: QuestItems,
    pub dungeon_items: AllDungeonItems,
    pub small_keys: SmallKeys,
    pub skull_tokens: u8,
    pub scene_flags: SceneFlags,
    pub gold_skulltulas: GoldSkulltulas,
    pub big_poes: u8,
    pub fishing_context: FishingContext,
    pub event_chk_inf: EventChkInf,
    pub item_get_inf: ItemGetInf,
    pub inf_table: InfTable,
    pub scarecrow_song_child: bool,
    pub game_mode: GameMode,
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
            magic: if get_offset!("has single magic", 0x003a) == 0 {
                try_eq!(0x003c, 0);
                MagicCapacity::None
            } else {
                if get_offset!("has double magic", 0x003c) == 0 {
                    MagicCapacity::Small
                } else {
                    MagicCapacity::Large
                }
            },
            biggoron_sword: match get_offset!("biggoron_sword", 0x003e) {
                0 => false,
                1 => true,
                value => return Err(DecodeError::UnexpectedValue { value, offset: 0x003e, field: "biggoron_sword" }),
            },
            dmt_biggoron_checked: {
                bitflags! {
                    struct DmtBiggoronCheckedFlags: u16 {
                        const DMT_BIGGORON_CHECKED = 0x0100;
                    }
                }

                impl TryFrom<Vec<u8>> for DmtBiggoronCheckedFlags {
                    type Error = Vec<u8>;

                    fn try_from(raw_data: Vec<u8>) -> Result<DmtBiggoronCheckedFlags, Vec<u8>> {
                        if raw_data.len() != 2 { return Err(raw_data) }
                        Ok(DmtBiggoronCheckedFlags::from_bits_truncate(BigEndian::read_u16(&raw_data)))
                    }
                }

                let flags: DmtBiggoronCheckedFlags = try_get_offset!("dmt_biggoron_checked", 0x0072, 0x2);
                flags.contains(DmtBiggoronCheckedFlags::DMT_BIGGORON_CHECKED)
            },
            inv: try_get_offset!("inv", 0x0074, 0x18),
            inv_amounts: try_get_offset!("inv_amounts", 0x008c, 0xf),
            equipment: try_get_offset!("equipment", 0x009c, 0x2),
            upgrades: try_get_offset!("upgrades", 0x00a0, 0x4),
            quest_items: try_get_offset!("quest_items", 0x00a4, 0x4),
            dungeon_items: try_get_offset!("dungeon_items", 0x00a8, 0x14),
            small_keys: try_get_offset!("small_keys", 0x00bc, 0x13),
            skull_tokens: BigEndian::read_i16(get_offset!("skull_tokens", 0x00d0, 0x2)).try_into()?,
            scene_flags: try_get_offset!("scene_flags", 0x00d4, 101 * 0x1c),
            gold_skulltulas: try_get_offset!("gold_skulltulas", 0x0e9c, 0x18),
            big_poes: (BigEndian::read_u32(get_offset!("big_poes", 0x0ebc, 0x4)) / 100).try_into()?,
            fishing_context: try_get_offset!("fishing_context", 0x0ec0, 0x4),
            event_chk_inf: try_get_offset!("event_chk_inf", 0x0ed4, 0x1c),
            item_get_inf: try_get_offset!("item_get_inf", 0x0ef0, 0x8),
            inf_table: try_get_offset!("inf_table", 0x0ef8, 0x3c),
            scarecrow_song_child: match get_offset!("scarecrow_song_child", 0x12c5) {
                0 => false,
                1 => true,
                value => return Err(DecodeError::UnexpectedValue { value, offset: 0x12c5, field: "scarecrow_song_child" }),
            },
            game_mode: try_get_offset!("game_mode", 0x135c, 0x4),
        })
    }

    pub(crate) fn to_save_data(&self) -> Vec<u8> {
        let mut buf = vec![0; SIZE];
        let Save {
            is_adult, time_of_day, magic, biggoron_sword, dmt_biggoron_checked, inv, inv_amounts,
            equipment, upgrades, quest_items, dungeon_items, small_keys, skull_tokens, scene_flags,
            gold_skulltulas, big_poes, fishing_context, event_chk_inf, item_get_inf, inf_table,
            scarecrow_song_child, game_mode,
        } = self;
        buf.splice(0x0004..0x0008, if *is_adult { 0i32 } else { 1 }.to_be_bytes().into_iter());
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
        buf[0x003e] = if *biggoron_sword { 1 } else { 0 };
        buf[0x0072] = if *dmt_biggoron_checked { 1 } else { 0 };
        buf.splice(0x0074..0x008c, Vec::from(inv));
        buf.splice(0x008c..0x009b, Vec::from(inv_amounts));
        buf.splice(0x009c..0x009e, Vec::from(equipment));
        buf.splice(0x00a0..0x00a4, Vec::from(upgrades));
        buf.splice(0x00a4..0x00a8, Vec::from(quest_items));
        buf.splice(0x00a8..0x00bc, Vec::from(dungeon_items));
        buf.splice(0x00bc..0x00cf, Vec::from(small_keys));
        buf.splice(0x00d0..0x00d2, i16::from(*skull_tokens).to_be_bytes().into_iter());
        buf.splice(0x00d4..0x00d4 + 101 * 0x1c, Vec::from(scene_flags));
        buf.splice(0x0e9c..0x0eb4, Vec::from(gold_skulltulas));
        buf.splice(0x0ebc..0x0ec0, u32::from(100 * big_poes).to_be_bytes().into_iter());
        buf.splice(0x0ec0..0x0ec4, Vec::from(fishing_context));
        buf.splice(0x0ed4..0x0ef0, Vec::from(event_chk_inf));
        buf.splice(0x0ef0..0x0ef8, Vec::from(item_get_inf));
        buf.splice(0x0ef8..0x0f34, Vec::from(inf_table));
        buf[0x12c5] = if *scarecrow_song_child { 1 } else { 0 };
        buf.splice(0x135c..0x1360, Vec::from(game_mode));
        buf
    }

    pub fn triforce_pieces(&self) -> u8 { //TODO move to Ram depending on how finding a triforce piece in the scene works
        self.scene_flags.windmill_and_dampes_grave.unused.bits().try_into().expect("too many triforce pieces")
    }

    pub fn set_triforce_pieces(&mut self, triforce_pieces: u8) {
        self.scene_flags.windmill_and_dampes_grave.unused = crate::scene::WindmillAndDampesGraveUnused::from_bits_truncate(triforce_pieces.into());
    }

    pub fn recv_mw_item(&mut self, item: u16) -> Result<(), ()> {
        self.inv_amounts.num_received_mw_items += 1;
        match item {
            0x0001 => {} // Bombs (5)
            0x0002 => {} // Deku Nuts (5)
            0x0003 => { // Bombchus (10)
                self.inv.bombchus = true;
                self.inv_amounts.bombchus = 50.min(self.inv_amounts.bombchus + 10);
            }
            0x0006 => self.inv.boomerang = true, // Boomerang
            0x0007 => {} // Deku Stick (1)
            0x000A => self.inv.lens = true, // Lens of Truth
            0x000B => self.inv.child_trade_item = ChildTradeItem::ZeldasLetter, // Zeldas Letter
            0x000D => self.inv.hammer = true, // Megaton Hammer
            0x000E => self.inv.adult_trade_item = AdultTradeItem::Cojiro, // Cojiro
            0x000F => { self.inv.add_bottle(Bottle::Empty); } // Bottle
            0x0014 => { self.inv.add_bottle(Bottle::MilkFull); } // Bottle with Milk
            0x0015 => { self.inv.add_bottle(Bottle::RutosLetter); } // Rutos Letter
            0x0016 => self.inv.beans = true, // Magic Bean
            0x0017 => self.inv.child_trade_item = ChildTradeItem::SkullMask, // Skull Mask
            0x0018 => self.inv.child_trade_item = ChildTradeItem::SpookyMask, // Spooky Mask
            0x001A => self.inv.child_trade_item = ChildTradeItem::KeatonMask, // Keaton Mask
            0x001B => self.inv.child_trade_item = ChildTradeItem::BunnyHood, // Bunny Hood
            0x001C => self.inv.child_trade_item = ChildTradeItem::MaskOfTruth, // Mask of Truth
            0x001D => self.inv.adult_trade_item = AdultTradeItem::PocketEgg, // Pocket Egg
            0x001E => self.inv.adult_trade_item = AdultTradeItem::PocketCucco, // Pocket Cucco
            0x001F => self.inv.adult_trade_item = AdultTradeItem::OddMushroom, // Odd Mushroom
            0x0020 => self.inv.adult_trade_item = AdultTradeItem::OddPotion, // Odd Potion
            0x0021 => self.inv.adult_trade_item = AdultTradeItem::PoachersSaw, // Poachers Saw
            0x0022 => self.inv.adult_trade_item = AdultTradeItem::BrokenSword, // Broken Sword
            0x0023 => self.inv.adult_trade_item = AdultTradeItem::Prescription, // Prescription
            0x0024 => self.inv.adult_trade_item = AdultTradeItem::EyeballFrog, // Eyeball Frog
            0x0025 => self.inv.adult_trade_item = AdultTradeItem::Eyedrops, // Eyedrops
            0x0026 => self.inv.adult_trade_item = AdultTradeItem::ClaimCheck, // Claim Check
            0x0027 => self.equipment.insert(Equipment::KOKIRI_SWORD), // Kokiri Sword
            0x0028 => self.equipment.insert(Equipment::GIANTS_KNIFE), // Giants Knife
            0x0029 => self.equipment.insert(Equipment::DEKU_SHIELD), // Deku Shield
            0x002A => self.equipment.insert(Equipment::HYLIAN_SHIELD), // Hylian Shield
            0x002B => self.equipment.insert(Equipment::MIRROR_SHIELD), // Mirror Shield
            0x002C => self.equipment.insert(Equipment::GORON_TUNIC), // Goron Tunic
            0x002D => self.equipment.insert(Equipment::ZORA_TUNIC), // Zora Tunic
            0x002E => self.equipment.insert(Equipment::IRON_BOOTS), // Iron Boots
            0x002F => self.equipment.insert(Equipment::HOVER_BOOTS), // Hover Boots
            0x0039 => self.quest_items.insert(QuestItems::STONE_OF_AGONY), // Stone of Agony
            0x003A => self.quest_items.insert(QuestItems::GERUDO_CARD), // Gerudo Membership Card
            0x003D => {} // Heart Container
            0x003E => {} // Piece of Heart
            0x003F => {} // Boss Key
            0x0040 => {} // Compass
            0x0041 => {} // Map
            0x0042 => {} // Small Key
            0x0047 => self.inv.child_trade_item = ChildTradeItem::WeirdEgg, // Weird Egg
            0x0048 => {} // Recovery Heart
            0x0049 => {} // Arrows (5)
            0x004A => {} // Arrows (10)
            0x004B => {} // Arrows (30)
            0x004C => {} // Rupee (1)
            0x004D => {} // Rupees (5)
            0x004E => {} // Rupees (20)
            0x0050 => {} // Milk
            0x0051 => self.inv.child_trade_item = ChildTradeItem::GoronMask, // Goron Mask
            0x0052 => self.inv.child_trade_item = ChildTradeItem::ZoraMask, // Zora Mask
            0x0053 => self.inv.child_trade_item = ChildTradeItem::GerudoMask, // Gerudo Mask
            0x0055 => {} // Rupees (50)
            0x0056 => {} // Rupees (200)
            0x0057 => { // Biggoron Sword
                self.equipment.insert(Equipment::GIANTS_KNIFE);
                self.biggoron_sword = true;
            }
            0x0058 => self.inv.fire_arrows = true, // Fire Arrows
            0x0059 => self.inv.ice_arrows = true, // Ice Arrows
            0x005A => self.inv.light_arrows = true, // Light Arrows
            0x005B => self.skull_tokens += 1, // Gold Skulltula Token
            0x005C => self.inv.dins_fire = true, // Dins Fire
            0x005D => self.inv.farores_wind = true, // Farores Wind
            0x005E => self.inv.nayrus_love = true, // Nayrus Love
            0x0064 => {} // Deku Nuts (10)
            0x0065 => {} // Bomb
            0x0066 => {} // Bombs (10)
            0x0067 => {} // Bombs (20)
            0x0068 => {} // Bombs (30)
            0x0069 => {} // Deku Seeds (30)
            0x006A => { // Bombchus (5)
                self.inv.bombchus = true;
                self.inv_amounts.bombchus = 50.min(self.inv_amounts.bombchus + 5);
            }
            0x006B => { // Bombchus (20)
                self.inv.bombchus = true;
                self.inv_amounts.bombchus = 50.min(self.inv_amounts.bombchus + 20);
            }
            0x0071 => self.small_keys.treasure_chest_game += 1, // Small Key (Treasure Chest Game)
            0x0072 => {} // Rupee (Treasure Chest Game)
            0x0073 => {} // Rupees (5) (Treasure Chest Game)
            0x0074 => {} // Rupees (20) (Treasure Chest Game)
            0x0075 => {} // Rupees (50) (Treasure Chest Game)
            0x0076 => {} // Piece of Heart (Treasure Chest Game)
            0x007C => {} // Ice Trap
            0x0080 => self.inv.hookshot = match self.inv.hookshot { // Progressive Hookshot
                Hookshot::None => Hookshot::Hookshot,
                Hookshot::Hookshot | Hookshot::Longshot => Hookshot::Longshot,
            },
            0x0081 => self.upgrades.set_strength(match self.upgrades.strength() { // Progressive Strength Upgrade
                Upgrades::GORON_BRACELET => Upgrades::SILVER_GAUNTLETS,
                Upgrades::SILVER_GAUNTLETS | Upgrades::GOLD_GAUNTLETS => Upgrades::GOLD_GAUNTLETS,
                _ => Upgrades::GORON_BRACELET,
            }),
            0x0082 => self.upgrades.set_bomb_bag(match self.upgrades.bomb_bag() { // Bomb Bag
                Upgrades::BOMB_BAG_20 => Upgrades::BOMB_BAG_30,
                Upgrades::BOMB_BAG_30 | Upgrades::BOMB_BAG_40 => Upgrades::BOMB_BAG_40,
                _ => Upgrades::BOMB_BAG_20,
            }),
            0x0083 => { // Bow
                self.inv.bow = true;
                self.upgrades.set_quiver(match self.upgrades.quiver() {
                    Upgrades::QUIVER_30 => Upgrades::QUIVER_40,
                    Upgrades::QUIVER_40 | Upgrades::QUIVER_50 => Upgrades::QUIVER_50,
                    _ => Upgrades::QUIVER_30,
                });
            }
            0x0084 => { // Slingshot
                self.inv.slingshot = true;
                self.upgrades.set_bullet_bag(match self.upgrades.bullet_bag() {
                    Upgrades::BULLET_BAG_30 => Upgrades::BULLET_BAG_40,
                    Upgrades::BULLET_BAG_40 | Upgrades::BULLET_BAG_50 => Upgrades::BULLET_BAG_50,
                    _ => Upgrades::BULLET_BAG_30,
                });
            }
            0x0085 => self.upgrades.set_wallet(match self.upgrades.wallet() { // Progressive Wallet
                Upgrades::ADULTS_WALLET => Upgrades::GIANTS_WALLET,
                Upgrades::GIANTS_WALLET | Upgrades::TYCOONS_WALLET => Upgrades::TYCOONS_WALLET,
                _ => Upgrades::ADULTS_WALLET,
            }),
            0x0086 => self.upgrades.set_scale(match self.upgrades.scale() { // Progressive Scale
                Upgrades::SILVER_SCALE | Upgrades::GOLD_SCALE => Upgrades::GOLD_SCALE,
                _ => Upgrades::SILVER_SCALE,
            }),
            0x0087 => self.upgrades.set_nut_capacity(match self.upgrades.nut_capacity() { // Deku Nut Capacity
                Upgrades::DEKU_NUT_CAPACITY_20 => Upgrades::DEKU_NUT_CAPACITY_30,
                Upgrades::DEKU_NUT_CAPACITY_30 | Upgrades::DEKU_NUT_CAPACITY_40 => Upgrades::DEKU_NUT_CAPACITY_40,
                _ => Upgrades::DEKU_NUT_CAPACITY_20,
            }),
            0x0088 => self.upgrades.set_stick_capacity(match self.upgrades.stick_capacity() { // Deku Stick Capacity
                Upgrades::DEKU_STICK_CAPACITY_10 => Upgrades::DEKU_STICK_CAPACITY_20,
                Upgrades::DEKU_STICK_CAPACITY_20 | Upgrades::DEKU_STICK_CAPACITY_30 => Upgrades::DEKU_STICK_CAPACITY_30,
                _ => Upgrades::DEKU_STICK_CAPACITY_10,
            }),
            0x0089 => self.inv.bombchus = true, // Bombchus
            0x008A => self.magic = match self.magic { // Magic Meter
                MagicCapacity::None => MagicCapacity::Small,
                MagicCapacity::Small | MagicCapacity::Large => MagicCapacity::Large,
            },
            0x008B => self.inv.ocarina = match self.inv.ocarina { // Ocarina
                Ocarina::None => Ocarina::FairyOcarina,
                Ocarina::FairyOcarina | Ocarina::OcarinaOfTime => Ocarina::OcarinaOfTime,
            },
            0x008C => { self.inv.add_bottle(Bottle::RedPotion); } // Bottle with Red Potion
            0x008D => { self.inv.add_bottle(Bottle::GreenPotion); } // Bottle with Green Potion
            0x008E => { self.inv.add_bottle(Bottle::BluePotion); } // Bottle with Blue Potion
            0x008F => { self.inv.add_bottle(Bottle::Fairy); } // Bottle with Fairy
            0x0090 => { self.inv.add_bottle(Bottle::Fish); } // Bottle with Fish
            0x0091 => { self.inv.add_bottle(Bottle::BlueFire); } // Bottle with Blue Fire
            0x0092 => { self.inv.add_bottle(Bottle::Bug); } // Bottle with Bugs
            0x0093 => { self.inv.add_bottle(Bottle::BigPoe); } // Bottle with Big Poe
            0x0094 => { self.inv.add_bottle(Bottle::Poe); } // Bottle with Poe
            0x0095 => self.dungeon_items.forest_temple.insert(DungeonItems::BOSS_KEY), // Boss Key (Forest Temple)
            0x0096 => self.dungeon_items.fire_temple.insert(DungeonItems::BOSS_KEY), // Boss Key (Fire Temple)
            0x0097 => self.dungeon_items.water_temple.insert(DungeonItems::BOSS_KEY), // Boss Key (Water Temple)
            0x0098 => self.dungeon_items.spirit_temple.insert(DungeonItems::BOSS_KEY), // Boss Key (Spirit Temple)
            0x0099 => self.dungeon_items.shadow_temple.insert(DungeonItems::BOSS_KEY), // Boss Key (Shadow Temple)
            0x009A => self.dungeon_items.ganons_castle.insert(DungeonItems::BOSS_KEY), // Boss Key (Ganons Castle)
            0x009B => self.dungeon_items.deku_tree.insert(DungeonItems::COMPASS), // Compass (Deku Tree)
            0x009C => self.dungeon_items.dodongos_cavern.insert(DungeonItems::COMPASS), // Compass (Dodongos Cavern)
            0x009D => self.dungeon_items.jabu_jabu.insert(DungeonItems::COMPASS), // Compass (Jabu Jabus Belly)
            0x009E => self.dungeon_items.forest_temple.insert(DungeonItems::COMPASS), // Compass (Forest Temple)
            0x009F => self.dungeon_items.fire_temple.insert(DungeonItems::COMPASS), // Compass (Fire Temple)
            0x00A0 => self.dungeon_items.water_temple.insert(DungeonItems::COMPASS), // Compass (Water Temple)
            0x00A1 => self.dungeon_items.spirit_temple.insert(DungeonItems::COMPASS), // Compass (Spirit Temple)
            0x00A2 => self.dungeon_items.shadow_temple.insert(DungeonItems::COMPASS), // Compass (Shadow Temple)
            0x00A3 => self.dungeon_items.bottom_of_the_well.insert(DungeonItems::COMPASS), // Compass (Bottom of the Well)
            0x00A4 => self.dungeon_items.ice_cavern.insert(DungeonItems::COMPASS), // Compass (Ice Cavern)
            0x00A5 => self.dungeon_items.deku_tree.insert(DungeonItems::MAP), // Map (Deku Tree)
            0x00A6 => self.dungeon_items.dodongos_cavern.insert(DungeonItems::MAP), // Map (Dodongos Cavern)
            0x00A7 => self.dungeon_items.jabu_jabu.insert(DungeonItems::MAP), // Map (Jabu Jabus Belly)
            0x00A8 => self.dungeon_items.forest_temple.insert(DungeonItems::MAP), // Map (Forest Temple)
            0x00A9 => self.dungeon_items.fire_temple.insert(DungeonItems::MAP), // Map (Fire Temple)
            0x00AA => self.dungeon_items.water_temple.insert(DungeonItems::MAP), // Map (Water Temple)
            0x00AB => self.dungeon_items.spirit_temple.insert(DungeonItems::MAP), // Map (Spirit Temple)
            0x00AC => self.dungeon_items.shadow_temple.insert(DungeonItems::MAP), // Map (Shadow Temple)
            0x00AD => self.dungeon_items.bottom_of_the_well.insert(DungeonItems::MAP), // Map (Bottom of the Well)
            0x00AE => self.dungeon_items.ice_cavern.insert(DungeonItems::MAP), // Map (Ice Cavern)
            0x00AF => self.small_keys.forest_temple += 1, // Small Key (Forest Temple)
            0x00B0 => self.small_keys.fire_temple += 1, // Small Key (Fire Temple)
            0x00B1 => self.small_keys.water_temple += 1, // Small Key (Water Temple)
            0x00B2 => self.small_keys.spirit_temple += 1, // Small Key (Spirit Temple)
            0x00B3 => self.small_keys.shadow_temple += 1, // Small Key (Shadow Temple)
            0x00B4 => self.small_keys.bottom_of_the_well += 1, // Small Key (Bottom of the Well)
            0x00B5 => self.small_keys.gerudo_training_ground += 1, // Small Key (Gerudo Training Ground)
            0x00B6 => self.small_keys.thieves_hideout += 1, // Small Key (Thieves Hideout)
            0x00B7 => self.small_keys.ganons_castle += 1, // Small Key (Ganons Castle)
            0x00B8 => {} // Double Defense
            0x00BB => self.quest_items.insert(QuestItems::MINUET_OF_FOREST), // Minuet of Forest
            0x00BC => self.quest_items.insert(QuestItems::BOLERO_OF_FIRE), // Bolero of Fire
            0x00BD => self.quest_items.insert(QuestItems::SERENADE_OF_WATER), // Serenade of Water
            0x00BE => self.quest_items.insert(QuestItems::REQUIEM_OF_SPIRIT), // Requiem of Spirit
            0x00BF => self.quest_items.insert(QuestItems::NOCTURNE_OF_SHADOW), // Nocturne of Shadow
            0x00C0 => self.quest_items.insert(QuestItems::PRELUDE_OF_LIGHT), // Prelude of Light
            0x00C1 => self.quest_items.insert(QuestItems::ZELDAS_LULLABY), // Zeldas Lullaby
            0x00C2 => self.quest_items.insert(QuestItems::EPONAS_SONG), // Eponas Song
            0x00C3 => self.quest_items.insert(QuestItems::SARIAS_SONG), // Sarias Song
            0x00C4 => self.quest_items.insert(QuestItems::SUNS_SONG), // Suns Song
            0x00C5 => self.quest_items.insert(QuestItems::SONG_OF_TIME), // Song of Time
            0x00C6 => self.quest_items.insert(QuestItems::SONG_OF_STORMS), // Song of Storms
            0x00C9 => self.inv.beans = true, // Magic Bean Pack
            0x00CA => self.set_triforce_pieces(self.triforce_pieces() + 1), // Triforce Piece
            0x00CB => self.small_keys.forest_temple = 10, // Small Key Ring (Forest Temple)
            0x00CC => self.small_keys.fire_temple = 10, // Small Key Ring (Fire Temple)
            0x00CD => self.small_keys.water_temple = 10, // Small Key Ring (Water Temple)
            0x00CE => self.small_keys.spirit_temple = 10, // Small Key Ring (Spirit Temple)
            0x00CF => self.small_keys.shadow_temple = 10, // Small Key Ring (Shadow Temple)
            0x00D0 => self.small_keys.bottom_of_the_well = 10, // Small Key Ring (Bottom of the Well)
            0x00D1 => self.small_keys.gerudo_training_ground = 10, // Small Key Ring (Gerudo Training Ground)
            0x00D2 => self.small_keys.thieves_hideout = 10, // Small Key Ring (Thieves Hideout)
            0x00D3 => self.small_keys.ganons_castle = 10, // Small Key Ring (Ganons Castle)
            0x00D4 => { // Bombchu Bag (20)
                self.inv.bombchus = true;
                self.inv_amounts.bombchus = 50.min(self.inv_amounts.bombchus + 20);
            }
            0x00D5 => { // Bombchu Bag (10)
                self.inv.bombchus = true;
                self.inv_amounts.bombchus = 50.min(self.inv_amounts.bombchus + 10);
            }
            0x00D6 => { // Bombchu Bag (5)
                self.inv.bombchus = true;
                self.inv_amounts.bombchus = 50.min(self.inv_amounts.bombchus + 5);
            }
            0x00D7 => self.small_keys.treasure_chest_game = 10, // Small Key Ring (Treasure Chest Game)
            0x00D8 => {} // Silver Rupee (Dodongos Cavern Staircase)
            0x00D9 => {} // Silver Rupee (Ice Cavern Spinning Scythe)
            0x00DA => {} // Silver Rupee (Ice Cavern Push Block)
            0x00DB => {} // Silver Rupee (Bottom of the Well Basement)
            0x00DC => {} // Silver Rupee (Shadow Temple Scythe Shortcut)
            0x00DD => {} // Silver Rupee (Shadow Temple Invisible Blades)
            0x00DE => {} // Silver Rupee (Shadow Temple Huge Pit)
            0x00DF => {} // Silver Rupee (Shadow Temple Invisible Spikes)
            0x00E0 => {} // Silver Rupee (Gerudo Training Ground Slopes)
            0x00E1 => {} // Silver Rupee (Gerudo Training Ground Lava)
            0x00E2 => {} // Silver Rupee (Gerudo Training Ground Water)
            0x00E3 => {} // Silver Rupee (Spirit Temple Child Early Torches)
            0x00E4 => {} // Silver Rupee (Spirit Temple Adult Boulders)
            0x00E5 => {} // Silver Rupee (Spirit Temple Lobby and Lower Adult)
            0x00E6 => {} // Silver Rupee (Spirit Temple Sun Block)
            0x00E7 => {} // Silver Rupee (Spirit Temple Adult Climb)
            0x00E8 => {} // Silver Rupee (Ganons Castle Spirit Trial)
            0x00E9 => {} // Silver Rupee (Ganons Castle Light Trial)
            0x00EA => {} // Silver Rupee (Ganons Castle Fire Trial)
            0x00EB => {} // Silver Rupee (Ganons Castle Shadow Trial)
            0x00EC => {} // Silver Rupee (Ganons Castle Water Trial)
            0x00ED => {} // Silver Rupee (Ganons Castle Forest Trial)
            0x00EE => {} // Silver Rupee Pouch (Dodongos Cavern Staircase)
            0x00EF => {} // Silver Rupee Pouch (Ice Cavern Spinning Scythe)
            0x00F0 => {} // Silver Rupee Pouch (Ice Cavern Push Block)
            0x00F1 => {} // Silver Rupee Pouch (Bottom of the Well Basement)
            0x00F2 => {} // Silver Rupee Pouch (Shadow Temple Scythe Shortcut)
            0x00F3 => {} // Silver Rupee Pouch (Shadow Temple Invisible Blades)
            0x00F4 => {} // Silver Rupee Pouch (Shadow Temple Huge Pit)
            0x00F5 => {} // Silver Rupee Pouch (Shadow Temple Invisible Spikes)
            0x00F6 => {} // Silver Rupee Pouch (Gerudo Training Ground Slopes)
            0x00F7 => {} // Silver Rupee Pouch (Gerudo Training Ground Lava)
            0x00F8 => {} // Silver Rupee Pouch (Gerudo Training Ground Water)
            0x00F9 => {} // Silver Rupee Pouch (Spirit Temple Child Early Torches)
            0x00FA => {} // Silver Rupee Pouch (Spirit Temple Adult Boulders)
            0x00FB => {} // Silver Rupee Pouch (Spirit Temple Lobby and Lower Adult)
            0x00FC => {} // Silver Rupee Pouch (Spirit Temple Sun Block)
            0x00FD => {} // Silver Rupee Pouch (Spirit Temple Adult Climb)
            0x00FE => {} // Silver Rupee Pouch (Ganons Castle Spirit Trial)
            0x00FF => {} // Silver Rupee Pouch (Ganons Castle Light Trial)
            0x0100 => {} // Silver Rupee Pouch (Ganons Castle Fire Trial)
            0x0101 => {} // Silver Rupee Pouch (Ganons Castle Shadow Trial)
            0x0102 => {} // Silver Rupee Pouch (Ganons Castle Water Trial)
            0x0103 => {} // Silver Rupee Pouch (Ganons Castle Forest Trial)
            0x0104 => {} // Ocarina A
            0x0105 => {} // Ocarina C up
            0x0106 => {} // Ocarina C down
            0x0107 => {} // Ocarina C left
            0x0108 => {} // Ocarina C right
            0x0109 => self.dungeon_items.forest_temple.insert(DungeonItems::BOSS_KEY), // Boss Key (Forest Temple)
            0x010A => self.dungeon_items.fire_temple.insert(DungeonItems::BOSS_KEY), // Boss Key (Fire Temple)
            0x010B => self.dungeon_items.water_temple.insert(DungeonItems::BOSS_KEY), // Boss Key (Water Temple)
            0x010C => self.dungeon_items.spirit_temple.insert(DungeonItems::BOSS_KEY), // Boss Key (Spirit Temple)
            0x010D => self.dungeon_items.shadow_temple.insert(DungeonItems::BOSS_KEY), // Boss Key (Shadow Temple)
            0x010E => self.dungeon_items.ganons_castle.insert(DungeonItems::BOSS_KEY), // Boss Key (Ganons Castle)
            0x010F => self.small_keys.forest_temple += 1, // Small Key (Forest Temple)
            0x0110 => self.small_keys.fire_temple += 1, // Small Key (Fire Temple)
            0x0111 => self.small_keys.water_temple += 1, // Small Key (Water Temple)
            0x0112 => self.small_keys.spirit_temple += 1, // Small Key (Spirit Temple)
            0x0113 => self.small_keys.shadow_temple += 1, // Small Key (Shadow Temple)
            0x0114 => self.small_keys.bottom_of_the_well += 1, // Small Key (Bottom of the Well)
            0x0115 => self.small_keys.gerudo_training_ground += 1, // Small Key (Gerudo Training Ground)
            0x0116 => self.small_keys.thieves_hideout += 1, // Small Key (Thieves Hideout)
            0x0117 => self.small_keys.ganons_castle += 1, // Small Key (Ganons Castle)
            0x0118 => self.small_keys.treasure_chest_game += 1, // Small Key (Treasure Chest Game)
            0x0119 => {} // Fairy
            0x011A => {} // Nothing :)
            0x011B => {} // Stalfos Soul
            0x011C => {} // Octorok Soul
            0x011D => {} // Wallmaster Soul
            0x011E => {} // Dodongo Soul
            0x011F => {} // Keese Soul
            0x0120 => {} // Tektite Soul
            0x0121 => {} // Peahat Soul
            0x0122 => {} // Lizalfos and Dinalfos Soul
            0x0123 => {} // Gohma Larvae Soul
            0x0124 => {} // Shabom Soul
            0x0125 => {} // Baby Dodongo Soul
            0x0126 => {} // Biri and Bari Soul
            0x0127 => {} // Tailpasaran Soul
            0x0128 => {} // Skulltula Soul
            0x0129 => {} // Torch Slug Soul
            0x012A => {} // Moblin Soul
            0x012B => {} // Armos Soul
            0x012C => {} // Deku Baba Soul
            0x012D => {} // Deku Scrub Soul
            0x012E => {} // Bubble Soul
            0x012F => {} // Beamos Soul
            0x0130 => {} // Floormaster Soul
            0x0131 => {} // Redead and Gibdo Soul
            0x0132 => {} // Skullwalltula Soul
            0x0133 => {} // Flare Dancer Soul
            0x0134 => {} // Dead hand Soul
            0x0135 => {} // Shell blade Soul
            0x0136 => {} // Like-like Soul
            0x0137 => {} // Spike Enemy Soul
            0x0138 => {} // Anubis Soul
            0x0139 => {} // Iron Knuckle Soul
            0x013A => {} // Skull Kid Soul
            0x013B => {} // Flying Pot Soul
            0x013C => {} // Freezard Soul
            0x013D => {} // Stinger Soul
            0x013E => {} // Wolfos Soul
            0x013F => {} // Guay Soul
            0x0140 => {} // Queen Gohma Soul
            0x0141 => {} // King Dodongo Soul
            0x0142 => {} // Barinade Soul
            0x0143 => {} // Phantom Ganon Soul
            0x0144 => {} // Volvagia Soul
            0x0145 => {} // Morpha Soul
            0x0146 => {} // Bongo Bongo Soul
            0x0147 => {} // Twinrova Soul
            0x0148 => {} // Jabu Jabu Tentacle Soul
            0x0149 => {} // Dark Link Soul
            0x1000 => self.set_triforce_pieces(self.triforce_pieces() + 1), // Easter Egg (Pink)
            0x1001 => self.set_triforce_pieces(self.triforce_pieces() + 1), // Easter Egg (Orange)
            0x1002 => self.set_triforce_pieces(self.triforce_pieces() + 1), // Easter Egg (Green)
            0x1003 => self.set_triforce_pieces(self.triforce_pieces() + 1), // Easter Egg (Blue)
            0x1004 => self.set_triforce_pieces(self.triforce_pieces() + 1), // Triforce of Power
            0x1005 => self.set_triforce_pieces(self.triforce_pieces() + 1), // Triforce of Wisdom
            0x1006 => self.set_triforce_pieces(self.triforce_pieces() + 1), // Triforce of Courage
            0x1007 => self.skull_tokens += 1, // Gold Skulltula Token (normal text)
            0x1008 => self.skull_tokens += 1, // Gold Skulltula Token (big chest, normal text)
            0x1009 => {} // Fairy
            0x100A => {} // Nothing :)
            0x100B => self.quest_items.insert(QuestItems::KOKIRI_EMERALD), // Kokiri Emerald
            0x100C => self.quest_items.insert(QuestItems::GORON_RUBY), // Goron Ruby
            0x100D => self.quest_items.insert(QuestItems::ZORA_SAPPHIRE), // Zora Sapphire
            0x100E => self.quest_items.insert(QuestItems::LIGHT_MEDALLION), // Light Medallion
            0x100F => self.quest_items.insert(QuestItems::FOREST_MEDALLION), // Forest Medallion
            0x1010 => self.quest_items.insert(QuestItems::FIRE_MEDALLION), // Fire Medallion
            0x1011 => self.quest_items.insert(QuestItems::WATER_MEDALLION), // Water Medallion
            0x1012 => self.quest_items.insert(QuestItems::SHADOW_MEDALLION), // Shadow Medallion
            0x1013 => self.quest_items.insert(QuestItems::SPIRIT_MEDALLION), // Spirit Medallion
            0x1014 => { // Forest Temple Key Ring (with boss key)
                self.small_keys.forest_temple = 10;
                self.dungeon_items.forest_temple.insert(DungeonItems::BOSS_KEY);
            }
            0x1015 => { // Fire Temple Key Ring (with boss key)
                self.small_keys.fire_temple = 10;
                self.dungeon_items.fire_temple.insert(DungeonItems::BOSS_KEY);
            }
            0x1016 => { // Water Temple Key Ring (with boss key)
                self.small_keys.water_temple = 10;
                self.dungeon_items.water_temple.insert(DungeonItems::BOSS_KEY);
            }
            0x1017 => { // Spirit Temple Key Ring (with boss key)
                self.small_keys.spirit_temple = 10;
                self.dungeon_items.spirit_temple.insert(DungeonItems::BOSS_KEY);
            }
            0x1018 => { // Shadow Temple Key Ring (with boss key)
                self.small_keys.shadow_temple = 10;
                self.dungeon_items.shadow_temple.insert(DungeonItems::BOSS_KEY);
            }
            0x1019 => self.inv.ice_arrows = true, // Blue Fire Arrow
            0x101A => self.skull_tokens += 1, // Gold Skulltula Token (big chest)
            0x101B => {} // Heart Container (big chest)
            0x101C => {} // Piece of Heart (big chest)
            0x101D => {} // Piece of Heart (Chest Game) (big chest)
            0x101E => self.equipment.insert(Equipment::DEKU_SHIELD), // Deku Shield (big chest)
            0x101F => self.equipment.insert(Equipment::HYLIAN_SHIELD), // Hylian Shield (big chest)
            0x1020 => { // Bombchu (5) (big chest)
                self.inv.bombchus = true;
                self.inv_amounts.bombchus = 50.min(self.inv_amounts.bombchus + 5);
            }
            0x1021 => { // Bombchu (10) (big chest)
                self.inv.bombchus = true;
                self.inv_amounts.bombchus = 50.min(self.inv_amounts.bombchus + 10);
            }
            0x1022 => { // Bombchu (20) (big chest)
                self.inv.bombchus = true;
                self.inv_amounts.bombchus = 50.min(self.inv_amounts.bombchus + 20);
            }
            0x1023 => self.upgrades.set_nut_capacity(match self.upgrades.nut_capacity() { // Progressive Nut Capacity (big chest)
                Upgrades::DEKU_NUT_CAPACITY_20 => Upgrades::DEKU_NUT_CAPACITY_30,
                Upgrades::DEKU_NUT_CAPACITY_30 | Upgrades::DEKU_NUT_CAPACITY_40 => Upgrades::DEKU_NUT_CAPACITY_40,
                _ => Upgrades::DEKU_NUT_CAPACITY_20,
            }),
            0x1024 => self.upgrades.set_stick_capacity(match self.upgrades.stick_capacity() { // Progressive Stick Capacity (big chest)
                Upgrades::DEKU_STICK_CAPACITY_10 => Upgrades::DEKU_STICK_CAPACITY_20,
                Upgrades::DEKU_STICK_CAPACITY_20 | Upgrades::DEKU_STICK_CAPACITY_30 => Upgrades::DEKU_STICK_CAPACITY_30,
                _ => Upgrades::DEKU_STICK_CAPACITY_10,
            }),
            0x2000 => {} // Stalfos Soul
            0x2001 => {} // Octorok Soul
            0x2002 => {} // Wallmaster Soul
            0x2003 => {} // Dodongo Soul
            0x2004 => {} // Keese Soul
            0x2005 => {} // Tektite Soul
            0x2006 => {} // Peahat Soul
            0x2007 => {} // Lizalfos and Dinalfos Soul
            0x2008 => {} // Gohma Larvae Soul
            0x2009 => {} // Shabom Soul
            0x200A => {} // Baby Dodongo Soul
            0x200B => {} // Biri and Bari Soul
            0x200C => {} // Tailpasaran Soul
            0x200D => {} // Skulltula Soul
            0x200E => {} // Torch Slug Soul
            0x200F => {} // Moblin Soul
            0x2010 => {} // Armos Soul
            0x2011 => {} // Deku Baba Soul
            0x2012 => {} // Deku Scrub Soul
            0x2013 => {} // Bubble Soul
            0x2014 => {} // Beamos Soul
            0x2015 => {} // Floormaster Soul
            0x2016 => {} // Redead and Gibdo Soul
            0x2017 => {} // Skullwalltula Soul
            0x2018 => {} // Flare Dancer Soul
            0x2019 => {} // Dead hand Soul
            0x201A => {} // Shell blade Soul
            0x201B => {} // Like-like Soul
            0x201C => {} // Spike Enemy Soul
            0x201D => {} // Anubis Soul
            0x201E => {} // Iron Knuckle Soul
            0x201F => {} // Skull Kid Soul
            0x2020 => {} // Flying Pot Soul
            0x2021 => {} // Freezard Soul
            0x2022 => {} // Stinger Soul
            0x2023 => {} // Wolfos Soul
            0x2024 => {} // Guay Soul
            0x2025 => {} // Queen Gohma Soul
            0x2026 => {} // King Dodongo Soul
            0x2027 => {} // Barinade Soul
            0x2028 => {} // Phantom Ganon Soul
            0x2029 => {} // Volvagia Soul
            0x202A => {} // Morpha Soul
            0x202B => {} // Bongo Bongo Soul
            0x202C => {} // Twinrova Soul
            0x202D => {} // Jabu Jabu Tentacle Soul
            0x202E => {} // Dark Link Soul
            _ => return Err(()),
        }
        Ok(())
    }
}

impl Protocol for Save {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Save, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let mut buf = vec![0; SIZE];
            stream.read_exact(&mut buf).await?;
            Ok(Save::from_save_data(&buf).map_err(|e| ReadError::Custom(format!("failed to decode save data: {e:?}")))?)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            let buf = self.to_save_data();
            assert_eq!(buf.len(), SIZE);
            sink.write_all(&buf).await?;
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let mut buf = vec![0; SIZE];
        stream.read_exact(&mut buf)?;
        Ok(Save::from_save_data(&buf).map_err(|e| ReadError::Custom(format!("failed to decode save data: {e:?}")))?)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        let buf = self.to_save_data();
        assert_eq!(buf.len(), SIZE);
        sink.write_all(&buf)?;
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
#[derive(Debug, Clone, Protocol)]
pub struct Delta(Vec<(u16, u8)>);
