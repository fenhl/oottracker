#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use {
    std::fmt,
    iced::{
        Application,
        Background,
        Color,
        Command,
        Element,
        Length,
        Settings,
        Subscription,
        widget::{
            Column,
            Image,
            Row,
            Text,
            button::{
                self,
                Button,
            },
            container::{
                self,
                Container,
            },
        },
        window,
    },
    structopt::StructOpt,
    oottracker::{
        checks::checked,
        info_tables::*,
        knowledge::*,
        save::*,
    },
};
#[cfg(not(target_arch = "wasm32"))] use {
    std::time::Duration,
    iced::image,
    tokio::time::delay_for,
    oottracker::proto::{
        self,
        Packet,
    },
};

#[cfg(not(target_arch = "wasm32"))] mod tcp_server;

macro_rules! embed_image {
    ($path:expr) => {{
        #[cfg(not(target_arch = "wasm32"))] {
            Image::new(image::Handle::from_memory(include_bytes!(concat!("../../../assets/", $path)).to_vec()))
        }
        #[cfg(target_arch = "wasm32")] {
            Image::new(concat!("assets/", $path))
        }
    }};
}

const WIDTH: u32 = 50 * 6 + 7; // 6 images, each 50px wide, plus 1px spacing
const HEIGHT: u32 = 18 + 50 * 7 + 9; // dungeon reward location text, 18px high, and 7 images, each 50px high, plus 1px spacing

struct ContainerStyle;

impl container::StyleSheet for ContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::BLACK)),
            ..container::Style::default()
        }
    }
}

macro_rules! cells {
    ($($cell:ident,)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        enum TrackerCell {
            $(
                $cell,
            )*
        }

        #[allow(non_snake_case)]
        #[derive(Debug, Default)]
        struct CellButtons {
            $(
                $cell: button::State,
            )*
        }
    }
}

cells! {
    LightMedallionLocation,
    ForestMedallionLocation,
    FireMedallionLocation,
    WaterMedallionLocation,
    ShadowMedallionLocation,
    SpiritMedallionLocation,
    LightMedallion,
    ForestMedallion,
    FireMedallion,
    WaterMedallion,
    ShadowMedallion,
    SpiritMedallion,
    AdultTrade,
    Skulltula,
    KokiriEmeraldLocation,
    KokiriEmerald,
    GoronRubyLocation,
    GoronRuby,
    ZoraSapphireLocation,
    ZoraSapphire,
    Bottle,
    Scale,
    Slingshot,
    Bombs,
    Boomerang,
    Strength,
    Magic,
    Spells,
    Hookshot,
    Bow,
    Arrows,
    Hammer,
    Boots,
    MirrorShield,
    ChildTrade,
    Ocarina,
    Beans,
    SwordCard,
    Tunics,
    Triforce,
    ZeldasLullaby,
    EponasSong,
    SariasSong,
    SunsSong,
    SongOfTime,
    SongOfStorms,
    Minuet,
    Bolero,
    Serenade,
    Requiem,
    Nocturne,
    Prelude,
}

