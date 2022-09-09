use {
    std::{
        fmt,
        str::FromStr,
    },
    async_proto::Protocol,
    enum_iterator::Sequence,
    quote_value::QuoteValue,
    serde::{
        Deserialize,
        Serialize,
    },
    serde_plain::{
        derive_deserialize_from_fromstr,
        derive_serialize_from_display,
    },
    crate::item::Item,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Protocol, QuoteValue)]
pub enum Dungeon {
    Main(MainDungeon),
    IceCavern,
    BottomOfTheWell,
    GerudoTrainingGround,
    GanonsCastle,
}

impl Dungeon {
    pub fn rando_name(&self) -> &'static str {
        match self {
            Self::Main(MainDungeon::DekuTree) => "Deku Tree",
            Self::Main(MainDungeon::DodongosCavern) => "Dodongos Cavern",
            Self::Main(MainDungeon::JabuJabu) => "Jabu Jabus Belly",
            Self::Main(MainDungeon::ForestTemple) => "Forest Temple",
            Self::Main(MainDungeon::FireTemple) => "Fire Temple",
            Self::Main(MainDungeon::WaterTemple) => "Water Temple",
            Self::Main(MainDungeon::ShadowTemple) => "Shadow Temple",
            Self::Main(MainDungeon::SpiritTemple) => "Spirit Temple",
            Self::IceCavern => "Ice Cavern",
            Self::BottomOfTheWell => "Bottom of the Well",
            Self::GerudoTrainingGround => "Gerudo Training Ground",
            Self::GanonsCastle => "Ganons Castle",
        }
    }
}

impl FromStr for Dungeon {
    type Err = ();

    fn from_str(s: &str) -> Result<Dungeon, ()> {
        MainDungeon::from_str(s).map(Dungeon::Main).or_else(|_| match s {
            "Ice Cavern" => Ok(Dungeon::IceCavern),
            "Bottom of the Well" => Ok(Dungeon::BottomOfTheWell),
            "Gerudo Training Ground" | "Gerudo Training Grounds" => Ok(Dungeon::GerudoTrainingGround),
            "Ganon's Castle" | "Ganons Castle" => Ok(Dungeon::GanonsCastle),
            _ => Err(()),
        })
    }
}

impl fmt::Display for Dungeon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Dungeon::Main(main) => main.fmt(f),
            Dungeon::IceCavern => write!(f, "Ice Cavern"),
            Dungeon::BottomOfTheWell => write!(f, "Bottom of the Well"),
            Dungeon::GerudoTrainingGround => write!(f, "Gerudo Training Ground"),
            Dungeon::GanonsCastle => write!(f, "Ganon's Castle"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sequence, Protocol)]
pub enum DungeonReward {
    Medallion(Medallion),
    Stone(Stone),
}

impl FromStr for DungeonReward {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(match s.parse() {
            Ok(med) => Self::Medallion(med),
            Err(()) => Self::Stone(s.parse()?),
        })
    }
}

impl TryFrom<Item> for DungeonReward {
    type Error = ();

    fn try_from(item: Item) -> Result<Self, ()> {
        item.0.parse()
    }
}

impl fmt::Display for DungeonReward {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Medallion(med) => med.fmt(f),
            Self::Stone(stone) => stone.fmt(f),
        }
    }
}

derive_deserialize_from_fromstr!(DungeonReward, "dungeon reward");
derive_serialize_from_display!(DungeonReward);

impl From<DungeonReward> for Item {
    fn from(reward: DungeonReward) -> Self {
        Self(reward.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Protocol)]
pub enum DungeonRewardLocation {
    LinksPocket,
    Dungeon(MainDungeon),
}

impl DungeonRewardLocation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LinksPocket => "Links Pocket",
            Self::Dungeon(dungeon) => dungeon.reward_location(),
        }
    }
}

impl FromStr for DungeonRewardLocation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(if s == "Links Pocket" {
            Self::LinksPocket
        } else {
            Self::Dungeon(MainDungeon::from_reward_location(s).ok_or(())?)
        })
    }
}

impl fmt::Display for DungeonRewardLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

derive_deserialize_from_fromstr!(DungeonRewardLocation, "dungeon reward location");
derive_serialize_from_display!(DungeonRewardLocation);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sequence, Protocol, QuoteValue)]
pub enum MainDungeon {
    DekuTree,
    DodongosCavern,
    JabuJabu,
    ForestTemple,
    FireTemple,
    WaterTemple,
    ShadowTemple,
    SpiritTemple,
}

impl MainDungeon {
    pub fn from_reward_location(loc: &str) -> Option<Self> {
        match loc {
            "Queen Gohma" => Some(Self::DekuTree),
            "King Dodongo" => Some(Self::DodongosCavern),
            "Barinade" => Some(Self::JabuJabu),
            "Phantom Ganon" => Some(Self::ForestTemple),
            "Volvagia" => Some(Self::FireTemple),
            "Morpha" => Some(Self::WaterTemple),
            "Bongo Bongo" => Some(Self::ShadowTemple),
            "Twinrova" => Some(Self::SpiritTemple),
            _ => None,
        }
    }

