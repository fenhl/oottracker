#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::ops::{
        AddAssign,
        Sub,
    },
    async_proto::Protocol,
    enum_iterator::all,
    itertools::Itertools as _,
    semver::Version,
    serde::{
        Deserialize,
        Serialize,
    },
    ootr::{
        check::Check,
        model::*,
    },
    crate::{
        info_tables::InfTable55,
        ram::Pad,
        save::{
            DungeonItems,
            GameMode,
        },
    },
};
pub use crate::{
    ctx::TrackerCtx,
    knowledge::Knowledge,
    ram::Ram,
    save::Save,
};

pub mod checks;
pub mod ctx;
pub mod github;
pub mod info_tables;
mod item_ids;
pub mod knowledge;
pub mod net;
pub mod proto;
pub mod ram;
pub mod region;
pub mod save;
mod scene;
mod text;
pub mod ui;
pub mod websocket;

#[derive(Debug, Default, Clone, PartialEq, Eq, Protocol, Deserialize, Serialize)]
pub struct ModelState {
    pub knowledge: Knowledge,
    pub tracker_ctx: TrackerCtx,
    pub ram: Ram,
}

impl ModelState {
    pub fn update_knowledge(&mut self) {
        if self.ram.save.game_mode != GameMode::Gameplay { return } //TODO read knowledge from inventory preview on file select?
        // immediate knowledge
        // read dungeon reward info if the player is looking at the dungeon info screen in the pause menu
        let button_pressed = match self.tracker_ctx.cfg_dungeon_info_enable {
            0 => false,
            1 => self.ram.input_p1_raw_pad.contains(Pad::A),
            2.. => self.ram.input_p1_raw_pad.contains(Pad::D_DOWN),
        };
        if button_pressed && self.ram.pause_state == 6 && self.ram.pause_screen_idx == 0 && !self.ram.pause_changing && self.tracker_ctx.cfg_dungeon_info_reward_enable {
            for (&location, &reward) in &self.tracker_ctx.cfg_dungeon_rewards {
                let mut known = true;
                if self.tracker_ctx.cfg_dungeon_info_reward_need_altar {
                    known &= match reward {
                        DungeonReward::Medallion(_) => self.ram.save.inf_table.55.contains(InfTable55::TOT_ALTAR_READ_MEDALLION_LOCATIONS),
                        DungeonReward::Stone(_) => self.ram.save.inf_table.55.contains(InfTable55::TOT_ALTAR_READ_STONE_LOCATIONS),
                    };
                }
                if self.tracker_ctx.cfg_dungeon_info_reward_need_compass {
                    match location {
                        DungeonRewardLocation::Dungeon(dungeon) => known &= self.ram.save.dungeon_items.get(Dungeon::Main(dungeon)).contains(DungeonItems::COMPASS),
                        DungeonRewardLocation::LinksPocket => {}
                    }
                }
                if known {
                    self.knowledge.dungeon_reward_locations.insert(reward, location);
                }
            }
        }
        // read the current text box for various pieces of information
        if self.ram.current_text_box_id != 0 {
            if let Ok(new_knowledge) = self.knowledge.clone() & text::read_knowledge(&self.ram.text_box_contents[..]) {
                self.knowledge = new_knowledge;
            } else {
                //TODO report/log error?
            }
        }

        // derived knowledge
        // dungeon reward shuffle doesn't exist yet, so if we have exactly 1 reward, it must have been on Links Pocket
        if let Ok(reward) = all().filter(|reward| self.ram.save.quest_items.has(reward)).exactly_one() {
            self.knowledge.dungeon_reward_locations.insert(reward, DungeonRewardLocation::LinksPocket);
        }
        // dungeon reward shuffle doesn't exist yet, so if we know the locations of all but 1 reward, the 9th can be determined by process of elimination
        if let Some((reward,)) = all().filter(|reward| !self.knowledge.dungeon_reward_locations.contains_key(reward)).collect_tuple() {
            let (dungeon,) = all().filter(|dungeon| !self.knowledge.dungeon_reward_locations.values().any(|&loc| loc == DungeonRewardLocation::Dungeon(*dungeon))).collect_tuple().expect("exactly one reward left but not exactly one reward location left");
            self.knowledge.dungeon_reward_locations.insert(reward, DungeonRewardLocation::Dungeon(dungeon));
        }
    }
}

impl AddAssign<ModelDelta> for ModelState {
    fn add_assign(&mut self, rhs: ModelDelta) {
        let ModelDelta { knowledge, tracker_ctx, ram } = rhs;
        self.knowledge = knowledge;
        if let Some(tracker_ctx) = tracker_ctx { self.tracker_ctx = tracker_ctx }
        self.ram += ram;
    }
}

impl<'a, 'b> Sub<&'b ModelState> for &'a ModelState {
    type Output = ModelDelta;

    fn sub(self, rhs: &ModelState) -> ModelDelta {
        let ModelState { knowledge, tracker_ctx, ram } = self;
        ModelDelta {
            knowledge: knowledge.clone(), //TODO only include new knowledge?
            tracker_ctx: (*tracker_ctx != rhs.tracker_ctx).then(|| tracker_ctx.clone()),
            ram: ram - &rhs.ram,
        }
    }
}

/// The difference between two model states.
#[derive(Debug, Clone, Protocol)]
pub struct ModelDelta {
    knowledge: Knowledge, //TODO use a separate knowledge delta format?\
    tracker_ctx: Option<TrackerCtx>,
    ram: ram::Delta,
}

pub fn version() -> Version {
    let version = Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version");
    assert_eq!(version, oottracker_derive::version!());
    version
}
