use {
    std::{
        convert::{
            TryFrom,
            TryInto as _,
        },
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
        event_chk_inf::EventChkInf,
        item_ids,
    },
};
#[cfg(not(target_arch = "wasm32"))] use {
    std::io::prelude::*,
    async_trait::async_trait,
    tokio::{
        net::TcpStream,
        prelude::*,
    },
    crate::proto::Protocol,
};

pub const SIZE: usize = 0x1450;

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
    pub bottles: u8, //TODO Ruto's letter
    pub adult_trade_item: AdultTradeItem,
    pub child_trade_item: ChildTradeItem,
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

        macro_rules! bottles {
            ($($offset:literal),+) => {{
                0 $(+ {
                    let raw_item = *raw_data.get($offset).ok_or_else(|| raw_data.clone())?;
                    if raw_item >= item_ids::EMPTY_BOTTLE && raw_item <= item_ids::POE {
                        1
                    } else if raw_item == item_ids::NONE {
                        0
                    } else {
                        return Err(raw_data)
                    }
                })+
            }};
        }

        Ok(Inventory {
            bow: bool_item!(0x03, item_ids::BOW),
            fire_arrows: bool_item!(0x04, item_ids::FIRE_ARROWS),
            dins_fire: bool_item!(0x05, item_ids::DINS_FIRE),
            slingshot: bool_item!(0x06, item_ids::SLINGSHOT),
            ocarina: bool_item!(0x07, item_ids::FAIRY_OCARINA | item_ids::OCARINA_OF_TIME),
            bombchus: bool_item!(0x08, item_ids::BOMBCHU_10),
            hookshot: Hookshot::try_from(*raw_data.get(0x09).ok_or_else(|| raw_data.clone())?).map_err(|_| raw_data.clone())?,
            ice_arrows: bool_item!(0x0a, item_ids::ICE_ARROWS),
            farores_wind: bool_item!(0x0b, item_ids::FARORES_WIND),
            boomerang: bool_item!(0x0c, item_ids::BOOMERANG),
            lens: bool_item!(0x0d, item_ids::LENS_OF_TRUTH),
            beans: bool_item!(0x0e, item_ids::MAGIC_BEAN),
            hammer: bool_item!(0x0f, item_ids::MEGATON_HAMMER),
            light_arrows: bool_item!(0x10, item_ids::LIGHT_ARROWS),
            nayrus_love: bool_item!(0x11, item_ids::NAYRUS_LOVE),
            bottles: bottles!(0x12, 0x13, 0x14, 0x15),
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

        macro_rules! bottle {
            ($min_count:literal) => {{
                if inv.bottles >= $min_count { item_ids::EMPTY_BOTTLE } else { item_ids::NONE }
            }};
        }

        [
            item_ids::NONE, item_ids::NONE, item_ids::NONE, bool_item!(bow, item_ids::BOW), bool_item!(fire_arrows, item_ids::FIRE_ARROWS), bool_item!(dins_fire, item_ids::DINS_FIRE),
            bool_item!(slingshot, item_ids::SLINGSHOT), bool_item!(ocarina, item_ids::FAIRY_OCARINA), bool_item!(bombchus, item_ids::BOMBCHU_10), inv.hookshot.into(), bool_item!(ice_arrows, item_ids::ICE_ARROWS), bool_item!(farores_wind, item_ids::FARORES_WIND),
            bool_item!(boomerang, item_ids::BOOMERANG), bool_item!(lens, item_ids::LENS_OF_TRUTH), bool_item!(beans, item_ids::MAGIC_BEAN), bool_item!(hammer, item_ids::MEGATON_HAMMER), bool_item!(light_arrows, item_ids::LIGHT_ARROWS), bool_item!(nayrus_love, item_ids::NAYRUS_LOVE),
            bottle!(1), bottle!(2), bottle!(3), bottle!(4), inv.adult_trade_item.into(), inv.child_trade_item.into(),
        ]
    }    
}

