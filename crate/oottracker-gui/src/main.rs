#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use {
    std::path::{
        Path,
        PathBuf,
    },
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
    oottracker::{
        event_chk_inf::*,
        knowledge::*,
        proto::{
            self,
            Packet,
        },
        save::*,
    },
};

mod tcp_server;

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
                ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask => ChildTradeItem::KeatonMask,
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
            TrackerCell::Triforce => if state.save.triforce_pieces == 100 { state.save.triforce_pieces = 0 } else { state.save.triforce_pieces += 1 },
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
        let xopar_images = asset_path().join("xopar-images");
        let xopar_images_count = asset_path().join("xopar-images-count");
        let xopar_images_dimmed = asset_path().join("xopar-images-dimmed");
        let xopar_images_overlay = asset_path().join("xopar-images-overlay");
        let xopar_images_overlay_dimmed = asset_path().join("xopar-images-overlay-dimmed");
        let content = match self {
            TrackerCell::LightMedallionLocation => Image::new(xopar_images.join(match state.knowledge.light_medallion_location {
                DungeonRewardLocation::Unknown => "unknown_text.png",
                DungeonRewardLocation::DekuTree => "deku_text.png",
                DungeonRewardLocation::DodongosCavern => "dc_text.png",
                DungeonRewardLocation::JabuJabu => "jabu_text.png",
                DungeonRewardLocation::ForestTemple => "forest_text.png",
                DungeonRewardLocation::FireTemple => "fire_text.png",
                DungeonRewardLocation::WaterTemple => "water_text.png",
                DungeonRewardLocation::ShadowTemple => "shadow_text.png",
                DungeonRewardLocation::SpiritTemple => "spirit_text.png",
                DungeonRewardLocation::LinksPocket => "free_text.png",
            })).width(Length::Units(50)),
            TrackerCell::ForestMedallionLocation => Image::new(xopar_images.join(match state.knowledge.forest_medallion_location {
                DungeonRewardLocation::Unknown => "unknown_text.png",
                DungeonRewardLocation::DekuTree => "deku_text.png",
                DungeonRewardLocation::DodongosCavern => "dc_text.png",
                DungeonRewardLocation::JabuJabu => "jabu_text.png",
                DungeonRewardLocation::ForestTemple => "forest_text.png",
                DungeonRewardLocation::FireTemple => "fire_text.png",
                DungeonRewardLocation::WaterTemple => "water_text.png",
                DungeonRewardLocation::ShadowTemple => "shadow_text.png",
                DungeonRewardLocation::SpiritTemple => "spirit_text.png",
                DungeonRewardLocation::LinksPocket => "free_text.png",
            })).width(Length::Units(50)),
            TrackerCell::FireMedallionLocation => Image::new(xopar_images.join(match state.knowledge.fire_medallion_location {
                DungeonRewardLocation::Unknown => "unknown_text.png",
                DungeonRewardLocation::DekuTree => "deku_text.png",
                DungeonRewardLocation::DodongosCavern => "dc_text.png",
                DungeonRewardLocation::JabuJabu => "jabu_text.png",
                DungeonRewardLocation::ForestTemple => "forest_text.png",
                DungeonRewardLocation::FireTemple => "fire_text.png",
                DungeonRewardLocation::WaterTemple => "water_text.png",
                DungeonRewardLocation::ShadowTemple => "shadow_text.png",
                DungeonRewardLocation::SpiritTemple => "spirit_text.png",
                DungeonRewardLocation::LinksPocket => "free_text.png",
            })).width(Length::Units(50)),
            TrackerCell::WaterMedallionLocation => Image::new(xopar_images.join(match state.knowledge.water_medallion_location {
                DungeonRewardLocation::Unknown => "unknown_text.png",
                DungeonRewardLocation::DekuTree => "deku_text.png",
                DungeonRewardLocation::DodongosCavern => "dc_text.png",
                DungeonRewardLocation::JabuJabu => "jabu_text.png",
                DungeonRewardLocation::ForestTemple => "forest_text.png",
                DungeonRewardLocation::FireTemple => "fire_text.png",
                DungeonRewardLocation::WaterTemple => "water_text.png",
                DungeonRewardLocation::ShadowTemple => "shadow_text.png",
                DungeonRewardLocation::SpiritTemple => "spirit_text.png",
                DungeonRewardLocation::LinksPocket => "free_text.png",
            })).width(Length::Units(50)),
            TrackerCell::ShadowMedallionLocation => Image::new(xopar_images.join(match state.knowledge.shadow_medallion_location {
                DungeonRewardLocation::Unknown => "unknown_text.png",
                DungeonRewardLocation::DekuTree => "deku_text.png",
                DungeonRewardLocation::DodongosCavern => "dc_text.png",
                DungeonRewardLocation::JabuJabu => "jabu_text.png",
                DungeonRewardLocation::ForestTemple => "forest_text.png",
                DungeonRewardLocation::FireTemple => "fire_text.png",
                DungeonRewardLocation::WaterTemple => "water_text.png",
                DungeonRewardLocation::ShadowTemple => "shadow_text.png",
                DungeonRewardLocation::SpiritTemple => "spirit_text.png",
                DungeonRewardLocation::LinksPocket => "free_text.png",
            })).width(Length::Units(50)),
            TrackerCell::SpiritMedallionLocation => Image::new(xopar_images.join(match state.knowledge.spirit_medallion_location {
                DungeonRewardLocation::Unknown => "unknown_text.png",
                DungeonRewardLocation::DekuTree => "deku_text.png",
                DungeonRewardLocation::DodongosCavern => "dc_text.png",
                DungeonRewardLocation::JabuJabu => "jabu_text.png",
                DungeonRewardLocation::ForestTemple => "forest_text.png",
                DungeonRewardLocation::FireTemple => "fire_text.png",
                DungeonRewardLocation::WaterTemple => "water_text.png",
                DungeonRewardLocation::ShadowTemple => "shadow_text.png",
                DungeonRewardLocation::SpiritTemple => "spirit_text.png",
                DungeonRewardLocation::LinksPocket => "free_text.png",
            })).width(Length::Units(50)),
            TrackerCell::LightMedallion => Image::new(if state.save.quest_items.contains(QuestItems::LIGHT_MEDALLION) { &xopar_images } else { &xopar_images_dimmed }.join("light_medallion.png")),
            TrackerCell::ForestMedallion => Image::new(if state.save.quest_items.contains(QuestItems::FOREST_MEDALLION) { &xopar_images } else { &xopar_images_dimmed }.join("forest_medallion.png")),
            TrackerCell::FireMedallion => Image::new(if state.save.quest_items.contains(QuestItems::FIRE_MEDALLION) { &xopar_images } else { &xopar_images_dimmed }.join("fire_medallion.png")),
            TrackerCell::WaterMedallion => Image::new(if state.save.quest_items.contains(QuestItems::WATER_MEDALLION) { &xopar_images } else { &xopar_images_dimmed }.join("water_medallion.png")),
            TrackerCell::ShadowMedallion => Image::new(if state.save.quest_items.contains(QuestItems::SHADOW_MEDALLION) { &xopar_images } else { &xopar_images_dimmed }.join("shadow_medallion.png")),
            TrackerCell::SpiritMedallion => Image::new(if state.save.quest_items.contains(QuestItems::SPIRIT_MEDALLION) { &xopar_images } else { &xopar_images_dimmed }.join("spirit_medallion.png")),
            TrackerCell::AdultTrade => Image::new(match state.save.inv.adult_trade_item {
                AdultTradeItem::None => xopar_images_dimmed.join("blue_egg.png"),
                AdultTradeItem::PocketEgg | AdultTradeItem::PocketCucco => xopar_images.join("blue_egg.png"),
                AdultTradeItem::Cojiro => xopar_images.join("cojiro.png"),
                AdultTradeItem::OddMushroom => xopar_images.join("odd_mushroom.png"),
                AdultTradeItem::OddPotion => xopar_images.join("odd_poultice.png"),
                AdultTradeItem::PoachersSaw => xopar_images.join("poachers_saw.png"),
                AdultTradeItem::BrokenSword => xopar_images.join("broken_sword.png"),
                AdultTradeItem::Prescription => xopar_images.join("prescription.png"),
                AdultTradeItem::EyeballFrog => xopar_images.join("eyeball_frog.png"),
                AdultTradeItem::Eyedrops => xopar_images.join("eye_drops.png"),
                AdultTradeItem::ClaimCheck => xopar_images.join("claim_check.png"),
            }),
            TrackerCell::Skulltula => Image::new(if state.save.skull_tokens == 0 { xopar_images_dimmed.join("golden_skulltula.png") } else { xopar_images_count.join(format!("skulls_{}.png", state.save.skull_tokens)) }),
            TrackerCell::KokiriEmeraldLocation => Image::new(xopar_images.join(match state.knowledge.kokiri_emerald_location {
                DungeonRewardLocation::Unknown => "unknown_text.png",
                DungeonRewardLocation::DekuTree => "deku_text.png",
                DungeonRewardLocation::DodongosCavern => "dc_text.png",
                DungeonRewardLocation::JabuJabu => "jabu_text.png",
                DungeonRewardLocation::ForestTemple => "forest_text.png",
                DungeonRewardLocation::FireTemple => "fire_text.png",
                DungeonRewardLocation::WaterTemple => "water_text.png",
                DungeonRewardLocation::ShadowTemple => "shadow_text.png",
                DungeonRewardLocation::SpiritTemple => "spirit_text.png",
                DungeonRewardLocation::LinksPocket => "free_text.png",
            })).width(Length::Units(33)),
            TrackerCell::KokiriEmerald => Image::new(if state.save.quest_items.contains(QuestItems::KOKIRI_EMERALD) { &xopar_images } else { &xopar_images_dimmed }.join("kokiri_emerald.png")).width(Length::Units(33)),
            TrackerCell::GoronRubyLocation => Image::new(xopar_images.join(match state.knowledge.goron_ruby_location {
                DungeonRewardLocation::Unknown => "unknown_text.png",
                DungeonRewardLocation::DekuTree => "deku_text.png",
                DungeonRewardLocation::DodongosCavern => "dc_text.png",
                DungeonRewardLocation::JabuJabu => "jabu_text.png",
                DungeonRewardLocation::ForestTemple => "forest_text.png",
                DungeonRewardLocation::FireTemple => "fire_text.png",
                DungeonRewardLocation::WaterTemple => "water_text.png",
                DungeonRewardLocation::ShadowTemple => "shadow_text.png",
                DungeonRewardLocation::SpiritTemple => "spirit_text.png",
                DungeonRewardLocation::LinksPocket => "free_text.png",
            })).width(Length::Units(34)),
            TrackerCell::GoronRuby => Image::new(if state.save.quest_items.contains(QuestItems::GORON_RUBY) { &xopar_images } else { &xopar_images_dimmed }.join("goron_ruby.png")).width(Length::Units(34)),
            TrackerCell::ZoraSapphireLocation => Image::new(xopar_images.join(match state.knowledge.zora_sapphire_location {
                DungeonRewardLocation::Unknown => "unknown_text.png",
                DungeonRewardLocation::DekuTree => "deku_text.png",
                DungeonRewardLocation::DodongosCavern => "dc_text.png",
                DungeonRewardLocation::JabuJabu => "jabu_text.png",
                DungeonRewardLocation::ForestTemple => "forest_text.png",
                DungeonRewardLocation::FireTemple => "fire_text.png",
                DungeonRewardLocation::WaterTemple => "water_text.png",
                DungeonRewardLocation::ShadowTemple => "shadow_text.png",
                DungeonRewardLocation::SpiritTemple => "spirit_text.png",
                DungeonRewardLocation::LinksPocket => "free_text.png",
            })).width(Length::Units(33)),
            TrackerCell::ZoraSapphire => Image::new(if state.save.quest_items.contains(QuestItems::ZORA_SAPPHIRE) { &xopar_images } else { &xopar_images_dimmed }.join("zora_sapphire.png")).width(Length::Units(33)),
            TrackerCell::Bottle => Image::new(if state.save.inv.bottles > 0 { &xopar_images } else { &xopar_images_dimmed }.join("bottle.png")), //TODO only undim if the bottle can be trivially emptied; Ruto's Letter support
            TrackerCell::Scale => Image::new(match state.save.upgrades.scale() {
                Upgrades::SILVER_SCALE => xopar_images.join("silver_scale.png"),
                Upgrades::GOLD_SCALE => xopar_images.join("gold_scale.png"),
                _ => xopar_images_dimmed.join("silver_scale.png"),
            }),
            TrackerCell::Slingshot => Image::new(if state.save.inv.slingshot { &xopar_images } else { &xopar_images_dimmed }.join("slingshot.png")),
            TrackerCell::Bombs => Image::new(match (state.save.upgrades.bomb_bag(), state.save.inv.bombchus) {
                (Upgrades::NONE, false) => xopar_images_dimmed.join("bomb_bag.png"),
                (Upgrades::NONE, true) => xopar_images_overlay_dimmed.join("bomb_bag_bombchu.png"),
                (_, false) => xopar_images.join("bomb_bag.png"),
                (_, true) => xopar_images_overlay.join("bomb_bag_bombchu.png"),
            }),
            TrackerCell::Boomerang => Image::new(if state.save.inv.boomerang { &xopar_images } else { &xopar_images_dimmed }.join("boomerang.png")),
            TrackerCell::Strength => Image::new(match state.save.upgrades.strength() {
                Upgrades::GORON_BRACELET => xopar_images.join("goron_bracelet.png"),
                Upgrades::SILVER_GAUNTLETS => xopar_images.join("silver_gauntlets.png"),
                Upgrades::GOLD_GAUNTLETS => xopar_images.join("gold_gauntlets.png"),
                _ => xopar_images_dimmed.join("goron_bracelet.png"),
            }),
            TrackerCell::Magic => Image::new(match (state.save.magic, state.save.inv.lens) {
                (MagicCapacity::None, false) => xopar_images_dimmed.join("magic.png"),
                (MagicCapacity::None, true) => xopar_images_overlay_dimmed.join("magic_lens.png"),
                (_, false) => xopar_images.join("magic.png"),
                (_, true) => xopar_images_overlay.join("magic_lens.png"),
            }),
            TrackerCell::Spells => Image::new(match (state.save.inv.dins_fire, state.save.inv.farores_wind) {
                (false, false) => xopar_images_dimmed.join("composite_magic.png"),
                (false, true) => xopar_images.join("faores_wind.png"),
                (true, false) => xopar_images.join("dins_fire.png"),
                (true, true) => xopar_images.join("composite_magic.png"),
            }),
            TrackerCell::Hookshot => Image::new(match state.save.inv.hookshot {
                Hookshot::None => xopar_images_dimmed.join("hookshot.png"),
                Hookshot::Hookshot => xopar_images.join("hookshot_accessible.png"),
                Hookshot::Longshot => xopar_images.join("longshot_accessible.png"),
            }),
            TrackerCell::Bow => Image::new(match (state.save.inv.bow, state.save.inv.ice_arrows) {
                (false, false) => xopar_images_dimmed.join("bow.png"),
                (false, true) => xopar_images_overlay_dimmed.join("bow_ice_arrows.png"),
                (true, false) => xopar_images.join("bow.png"),
                (true, true) => xopar_images_overlay.join("bow_ice_arrows.png"),
            }),
            TrackerCell::Arrows => Image::new(match (state.save.inv.fire_arrows, state.save.inv.light_arrows) {
                (false, false) => xopar_images_dimmed.join("composite_arrows.png"),
                (false, true) => xopar_images.join("light_arrows.png"),
                (true, false) => xopar_images.join("fire_arrows.png"),
                (true, true) => xopar_images.join("composite_arrows.png"),
            }),
            TrackerCell::Hammer => Image::new(if state.save.inv.hammer { &xopar_images } else { &xopar_images_dimmed }.join("hammer.png")),
            TrackerCell::Boots => Image::new(match (state.save.equipment.contains(Equipment::IRON_BOOTS), state.save.equipment.contains(Equipment::HOVER_BOOTS)) {
                (false, false) => xopar_images_dimmed.join("composite_boots.png"),
                (false, true) => xopar_images.join("hover_boots.png"),
                (true, false) => xopar_images.join("iron_boots.png"),
                (true, true) => xopar_images.join("composite_boots.png"),
            }),
            TrackerCell::MirrorShield => Image::new(if state.save.equipment.contains(Equipment::MIRROR_SHIELD) { &xopar_images } else { &xopar_images_dimmed }.join("mirror_shield.png")),
            TrackerCell::ChildTrade => Image::new(match state.save.inv.child_trade_item {
                ChildTradeItem::None => xopar_images_dimmed.join("white_egg.png"),
                ChildTradeItem::WeirdEgg => xopar_images.join("white_egg.png"),
                ChildTradeItem::Chicken => xopar_images.join("white_chicken.png"),
                ChildTradeItem::ZeldasLetter | ChildTradeItem::GoronMask | ChildTradeItem::ZoraMask | ChildTradeItem::GerudoMask => xopar_images.join("zelda_letter.png"),
                ChildTradeItem::KeatonMask => xopar_images.join("keaton_mask.png"),
                ChildTradeItem::SkullMask => xopar_images.join("skull_mask.png"),
                ChildTradeItem::SpookyMask => xopar_images.join("spooky_mask.png"),
                ChildTradeItem::BunnyHood => xopar_images.join("bunny_hood.png"),
                ChildTradeItem::MaskOfTruth => xopar_images.join("mask_of_truth.png"),
            }),
            TrackerCell::Ocarina => Image::new(match (state.save.inv.ocarina, state.save.event_chk_inf.9.contains(EventChkInf9::SCARECROW_SONG)) { //TODO only show free Scarecrow's Song once it's known (by settings string input or by check)
                (false, false) => xopar_images_dimmed.join("ocarina.png"),
                (false, true) => xopar_images_overlay_dimmed.join("ocarina_scarecrow.png"),
                (true, false) => xopar_images.join("ocarina.png"),
                (true, true) => xopar_images_overlay.join("ocarina_scarecrow.png"),
            }),
            TrackerCell::Beans => Image::new(if state.save.inv.beans { &xopar_images } else { &xopar_images_dimmed }.join("beans.png")), //TODO overlay with number bought if autotracker is on?
            TrackerCell::SwordCard => Image::new(match (state.save.equipment.contains(Equipment::KOKIRI_SWORD), state.save.quest_items.contains(QuestItems::GERUDO_CARD)) {
                (false, false) => xopar_images_dimmed.join("composite_ksword_gcard.png"),
                (false, true) => xopar_images.join("gerudo_card.png"),
                (true, false) => xopar_images.join("kokiri_sword.png"),
                (true, true) => xopar_images.join("composite_ksword_gcard.png"),
            }),
            TrackerCell::Tunics => Image::new(match (state.save.equipment.contains(Equipment::GORON_TUNIC), state.save.equipment.contains(Equipment::ZORA_TUNIC)) {
                (false, false) => xopar_images_dimmed.join("composite_tunics.png"),
                (false, true) => xopar_images.join("zora_tunic.png"),
                (true, false) => xopar_images.join("goron_tunic.png"),
                (true, true) => xopar_images.join("composite_tunics.png"),
            }),
            TrackerCell::Triforce => Image::new(if state.save.triforce_pieces == 0 { xopar_images_dimmed.join("triforce.png") } else { xopar_images_count.join(format!("force_{}.png", state.save.triforce_pieces)) }),
            TrackerCell::ZeldasLullaby => Image::new(if state.save.quest_items.contains(QuestItems::ZELDAS_LULLABY) { &xopar_images } else { &xopar_images_dimmed }.join("lullaby.png")),
            TrackerCell::EponasSong => Image::new(if state.save.quest_items.contains(QuestItems::EPONAS_SONG) { &xopar_images } else { &xopar_images_dimmed }.join("epona.png")),
            TrackerCell::SariasSong => Image::new(if state.save.quest_items.contains(QuestItems::SARIAS_SONG) { &xopar_images } else { &xopar_images_dimmed }.join("saria.png")),
            TrackerCell::SunsSong => Image::new(if state.save.quest_items.contains(QuestItems::SUNS_SONG) { &xopar_images } else { &xopar_images_dimmed }.join("sun.png")),
            TrackerCell::SongOfTime => Image::new(if state.save.quest_items.contains(QuestItems::SONG_OF_TIME) { &xopar_images } else { &xopar_images_dimmed }.join("time.png")),
            TrackerCell::SongOfStorms => Image::new(if state.save.quest_items.contains(QuestItems::SONG_OF_STORMS) { &xopar_images } else { &xopar_images_dimmed }.join("storms.png")),
            TrackerCell::Minuet => Image::new(if state.save.quest_items.contains(QuestItems::MINUET_OF_FOREST) { &xopar_images } else { &xopar_images_dimmed }.join("minuet.png")),
            TrackerCell::Bolero => Image::new(if state.save.quest_items.contains(QuestItems::BOLERO_OF_FIRE) { &xopar_images } else { &xopar_images_dimmed }.join("bolero.png")),
            TrackerCell::Serenade => Image::new(if state.save.quest_items.contains(QuestItems::SERENADE_OF_WATER) { &xopar_images } else { &xopar_images_dimmed }.join("serenade.png")),
            TrackerCell::Requiem => Image::new(if state.save.quest_items.contains(QuestItems::REQUIEM_OF_SPIRIT) { &xopar_images } else { &xopar_images_dimmed }.join("requiem.png")),
            TrackerCell::Nocturne => Image::new(if state.save.quest_items.contains(QuestItems::NOCTURNE_OF_SHADOW) { &xopar_images } else { &xopar_images_dimmed }.join("nocturne.png")),
            TrackerCell::Prelude => Image::new(if state.save.quest_items.contains(QuestItems::PRELUDE_OF_LIGHT) { &xopar_images } else { &xopar_images_dimmed }.join("prelude.png")),
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
    ClientConnected,
    ClientDisconnected,
    DismissNotification,
    LeftClick(TrackerCell),
    NetworkError(proto::ReadError),
    Packet(Packet),
}

