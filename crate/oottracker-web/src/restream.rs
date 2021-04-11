use {
    std::{
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
        Medallion,
        Stone,
    },
    oottracker::{
        Knowledge,
        Ram,
        ui::TrackerCellId,
    },
};

pub(crate) struct RestreamState {
    worlds: Vec<(Sender<()>, Receiver<()>, Knowledge, HashMap<String, Ram>)>,
}

impl RestreamState {
    pub(crate) fn new(worlds: impl IntoIterator<Item = (Knowledge, HashMap<String, Ram>)>) -> RestreamState {
        RestreamState {
            worlds: worlds.into_iter().map(|(knowledge, players)| {
                let (tx, rx) = channel(());
                (tx, rx, knowledge, players)
            }).collect(),
        }
    }

    pub(crate) fn layout(&self) -> TrackerLayout {
        TrackerLayout::default() //TODO allow restreamer to set different tracker layouts
    }

    pub(crate) fn runner(&mut self, runner: &str) -> Option<(&Sender<()>, &Receiver<()>, ModelStateView<'_>)> {
        self.worlds.iter_mut().filter_map(|(tx, rx, knowledge, players)| players.get_mut(runner).map(move |ram| (&*tx, &*rx, ModelStateView { knowledge, ram }))).next()
    }
}

pub(crate) struct ModelStateView<'a> {
    knowledge: &'a mut Knowledge,
    ram: &'a mut Ram,
}

impl<'a> oottracker::ModelStateView for ModelStateView<'a> {
    fn knowledge(&self) -> &Knowledge { self.knowledge }
    fn ram(&self) -> &Ram { self.ram }
    fn knowledge_mut(&mut self) -> &mut Knowledge { self.knowledge }
    fn ram_mut(&mut self) -> &mut Ram { self.ram }
}

#[derive(SmartDefault, Protocol)]
pub(crate) enum TrackerLayout {
    #[default]
    Default,
    MultiworldExpanded,
    MultiworldCollapsed,
    MultiworldEdit,
}

impl TrackerLayout {
    pub(crate) fn cells(&self) -> Box<dyn Iterator<Item = (TrackerCellId, u8, bool)>> {
        use TrackerCellId::*;

        match self {
            TrackerLayout::Default => {
                let layout = oottracker::ui::TrackerLayout::default();
                Box::new(layout.meds.into_iter().map(|med| (TrackerCellId::med_location(med), 3, true))
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
                    .chain(layout.warp_songs.into_iter().map(|med| (TrackerCellId::warp_song(med), 3, false))))
            }
            TrackerLayout::MultiworldExpanded => Box::new(vec![
                KokiriSword, Slingshot, Skulltula, GoBk,
                Bombs, Bow, ZeldasLullaby, Minuet,
                Boomerang, Hammer, EponasSong, Bolero,
                Hookshot, Spells, SariasSong, Serenade,
                Bottle, Arrows, SunsSong, Requiem,
                MirrorShield, Strength, SongOfTime, Nocturne,
                Boots, Scale, SongOfStorms, Prelude,
            ].into_iter().map(|cell| (cell, 3, false))),
            TrackerLayout::MultiworldCollapsed => Box::new(vec![
                KokiriSword, Bottle, Skulltula, Strength, Scale, Spells, Slingshot, Bombs, Boomerang, GoBk,
                ZeldasLullaby, EponasSong, SariasSong, SunsSong, SongOfTime, SongOfStorms, Hookshot, Bow, Hammer, Magic,
                Minuet, Bolero, Serenade, Nocturne, Requiem, Prelude, MirrorShield, Boots, Arrows, Tunics, //TODO replace tunics with wallets once images exist
            ].into_iter().map(|cell| (cell, 3, false))),
            TrackerLayout::MultiworldEdit => Box::new(vec![
                KokiriEmeraldLocation, GoronRubyLocation, ZoraSapphireLocation, LightMedallionLocation, ForestMedallionLocation, FireMedallionLocation, WaterMedallionLocation, ShadowMedallionLocation, SpiritMedallionLocation,
            ].into_iter().map(|cell| (cell, 2, true)).chain(vec![
                KokiriEmerald, GoronRuby, ZoraSapphire, LightMedallion, ForestMedallion, FireMedallion, WaterMedallion, ShadowMedallion, SpiritMedallion,
            ].into_iter().map(|cell| (cell, 2, false))).chain(vec![
                KokiriSword, Bottle, Skulltula, Scale, Tunics, GoBk, //TODO replace tunics with wallets once images exist
                Slingshot, Bombs, Boomerang, Strength, Magic, Spells,
                Hookshot, Bow, Arrows, Hammer, Boots, MirrorShield,
                ZeldasLullaby, EponasSong, SariasSong, SunsSong, SongOfTime, SongOfStorms,
                Minuet, Bolero, Serenade, Nocturne, Requiem, Prelude,
            ].into_iter().map(|cell| (cell, 3, false))))
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
        }
    }
}

impl UriDisplay<Path> for TrackerLayout {
    fn fmt(&self, f: &mut Formatter<'_, Path>) -> fmt::Result {
        f.write_raw(format!("{}", self))
    }
}

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
