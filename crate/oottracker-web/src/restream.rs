use {
    std::{
        borrow::Cow,
        collections::HashMap,
    },
    tokio::sync::watch::*,
    ootr::model::{
        DungeonReward,
        DungeonRewardLocation,
        MainDungeon,
        Stone,
    },
    oottracker::{
        ModelState,
        ui::{
            CellOverlay,
            CellRender,
            CellStyle,
            LocationStyle,
            TrackerLayout,
        },
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
        TrackerLayout::default() //TODO allow restreamer to set different default layouts?
    }

    pub(crate) fn runner(&self, runner: &str) -> Option<(&Sender<()>, &Receiver<()>, &ModelState)> {
        self.worlds.iter().filter_map(|(tx, rx, players)| players.get(runner).map(move |state| (&*tx, &*rx, state))).next()
    }

    pub(crate) fn runner_mut(&mut self, runner: &str) -> Option<(&Sender<()>, &Receiver<()>, &mut ModelState)> {
        self.worlds.iter_mut().filter_map(|(tx, rx, players)| players.get_mut(runner).map(move |state| (&*tx, &*rx, state))).next()
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
            loc_dir: Cow::Borrowed("xopar-images"),
            loc_img: Cow::Borrowed(loc_img_filename),
            style: if location.is_some() { LocationStyle::Normal } else { LocationStyle::Dimmed },
        },
    }
}
