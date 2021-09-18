use {
    std::{
        fmt,
        str::FromStr,
    },
    async_proto::Protocol,
    enum_iterator::IntoEnumIterator,
    quote_value::QuoteValue,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Protocol, QuoteValue)]
pub enum Dungeon {
    Main(MainDungeon),
    IceCavern,
    BottomOfTheWell,
    GerudoTrainingGrounds,
    GanonsCastle,
}

impl FromStr for Dungeon {
    type Err = ();

    fn from_str(s: &str) -> Result<Dungeon, ()> {
        MainDungeon::from_str(s).map(Dungeon::Main).or_else(|_| match s {
            "Ice Cavern" => Ok(Dungeon::IceCavern),
            "Bottom of the Well" => Ok(Dungeon::BottomOfTheWell),
            "Gerudo Training Ground" | "Gerudo Training Grounds" => Ok(Dungeon::GerudoTrainingGrounds),
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
            Dungeon::GerudoTrainingGrounds => write!(f, "Gerudo Training Grounds"),
            Dungeon::GanonsCastle => write!(f, "Ganon's Castle"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoEnumIterator, Protocol)]
pub enum DungeonReward {
    Medallion(Medallion),
    Stone(Stone),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoEnumIterator, Protocol)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoEnumIterator, Protocol, QuoteValue)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoEnumIterator, Protocol, QuoteValue)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoEnumIterator, Protocol)]
pub enum Stone {
    KokiriEmerald,
    GoronRuby,
    ZoraSapphire,
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
