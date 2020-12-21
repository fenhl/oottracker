#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use {
    std::{
        collections::HashMap,
        fmt,
        path::Path,
    },
    derive_more::From,
    iced::{
        Application,
        Background,
        Color,
        Command,
        Element,
        Length,
        Settings,
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
    iced_futures::Subscription,
    itertools::Itertools as _,
    structopt::StructOpt,
    oottracker::{
        Check,
        ModelState,
        checks::{
            CheckExt as _,
            CheckStatus,
        },
        info_tables::*,
        model::{
            DungeonReward,
            DungeonRewardLocation,
            MainDungeon,
            Medallion,
            Stone,
        },
        save::*,
    },
};
#[cfg(not(target_arch = "wasm32"))] use {
    std::time::Duration,
    iced::window::Icon,
    iced_native::keyboard::Modifiers as KeyboardModifiers,
    image::DynamicImage,
    tokio::time::sleep,
    oottracker::{
        Rando,
        checks::{
            self,
            CheckStatusError,
        },
        proto::{
            self,
            Packet,
        },
    },
};

mod lang;
#[cfg(not(target_arch = "wasm32"))] mod tcp_server;

pub trait FromEmbeddedImage {
    fn from_embedded_image(name: &Path, contents: &[u8]) -> Self;
}

impl FromEmbeddedImage for Image {
    #[cfg(not(target_arch = "wasm32"))]
    fn from_embedded_image(_: &Path, contents: &[u8]) -> Image {
        Image::new(iced::image::Handle::from_memory(contents.to_vec()))
    }

    #[cfg(target_arch = "wasm32")]
    fn from_embedded_image(path: &Path, _: &[u8]) -> Image {
        Image::new(iced::image::Handle::from_path(path))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl FromEmbeddedImage for DynamicImage {
    fn from_embedded_image(_: &Path, contents: &[u8]) -> DynamicImage {
        image::load_from_memory(contents).expect("failed to load embedded DynamicImage")
    }
}

mod images {
    use super::FromEmbeddedImage;

    oottracker_derive::embed_images!("assets/xopar-images");
    oottracker_derive::embed_images!("assets/xopar-images-count");
    oottracker_derive::embed_images!("assets/xopar-images-dimmed");
    oottracker_derive::embed_images!("assets/xopar-images-overlay");
    oottracker_derive::embed_images!("assets/xopar-images-overlay-dimmed");
    oottracker_derive::embed_image!("assets/icon.ico");
}

const WIDTH: u32 = 50 * 6 + 7; // 6 images, each 50px wide, plus 1px spacing
const HEIGHT: u32 = 18 + 50 * 7 + 9; // dungeon reward location text, 18px high, and 7 images, each 50px high, plus 1px spacing

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Default)]
struct KeyboardModifiers {
    control: bool,
}

struct ContainerStyle;

impl container::StyleSheet for ContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::BLACK)),
            ..container::Style::default()
        }
    }
}

trait DungeonRewardLocationExt {
    fn increment(&mut self, key: DungeonReward);
    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(not(target_arch = "wasm32"))]
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

enum TrackerCellKind {
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
    MedallionLocation(Medallion),
    OptionalOverlay {
        main_img: &'static str,
        overlay_img: &'static str,
        active: Box<dyn Fn(&ModelState) -> (bool, bool)>,
        toggle_main: Box<dyn Fn(&mut ModelState)>,
        #[cfg(not(target_arch = "wasm32"))]
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
        #[cfg(not(target_arch = "wasm32"))]
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
        #[cfg(not(target_arch = "wasm32"))]
        toggle_overlay: Box<dyn Fn(&mut EventChkInf)>,
    },
    Stone(Stone),
    StoneLocation(Stone),
}

use TrackerCellKind::*;