    pub fn reward_location(&self) -> &'static str {
        match self {
            Self::DekuTree => "Queen Gohma",
            Self::DodongosCavern => "King Dodongo",
            Self::JabuJabu => "Barinade",
            Self::ForestTemple => "Phantom Ganon",
            Self::FireTemple => "Volvagia",
            Self::WaterTemple => "Morpha",
            Self::ShadowTemple => "Bongo Bongo",
            Self::SpiritTemple => "Twinrova",
        }
    }
}

impl FromStr for MainDungeon {
    type Err = ();

    fn from_str(s: &str) -> Result<MainDungeon, ()> {
        match s {
            "Deku Tree" => Ok(MainDungeon::DekuTree),
            "Dodongo's Cavern" | "Dodongos Cavern" => Ok(MainDungeon::DodongosCavern),
            "Jabu-Jabu" | "Jabu Jabus Belly" => Ok(MainDungeon::JabuJabu),
            "Forest Temple" => Ok(MainDungeon::ForestTemple),
            "Fire Temple" => Ok(MainDungeon::FireTemple),
            "Water Temple" => Ok(MainDungeon::WaterTemple),
            "Shadow Temple" => Ok(MainDungeon::ShadowTemple),
            "Spirit Temple" => Ok(MainDungeon::SpiritTemple),
            _ => Err(()),
        }
    }
}

impl fmt::Display for MainDungeon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MainDungeon::DekuTree => write!(f, "Deku Tree"),
            MainDungeon::DodongosCavern => write!(f, "Dodongo's Cavern"),
            MainDungeon::JabuJabu => write!(f, "Jabu-Jabu"),
            MainDungeon::ForestTemple => write!(f, "Forest Temple"),
            MainDungeon::FireTemple => write!(f, "Fire Temple"),
            MainDungeon::WaterTemple => write!(f, "Water Temple"),
            MainDungeon::ShadowTemple => write!(f, "Shadow Temple"),
            MainDungeon::SpiritTemple => write!(f, "Spirit Temple"),
        }
    }
}

derive_deserialize_from_fromstr!(MainDungeon, "main dungeon");
derive_serialize_from_display!(MainDungeon);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sequence, Protocol, Deserialize, Serialize, QuoteValue)]
pub enum Medallion {
    Light,
    Forest,
    Fire,
    Water,
    Shadow,
    Spirit,
}

impl Medallion {
    /// Returns the medallion's element, e.g. `"Light"` for the Light Medallion.
    pub fn element(&self) -> &'static str {
        match self {
            Medallion::Light => "Light",
            Medallion::Forest => "Forest",
            Medallion::Fire => "Fire",
            Medallion::Water => "Water",
            Medallion::Shadow => "Shadow",
            Medallion::Spirit => "Spirit",
        }
    }
}

impl FromStr for Medallion {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(match s {
            "Light Medallion" => Self::Light,
            "Forest Medallion" => Self::Forest,
            "Fire Medallion" => Self::Fire,
            "Water Medallion" => Self::Water,
            "Shadow Medallion" => Self::Shadow,
            "Spirit Medallion" => Self::Spirit,
            _ => return Err(()),
        })
    }
}

impl TryFrom<Item> for Medallion {
    type Error = ();

    fn try_from(item: Item) -> Result<Self, ()> {
        item.0.parse()
    }
}

impl fmt::Display for Medallion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} Medallion", self.element())
    }
}

impl From<Medallion> for Item {
    fn from(med: Medallion) -> Self {
        Self(med.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sequence, Protocol)]
pub enum Stone {
    KokiriEmerald,
    GoronRuby,
    ZoraSapphire,
}

impl FromStr for Stone {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(match s {
            "Kokiri Emerald" => Self::KokiriEmerald,
            "Goron Ruby" => Self::GoronRuby,
            "Zora Sapphire" => Self::ZoraSapphire,
            _ => return Err(()),
        })
    }
}

impl TryFrom<Item> for Stone {
    type Error = ();

    fn try_from(item: Item) -> Result<Self, ()> {
        item.0.parse()
    }
}

impl fmt::Display for Stone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KokiriEmerald => write!(f, "Kokiri Emerald"),
            Self::GoronRuby => write!(f, "Goron Ruby"),
            Self::ZoraSapphire => write!(f, "Zora Sapphire"),
        }
    }
}

impl From<Stone> for Item {
    fn from(stone: Stone) -> Self {
        Self(stone.to_string())
    }
}

#[derive(Debug, Clone, Copy, QuoteValue)]
pub enum TimeRange {
    /// 06:00–18:00.
    ///
    /// Playing Sun's Song during `Night` sets the time to 12:00.
    Day,
    /// 18:00–06:00.
    ///
    /// Playing Sun's Song during `Day` sets the time to 00:00.
    Night,
    /// The time of day when Dampé's Heart-Pounding Gravedigging Tour is available: 18:00–21:00, a subset of `Night`.
    ///
    /// Going to outside Ganon's Castle sets the time to 18:01.
    Dampe,
}