#[derive(Debug, Default)]
struct State {
    cell_buttons: CellButtons,
    client_connected: bool,
    model: ModelState,
    notification: Option<Message>,
    dismiss_notification_button: button::State,
}

impl State {
    /// Adds a visible notification/alert/log message.
    ///
    /// Implemented as a separate method in case the way this is displayed is changed later, e.g. to allow multiple notifications.
    fn notify(&mut self, message: Message) {
        self.notification = Some(message);
    }
}

impl Application for State {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new((): ()) -> (State, Command<Message>) { (State::default(), Command::none()) }
    fn title(&self) -> String { format!("OoT Tracker") }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ClientConnected => {
                self.client_connected = true;
                self.notify(message); //TODO automatically hide message after some amount of time
            }
            Message::ClientDisconnected => {
                self.client_connected = false;
                self.notify(message); //TODO automatically hide message after some amount of time
            }
            Message::DismissNotification => self.notification = None,
            Message::LeftClick(cell) => cell.left_click(&mut self.model),
            Message::NetworkError(_) => self.notify(message),
            Message::Packet(packet) => match packet {
                Packet::Goodbye => unreachable!(), // Goodbye is not yielded from proto::read
                Packet::SaveDelta(delta) => self.model.save = &self.model.save + &delta,
                Packet::SaveInit(save) => self.model.save = save,
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
            )
            .push(Row::new()
                .push(cell!(LightMedallion))
                .push(cell!(ForestMedallion))
                .push(cell!(FireMedallion))
                .push(cell!(WaterMedallion))
                .push(cell!(ShadowMedallion))
                .push(cell!(SpiritMedallion))
            )
            .push(Row::new()
                .push(cell!(AdultTrade))
                .push(cell!(Skulltula))
                .push(Column::new()
                    .push(cell!(KokiriEmeraldLocation))
                    .push(cell!(KokiriEmerald))
                )
                .push(Column::new()
                    .push(cell!(GoronRubyLocation))
                    .push(cell!(GoronRuby))
                )
                .push(Column::new()
                    .push(cell!(ZoraSapphireLocation))
                    .push(cell!(ZoraSapphire))
                )
                .push(cell!(Bottle))
                .push(cell!(Scale))
            )
            .push(Row::new()
                .push(cell!(Slingshot))
                .push(cell!(Bombs))
                .push(cell!(Boomerang))
                .push(cell!(Strength))
                .push(cell!(Magic))
                .push(cell!(Spells))
            )
            .push(Row::new()
                .push(cell!(Hookshot))
                .push(cell!(Bow))
                .push(cell!(Arrows))
                .push(cell!(Hammer)) 
                .push(cell!(Boots))
                .push(cell!(MirrorShield))
            )
            .push(Row::new()
                .push(cell!(ChildTrade))
                .push(cell!(Ocarina))
                .push(cell!(Beans))
                .push(cell!(SwordCard))
                .push(cell!(Tunics))
                .push(cell!(Triforce)) //TODO if triforce hunt is off and autotracker is on, replace with something else (big poes?)
            );
        let view = if let Some(ref notification) = self.notification {
            view.push(Row::new()
                .push(Text::new(format!("{:?}", notification)).color([1.0, 1.0, 1.0])) //TODO Display instead of Debug
                .push(Button::new(&mut self.dismiss_notification_button, Text::new("X").color([1.0, 0.0, 0.0])).on_press(Message::DismissNotification))
                .height(Length::Units(100))
            )
        } else {
            view.push(Row::new() //TODO overlay with song checks
                    .push(cell!(ZeldasLullaby))
                    .push(cell!(EponasSong))
                    .push(cell!(SariasSong))
                    .push(cell!(SunsSong))
                    .push(cell!(SongOfTime))
                    .push(cell!(SongOfStorms))
                )
                .push(Row::new() //TODO overlay with song checks
                    .push(cell!(Minuet))
                    .push(cell!(Bolero))
                    .push(cell!(Serenade))
                    .push(cell!(Requiem))
                    .push(cell!(Nocturne))
                    .push(cell!(Prelude))
                )
        };
        Container::new(view).style(ContainerStyle).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::from_recipe(tcp_server::Subscription)
    }
}

fn asset_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().expect("crate dir has no parent")
        .parent().expect("crates dir has no parent")
        .join("assets")
}

fn main() {
    State::run(Settings {
        window: window::Settings {
            size: (50 * 6, 18 + 50 * 7),
            resizable: false,
            ..window::Settings::default()
        },
        ..Settings::default()
    });
}