impl TrackerCellKind {
    #[cfg(not(target_arch = "wasm32"))]
    fn width(&self) -> u16 {
        match self {
            StoneLocation(_) | Stone(_) => 33,
            _ => 50,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn height(&self) -> u16 {
        match self {
            MedallionLocation(_) => 18,
            StoneLocation(_) => 12,
            _ => 50,
        }
    }

    fn render(&self, state: &ModelState) -> Image {
        match self {
            Composite { left_img, right_img, both_img, active, .. } => match active(state) {
                (false, false) => images::xopar_images_dimmed(both_img, "png"),
                (false, true) => images::xopar_images(right_img, "png"),
                (true, false) => images::xopar_images(left_img, "png"),
                (true, true) => images::xopar_images(both_img, "png"),
            },
            Count { dimmed_img, img, get, .. } => {
                let count = get(state);
                if count == 0 {
                    images::xopar_images_dimmed(dimmed_img, "png")
                } else {
                    images::xopar_images_count(&format!("{}_{}", img, count), "png")
                }
            }
            MedallionLocation(med) => match state.knowledge.dungeon_reward_locations.get(&DungeonReward::Medallion(*med)) {
                None => images::xopar_images_dimmed::<Image>("unknown_text", "png"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)) => images::xopar_images("deku_text", "png"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)) => images::xopar_images("dc_text", "png"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)) => images::xopar_images("jabu_text", "png"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)) => images::xopar_images("forest_text", "png"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)) => images::xopar_images("fire_text", "png"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)) => images::xopar_images("water_text", "png"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)) => images::xopar_images("shadow_text", "png"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)) => images::xopar_images("spirit_text", "png"),
                Some(DungeonRewardLocation::LinksPocket) => images::xopar_images("free_text", "png"),
            }.width(Length::Units(50)),
            OptionalOverlay { main_img, overlay_img, active, .. } | Overlay { main_img, overlay_img, active, .. } => match active(state) {
                (false, false) => images::xopar_images_dimmed(main_img, "png"),
                (false, true) => images::xopar_images_overlay_dimmed(&format!("{}_{}", main_img, overlay_img), "png"),
                (true, false) => images::xopar_images(main_img, "png"),
                (true, true) => images::xopar_images_overlay(&format!("{}_{}", main_img, overlay_img), "png"),
            },
            Sequence { img, .. } => match img(state) {
                (false, img) => images::xopar_images_dimmed(img, "png"),
                (true, img) => images::xopar_images(img, "png"),
            },
            Simple { img, active, .. } => if active(state) {
                images::xopar_images(img, "png")
            } else {
                images::xopar_images_dimmed(img, "png")
            },
            Song { song, check, .. } => {
                let song_filename = match *song {
                    QuestItems::ZELDAS_LULLABY => "lullaby",
                    QuestItems::EPONAS_SONG => "epona",
                    QuestItems::SARIAS_SONG => "saria",
                    QuestItems::SUNS_SONG => "sun",
                    QuestItems::SONG_OF_TIME => "time",
                    QuestItems::SONG_OF_STORMS => "storms",
                    QuestItems::MINUET_OF_FOREST => "minuet",
                    QuestItems::BOLERO_OF_FIRE => "bolero",
                    QuestItems::SERENADE_OF_WATER => "serenade",
                    QuestItems::NOCTURNE_OF_SHADOW => "nocturne",
                    QuestItems::REQUIEM_OF_SPIRIT => "requiem",
                    QuestItems::PRELUDE_OF_LIGHT => "prelude",
                    _ => unreachable!(),
                };
                match (state.ram.save.quest_items.contains(*song), Check::Location(check.to_string()).checked(state).unwrap_or(false)) {
                    (false, false) => images::xopar_images_dimmed(song_filename, "png"),
                    (false, true) => images::xopar_images_overlay_dimmed(&format!("{}_check", song_filename), "png"),
                    (true, false) => images::xopar_images(song_filename, "png"),
                    (true, true) => images::xopar_images_overlay(&format!("{}_check", song_filename), "png"),
                }
            }
            Stone(stone) => {
                let stone_filename = match *stone {
                    Stone::KokiriEmerald => "kokiri_emerald",
                    Stone::GoronRuby => "goron_ruby",
                    Stone::ZoraSapphire => "zora_sapphire",
                };
                if state.ram.save.quest_items.has(*stone) {
                    images::xopar_images::<Image>(stone_filename, "png")
                } else {
                    images::xopar_images_dimmed(stone_filename, "png")
                }.width(Length::Units(33))
            }
            StoneLocation(stone) => match state.knowledge.dungeon_reward_locations.get(&DungeonReward::Stone(*stone)) {
                None => images::xopar_images_dimmed::<Image>("unknown_text", "png"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)) => images::xopar_images("deku_text", "png"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)) => images::xopar_images("dc_text", "png"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)) => images::xopar_images("jabu_text", "png"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)) => images::xopar_images("forest_text", "png"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)) => images::xopar_images("fire_text", "png"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)) => images::xopar_images("water_text", "png"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)) => images::xopar_images("shadow_text", "png"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)) => images::xopar_images("spirit_text", "png"),
                Some(DungeonRewardLocation::LinksPocket) => images::xopar_images("free_text", "png"),
            }.width(Length::Units(33)),
        }
    }

    fn left_click(&self, #[cfg_attr(not(target_os = "macos"), allow(unused))] keyboard_modifiers: KeyboardModifiers, state: &mut ModelState) {
        #[cfg(target_os = "macos")] if keyboard_modifiers.control {
            self.right_click(state);
            return
        }
        match self {
            Composite { toggle_left: toggle, .. } | OptionalOverlay { toggle_main: toggle, .. } | Overlay { toggle_main: toggle, .. } | Simple { toggle, .. } => toggle(state),
            Count { get, set, max, .. } => {
                let current = get(state);
                if current == *max { set(state, 0) } else { set(state, current + 1) }
            }
            MedallionLocation(med) => state.knowledge.dungeon_reward_locations.increment(DungeonReward::Medallion(*med)),
            Sequence { increment, .. } => increment(state),
            Song { song: quest_item, .. } => state.ram.save.quest_items.toggle(*quest_item),
            Stone(stone) => state.ram.save.quest_items.toggle(QuestItems::from(stone)),
            StoneLocation(stone) => state.knowledge.dungeon_reward_locations.increment(DungeonReward::Stone(*stone)),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn right_click(&self, state: &mut ModelState) {
        match self {
            Composite { toggle_right: toggle, .. } | OptionalOverlay { toggle_overlay: toggle, .. } | Overlay { toggle_overlay: toggle, .. } => toggle(state),
            Count { get, set, max, .. } => {
                let current = get(state);
                if current == 0 { set(state, *max) } else { set(state, current - 1) }
            }
            MedallionLocation(med) => state.knowledge.dungeon_reward_locations.decrement(DungeonReward::Medallion(*med)),
            Sequence { decrement, .. } => decrement(state),
            Simple { .. } | Stone(_) => {}
            Song { toggle_overlay, .. } => toggle_overlay(&mut state.ram.save.event_chk_inf),
            StoneLocation(stone) => state.knowledge.dungeon_reward_locations.decrement(DungeonReward::Stone(*stone)),
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn click(&self, state: &mut ModelState) {
        match self {
            Composite { state: state_fn, toggle_left, toggle_right } | Overlay { state: state_fn, toggle_main: toggle_left, toggle_overlay: toggle_right } => {
                let (left, _) = state_fn(state);
                if left { toggle_right(state) }
                toggle_left(state);
            }
            _ => self.left_click(KeyboardModifiers::default(), state),
        }
    }
}

macro_rules! cells {
    ($([$($cell:ident: $kind:expr,)*],)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        enum TrackerCellId {
            $(
                $(
                    $cell,
                )*
            )*
        }

        impl TrackerCellId {
            #[cfg(not(target_arch = "wasm32"))]
            #[allow(unused_assignments)]
            fn at([x, y]: [f32; 2], include_songs: bool) -> Option<TrackerCellId> {
                if x < 0.0 || y < 0.0 { return None }
                let x = x as u16;
                let y = y as u16;
                let mut max_x;
                let mut max_y = 1;
                $({
                    max_x = 0;
                    let mut row_max_y = 0;
                    $({
                        let kind = $kind;
                        if max_x == 0 { row_max_y = kind.height() }
                        if !matches!(kind, Stone(_)) { max_x += kind.width() + 1 }
                        if (include_songs || !matches!(kind, Song { .. })) && (x < max_x && y < max_y + kind.height()) {
                            return Some(TrackerCellId::$cell)
                        }
                    })*
                    max_y += row_max_y + 1;
                })*
                None
            }

            fn kind(&self) -> TrackerCellKind {
                match self {
                    $($(TrackerCellId::$cell => $kind,)*)*
                }
            }
        }

        #[allow(non_snake_case)]
        #[derive(Debug, Default)]
        struct CellButtons {
            $(
                $(
                    $cell: button::State,
                )*
            )*
        }
    }
}

cells! {
    [
        LightMedallionLocation: MedallionLocation(Medallion::Light),
        ForestMedallionLocation: MedallionLocation(Medallion::Forest),
        FireMedallionLocation: MedallionLocation(Medallion::Fire),
        WaterMedallionLocation: MedallionLocation(Medallion::Water),
        ShadowMedallionLocation: MedallionLocation(Medallion::Shadow),
        SpiritMedallionLocation: MedallionLocation(Medallion::Spirit),
    ],
    [
        LightMedallion: Simple {
            img: "light_medallion",
            active: Box::new(|state| state.ram.save.quest_items.contains(QuestItems::LIGHT_MEDALLION)),
            toggle: Box::new(|state| state.ram.save.quest_items.toggle(QuestItems::LIGHT_MEDALLION)),
        },
        ForestMedallion: Simple {
            img: "forest_medallion",
            active: Box::new(|state| state.ram.save.quest_items.contains(QuestItems::FOREST_MEDALLION)),
            toggle: Box::new(|state| state.ram.save.quest_items.toggle(QuestItems::FOREST_MEDALLION)),
        },
        FireMedallion: Simple {
            img: "fire_medallion",
            active: Box::new(|state| state.ram.save.quest_items.contains(QuestItems::FIRE_MEDALLION)),
            toggle: Box::new(|state| state.ram.save.quest_items.toggle(QuestItems::FIRE_MEDALLION)),
        },
        WaterMedallion: Simple {
            img: "water_medallion",
            active: Box::new(|state| state.ram.save.quest_items.contains(QuestItems::WATER_MEDALLION)),
            toggle: Box::new(|state| state.ram.save.quest_items.toggle(QuestItems::WATER_MEDALLION)),
        },
        ShadowMedallion: Simple {
            img: "shadow_medallion",
            active: Box::new(|state| state.ram.save.quest_items.contains(QuestItems::SHADOW_MEDALLION)),
            toggle: Box::new(|state| state.ram.save.quest_items.toggle(QuestItems::SHADOW_MEDALLION)),
        },
        SpiritMedallion: Simple {
            img: "spirit_medallion",
            active: Box::new(|state| state.ram.save.quest_items.contains(QuestItems::SPIRIT_MEDALLION)),
            toggle: Box::new(|state| state.ram.save.quest_items.toggle(QuestItems::SPIRIT_MEDALLION)),
        },
    ],
    [
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
            #[cfg(not(target_arch = "wasm32"))]
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
            active: Box::new(|state| (state.ram.save.inv.has_emptiable_bottle(), state.ram.save.inv.has_rutos_letter())), //TODO also show Ruto's letter as active if the pit has been delivered
            toggle_main: Box::new(|state| state.ram.save.inv.toggle_emptiable_bottle()),
            #[cfg(not(target_arch = "wasm32"))]
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
            #[cfg(not(target_arch = "wasm32"))]
            decrement: Box::new(|state| state.ram.save.upgrades.set_scale(match state.ram.save.upgrades.scale() {
                Upgrades::SILVER_SCALE => Upgrades::NONE,
                Upgrades::GOLD_SCALE => Upgrades::SILVER_SCALE,
                _ => Upgrades::GOLD_SCALE,
            })),
        },
    ],
    [
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
            #[cfg(not(target_arch = "wasm32"))]
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
    ],
    [
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
            #[cfg(not(target_arch = "wasm32"))]
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
            #[cfg(not(target_arch = "wasm32"))]
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
    ],
    [
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
            #[cfg(not(target_arch = "wasm32"))]
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
        Beans: Simple { //TODO overlay with number bought if autotracker is on?
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
        Triforce: Count {
            dimmed_img: "triforce",
            img: "force",
            get: Box::new(|state| state.ram.save.triforce_pieces()),
            set: Box::new(|state, value| state.ram.save.set_triforce_pieces(value)),
            max: 100,
        },
    ],
    [
        ZeldasLullaby: Song {
            song: QuestItems::ZELDAS_LULLABY,
            check: "Song from Impa",
            #[cfg(not(target_arch = "wasm32"))]
            toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_IMPA)),
        },
        EponasSong: Song {
            song: QuestItems::EPONAS_SONG,
            check: "Song from Malon",
            #[cfg(not(target_arch = "wasm32"))]
            toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_MALON)),
        },
        SariasSong: Song {
            song: QuestItems::SARIAS_SONG,
            check: "Song from Saria",
            #[cfg(not(target_arch = "wasm32"))]
            toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_SARIA)),
        },
        SunsSong: Song {
            song: QuestItems::SUNS_SONG,
            check: "Song from Composers Grave",
            #[cfg(not(target_arch = "wasm32"))]
            toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_COMPOSERS_GRAVE)),
        },
        SongOfTime: Song {
            song: QuestItems::SONG_OF_TIME,
            check: "Song from Ocarina of Time",
            #[cfg(not(target_arch = "wasm32"))]
            toggle_overlay: Box::new(|eci| eci.10.toggle(EventChkInf10::SONG_FROM_OCARINA_OF_TIME)),
        },
        SongOfStorms: Song {
            song: QuestItems::SONG_OF_STORMS,
            check: "Song from Windmill",
            #[cfg(not(target_arch = "wasm32"))]
            toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SONG_FROM_WINDMILL)),
        },
    ],
    [
        Minuet: Song {
            song: QuestItems::MINUET_OF_FOREST,
            check: "Sheik in Forest",
            #[cfg(not(target_arch = "wasm32"))]
            toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_FOREST)),
        },
        Bolero: Song {
            song: QuestItems::BOLERO_OF_FIRE,
            check: "Sheik in Crater",
            #[cfg(not(target_arch = "wasm32"))]
            toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_CRATER)),
        },
        Serenade: Song {
            song: QuestItems::SERENADE_OF_WATER,
            check: "Sheik in Ice Cavern",
            #[cfg(not(target_arch = "wasm32"))]
            toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_ICE_CAVERN)),
        },
        Requiem: Song {
            song: QuestItems::REQUIEM_OF_SPIRIT,
            check: "Sheik at Colossus",
            #[cfg(not(target_arch = "wasm32"))]
            toggle_overlay: Box::new(|eci| eci.10.toggle(EventChkInf10::SHEIK_AT_COLOSSUS)),
        },
        Nocturne: Song {
            song: QuestItems::NOCTURNE_OF_SHADOW,
            check: "Sheik in Kakariko",
            #[cfg(not(target_arch = "wasm32"))]
            toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_IN_KAKARIKO)),
        },
        Prelude: Song {
            song: QuestItems::PRELUDE_OF_LIGHT,
            check: "Sheik at Temple",
            #[cfg(not(target_arch = "wasm32"))]
            toggle_overlay: Box::new(|eci| eci.5.toggle(EventChkInf5::SHEIK_AT_TEMPLE)),
        },
    ],
}

