use {
    std::{
        borrow::Cow,
        collections::HashMap,
        fmt,
    },
    async_proto::Protocol,
    itertools::Itertools as _,
    rocket::{
        http::uri::{
            Formatter,
            Path,
            UriDisplay,
        },
        request::FromParam,
    },
    smart_default::SmartDefault,
    tokio::sync::watch::*,
    ootr::model::{
        DungeonReward,
        DungeonRewardLocation,
        MainDungeon,
        Medallion,
        Stone,
    },
    oottracker::{
        ModelState,
        ui::TrackerCellId,
    },
    crate::{
        CellOverlay,
        CellRender,
        CellStyle,
        LocationStyle,
    },
};

pub(crate) struct RestreamState {
    worlds: Vec<(Sender<()>, Receiver<()>, HashMap<String, ModelState>)>,
}

impl RestreamState {
    pub(crate) fn new(worlds: Vec<Vec<&str>>) -> RestreamState {
        RestreamState {
            worlds: worlds.into_iter().map(|players| {
                let (tx, rx) = channel(());
                (tx, rx, players.into_iter().map(|player| (player.to_owned(), ModelState::default())).collect())
            }).collect(),
        }
    }

    pub(crate) fn layout(&self) -> TrackerLayout {
        TrackerLayout::default() //TODO allow restreamer to set different tracker layouts
    }

    pub(crate) fn runner(&self, runner: &str) -> Option<(&Sender<()>, &Receiver<()>, &ModelState)> {
        self.worlds.iter().filter_map(|(tx, rx, players)| players.get(runner).map(move |state| (&*tx, &*rx, state))).next()
    }

    pub(crate) fn runner_mut(&mut self, runner: &str) -> Option<(&Sender<()>, &Receiver<()>, &mut ModelState)> {
        self.worlds.iter_mut().filter_map(|(tx, rx, players)| players.get_mut(runner).map(move |state| (&*tx, &*rx, state))).next()
    }
}

#[derive(SmartDefault, Protocol)]
pub(crate) enum TrackerLayout {
    #[default]
    Default,
    MultiworldExpanded,
    MultiworldCollapsed,
    MultiworldEdit,
    RslLeft,
    RslRight,
    RslEdit,
}

