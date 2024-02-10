use {
    std::collections::HashMap,
    async_proto::Protocol,
    serde::{
        Deserialize,
        Serialize,
    },
    ootr::model::{
        DungeonReward,
        DungeonRewardLocation,
    },
};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Protocol)]
pub struct TrackerCtx {
    pub cfg_dungeon_info_enable: u32,
    pub cfg_dungeon_info_reward_enable: bool,
    pub cfg_dungeon_info_reward_need_compass: bool,
    pub cfg_dungeon_info_reward_need_altar: bool,
    pub cfg_dungeon_rewards: HashMap<DungeonRewardLocation, DungeonReward>,
}

impl Default for TrackerCtx {
    fn default() -> Self {
        Self {
            cfg_dungeon_info_enable: 0,
            cfg_dungeon_info_reward_enable: false,
            cfg_dungeon_info_reward_need_compass: true,
            cfg_dungeon_info_reward_need_altar: true,
            cfg_dungeon_rewards: HashMap::default(),
        }
    }
}