impl TrackerCell {
    fn left_click(&self, state: &mut ModelState) {
        match self {
            TrackerCell::LightMedallionLocation => state.knowledge.light_medallion_location = match state.knowledge.light_medallion_location {
                DungeonRewardLocation::Unknown => DungeonRewardLocation::DekuTree,
                DungeonRewardLocation::DekuTree => DungeonRewardLocation::DodongosCavern,
                DungeonRewardLocation::DodongosCavern => DungeonRewardLocation::JabuJabu,
                DungeonRewardLocation::JabuJabu => DungeonRewardLocation::ForestTemple,
                DungeonRewardLocation::ForestTemple => DungeonRewardLocation::FireTemple,
                DungeonRewardLocation::FireTemple => DungeonRewardLocation::WaterTemple,
                DungeonRewardLocation::WaterTemple => DungeonRewardLocation::ShadowTemple,
                DungeonRewardLocation::ShadowTemple => DungeonRewardLocation::SpiritTemple,
                DungeonRewardLocation::SpiritTemple => DungeonRewardLocation::LinksPocket,
                DungeonRewardLocation::LinksPocket => DungeonRewardLocation::Unknown,
            },
            TrackerCell::ForestMedallionLocation => state.knowledge.forest_medallion_location = match state.knowledge.forest_medallion_location {
                DungeonRewardLocation::Unknown => DungeonRewardLocation::DekuTree,
                DungeonRewardLocation::DekuTree => DungeonRewardLocation::DodongosCavern,
                DungeonRewardLocation::DodongosCavern => DungeonRewardLocation::JabuJabu,
                DungeonRewardLocation::JabuJabu => DungeonRewardLocation::ForestTemple,
                DungeonRewardLocation::ForestTemple => DungeonRewardLocation::FireTemple,
                DungeonRewardLocation::FireTemple => DungeonRewardLocation::WaterTemple,
                DungeonRewardLocation::WaterTemple => DungeonRewardLocation::ShadowTemple,
                DungeonRewardLocation::ShadowTemple => DungeonRewardLocation::SpiritTemple,
                DungeonRewardLocation::SpiritTemple => DungeonRewardLocation::LinksPocket,
                DungeonRewardLocation::LinksPocket => DungeonRewardLocation::Unknown,
            },
            TrackerCell::FireMedallionLocation => state.knowledge.fire_medallion_location = match state.knowledge.fire_medallion_location {
                DungeonRewardLocation::Unknown => DungeonRewardLocation::DekuTree,
                DungeonRewardLocation::DekuTree => DungeonRewardLocation::DodongosCavern,
                DungeonRewardLocation::DodongosCavern => DungeonRewardLocation::JabuJabu,
                DungeonRewardLocation::JabuJabu => DungeonRewardLocation::ForestTemple,
                DungeonRewardLocation::ForestTemple => DungeonRewardLocation::FireTemple,
                DungeonRewardLocation::FireTemple => DungeonRewardLocation::WaterTemple,
                DungeonRewardLocation::WaterTemple => DungeonRewardLocation::ShadowTemple,
                DungeonRewardLocation::ShadowTemple => DungeonRewardLocation::SpiritTemple,
                DungeonRewardLocation::SpiritTemple => DungeonRewardLocation::LinksPocket,
                DungeonRewardLocation::LinksPocket => DungeonRewardLocation::Unknown,
            },
            TrackerCell::WaterMedallionLocation => state.knowledge.water_medallion_location = match state.knowledge.water_medallion_location {
                DungeonRewardLocation::Unknown => DungeonRewardLocation::DekuTree,
                DungeonRewardLocation::DekuTree => DungeonRewardLocation::DodongosCavern,
                DungeonRewardLocation::DodongosCavern => DungeonRewardLocation::JabuJabu,
                DungeonRewardLocation::JabuJabu => DungeonRewardLocation::ForestTemple,
                DungeonRewardLocation::ForestTemple => DungeonRewardLocation::FireTemple,
                DungeonRewardLocation::FireTemple => DungeonRewardLocation::WaterTemple,
                DungeonRewardLocation::WaterTemple => DungeonRewardLocation::ShadowTemple,
                DungeonRewardLocation::ShadowTemple => DungeonRewardLocation::SpiritTemple,
                DungeonRewardLocation::SpiritTemple => DungeonRewardLocation::LinksPocket,
                DungeonRewardLocation::LinksPocket => DungeonRewardLocation::Unknown,
            },
            TrackerCell::ShadowMedallionLocation => state.knowledge.shadow_medallion_location = match state.knowledge.shadow_medallion_location {
                DungeonRewardLocation::Unknown => DungeonRewardLocation::DekuTree,
                DungeonRewardLocation::DekuTree => DungeonRewardLocation::DodongosCavern,
                DungeonRewardLocation::DodongosCavern => DungeonRewardLocation::JabuJabu,
                DungeonRewardLocation::JabuJabu => DungeonRewardLocation::ForestTemple,
                DungeonRewardLocation::ForestTemple => DungeonRewardLocation::FireTemple,
                DungeonRewardLocation::FireTemple => DungeonRewardLocation::WaterTemple,
                DungeonRewardLocation::WaterTemple => DungeonRewardLocation::ShadowTemple,
                DungeonRewardLocation::ShadowTemple => DungeonRewardLocation::SpiritTemple,
                DungeonRewardLocation::SpiritTemple => DungeonRewardLocation::LinksPocket,
                DungeonRewardLocation::LinksPocket => DungeonRewardLocation::Unknown,
            },
            TrackerCell::SpiritMedallionLocation => state.knowledge.spirit_medallion_location = match state.knowledge.spirit_medallion_location {
                DungeonRewardLocation::Unknown => DungeonRewardLocation::DekuTree,
                DungeonRewardLocation::DekuTree => DungeonRewardLocation::DodongosCavern,
                DungeonRewardLocation::DodongosCavern => DungeonRewardLocation::JabuJabu,
                DungeonRewardLocation::JabuJabu => DungeonRewardLocation::ForestTemple,
                DungeonRewardLocation::ForestTemple => DungeonRewardLocation::FireTemple,
                DungeonRewardLocation::FireTemple => DungeonRewardLocation::WaterTemple,
                DungeonRewardLocation::WaterTemple => DungeonRewardLocation::ShadowTemple,
                DungeonRewardLocation::ShadowTemple => DungeonRewardLocation::SpiritTemple,
                DungeonRewardLocation::SpiritTemple => DungeonRewardLocation::LinksPocket,
                DungeonRewardLocation::LinksPocket => DungeonRewardLocation::Unknown,
            },
            TrackerCell::LightMedallion => state.save.quest_items.toggle(QuestItems::LIGHT_MEDALLION),
            TrackerCell::ForestMedallion => state.save.quest_items.toggle(QuestItems::FOREST_MEDALLION),
            TrackerCell::FireMedallion => state.save.quest_items.toggle(QuestItems::FIRE_MEDALLION),
            TrackerCell::WaterMedallion => state.save.quest_items.toggle(QuestItems::WATER_MEDALLION),
            TrackerCell::ShadowMedallion => state.save.quest_items.toggle(QuestItems::SHADOW_MEDALLION),
            TrackerCell::SpiritMedallion => state.save.quest_items.toggle(QuestItems::SPIRIT_MEDALLION),
            TrackerCell::AdultTrade => state.save.inv.adult_trade_item = match state.save.inv.adult_trade_item {
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
            },
            TrackerCell::Skulltula => if state.save.skull_tokens == 100 { state.save.skull_tokens = 0 } else { state.save.skull_tokens += 1 },
            TrackerCell::KokiriEmeraldLocation => state.knowledge.kokiri_emerald_location = match state.knowledge.kokiri_emerald_location {
                DungeonRewardLocation::Unknown => DungeonRewardLocation::DekuTree,
                DungeonRewardLocation::DekuTree => DungeonRewardLocation::DodongosCavern,
                DungeonRewardLocation::DodongosCavern => DungeonRewardLocation::JabuJabu,
                DungeonRewardLocation::JabuJabu => DungeonRewardLocation::ForestTemple,
                DungeonRewardLocation::ForestTemple => DungeonRewardLocation::FireTemple,
                DungeonRewardLocation::FireTemple => DungeonRewardLocation::WaterTemple,
                DungeonRewardLocation::WaterTemple => DungeonRewardLocation::ShadowTemple,
                DungeonRewardLocation::ShadowTemple => DungeonRewardLocation::SpiritTemple,
                DungeonRewardLocation::SpiritTemple => DungeonRewardLocation::LinksPocket,
                DungeonRewardLocation::LinksPocket => DungeonRewardLocation::Unknown,
            },
            TrackerCell::KokiriEmerald => state.save.quest_items.toggle(QuestItems::KOKIRI_EMERALD),
            TrackerCell::GoronRubyLocation => state.knowledge.goron_ruby_location = match state.knowledge.goron_ruby_location {
                DungeonRewardLocation::Unknown => DungeonRewardLocation::DekuTree,
                DungeonRewardLocation::DekuTree => DungeonRewardLocation::DodongosCavern,
                DungeonRewardLocation::DodongosCavern => DungeonRewardLocation::JabuJabu,
                DungeonRewardLocation::JabuJabu => DungeonRewardLocation::ForestTemple,
                DungeonRewardLocation::ForestTemple => DungeonRewardLocation::FireTemple,
                DungeonRewardLocation::FireTemple => DungeonRewardLocation::WaterTemple,
                DungeonRewardLocation::WaterTemple => DungeonRewardLocation::ShadowTemple,
                DungeonRewardLocation::ShadowTemple => DungeonRewardLocation::SpiritTemple,
                DungeonRewardLocation::SpiritTemple => DungeonRewardLocation::LinksPocket,
                DungeonRewardLocation::LinksPocket => DungeonRewardLocation::Unknown,
            },
            TrackerCell::GoronRuby => state.save.quest_items.toggle(QuestItems::GORON_RUBY),
            TrackerCell::ZoraSapphireLocation => state.knowledge.zora_sapphire_location = match state.knowledge.zora_sapphire_location {
                DungeonRewardLocation::Unknown => DungeonRewardLocation::DekuTree,
                DungeonRewardLocation::DekuTree => DungeonRewardLocation::DodongosCavern,
                DungeonRewardLocation::DodongosCavern => DungeonRewardLocation::JabuJabu,
                DungeonRewardLocation::JabuJabu => DungeonRewardLocation::ForestTemple,
                DungeonRewardLocation::ForestTemple => DungeonRewardLocation::FireTemple,
                DungeonRewardLocation::FireTemple => DungeonRewardLocation::WaterTemple,
                DungeonRewardLocation::WaterTemple => DungeonRewardLocation::ShadowTemple,
                DungeonRewardLocation::ShadowTemple => DungeonRewardLocation::SpiritTemple,
                DungeonRewardLocation::SpiritTemple => DungeonRewardLocation::LinksPocket,
                DungeonRewardLocation::LinksPocket => DungeonRewardLocation::Unknown,
            },
            TrackerCell::ZoraSapphire => state.save.quest_items.toggle(QuestItems::ZORA_SAPPHIRE),
            TrackerCell::Bottle => state.save.inv.bottles = if state.save.inv.bottles == 0 { 1 } else { 0 }, //TODO Ruto's Letter support
            TrackerCell::Scale => state.save.upgrades.set_scale(match state.save.upgrades.scale() {
                Upgrades::SILVER_SCALE => Upgrades::GOLD_SCALE,
                Upgrades::GOLD_SCALE => Upgrades::NONE,
                _ => Upgrades::SILVER_SCALE,
            }),
            TrackerCell::Slingshot => state.save.inv.slingshot = !state.save.inv.slingshot,
            TrackerCell::Bombs => if state.save.upgrades.bomb_bag() != Upgrades::NONE {
                state.save.upgrades.set_bomb_bag(Upgrades::NONE);
                state.save.inv.bombchus = !state.save.inv.bombchus;
            } else {
                state.save.upgrades.set_bomb_bag(Upgrades::BOMB_BAG);
            },
            TrackerCell::Boomerang => state.save.inv.boomerang = !state.save.inv.boomerang,
            TrackerCell::Strength => state.save.upgrades.set_strength(match state.save.upgrades.strength() {
                Upgrades::GORON_BRACELET => Upgrades::SILVER_GAUNTLETS,
                Upgrades::SILVER_GAUNTLETS => Upgrades::GOLD_GAUNTLETS,
                Upgrades::GOLD_GAUNTLETS => Upgrades::NONE,
                _ => Upgrades::GORON_BRACELET,
            }),
            TrackerCell::Magic => if state.save.magic != MagicCapacity::None {
                state.save.magic = MagicCapacity::None;
                state.save.inv.lens = !state.save.inv.lens;
            } else {
                state.save.magic = MagicCapacity::Small;
            },
            TrackerCell::Spells => if state.save.inv.dins_fire {
                state.save.inv.dins_fire = false;
                state.save.inv.farores_wind = !state.save.inv.farores_wind;
            } else {
                state.save.inv.dins_fire = true;
            },
            TrackerCell::Hookshot => state.save.inv.hookshot = match state.save.inv.hookshot {
                Hookshot::None => Hookshot::Hookshot,
                Hookshot::Hookshot => Hookshot::Longshot,
                Hookshot::Longshot => Hookshot::None,
            },
            TrackerCell::Bow => if state.save.inv.bow {
                state.save.inv.bow = false;
                state.save.inv.ice_arrows = !state.save.inv.ice_arrows;
            } else {
                state.save.inv.bow = true;
            },
            TrackerCell::Arrows => if state.save.inv.fire_arrows {
                state.save.inv.fire_arrows = false;
                state.save.inv.light_arrows = !state.save.inv.light_arrows;
            } else {
                state.save.inv.fire_arrows = true;
            },
            TrackerCell::Hammer => state.save.inv.hammer = !state.save.inv.hammer,
            TrackerCell::Boots => if state.save.equipment.contains(Equipment::IRON_BOOTS) {
                state.save.equipment.remove(Equipment::IRON_BOOTS);
                state.save.equipment.toggle(Equipment::HOVER_BOOTS);
            } else {
                state.save.equipment.insert(Equipment::IRON_BOOTS);
            },
            TrackerCell::MirrorShield => state.save.equipment.toggle(Equipment::MIRROR_SHIELD),
            TrackerCell::ChildTrade => state.save.inv.child_trade_item = match state.save.inv.child_trade_item {
                ChildTradeItem::None => ChildTradeItem::WeirdEgg,
                ChildTradeItem::WeirdEgg => ChildTradeItem::Chicken,
                ChildTradeItem::Chicken => ChildTradeItem::ZeldasLetter,
                ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => ChildTradeItem::KeatonMask, //TODO for SOLD OUT, check trade quest progress
                ChildTradeItem::KeatonMask => ChildTradeItem::SkullMask,
                ChildTradeItem::SkullMask => ChildTradeItem::SpookyMask,
                ChildTradeItem::SpookyMask => ChildTradeItem::BunnyHood,
                ChildTradeItem::BunnyHood => ChildTradeItem::MaskOfTruth,
                ChildTradeItem::MaskOfTruth => ChildTradeItem::None,
            },
            TrackerCell::Ocarina => if state.save.inv.ocarina {
                state.save.inv.ocarina = false;
                state.save.event_chk_inf.9.toggle(EventChkInf9::SCARECROW_SONG);
            } else {
                state.save.inv.ocarina = true;
            },
            TrackerCell::Beans => state.save.inv.beans = !state.save.inv.beans,
            TrackerCell::SwordCard => if state.save.equipment.contains(Equipment::KOKIRI_SWORD) {
                state.save.equipment.remove(Equipment::KOKIRI_SWORD);
                state.save.quest_items.toggle(QuestItems::GERUDO_CARD);
            } else {
                state.save.equipment.insert(Equipment::KOKIRI_SWORD);
            },
            TrackerCell::Tunics => if state.save.equipment.contains(Equipment::GORON_TUNIC) {
                state.save.equipment.remove(Equipment::GORON_TUNIC);
                state.save.equipment.toggle(Equipment::ZORA_TUNIC);
            } else {
                state.save.equipment.insert(Equipment::GORON_TUNIC);
            },
            TrackerCell::Triforce => state.save.set_triforce_pieces(if state.save.triforce_pieces() == 100 { 0 } else { state.save.triforce_pieces() + 1 }),
            TrackerCell::ZeldasLullaby => state.save.quest_items.toggle(QuestItems::ZELDAS_LULLABY),
            TrackerCell::EponasSong => state.save.quest_items.toggle(QuestItems::EPONAS_SONG),
            TrackerCell::SariasSong => state.save.quest_items.toggle(QuestItems::SARIAS_SONG),
            TrackerCell::SunsSong => state.save.quest_items.toggle(QuestItems::SUNS_SONG),
            TrackerCell::SongOfTime => state.save.quest_items.toggle(QuestItems::SONG_OF_TIME),
            TrackerCell::SongOfStorms => state.save.quest_items.toggle(QuestItems::SONG_OF_STORMS),
            TrackerCell::Minuet => state.save.quest_items.toggle(QuestItems::MINUET_OF_FOREST),
            TrackerCell::Bolero => state.save.quest_items.toggle(QuestItems::BOLERO_OF_FIRE),
            TrackerCell::Serenade => state.save.quest_items.toggle(QuestItems::SERENADE_OF_WATER),
            TrackerCell::Requiem => state.save.quest_items.toggle(QuestItems::REQUIEM_OF_SPIRIT),
            TrackerCell::Nocturne => state.save.quest_items.toggle(QuestItems::NOCTURNE_OF_SHADOW),
            TrackerCell::Prelude => state.save.quest_items.toggle(QuestItems::PRELUDE_OF_LIGHT),
        }
    }