impl TrackerCellId {
    fn view<'a>(&self, state: &ModelState, cell_button: Option<&'a mut button::State>) -> Element<'a, Message> {
        let content = self.kind().render(state);
        if let Some(cell_button) = cell_button {
            Button::new(cell_button, content).on_press(Message::LeftClick(*self)).padding(0).style(*self).into()
        } else {
            content.into()
        }
    }
}

impl button::StyleSheet for TrackerCellId {
    fn active(&self) -> button::Style { button::Style::default() }
}

#[derive(Debug, Clone)]
enum Message {
    #[cfg(not(target_arch = "wasm32"))]
    AutoDismissNotification,
    #[cfg(not(target_arch = "wasm32"))]
    CheckStatusError(CheckStatusError),
    #[cfg(not(target_arch = "wasm32"))]
    ClientConnected,
    #[cfg(not(target_arch = "wasm32"))]
    ClientDisconnected,
    DismissNotification,
    #[cfg(not(target_arch = "wasm32"))]
    KeyboardModifiers(KeyboardModifiers),
    LeftClick(TrackerCellId),
    #[cfg(not(target_arch = "wasm32"))]
    MouseMoved([f32; 2]),
    #[cfg(not(target_arch = "wasm32"))]
    NetworkError(proto::ReadError),
    #[cfg(not(target_arch = "wasm32"))]
    Packet(Packet),
    #[cfg(not(target_arch = "wasm32"))]
    RightClick,
    #[cfg(not(target_arch = "wasm32"))]
    UpdateAvailableChecks(HashMap<Check, CheckStatus>),
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Message::CheckStatusError(e) => write!(f, "error calculating checks: {}", e),
            #[cfg(not(target_arch = "wasm32"))]
            Message::ClientConnected => write!(f, "auto-tracker connected"),
            #[cfg(not(target_arch = "wasm32"))]
            Message::ClientDisconnected => write!(f, "auto-tracker disconnected"),
            #[cfg(not(target_arch = "wasm32"))]
            Message::NetworkError(e) => write!(f, "network error: {}", e),
            _ => write!(f, "{:?}", self), // these messages are not notifications so just fall back to Debug
        }
    }
}