impl TrackerLayout {
    pub(crate) fn cells(&self) -> Vec<(TrackerCellId, u8, bool)> {
        use TrackerCellId::*;

        match self {
            TrackerLayout::Default => {
                let layout = oottracker::ui::TrackerLayout::default();
                layout.meds.into_iter().map(|med| (TrackerCellId::med_location(med), 3, true))
                    .chain(layout.meds.into_iter().map(|med| (TrackerCellId::from(med), 3, false)))
                    .chain(vec![
                        (layout.row2[0], 3, false),
                        (layout.row2[1], 3, false),
                        (KokiriEmeraldLocation, 2, true),
                        (GoronRubyLocation, 2, true),
                        (ZoraSapphireLocation, 2, true),
                        (layout.row2[2], 3, false),
                        (layout.row2[3], 3, false),
                        (KokiriEmerald, 2, false),
                        (GoronRuby, 2, false),
                        (ZoraSapphire, 2, false),
                    ])
                    .chain(layout.rest.iter().flat_map(|row|
                        row.iter().map(|&cell| (cell, 3, false))
                    ).collect_vec())
                    .chain(layout.warp_songs.into_iter().map(|med| (TrackerCellId::warp_song(med), 3, false)))
                    .collect()
            }
            TrackerLayout::MultiworldExpanded => vec![
                SwordCard, Slingshot, Skulltula, GoBk,
                Bombs, Bow, ZeldasLullaby, Minuet,
                Boomerang, Hammer, EponasSong, Bolero,
                Hookshot, Spells, SariasSong, Serenade,
                Bottle, Arrows, SunsSong, Requiem,
                MirrorShield, Strength, SongOfTime, Nocturne,
                Boots, Scale, SongOfStorms, Prelude,
            ].into_iter().map(|cell| (cell, 3, false)).collect(),
            TrackerLayout::MultiworldCollapsed => vec![
                SwordCard, Bottle, Skulltula, Strength, Scale, Spells, Slingshot, Bombs, Boomerang, GoBk,
                ZeldasLullaby, EponasSong, SariasSong, SunsSong, SongOfTime, SongOfStorms, Hookshot, Bow, Hammer, Magic,
                Minuet, Bolero, Serenade, Requiem, Nocturne, Prelude, MirrorShield, Boots, Arrows, Tunics, //TODO replace tunics with wallets once images exist
            ].into_iter().map(|cell| (cell, 3, false)).collect(),
            TrackerLayout::MultiworldEdit => vec![
                KokiriEmeraldLocation, GoronRubyLocation, ZoraSapphireLocation, LightMedallionLocation, ForestMedallionLocation, FireMedallionLocation, WaterMedallionLocation, ShadowMedallionLocation, SpiritMedallionLocation,
            ].into_iter().map(|cell| (cell, 2, true)).chain(vec![
                KokiriEmerald, GoronRuby, ZoraSapphire, LightMedallion, ForestMedallion, FireMedallion, WaterMedallion, ShadowMedallion, SpiritMedallion,
            ].into_iter().map(|cell| (cell, 2, false))).chain(vec![
                SwordCard, Bottle, Skulltula, Scale, Tunics, GoBk, //TODO replace tunics with wallets once images exist
                Slingshot, Bombs, Boomerang, Strength, Magic, Spells,
                Hookshot, Bow, Arrows, Hammer, Boots, MirrorShield,
                ZeldasLullaby, EponasSong, SariasSong, SunsSong, SongOfTime, SongOfStorms,
                Minuet, Bolero, Serenade, Requiem, Nocturne, Prelude,
            ].into_iter().map(|cell| (cell, 3, false))).collect(),
            TrackerLayout::RslLeft => vec![
                Slingshot, Bombs, Boomerang, Skulltula, GoMode, GanonMq, GanonKeys, DekuMq, Blank,
                Hookshot, Bow, Hammer, ZeldasLullaby, Minuet, ForestMq, ForestKeys, DcMq, Blank,
                Bottle, Strength, Scale, EponasSong, Bolero, FireMq, FireKeys, JabuMq, Blank,
                ChildTrade, Beans, SwordCard, SariasSong, Serenade, WaterMq, WaterKeys, IceMq, Blank,
                AdultTrade, Tunics, Triforce, SunsSong, Requiem, SpiritMq, SpiritKeys, WellMq, WellSmallKeys,
                Magic, Spells, Arrows, SongOfTime, Nocturne, ShadowMq, ShadowKeys, FortressMq, FortressSmallKeys,
                MirrorShield, Boots, Ocarina, SongOfStorms, Prelude, FreeReward, Blank, GtgMq, GtgSmallKeys,
            ].into_iter().map(|cell| (cell, 3, false)).collect(),
            TrackerLayout::RslRight => TrackerLayout::RslLeft.cells()
                .into_iter()
                .chunks(9)
                .into_iter()
                .flat_map(|row| row.collect_vec().into_iter().rev())
                .collect(),
            TrackerLayout::RslEdit => {
                let mut cells = TrackerLayout::MultiworldEdit.cells();
                cells[5].0 = GoMode; // unlike multiworld, RSL doesn't track BK mode
                cells.extend(vec![
                    ForestMq, FireMq, WaterMq, SpiritMq, ShadowMq, GanonMq,
                    ForestKeys, FireKeys, WaterKeys, SpiritKeys, ShadowKeys, GanonKeys,
                    DekuMq, DcMq, JabuMq, WellMq, FortressMq, GtgMq,
                    ChildTrade, Beans, IceMq, WellSmallKeys, FortressSmallKeys, GtgSmallKeys,
                    AdultTrade, Triforce, Ocarina, Blank, Blank, Blank,
                ].into_iter().map(|cell| (cell, 3, false)));
                cells
            }
        }
    }
}

impl<'a> FromParam<'a> for TrackerLayout {
    type Error = ();

    fn from_param(param: &'a str) -> Result<TrackerLayout, ()> {
        Ok(match param {
            "default" => TrackerLayout::Default,
            "mw-expanded" => TrackerLayout::MultiworldExpanded,
            "mw-collapsed" => TrackerLayout::MultiworldCollapsed,
            "mw-edit" => TrackerLayout::MultiworldEdit,
            "rsl-left" => TrackerLayout::RslLeft,
            "rsl-right" => TrackerLayout::RslRight,
            "rsl-edit" => TrackerLayout::RslEdit,
            _ => return Err(()),
        })
    }
}