impl<'a> From<&'a Inventory> for Vec<u8> {
    fn from(inv: &Inventory) -> Vec<u8> {
        <[u8; 0x18]>::from(inv).into()
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

#[derive(Debug, From, Clone)]
pub enum SaveDataDecodeError {
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
    pub magic: MagicCapacity,
    pub inv: Inventory,
    pub equipment: Equipment,
    pub upgrades: Upgrades,
    pub quest_items: QuestItems,
    pub skull_tokens: u8,
    pub triforce_pieces: u8,
    pub event_chk_inf: EventChkInf,
}

impl Save {
    /// Converts *Ocarina of Time* save data into a `Save`.
    ///
    /// # Panics
    ///
    /// This method may panic if `save_data`'s size is less than `0x1450` bytes, or if it doesn't contain valid OoT save data.
    pub fn from_save_data(save_data: &[u8]) -> Result<Save, SaveDataDecodeError> {
        macro_rules! get_offset {
            ($name:expr, $offset:expr) => {{
                *save_data.get($offset).ok_or(SaveDataDecodeError::Index($offset))?
            }};
            ($name:expr, $offset:expr, $len:expr) => {{
                save_data.get($offset..$offset + $len).ok_or(SaveDataDecodeError::IndexRange { start: $offset, end: $offset + $len })?
            }};
        }

        macro_rules! try_get_offset {
            ($name:expr, $offset:expr) => {{
                let raw = *save_data.get($offset).ok_or(SaveDataDecodeError::Index($offset))?;
                raw.try_into().map_err(|value| SaveDataDecodeError::UnexpectedValue { value, offset: $offset, field: $name })?
            }};
            ($name:expr, $offset:expr, $len:expr) => {{
                let raw = save_data.get($offset..$offset + $len).ok_or(SaveDataDecodeError::IndexRange { start: $offset, end: $offset + $len })?.to_vec();
                raw.try_into().map_err(|value| SaveDataDecodeError::UnexpectedValueRange { value, start: $offset, end: $offset + $len, field: $name })?
            }};
        }

        macro_rules! try_eq {
            ($offset:literal, $val:expr) => {{
                let expected = $val;
                let found = *save_data.get($offset).ok_or(SaveDataDecodeError::Index($offset))?;
                if expected != found { return Err(SaveDataDecodeError::AssertEq { expected, found, offset: $offset }) }
            }};
            ($start:literal..$end:literal, $val:expr) => {{
                let expected = $val;
                let found = save_data.get($start..$end).ok_or(SaveDataDecodeError::IndexRange { start: $start, end: $end })?;
                if expected != found { return Err(SaveDataDecodeError::AssertEqRange { start: $start, end: $end, expected: expected.to_vec(), found: found.to_vec() }) }
            }};
        }

        if save_data.len() != SIZE { return Err(SaveDataDecodeError::Size(save_data.len())) }
        try_eq!(0x001c..0x0022, b"ZELDAZ");
        Ok(Save {
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
            equipment: try_get_offset!("equipment", 0x009c, 0x2),
            upgrades: try_get_offset!("upgrades", 0x00a0, 0x4),
            quest_items: try_get_offset!("quest_items", 0x00a4, 0x4),
            skull_tokens: BigEndian::read_i16(get_offset!("skull_tokens", 0x00d0, 0x2)).try_into()?,
            triforce_pieces: BigEndian::read_i32(get_offset!("triforce_pieces", 0x00d4 + 0x48 * 0x1c + 0x10, 0x4)).try_into()?, // unused scene flag in scene 0x48
            event_chk_inf: try_get_offset!("event_chk_inf", 0x0ed4, 0x1c),
        })
    }

    fn to_save_data(&self) -> Vec<u8> {
        let mut buf = vec![0; SIZE];
        let Save { magic, inv, equipment, upgrades, quest_items, skull_tokens, triforce_pieces, event_chk_inf } = self;
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
        buf.splice(0x009c..0x009e, Vec::from(equipment));
        buf.splice(0x00a0..0x00a4, Vec::from(upgrades));
        buf.splice(0x00a4..0x00a8, Vec::from(quest_items));
        buf.splice(0x00d0..0x00d2, i16::from(*skull_tokens).to_be_bytes().iter().copied());
        buf.splice(0x00d4 + 0x48 * 0x1c + 0x10..0x00d4 + 0x48 * 0x1c + 0x10 + 0x4, i32::from(*triforce_pieces).to_be_bytes().iter().copied()); // unused scene flag in scene 0x48
        buf.splice(0x0ed4..0x0ef0, Vec::from(event_chk_inf));
        buf
    }
}

#[derive(Debug, From, Clone)]
pub enum SaveDataReadError {
    #[from]
    Decode(SaveDataDecodeError),
    Io(Arc<io::Error>),
}

impl From<io::Error> for SaveDataReadError {
    fn from(e: io::Error) -> SaveDataReadError {
        SaveDataReadError::Io(Arc::new(e))
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
