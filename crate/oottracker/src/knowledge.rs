use smart_default::SmartDefault;

#[derive(Debug, SmartDefault)]
pub enum DungeonRewardLocation {
    #[default]
    Unknown,
    DekuTree,
    DodongosCavern,
    JabuJabu,
    ForestTemple,
    FireTemple,
    WaterTemple,
    ShadowTemple,
    SpiritTemple,
    LinksPocket,
}

#[derive(Debug, Default)]
pub struct Knowledge {
    pub kokiri_emerald_location: DungeonRewardLocation,
    pub goron_ruby_location: DungeonRewardLocation,
    pub zora_sapphire_location: DungeonRewardLocation,
    pub forest_medallion_location: DungeonRewardLocation,
    pub fire_medallion_location: DungeonRewardLocation,
    pub water_medallion_location: DungeonRewardLocation,
    pub shadow_medallion_location: DungeonRewardLocation,
    pub spirit_medallion_location: DungeonRewardLocation,
    pub light_medallion_location: DungeonRewardLocation,
}