    fn view<'a>(&self, state: &ModelState, cell_button: Option<&'a mut button::State>) -> Element<'a, Message> {
        macro_rules! xopar_image {
            (@count_inner $filename:ident $count:expr, $($n:literal),*) => {{
                match $count {
                    $(
                        $n => embed_image!(concat!("xopar-images-count/", stringify!($filename), "_", stringify!($n), ".png")),
                    )*
                    _ => unreachable!(),
                }
            }};
            ($filename:ident) => {{
                embed_image!(concat!("xopar-images/", stringify!($filename), ".png"))
            }};
            (count = $count:expr, $filename:ident) => {{
                xopar_image!(@count_inner $filename $count,
                    1, 2, 3, 4, 5, 6, 7, 8, 9,
                    10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
                    20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
                    30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
                    40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
                    50, 51, 52, 53, 54, 55, 56, 57, 58, 59,
                    60, 61, 62, 63, 64, 65, 66, 67, 68, 69,
                    70, 71, 72, 73, 74, 75, 76, 77, 78, 79,
                    80, 81, 82, 83, 84, 85, 86, 87, 88, 89,
                    90, 91, 92, 93, 94, 95, 96, 97, 98, 99,
                    100
                )
            }};
            (dimmed $filename:ident) => {{
                embed_image!(concat!("xopar-images-dimmed/", stringify!($filename), ".png"))
            }};
            (undim = $undim:expr, $filename:ident) => {{
                if $undim {
                    xopar_image!($filename)
                } else {
                    xopar_image!(dimmed $filename)
                }
            }};
            (undim = $undim:expr, $filename:ident, overlay = $overlay:expr, $overlay_filename:ident) => {{
                match ($undim, $overlay) {
                    (false, false) => xopar_image!(dimmed $filename),
                    (false, true) => xopar_image!(overlay_dimmed $overlay_filename),
                    (true, false) => xopar_image!($filename),
                    (true, true) => xopar_image!(overlay $overlay_filename),
                }
            }};
            (overlay $filename:ident) => {{
                embed_image!(concat!("xopar-images-overlay/", stringify!($filename), ".png"))
            }};
            (overlay_dimmed $filename:ident) => {{
                embed_image!(concat!("xopar-images-overlay-dimmed/", stringify!($filename), ".png"))
            }};
            (composite = $left:expr, $left_filename:ident, $right:expr, $right_filename:ident, $composite_filename:ident) => {{
                match ($left, $right) {
                    (false, false) => xopar_image!(dimmed $composite_filename),
                    (false, true) => xopar_image!($right_filename),
                    (true, false) => xopar_image!($left_filename),
                    (true, true) => xopar_image!($composite_filename),
                }
            }};
        }

        let content = match self {
            TrackerCell::LightMedallionLocation => match state.knowledge.light_medallion_location {
                DungeonRewardLocation::Unknown => xopar_image!(unknown_text),
                DungeonRewardLocation::DekuTree => xopar_image!(deku_text),
                DungeonRewardLocation::DodongosCavern => xopar_image!(dc_text),
                DungeonRewardLocation::JabuJabu => xopar_image!(jabu_text),
                DungeonRewardLocation::ForestTemple => xopar_image!(forest_text),
                DungeonRewardLocation::FireTemple => xopar_image!(fire_text),
                DungeonRewardLocation::WaterTemple => xopar_image!(water_text),
                DungeonRewardLocation::ShadowTemple => xopar_image!(shadow_text),
                DungeonRewardLocation::SpiritTemple => xopar_image!(spirit_text),
                DungeonRewardLocation::LinksPocket => xopar_image!(free_text),
            }.width(Length::Units(50)),
            TrackerCell::ForestMedallionLocation => match state.knowledge.forest_medallion_location {
                DungeonRewardLocation::Unknown => xopar_image!(unknown_text),
                DungeonRewardLocation::DekuTree => xopar_image!(deku_text),
                DungeonRewardLocation::DodongosCavern => xopar_image!(dc_text),
                DungeonRewardLocation::JabuJabu => xopar_image!(jabu_text),
                DungeonRewardLocation::ForestTemple => xopar_image!(forest_text),
                DungeonRewardLocation::FireTemple => xopar_image!(fire_text),
                DungeonRewardLocation::WaterTemple => xopar_image!(water_text),
                DungeonRewardLocation::ShadowTemple => xopar_image!(shadow_text),
                DungeonRewardLocation::SpiritTemple => xopar_image!(spirit_text),
                DungeonRewardLocation::LinksPocket => xopar_image!(free_text),
            }.width(Length::Units(50)),
            TrackerCell::FireMedallionLocation => match state.knowledge.fire_medallion_location {
                DungeonRewardLocation::Unknown => xopar_image!(unknown_text),
                DungeonRewardLocation::DekuTree => xopar_image!(deku_text),
                DungeonRewardLocation::DodongosCavern => xopar_image!(dc_text),
                DungeonRewardLocation::JabuJabu => xopar_image!(jabu_text),
                DungeonRewardLocation::ForestTemple => xopar_image!(forest_text),
                DungeonRewardLocation::FireTemple => xopar_image!(fire_text),
                DungeonRewardLocation::WaterTemple => xopar_image!(water_text),
                DungeonRewardLocation::ShadowTemple => xopar_image!(shadow_text),
                DungeonRewardLocation::SpiritTemple => xopar_image!(spirit_text),
                DungeonRewardLocation::LinksPocket => xopar_image!(free_text),
            }.width(Length::Units(50)),
            TrackerCell::WaterMedallionLocation => match state.knowledge.water_medallion_location {
                DungeonRewardLocation::Unknown => xopar_image!(unknown_text),
                DungeonRewardLocation::DekuTree => xopar_image!(deku_text),
                DungeonRewardLocation::DodongosCavern => xopar_image!(dc_text),
                DungeonRewardLocation::JabuJabu => xopar_image!(jabu_text),
                DungeonRewardLocation::ForestTemple => xopar_image!(forest_text),
                DungeonRewardLocation::FireTemple => xopar_image!(fire_text),
                DungeonRewardLocation::WaterTemple => xopar_image!(water_text),
                DungeonRewardLocation::ShadowTemple => xopar_image!(shadow_text),
                DungeonRewardLocation::SpiritTemple => xopar_image!(spirit_text),
                DungeonRewardLocation::LinksPocket => xopar_image!(free_text),
            }.width(Length::Units(50)),
            TrackerCell::ShadowMedallionLocation => match state.knowledge.shadow_medallion_location {
                DungeonRewardLocation::Unknown => xopar_image!(unknown_text),
                DungeonRewardLocation::DekuTree => xopar_image!(deku_text),
                DungeonRewardLocation::DodongosCavern => xopar_image!(dc_text),
                DungeonRewardLocation::JabuJabu => xopar_image!(jabu_text),
                DungeonRewardLocation::ForestTemple => xopar_image!(forest_text),
                DungeonRewardLocation::FireTemple => xopar_image!(fire_text),
                DungeonRewardLocation::WaterTemple => xopar_image!(water_text),
                DungeonRewardLocation::ShadowTemple => xopar_image!(shadow_text),
                DungeonRewardLocation::SpiritTemple => xopar_image!(spirit_text),
                DungeonRewardLocation::LinksPocket => xopar_image!(free_text),
            }.width(Length::Units(50)),
            TrackerCell::SpiritMedallionLocation => match state.knowledge.spirit_medallion_location {
                DungeonRewardLocation::Unknown => xopar_image!(unknown_text),
                DungeonRewardLocation::DekuTree => xopar_image!(deku_text),
                DungeonRewardLocation::DodongosCavern => xopar_image!(dc_text),
                DungeonRewardLocation::JabuJabu => xopar_image!(jabu_text),
                DungeonRewardLocation::ForestTemple => xopar_image!(forest_text),
                DungeonRewardLocation::FireTemple => xopar_image!(fire_text),
                DungeonRewardLocation::WaterTemple => xopar_image!(water_text),
                DungeonRewardLocation::ShadowTemple => xopar_image!(shadow_text),
                DungeonRewardLocation::SpiritTemple => xopar_image!(spirit_text),
                DungeonRewardLocation::LinksPocket => xopar_image!(free_text),
            }.width(Length::Units(50)),
            TrackerCell::LightMedallion => xopar_image!(undim = state.save.quest_items.contains(QuestItems::LIGHT_MEDALLION), light_medallion),
            TrackerCell::ForestMedallion => xopar_image!(undim = state.save.quest_items.contains(QuestItems::FOREST_MEDALLION), forest_medallion),
            TrackerCell::FireMedallion => xopar_image!(undim = state.save.quest_items.contains(QuestItems::FIRE_MEDALLION), fire_medallion),
            TrackerCell::WaterMedallion => xopar_image!(undim = state.save.quest_items.contains(QuestItems::WATER_MEDALLION), water_medallion),
            TrackerCell::ShadowMedallion => xopar_image!(undim = state.save.quest_items.contains(QuestItems::SHADOW_MEDALLION), shadow_medallion),
            TrackerCell::SpiritMedallion => xopar_image!(undim = state.save.quest_items.contains(QuestItems::SPIRIT_MEDALLION), spirit_medallion),
            TrackerCell::AdultTrade => match state.save.inv.adult_trade_item {
                AdultTradeItem::None => xopar_image!(dimmed blue_egg),
                AdultTradeItem::PocketEgg | AdultTradeItem::PocketCucco => xopar_image!(blue_egg),
                AdultTradeItem::Cojiro => xopar_image!(cojiro),
                AdultTradeItem::OddMushroom => xopar_image!(odd_mushroom),
                AdultTradeItem::OddPotion => xopar_image!(odd_poultice),
                AdultTradeItem::PoachersSaw => xopar_image!(poachers_saw),
                AdultTradeItem::BrokenSword => xopar_image!(broken_sword),
                AdultTradeItem::Prescription => xopar_image!(prescription),
                AdultTradeItem::EyeballFrog => xopar_image!(eyeball_frog),
                AdultTradeItem::Eyedrops => xopar_image!(eye_drops),
                AdultTradeItem::ClaimCheck => xopar_image!(claim_check),
            },
            TrackerCell::Skulltula => if state.save.skull_tokens == 0 { xopar_image!(dimmed golden_skulltula) } else { xopar_image!(count = state.save.skull_tokens, skulls) },
            TrackerCell::KokiriEmeraldLocation => match state.knowledge.kokiri_emerald_location {
                DungeonRewardLocation::Unknown => xopar_image!(unknown_text),
                DungeonRewardLocation::DekuTree => xopar_image!(deku_text),
                DungeonRewardLocation::DodongosCavern => xopar_image!(dc_text),
                DungeonRewardLocation::JabuJabu => xopar_image!(jabu_text),
                DungeonRewardLocation::ForestTemple => xopar_image!(forest_text),
                DungeonRewardLocation::FireTemple => xopar_image!(fire_text),
                DungeonRewardLocation::WaterTemple => xopar_image!(water_text),
                DungeonRewardLocation::ShadowTemple => xopar_image!(shadow_text),
                DungeonRewardLocation::SpiritTemple => xopar_image!(spirit_text),
                DungeonRewardLocation::LinksPocket => xopar_image!(free_text),
            }.width(Length::Units(33)),
            TrackerCell::KokiriEmerald => xopar_image!(undim = state.save.quest_items.contains(QuestItems::KOKIRI_EMERALD), kokiri_emerald).width(Length::Units(33)),
            TrackerCell::GoronRubyLocation => match state.knowledge.goron_ruby_location {
                DungeonRewardLocation::Unknown => xopar_image!(unknown_text),
                DungeonRewardLocation::DekuTree => xopar_image!(deku_text),
                DungeonRewardLocation::DodongosCavern => xopar_image!(dc_text),
                DungeonRewardLocation::JabuJabu => xopar_image!(jabu_text),
                DungeonRewardLocation::ForestTemple => xopar_image!(forest_text),
                DungeonRewardLocation::FireTemple => xopar_image!(fire_text),
                DungeonRewardLocation::WaterTemple => xopar_image!(water_text),
                DungeonRewardLocation::ShadowTemple => xopar_image!(shadow_text),
                DungeonRewardLocation::SpiritTemple => xopar_image!(spirit_text),
                DungeonRewardLocation::LinksPocket => xopar_image!(free_text),
            }.width(Length::Units(33)),
            TrackerCell::GoronRuby => xopar_image!(undim = state.save.quest_items.contains(QuestItems::GORON_RUBY), goron_ruby).width(Length::Units(33)),
            TrackerCell::ZoraSapphireLocation => match state.knowledge.zora_sapphire_location {
                DungeonRewardLocation::Unknown => xopar_image!(unknown_text),
                DungeonRewardLocation::DekuTree => xopar_image!(deku_text),
                DungeonRewardLocation::DodongosCavern => xopar_image!(dc_text),
                DungeonRewardLocation::JabuJabu => xopar_image!(jabu_text),
                DungeonRewardLocation::ForestTemple => xopar_image!(forest_text),
                DungeonRewardLocation::FireTemple => xopar_image!(fire_text),
                DungeonRewardLocation::WaterTemple => xopar_image!(water_text),
                DungeonRewardLocation::ShadowTemple => xopar_image!(shadow_text),
                DungeonRewardLocation::SpiritTemple => xopar_image!(spirit_text),
                DungeonRewardLocation::LinksPocket => xopar_image!(free_text),
            }.width(Length::Units(33)),
            TrackerCell::ZoraSapphire => xopar_image!(undim = state.save.quest_items.contains(QuestItems::ZORA_SAPPHIRE), zora_sapphire).width(Length::Units(33)),
            TrackerCell::Bottle => xopar_image!(undim = state.save.inv.bottles > 0, bottle), //TODO only undim if the bottle can be trivially emptied; Ruto's Letter support
            TrackerCell::Scale => match state.save.upgrades.scale() {
                Upgrades::SILVER_SCALE => xopar_image!(silver_scale),
                Upgrades::GOLD_SCALE => xopar_image!(gold_scale),
                _ => xopar_image!(dimmed silver_scale),
            },
            TrackerCell::Slingshot => xopar_image!(undim = state.save.inv.slingshot, slingshot),
            TrackerCell::Bombs => xopar_image!(undim = state.save.upgrades.bomb_bag() != Upgrades::NONE, bomb_bag, overlay = state.save.inv.bombchus, bomb_bag_bombchu),
            TrackerCell::Boomerang => xopar_image!(undim = state.save.inv.boomerang, boomerang),
            TrackerCell::Strength => match state.save.upgrades.strength() {
                Upgrades::GORON_BRACELET => xopar_image!(goron_bracelet),
                Upgrades::SILVER_GAUNTLETS => xopar_image!(silver_gauntlets),
                Upgrades::GOLD_GAUNTLETS => xopar_image!(gold_gauntlets),
                _ => xopar_image!(dimmed goron_bracelet),
            },
            TrackerCell::Magic => xopar_image!(undim = state.save.magic != MagicCapacity::None, magic, overlay = state.save.inv.lens, magic_lens),
            TrackerCell::Spells => xopar_image!(composite = state.save.inv.dins_fire, dins_fire, state.save.inv.farores_wind, faores_wind, composite_magic),
            TrackerCell::Hookshot => match state.save.inv.hookshot {
                Hookshot::None => xopar_image!(dimmed hookshot),
                Hookshot::Hookshot => xopar_image!(hookshot_accessible),
                Hookshot::Longshot => xopar_image!(longshot_accessible),
            },
            TrackerCell::Bow => xopar_image!(undim = state.save.inv.bow, bow, overlay = state.save.inv.ice_arrows, bow_ice_arrows),
            TrackerCell::Arrows => xopar_image!(composite = state.save.inv.fire_arrows, fire_arrows, state.save.inv.light_arrows, light_arrows, composite_arrows),
            TrackerCell::Hammer => xopar_image!(undim = state.save.inv.hammer, hammer),
            TrackerCell::Boots => xopar_image!(composite = state.save.equipment.contains(Equipment::IRON_BOOTS), iron_boots, state.save.equipment.contains(Equipment::HOVER_BOOTS), hover_boots, composite_boots),
            TrackerCell::MirrorShield => xopar_image!(undim = state.save.equipment.contains(Equipment::MIRROR_SHIELD), mirror_shield),
            TrackerCell::ChildTrade => match state.save.inv.child_trade_item {
                ChildTradeItem::None => xopar_image!(dimmed white_egg),
                ChildTradeItem::WeirdEgg => xopar_image!(white_egg),
                ChildTradeItem::Chicken => xopar_image!(white_chicken),
                ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask | ChildTradeItem::SoldOut => xopar_image!(zelda_letter), //TODO for SOLD OUT, check trade quest progress
                ChildTradeItem::KeatonMask => xopar_image!(keaton_mask),
                ChildTradeItem::SkullMask => xopar_image!(skull_mask),
                ChildTradeItem::SpookyMask => xopar_image!(spooky_mask),
                ChildTradeItem::BunnyHood => xopar_image!(bunny_hood),
                ChildTradeItem::MaskOfTruth => xopar_image!(mask_of_truth),
            },
            TrackerCell::Ocarina => xopar_image!(undim = state.save.inv.ocarina, ocarina, overlay = state.save.event_chk_inf.9.contains(EventChkInf9::SCARECROW_SONG), ocarina_scarecrow), //TODO only show free Scarecrow's Song once it's known (by settings string input or by check)
            TrackerCell::Beans => xopar_image!(undim = state.save.inv.beans, beans), //TODO overlay with number bought if autotracker is on?
            TrackerCell::SwordCard => xopar_image!(composite = state.save.equipment.contains(Equipment::KOKIRI_SWORD), kokiri_sword, state.save.quest_items.contains(QuestItems::GERUDO_CARD), gerudo_card, composite_ksword_gcard),
            TrackerCell::Tunics => xopar_image!(composite = state.save.equipment.contains(Equipment::GORON_TUNIC), goron_tunic, state.save.equipment.contains(Equipment::ZORA_TUNIC), zora_tunic, composite_tunics),
            TrackerCell::Triforce => if state.save.triforce_pieces() == 0 { xopar_image!(dimmed triforce) } else { xopar_image!(count = state.save.triforce_pieces(), force) },
            TrackerCell::ZeldasLullaby => xopar_image!(undim = state.save.quest_items.contains(QuestItems::ZELDAS_LULLABY), lullaby, overlay = checked(&state.save, "Song from Impa").unwrap_or(false), lullaby_check),
            TrackerCell::EponasSong => xopar_image!(undim = state.save.quest_items.contains(QuestItems::EPONAS_SONG), epona, overlay = checked(&state.save, "Song from Malon").unwrap_or(false), epona_check),
            TrackerCell::SariasSong => xopar_image!(undim = state.save.quest_items.contains(QuestItems::SARIAS_SONG), saria, overlay = checked(&state.save, "Song from Saria").unwrap_or(false), saria_check),
            TrackerCell::SunsSong => xopar_image!(undim = state.save.quest_items.contains(QuestItems::SUNS_SONG), sun, overlay = checked(&state.save, "Song from Composers Grave").unwrap_or(false), sun_check),
            TrackerCell::SongOfTime => xopar_image!(undim = state.save.quest_items.contains(QuestItems::SONG_OF_TIME), time, overlay = checked(&state.save, "Song from Ocarina of Time").unwrap_or(false), time_check),
            TrackerCell::SongOfStorms => xopar_image!(undim = state.save.quest_items.contains(QuestItems::SONG_OF_STORMS), storms, overlay = checked(&state.save, "Song from Windmill").unwrap_or(false), storms_check),
            TrackerCell::Minuet => xopar_image!(undim = state.save.quest_items.contains(QuestItems::MINUET_OF_FOREST), minuet, overlay = checked(&state.save, "Sheik in Forest").unwrap_or(false), minuet_check),
            TrackerCell::Bolero => xopar_image!(undim = state.save.quest_items.contains(QuestItems::BOLERO_OF_FIRE), bolero, overlay = checked(&state.save, "Sheik in Crater").unwrap_or(false), bolero_check),
            TrackerCell::Serenade => xopar_image!(undim = state.save.quest_items.contains(QuestItems::SERENADE_OF_WATER), serenade, overlay = checked(&state.save, "Sheik in Ice Cavern").unwrap_or(false), serenade_check),
            TrackerCell::Requiem => xopar_image!(undim = state.save.quest_items.contains(QuestItems::REQUIEM_OF_SPIRIT), requiem, overlay = checked(&state.save, "Sheik at Colossus").unwrap_or(false), requiem_check),
            TrackerCell::Nocturne => xopar_image!(undim = state.save.quest_items.contains(QuestItems::NOCTURNE_OF_SHADOW), nocturne, overlay = checked(&state.save, "Sheik in Kakariko").unwrap_or(false), nocturne_check),
            TrackerCell::Prelude => xopar_image!(undim = state.save.quest_items.contains(QuestItems::PRELUDE_OF_LIGHT), prelude, overlay = checked(&state.save, "Sheik at Temple").unwrap_or(false), prelude_check),
        };
        if let Some(cell_button) = cell_button {
            Button::new(cell_button, content).on_press(Message::LeftClick(*self)).padding(0).style(*self).into()
        } else {
            content.into()
        }
    }
}