rocket::http::impl_from_uri_param_identity!([Path] TrackerLayout);

impl fmt::Display for TrackerLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrackerLayout::Default => write!(f, "default"),
            TrackerLayout::MultiworldExpanded => write!(f, "mw-expanded"),
            TrackerLayout::MultiworldCollapsed => write!(f, "mw-collapsed"),
            TrackerLayout::MultiworldEdit => write!(f, "mw-edit"),
            TrackerLayout::RslLeft => write!(f, "rsl-left"),
            TrackerLayout::RslRight => write!(f, "rsl-right"),
            TrackerLayout::RslEdit => write!(f, "rsl-edit"),
        }
    }
}

impl UriDisplay<Path> for TrackerLayout {
    fn fmt(&self, f: &mut Formatter<'_, Path>) -> fmt::Result {
        f.write_raw(format!("{}", self))
    }
}

#[derive(Protocol)]
pub(crate) enum DoubleTrackerLayout {
    DungeonRewards,
}

impl DoubleTrackerLayout {
    pub(crate) fn cells(&self) -> Vec<DungeonReward> {
        match self {
            DoubleTrackerLayout::DungeonRewards => vec![
                DungeonReward::Stone(Stone::KokiriEmerald),
                DungeonReward::Stone(Stone::GoronRuby),
                DungeonReward::Stone(Stone::ZoraSapphire),
                DungeonReward::Medallion(Medallion::Forest),
                DungeonReward::Medallion(Medallion::Fire),
                DungeonReward::Medallion(Medallion::Water),
                DungeonReward::Medallion(Medallion::Shadow),
                DungeonReward::Medallion(Medallion::Spirit),
                DungeonReward::Medallion(Medallion::Light),
            ],
        }
    }
}

impl<'a> FromParam<'a> for DoubleTrackerLayout {
    type Error = ();

    fn from_param(param: &'a str) -> Result<DoubleTrackerLayout, ()> {
        Ok(match param {
            "dungeon-rewards" => DoubleTrackerLayout::DungeonRewards,
            _ => return Err(()),
        })
    }
}

impl fmt::Display for DoubleTrackerLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DoubleTrackerLayout::DungeonRewards => write!(f, "dungeon-rewards"),
        }
    }
}

pub(crate) fn render_double_cell(runner1: &ModelState, runner2: &ModelState, reward: DungeonReward) -> CellRender {
    let img_filename = match reward {
        DungeonReward::Medallion(med) => Cow::Owned(format!("{}_medallion", med.element().to_ascii_lowercase())),
        DungeonReward::Stone(Stone::KokiriEmerald) => Cow::Borrowed("kokiri_emerald"),
        DungeonReward::Stone(Stone::GoronRuby) => Cow::Borrowed("goron_ruby"),
        DungeonReward::Stone(Stone::ZoraSapphire) => Cow::Borrowed("zora_sapphire"),
    };
    let style = match (runner1.ram.save.quest_items.has(reward), runner2.ram.save.quest_items.has(reward)) {
        (false, false) => CellStyle::Dimmed,
        (false, true) => CellStyle::LeftDimmed,
        (true, false) => CellStyle::RightDimmed,
        (true, true) => CellStyle::Normal,
    };
    let location = (runner1.knowledge.clone() & runner2.knowledge.clone()).map(|knowledge| knowledge.dungeon_reward_locations.get(&reward).copied()).unwrap_or_default(); //TODO display contradiction errors differently?
    let loc_img_filename = match location {
        None => "unknown_text",
        Some(DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)) => "deku_text",
        Some(DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)) => "dc_text",
        Some(DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)) => "jabu_text",
        Some(DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)) => "forest_text",
        Some(DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)) => "fire_text",
        Some(DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)) => "water_text",
        Some(DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)) => "shadow_text",
        Some(DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)) => "spirit_text",
        Some(DungeonRewardLocation::LinksPocket) => "free_text",
    };
    CellRender {
        img_dir: Cow::Borrowed("xopar-images"),
        img_filename,
        style,
        overlay: CellOverlay::Location {
            style: if location.is_some() { LocationStyle::Normal } else { LocationStyle::Dimmed },
            loc_img: Cow::Borrowed(loc_img_filename),
        },
    }
}
