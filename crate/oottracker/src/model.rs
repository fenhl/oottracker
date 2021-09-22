use {
    std::{
        convert::TryFrom,
        fmt,
        str::FromStr,
    },
    async_proto::Protocol,
    enum_iterator::IntoEnumIterator,
    crate::item::Item,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoEnumIterator, Protocol)]
pub enum Dungeon {
    Main(MainDungeon),
    IceCavern,
    BottomOfTheWell,
    GerudoTrainingGround,
    GanonsCastle,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoEnumIterator, Protocol)]
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

    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "Deku Tree" => Ok(Self::DekuTree),
            "Dodongo's Cavern" | "Dodongos Cavern" => Ok(Self::DodongosCavern),
            "Jabu-Jabu" | "Jabu Jabus Belly" => Ok(Self::JabuJabu),
            "Forest Temple" => Ok(Self::ForestTemple),
            "Fire Temple" => Ok(Self::FireTemple),
            "Water Temple" => Ok(Self::WaterTemple),
            "Shadow Temple" => Ok(Self::ShadowTemple),
            "Spirit Temple" => Ok(Self::SpiritTemple),
            _ => Err(()),
        }
    }
}

impl fmt::Display for MainDungeon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DekuTree => write!(f, "Deku Tree"),
            Self::DodongosCavern => write!(f, "Dodongo's Cavern"),
            Self::JabuJabu => write!(f, "Jabu-Jabu"),
            Self::ForestTemple => write!(f, "Forest Temple"),
            Self::FireTemple => write!(f, "Fire Temple"),
            Self::WaterTemple => write!(f, "Water Temple"),
            Self::ShadowTemple => write!(f, "Shadow Temple"),
            Self::SpiritTemple => write!(f, "Spirit Temple"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoEnumIterator, Protocol)]
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
            Self::Light => "Light",
            Self::Forest => "Forest",
            Self::Fire => "Fire",
            Self::Water => "Water",
            Self::Shadow => "Shadow",
            Self::Spirit => "Spirit",
        }
    }
}

impl TryFrom<Item> for Medallion {
    type Error = ();

    fn try_from(item: Item) -> Result<Self, ()> {
        match item {
            Item::LightMedallion => Ok(Self::Light),
            Item::ForestMedallion => Ok(Self::Forest),
            Item::FireMedallion => Ok(Self::Fire),
            Item::WaterMedallion => Ok(Self::Water),
            Item::ShadowMedallion => Ok(Self::Shadow),
            Item::SpiritMedallion => Ok(Self::Spirit),
            _ => Err(()),
        }
    }
}

impl From<Medallion> for Item {
    fn from(med: Medallion) -> Self {
        match med {
            Medallion::Light => Self::LightMedallion,
            Medallion::Forest => Self::ForestMedallion,
            Medallion::Fire => Self::FireMedallion,
            Medallion::Water => Self::WaterMedallion,
            Medallion::Shadow => Self::ShadowMedallion,
            Medallion::Spirit => Self::SpiritMedallion,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoEnumIterator)]
pub enum Stone {
    KokiriEmerald,
    GoronRuby,
    ZoraSapphire,
}

impl TryFrom<Item> for Stone {
    type Error = ();

    fn try_from(item: Item) -> Result<Self, ()> {
        match item {
            Item::KokiriEmerald => Ok(Self::KokiriEmerald),
            Item::GoronRuby => Ok(Self::GoronRuby),
            Item::ZoraSapphire => Ok(Self::ZoraSapphire),
            _ => Err(()),
        }
    }
}

impl From<Stone> for Item {
    fn from(stone: Stone) -> Self {
        match stone {
            Stone::KokiriEmerald => Self::KokiriEmerald,
            Stone::GoronRuby => Self::GoronRuby,
            Stone::ZoraSapphire => Self::ZoraSapphire,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoEnumIterator)]
pub enum DungeonReward {
    Medallion(Medallion),
    Stone(Stone),
}

impl TryFrom<Item> for DungeonReward {
    type Error = ();

    fn try_from(item: Item) -> Result<Self, ()> {
        Ok(match Medallion::try_from(item) {
            Ok(med) => Self::Medallion(med),
            Err(()) => Self::Stone(Stone::try_from(item)?),
        })
    }
}

impl From<DungeonReward> for Item {
    fn from(reward: DungeonReward) -> Self {
        match reward {
            DungeonReward::Medallion(med) => med.into(),
            DungeonReward::Stone(stone) => stone.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoEnumIterator)]
pub enum DungeonRewardLocation {
    LinksPocket,
    Dungeon(MainDungeon),
}

impl DungeonRewardLocation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LinksPocket => "Links Pocket",
            Self::Dungeon(MainDungeon::DekuTree) => "Queen Gohma",
            Self::Dungeon(MainDungeon::DodongosCavern) => "King Dodongo",
            Self::Dungeon(MainDungeon::JabuJabu) => "Barinade",
            Self::Dungeon(MainDungeon::ForestTemple) => "Phantom Ganon",
            Self::Dungeon(MainDungeon::FireTemple) => "Volvagia",
            Self::Dungeon(MainDungeon::WaterTemple) => "Morpha",
            Self::Dungeon(MainDungeon::ShadowTemple) => "Bongo Bongo",
            Self::Dungeon(MainDungeon::SpiritTemple) => "Twinrova",
        }
    }
}

impl fmt::Display for DungeonRewardLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

#[derive(Debug, Clone, Copy)]
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