impl button::StyleSheet for TrackerCell {
    fn active(&self) -> button::Style { button::Style::default() }
}

#[derive(Debug, Default)]
struct ModelState {
    knowledge: Knowledge,
    save: Save,
}

#[derive(Debug, Clone)]
enum Message {
    #[cfg(not(target_arch = "wasm32"))]
    AutoDismissNotification,
    #[cfg(not(target_arch = "wasm32"))]
    ClientConnected,
    #[cfg(not(target_arch = "wasm32"))]
    ClientDisconnected,
    DismissNotification,
    LeftClick(TrackerCell),
    #[cfg(not(target_arch = "wasm32"))]
    NetworkError(proto::ReadError),
    #[cfg(not(target_arch = "wasm32"))]
    Packet(Packet),
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::ClientConnected => write!(f, "auto-tracker connected"),
            Message::ClientDisconnected => write!(f, "auto-tracker disconnected"),
            Message::NetworkError(e) => write!(f, "network error: {}", e),
            Message::AutoDismissNotification | Message::DismissNotification | Message::LeftClick(_) | Message::Packet(_) => write!(f, "{:?}", self), // these messages are not notifications so just fall back to Debug
        }
    }
}

#[derive(Debug, Default)]
struct State {
    cell_buttons: CellButtons,
    client_connected: bool,
    model: ModelState,
    notification: Option<(bool, Message)>,
    dismiss_notification_button: button::State,
}

