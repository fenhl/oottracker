#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::ops::{
        AddAssign,
        Sub,
    },
    async_proto::Protocol,
    enum_iterator::IntoEnumIterator as _,
    itertools::Itertools as _,
    semver::Version,
    crate::{
        model::{
            DungeonReward,
            DungeonRewardLocation,
        },
        save::GameMode,
    },
};
pub use crate::{
    knowledge::Knowledge,
    ram::Ram,
    save::Save,
};

pub mod check;
pub mod checks;
pub mod firebase;
pub mod github;
pub mod info_tables;
mod item;
mod item_ids;
pub mod knowledge;
pub mod model;
pub mod net;
pub mod proto;
pub mod ram;
pub mod region;
pub mod save;
mod scene;
mod settings;
pub mod ui;
pub mod websocket;

#[derive(Debug, Default, Clone, PartialEq, Eq, Protocol)]
pub struct ModelState {
    pub knowledge: Knowledge,
    pub ram: Ram,
}

impl ModelState {
    pub fn update_knowledge(&mut self) {
        if self.ram.save.game_mode != GameMode::Gameplay { return } //TODO read knowledge from inventory preview on file select?
        if !self.knowledge.settings.starting_age.is_known() {
            self.knowledge.settings.starting_age = if self.ram.save.is_adult { settings::StartingAgeKnowledge::adult() } else { settings::StartingAgeKnowledge::child() }
        } //TODO handle random starting age with unknown randomized_settings similarly
        if let Ok(reward) = DungeonReward::into_enum_iter().filter(|reward| self.ram.save.quest_items.has(reward)).exactly_one() {
            self.knowledge.set_dungeon_reward_location(reward, DungeonRewardLocation::LinksPocket);
        }
    }
}

impl AddAssign<ModelDelta> for ModelState {
    fn add_assign(&mut self, rhs: ModelDelta) {
        self.knowledge = rhs.knowledge;
        self.ram += rhs.ram;
    }
}

impl<'a, 'b> Sub<&'b ModelState> for &'a ModelState {
    type Output = ModelDelta;

    fn sub(self, rhs: &ModelState) -> ModelDelta {
        ModelDelta {
            knowledge: self.knowledge.clone(), //TODO only include new knowledge?
            ram: &self.ram - &rhs.ram,
        }
    }
}

/// The difference between two model states.
#[derive(Debug, Clone, Protocol)]
pub struct ModelDelta {
    knowledge: Knowledge, //TODO use a separate knowledge delta format?
    ram: ram::Delta,
}

pub fn version() -> Version {
    let version = Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version");
    assert_eq!(version, oottracker_derive::version!());
    version
}