#[derive(Debug, Default)]
struct State {
    flags: bool,
    client_connected: bool,
    keyboard_modifiers: KeyboardModifiers,
    last_cursor_pos: [f32; 2],
    cell_buttons: CellButtons,
    model: ModelState,
    checks: HashMap<Check, CheckStatus>,
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
        async { sleep(Duration::from_secs(10)).await; Message::AutoDismissNotification }.into()
    }
}

impl From<bool> for State {
    fn from(flags: bool) -> State {
        State {
            flags,
            ..State::default()
        }
    }
}

impl Application for State {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = bool;

    fn new(flags: bool) -> (State, Command<Message>) {
        (State::from(flags), Command::none())
    }

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
            Message::CheckStatusError(_) => self.notify(message),
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
            #[cfg(not(target_arch = "wasm32"))]
            Message::KeyboardModifiers(modifiers) => self.keyboard_modifiers = modifiers,
            #[cfg(not(target_arch = "wasm32"))]
            Message::MouseMoved(pos) => self.last_cursor_pos = pos,
            Message::LeftClick(cell) => {
                #[cfg(not(target_arch = "wasm32"))] cell.kind().left_click(self.keyboard_modifiers, &mut self.model);
                #[cfg(target_arch = "wasm32")] cell.kind().click(&mut self.model);
            }
            #[cfg(not(target_arch = "wasm32"))]
            Message::NetworkError(_) => self.notify(message),
            #[cfg(not(target_arch = "wasm32"))]
            Message::Packet(packet) => {
                match packet {
                    Packet::Goodbye => unreachable!(), // Goodbye is not yielded from proto::read
                    Packet::SaveDelta(delta) => self.model.ram.save = &self.model.ram.save + &delta,
                    Packet::SaveInit(save) => self.model.ram.save = save,
                    Packet::KnowledgeInit(knowledge) => self.model.knowledge = knowledge,
                }
                if self.flags { // show available checks
                    let model = self.model.clone();
                    return async move {
                        let rando = Rando::dynamic("C:\\Users\\Fenhl\\git\\github.com\\fenhl\\OoT-Randomizer\\stage"); //TODO use precompiled data by default, allow config override
                        match checks::status(&rando, &model) {
                            Ok(status) => Message::UpdateAvailableChecks(status),
                            Err(e) => Message::CheckStatusError(e),
                        }
                    }.into()
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            Message::RightClick => {
                if let Some(cell) = TrackerCellId::at(self.last_cursor_pos, self.notification.is_none()) {
                    cell.kind().right_click(&mut self.model);
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            Message::UpdateAvailableChecks(checks) => self.checks = checks,
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Message> {
        let cell_buttons = &mut self.cell_buttons;

        macro_rules! cell {
            ($cell:ident) => {{
                TrackerCellId::$cell.view(&self.model, if self.client_connected { None } else { Some(&mut cell_buttons.$cell) })
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
        let items_container = Container::new(Container::new(view.spacing(1).padding(1))
                .width(Length::Units(WIDTH as u16))
                .height(Length::Units(HEIGHT as u16))
            )
            .width(Length::Fill)
            .center_x()
            .center_y()
            .style(ContainerStyle);
        if self.flags { // show available checks
            let check_status_map = self.checks.iter().map(|(check, status)| (status, check)).into_group_map();
            let mut col = Column::new()
                .push(Text::new(format!("{} checked", lang::plural(check_status_map.get(&CheckStatus::Checked).map_or(0, Vec::len), "location"))))
                .push(Text::new(format!("{} currently inaccessible", lang::plural(check_status_map.get(&CheckStatus::NotYetReachable).map_or(0, Vec::len), "location"))))
                .push(Text::new(format!("{} accessible:", lang::plural(check_status_map.get(&CheckStatus::Reachable).map_or(0, Vec::len), "location"))));
            for check in check_status_map.get(&CheckStatus::Reachable).into_iter().flatten() {
                col = col.push(Text::new(format!("{}", check)));
            }
            Column::new()
                .push(items_container.height(Length::Units(HEIGHT as u16 + 2)))
                .push(col)
                .into()
        } else {
            items_container
                .height(Length::Fill)
                .into()
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        #[cfg(not(target_arch = "wasm32"))] {
            Subscription::batch(vec![
                iced_native::subscription::events_with(|event, status| match (event, status) {
                    (iced_native::Event::Keyboard(iced_native::keyboard::Event::ModifiersChanged(modifiers)), _) => Some(Message::KeyboardModifiers(modifiers)),
                    (iced_native::Event::Mouse(iced_native::mouse::Event::CursorMoved { x, y }), _) => Some(Message::MouseMoved([x, y])),
                    (iced_native::Event::Mouse(iced_native::mouse::Event::ButtonReleased(iced_native::mouse::Button::Right)), iced_native::event::Status::Ignored) => Some(Message::RightClick),
                    _ => None,
                }),
                Subscription::from_recipe(tcp_server::Subscription),
            ])
        }
        #[cfg(target_arch = "wasm32")] {
            Subscription::none()
        }
    }
}

#[derive(StructOpt)]
struct Args {
    #[structopt(long = "checks")]
    show_available_checks: bool,
}

#[derive(From)]
enum Error {
    Iced(iced::Error),
    #[cfg(not(target_arch = "wasm32"))]
    Icon(iced::window::icon::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Iced(e) => e.fmt(f),
            #[cfg(not(target_arch = "wasm32"))]
            Error::Icon(e) => write!(f, "failed to set app icon: {}", e),
        }
    }
}

#[wheel::main]
fn main(Args { show_available_checks }: Args) -> Result<(), Error> {
    #[cfg(not(target_arch = "wasm32"))]
    let icon = images::icon::<DynamicImage>().to_rgba8();
    State::run(Settings {
        window: window::Settings {
            size: (WIDTH, HEIGHT + if show_available_checks { 400 } else { 0 }),
            min_size: Some((WIDTH, HEIGHT)),
            max_size: if show_available_checks { Some((WIDTH, u32::MAX)) } else { Some((WIDTH, HEIGHT)) },
            resizable: show_available_checks,
            #[cfg(not(target_arch = "wasm32"))]
            icon: Some(Icon::from_rgba(icon.as_flat_samples().as_slice().to_owned(), icon.width(), icon.height())?),
            ..window::Settings::default()
        },
        flags: show_available_checks,
        ..Settings::default()
    })?;
    Ok(())
}