impl State {
    /// Adds a visible notification/alert/log message.
    ///
    /// Implemented as a separate method in case the way this is displayed is changed later, e.g. to allow multiple notifications.
    #[cfg(not(target_arch = "wasm32"))]
    fn notify(&mut self, message: Message) {
        self.notification = Some((false, message));
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn notify_temp(&mut self, message: Message) -> Command<Message> {
        self.notification = Some((true, message));
        async { delay_for(Duration::from_secs(10)).await; Message::AutoDismissNotification }.into()
    }
}

impl Application for State {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new((): ()) -> (State, Command<Message>) { (State::default(), Command::none()) }

    fn title(&self) -> String {
        if self.client_connected {
            format!("OoT Tracker (auto-tracker connected)")
        } else {
            format!("OoT Tracker")
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            #[cfg(not(target_arch = "wasm32"))]
            Message::AutoDismissNotification => if let Some((true, _)) = self.notification {
                self.notification = None;
            },
            #[cfg(not(target_arch = "wasm32"))]
            Message::ClientConnected => {
                self.client_connected = true;
                return self.notify_temp(message)
            }
            #[cfg(not(target_arch = "wasm32"))]
            Message::ClientDisconnected => {
                self.client_connected = false;
                self.notify(message);
            }
            Message::DismissNotification => self.notification = None,
            Message::LeftClick(cell) => cell.left_click(&mut self.model),
            #[cfg(not(target_arch = "wasm32"))]
            Message::NetworkError(_) => self.notify(message),
            #[cfg(not(target_arch = "wasm32"))]
            Message::Packet(packet) => match packet {
                Packet::Goodbye => unreachable!(), // Goodbye is not yielded from proto::read
                Packet::SaveDelta(delta) => self.model.save = &self.model.save + &delta,
                Packet::SaveInit(save) => self.model.save = save,
                Packet::KnowledgeInit(knowledge) => self.model.knowledge = knowledge,
            },
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Message> {
        let cell_buttons = &mut self.cell_buttons;

        macro_rules! cell {
            ($cell:ident) => {{
                TrackerCell::$cell.view(&self.model, if self.client_connected { None } else { Some(&mut cell_buttons.$cell) })
            }}
        }

        let view = Column::new()
            .push(Row::new()
                .push(cell!(LightMedallionLocation))
                .push(cell!(ForestMedallionLocation))
                .push(cell!(FireMedallionLocation))
                .push(cell!(WaterMedallionLocation))
                .push(cell!(ShadowMedallionLocation))
                .push(cell!(SpiritMedallionLocation))
                .spacing(1)
            )
            .push(Row::new()
                .push(cell!(LightMedallion))
                .push(cell!(ForestMedallion))
                .push(cell!(FireMedallion))
                .push(cell!(WaterMedallion))
                .push(cell!(ShadowMedallion))
                .push(cell!(SpiritMedallion))
                .spacing(1)
            )
            .push(Row::new()
                .push(cell!(AdultTrade))
                .push(cell!(Skulltula))
                .push(Column::new()
                    .push(cell!(KokiriEmeraldLocation))
                    .push(cell!(KokiriEmerald))
                    .spacing(1)
                )
                .push(Column::new()
                    .push(cell!(GoronRubyLocation))
                    .push(cell!(GoronRuby))
                    .spacing(1)
                )
                .push(Column::new()
                    .push(cell!(ZoraSapphireLocation))
                    .push(cell!(ZoraSapphire))
                    .spacing(1)
                )
                .push(cell!(Bottle))
                .push(cell!(Scale))
                .spacing(1)
            )
            .push(Row::new()
                .push(cell!(Slingshot))
                .push(cell!(Bombs))
                .push(cell!(Boomerang))
                .push(cell!(Strength))
                .push(cell!(Magic))
                .push(cell!(Spells))
                .spacing(1)
            )
            .push(Row::new()
                .push(cell!(Hookshot))
                .push(cell!(Bow))
                .push(cell!(Arrows))
                .push(cell!(Hammer)) 
                .push(cell!(Boots))
                .push(cell!(MirrorShield))
                .spacing(1)
            )
            .push(Row::new()
                .push(cell!(ChildTrade))
                .push(cell!(Ocarina))
                .push(cell!(Beans))
                .push(cell!(SwordCard))
                .push(cell!(Tunics))
                .push(cell!(Triforce)) //TODO if triforce hunt is off and autotracker is on, replace with something else (big poes?)
                .spacing(1)
            );
        let view = if let Some((is_temp, ref notification)) = self.notification {
            let mut row = Row::new()
                .push(Text::new(format!("{}", notification)).color([1.0, 1.0, 1.0])); //TODO Display instead of Debug
            if !is_temp {
                row = row.push(Button::new(&mut self.dismiss_notification_button, Text::new("X").color([1.0, 0.0, 0.0])).on_press(Message::DismissNotification));
            }
            view.push(row.height(Length::Units(101)))
        } else {
            view.push(Row::new()
                    .push(cell!(ZeldasLullaby))
                    .push(cell!(EponasSong))
                    .push(cell!(SariasSong))
                    .push(cell!(SunsSong))
                    .push(cell!(SongOfTime))
                    .push(cell!(SongOfStorms))
                    .spacing(1)
                )
                .push(Row::new()
                    .push(cell!(Minuet))
                    .push(cell!(Bolero))
                    .push(cell!(Serenade))
                    .push(cell!(Requiem))
                    .push(cell!(Nocturne))
                    .push(cell!(Prelude))
                    .spacing(1)
                )
        };
        Container::new(Container::new(view.spacing(1).padding(1))
                .width(Length::Units(WIDTH as u16))
                .height(Length::Units(HEIGHT as u16))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(ContainerStyle)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        #[cfg(not(target_arch = "wasm32"))] {
            Subscription::from_recipe(tcp_server::Subscription)
        }
        #[cfg(target_arch = "wasm32")] {
            Subscription::none()
        }
    }
}

#[derive(StructOpt)]
struct Args {}

#[wheel::main]
fn main(_: Args) {
    State::run(Settings {
        window: window::Settings {
            size: (WIDTH, HEIGHT),
            resizable: false,
            ..window::Settings::default()
        },
        ..Settings::default()
    });
}
