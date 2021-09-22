#![allow(unused, warnings)] //TODO

//! There are two distinct concepts this tracker deals with that are both called “scenes”:
//!
//! * What I will be calling “game scenes” are sections of the game world that are loaded individually. They are listed at <https://wiki.cloudmodding.com/oot/Scene_Table/NTSC_1.0>. Some of the game state is organized by game scene.
//! * What I will be calling “ER scenes” are groups of regions as defined in the randomizer's region files. The entrance randomizer code will not connect an exit to an entrance in the same ER scene. This is the only situation ER scenes are used. Regions in dungeons are considered to be in an ER scene with the same name as the dungeon.
//!
//! There is a many-to-many relationship between ER scenes and game scenes. For example:
//!
//! * The “Market Entrance” ER scene consists of multiple game scenes, which are loaded depending on the current time of day and age.
//! * The “Gerudo Fortress” ER scene contains both the exterior of the fortress and the “Thieves' Hideout” interior.
//! * Both bazaars are in the same game scene, but they are considered separate ER scenes.
//! * The “Kak Potion Shop Front” and “Kak Potion Shop Back” regions in the randomizer each have their own ER scene, though this may be a bug.
//!
//! This module only concerns itself with game scenes. ER scenes are currently not used in this tracker since ER logic is unimplemented.
//!
//! This module also contains types representing the “permanent scene flags” and “gold skulltulas” sections of [save data](crate::save::Save).
//!
//! The entry points are the types [`SceneFlags`] and [`GoldSkulltulas`]. All other types appear in their fields.

use crate::Ram;

pub(crate) trait FlagsScene {
    fn set_chests(&mut self, chests: u32);
    fn set_switches(&mut self, switches: u32);
    fn set_room_clear(&mut self, room_clear: u32);
}

oottracker_derive::scene_flags! {
    pub struct SceneFlags {
        0x00: DekuTree {
            //TODO return region name based on room number and/or coords, missing regions: Deku Tree Slingshot Room, Deku Tree Basement Backroom, Deku Tree Boss Room
            //TODO MQ support
            region_name: |_| DekuTreeLobby,
            chests: {
                "Deku Tree Compass Room Side Chest" = 0x0000_0040,
                "Deku Tree Slingshot Room Side Chest" = 0x0000_0020,
                "Deku Tree Basement Chest" = 0x0000_0010,
                "Deku Tree Map Chest" = 0x0000_0008,
                "Deku Tree Compass Chest" = 0x0000_0004,
                "Deku Tree Slingshot Chest" = 0x0000_0002,
            },
            switches: {
                1 for "Deku Tree GS Basement Backroom" = 0x0004_0000,
                BASEMENT_PUSHED_BLOCK /*vanilla*/ = 0x0001_0000,
                BASEMENT_BURN_FIRST_WEB_TO_BACK_ROOM /*vanilla*/ = 0x0000_0200,
                0 for "Deku Tree GS Basement Backroom" = 0x0000_0100,
                0 for DekuTreeLobby -> DekuTreeBossRoom = 0x0000_0040,
                1 for DekuTreeLobby -> DekuTreeBasementBackroom = 0x0000_0010,
                LIGHT_TORCHES_AFTER_WATER_ROOM /*vanilla*/ = 0x0000_0008,
            },
            room_clear: {
                SCRUBS_231_PUZZLE = 0x0000_0200,
                0 for DekuTreeLobby -> DekuTreeSlingshotRoom = 0x0000_0002,
            },
        },
        0x01: DodongosCavern {
            chests: {
                "Dodongos Cavern End of Bridge Chest" = 0x0000_0400,
                "Dodongos Cavern Map Chest" = 0x0000_0100,
                "Dodongos Cavern Bomb Flower Platform Chest" = 0x0000_0040,
                "Dodongos Cavern Compass Chest" = 0x0000_0020,
                "Dodongos Cavern Bomb Bag Chest" = 0x0000_0010,
            },
            switches: {
                0 for MqDodongosCavernLobby -> MqDodongosCavernLowerRightSide = 0x8000_0000,
                0 for MqDodongosCavernLowerRightSide -> MqDodongosCavernBombBagArea = 0x0800_0000, //TODO confirm, logic says this entrance needs slingshot but that doesn't seem to be the case
                0 for DodongosCavernLobby -> DodongosCavernStaircaseRoom = 0x0200_0000,
                0 for DodongosCavernLobby -> DodongosCavernFarBridge = 0x0000_0400,
                0 for DodongosCavernBeginning -> AmbigDodongosCavernLobby = 0x0000_0080,
            },
        },
        0x02: JabuJabu {
            chests: {
                "Jabu Jabus Belly Compass Chest" = 0x0000_0010,
                "Jabu Jabus Belly Map Chest" = 0x0000_0004,
                "Jabu Jabus Belly Boomerang Chest" = 0x0000_0002,
            },
        },
        0x03: ForestTemple {
            chests: {
                "Forest Temple Blue Poe Chest" = 0x0000_8000,
                "Forest Temple Boss Key Chest" = 0x0000_4000,
                "Forest Temple Red Poe Chest" = 0x0000_2000,
                "Forest Temple Bow Chest" = 0x0000_1000,
                "Forest Temple Basement Chest" = 0x0000_0800,
                "Forest Temple Well Chest" = 0x0000_0200,
                "Forest Temple Falling Ceiling Room Chest" = 0x0000_0080,
                "Forest Temple Raised Island Courtyard Chest" = 0x0000_0020,
                "Forest Temple Eye Switch Chest" = 0x0000_0010,
                "Forest Temple First Room Chest" = 0x0000_0008,
                "Forest Temple Floormaster Chest" = 0x0000_0004,
                "Forest Temple Map Chest" = 0x0000_0002,
                "Forest Temple First Stalfos Chest" = 0x0000_0001,
            },
            switches: {
                BETH_DEFEATED /*vanilla*/ = 0x4000_0000,
                JOELLE_DEFEATED /*vanilla*/ = 0x2000_0000,
            },
            room_clear: {
                0 for ForestTempleNwOutdoors -> ForestTempleOutdoorsHighBalconies = 0x0000_0400,
            },
            gold_skulltulas: {
                "Forest Temple GS Level Island Courtyard" = 0x04,
            },
        },
        0x04: FireTemple {
            chests: {
                "Fire Temple Scarecrow Chest" = 0x0000_2000,
                "Fire Temple Boss Key Chest" = 0x0000_1000,
                "Fire Temple Boulder Maze Shortcut Chest" = 0x0000_0800,
                "Fire Temple Map Chest" = 0x0000_0400,
                "Fire Temple Highest Goron Chest" = 0x0000_0200,
                "Fire Temple Boulder Maze Side Room Chest" = 0x0000_0100,
                "Fire Temple Compass Chest" = 0x0000_0080,
                "Fire Temple Boulder Maze Upper Chest" = 0x0000_0040,
                "Fire Temple Megaton Hammer Chest" = 0x0000_0020,
                "Fire Temple Big Lava Room Lower Open Door Chest" = 0x0000_0010,
                "Fire Temple Boulder Maze Lower Chest" = 0x0000_0008,
                "Fire Temple Big Lava Room Blocked Door Chest" = 0x0000_0004,
                "Fire Temple Near Boss Chest" = 0x0000_0002,
                "Fire Temple Flare Dancer Chest" = 0x0000_0001,
            },
        },
        0x05: WaterTemple {
            chests: {
                "Water Temple Dragon Chest" = 0x0000_0400,
                "Water Temple Compass Chest" = 0x0000_0200,
                "Water Temple Central Bow Target Chest" = 0x0000_0100,
                "Water Temple Longshot Chest" = 0x0000_0080,
                "Water Temple Central Pillar Chest" = 0x0000_0040,
                "Water Temple Boss Key Chest" = 0x0000_0020,
                "Water Temple River Chest" = 0x0000_0008,
                "Water Temple Map Chest" = 0x0000_0004,
                "Water Temple Torches Chest" = 0x0000_0002,
                "Water Temple Cracked Wall Chest" = 0x0000_0001,
            },
            switches: {
                event "Raise Water Level" /*vanilla*/ = 0x4000_0000,
                //WATER_LEVEL_MID /*vanilla*/ = 0x2000_0000,
                //WATER_LEVEL_LOW /*vanilla*/ = 0x1000_0000,
            },
        },
        0x06: SpiritTemple {
            chests: {
                "Spirit Temple Hallway Left Invisible Chest" = 0x0020_0000,
                "Spirit Temple Hallway Right Invisible Chest" = 0x0010_0000,
                "Spirit Temple Topmost Chest" = 0x0004_0000,
                "Spirit Temple Statue Room Northeast Chest" = 0x0000_8000,
                "Spirit Temple First Mirror Right Chest" = 0x0000_4000,
                "Spirit Temple First Mirror Left Chest" = 0x0000_2000,
                "Spirit Temple Child Climb East Chest" = 0x0000_1000,
                "Spirit Temple Boss Key Chest" = 0x0000_0400,
                "Spirit Temple Child Bridge Chest" = 0x0000_0100,
                "Spirit Temple Early Adult Right Chest" = 0x0000_0080,
                "Spirit Temple Child Climb North Chest" = 0x0000_0040,
                "Spirit Temple Near Four Armos Chest" = 0x0000_0020,
                "Spirit Temple Compass Chest" = 0x0000_0010,
                "Spirit Temple Map Chest" = 0x0000_0008,
                "Spirit Temple Statue Room Hand Chest" = 0x0000_0004,
                "Spirit Temple Sun Block Room Chest" = 0x0000_0002,
                "Spirit Temple Child Early Torches Chest" = 0x0000_0001,
            },
        },
        0x07: ShadowTemple {
            chests: {
                "Shadow Temple Invisible Blades Invisible Chest" = 0x0040_0000,
                "Shadow Temple Wind Hint Chest" = 0x0020_0000,
                "Shadow Temple After Wind Hidden Chest" = 0x0010_0000,
                "Shadow Temple Invisible Floormaster Chest" = 0x0000_2000,
                "Shadow Temple Invisible Blades Visible Chest" = 0x0000_1000,
                "Shadow Temple Boss Key Chest" = 0x0000_0800,
                "Shadow Temple Spike Walls Left Chest" = 0x0000_0400,
                "Shadow Temple Invisible Spikes Chest" = 0x0000_0200,
                "Shadow Temple After Wind Enemy Chest" = 0x0000_0100,
                "Shadow Temple Hover Boots Chest" = 0x0000_0080,
                "Shadow Temple Falling Spikes Upper Chest" = 0x0000_0040,
                "Shadow Temple Falling Spikes Lower Chest" = 0x0000_0020,
                "Shadow Temple Falling Spikes Switch Chest" = 0x0000_0010,
                "Shadow Temple Compass Chest" = 0x0000_0008,
                "Shadow Temple Early Silver Rupee Chest" = 0x0000_0004,
                "Shadow Temple Map Chest" = 0x0000_0002,
            },
            collectible: {
                "Shadow Temple Freestanding Key" = 0x0000_0002,
            },
        },
        0x08: BottomOfTheWell {
            chests: {
                "Bottom of the Well Invisible Chest" = 0x0010_0000,
                "Bottom of the Well Underwater Front Chest" = 0x0001_0000,
                "Bottom of the Well Center Skulltula Chest" = 0x0000_4000,
                "Bottom of the Well Like Like Chest" = 0x0000_1000,
                "Bottom of the Well Fire Keese Chest" = 0x0000_0400,
                "Bottom of the Well Underwater Left Chest" = 0x0000_0200,
                "Bottom of the Well Front Left Fake Wall Chest" = 0x0000_0100,
                "Bottom of the Well Map Chest" = 0x0000_0080,
                "Bottom of the Well Right Bottom Fake Wall Chest" = 0x0000_0020,
                "Bottom of the Well Back Left Bombable Chest" = 0x0000_0010,
                "Bottom of the Well Lens of Truth Chest" = 0x0000_0008,
                "Bottom of the Well Front Center Bombable Chest" = 0x0000_0004,
                "Bottom of the Well Compass Chest" = 0x0000_0002,
            },
            collectible: {
                "Bottom of the Well Freestanding Key" = 0x0000_0002,
            },
        },
        0x09: IceCavern {
            chests: {
                "Ice Cavern Iron Boots Chest" = 0x0000_0004,
                "Ice Cavern Compass Chest" = 0x0000_0002,
                "Ice Cavern Map Chest" = 0x0000_0001,
            },
            collectible: {
                "Ice Cavern Freestanding PoH" = 0x0000_0002,
            },
        },
        0x0a: GanonsCastleTower {
            chests: {
                "Ganons Tower Boss Key Chest" = 0x0000_0800,
            },
        },
        0x0b: GerudoTrainingGround {
            chests: {
                "Gerudo Training Ground Heavy Block Third Chest" = 0x0010_0000,
                "Gerudo Training Ground Lobby Left Chest" = 0x0008_0000,
                "Gerudo Training Ground Hammer Room Clear Chest" = 0x0004_0000,
                "Gerudo Training Ground Before Heavy Block Chest" = 0x0002_0000,
                "Gerudo Training Ground Hammer Room Switch Chest" = 0x0001_0000,
                "Gerudo Training Ground Heavy Block First Chest" = 0x0000_8000,
                "Gerudo Training Ground Heavy Block Second Chest" = 0x000_4000,
                "Gerudo Training Ground Underwater Silver Rupee Chest" = 0x0000_2000,
                "Gerudo Training Ground Maze Path Final Chest" = 0x0000_1000,
                "Gerudo Training Ground Hidden Ceiling Chest" = 0x0000_0800,
                "Gerudo Training Ground Maze Path Second Chest" = 0x0000_0400,
                "Gerudo Training Ground Maze Path Third Chest" = 0x0000_0200,
                "Gerudo Training Ground Maze Right Side Chest" = 0x0000_0100,
                "Gerudo Training Ground Lobby Right Chest" = 0x0000_0080,
                "Gerudo Training Ground Maze Path First Chest" = 0x0000_0040,
                "Gerudo Training Ground Maze Right Central Chest" = 0x0000_0020,
                "Gerudo Training Ground Near Scarecrow Chest" = 0x0000_0010,
                "Gerudo Training Ground Eye Statue Chest" = 0x0000_0008,
                "Gerudo Training Ground Heavy Block Fourth Chest" = 0x0000_0004,
                "Gerudo Training Ground Beamos Chest" = 0x0000_0002,
                "Gerudo Training Ground Stalfos Chest" = 0x0000_0001,
            },
            switches: {
                0 for GerudoTrainingGroundLobby -> GerudoTrainingGroundLavaRoom = 0x4000_0000,
            },
            collectible: {
                "Gerudo Training Ground Freestanding Key" = 0x0000_0002,
            },
        },
        0x0c: ThievesHideout {
            region_name: GerudoFortress,
            switches: {
                "GF Gerudo Membership Card" = 0x0000_0004,
            },
            collectible: {
                "GF South F2 Carpenter" = 0x0000_8000,
                "GF South F1 Carpenter" = 0x0000_4000,
                "GF North F1 Carpenter" = 0x0000_1000,
                "GF North F2 Carpenter" = 0x0000_0400,
            },
        },
        0x0d: InsideGanonsCastle {
            chests: {
                "Ganons Castle Spirit Trial Invisible Chest" = 0x0010_0000,
                "Ganons Castle Spirit Trial Crystal Switch Chest" = 0x0004_0000,
                "Ganons Castle Light Trial Lullaby Chest" = 0x0002_0000,
                "Ganons Castle Light Trial Invisible Enemies Chest" = 0x0001_0000,
                "Ganons Castle Light Trial Third Right Chest" = 0x0000_8000,
                "Ganons Castle Light Trial First Right Chest" = 0x0000_4000,
                "Ganons Castle Light Trial Third Left Chest" = 0x0000_2000,
                "Ganons Castle Light Trial First Left Chest" = 0x0000_1000,
                "Ganons Castle Light Trial Second Left Chest" = 0x0000_0800,
                "Ganons Castle Light Trial Second Right Chest" = 0x0000_0400,
                "Ganons Castle Forest Trial Chest" = 0x0000_0200,
                "Ganons Castle Shadow Trial Front Chest" = 0x0000_0100,
                "Ganons Castle Water Trial Left Chest" = 0x0000_0080,
                "Ganons Castle Water Trial Right Chest" = 0x0000_0040,
                "Ganons Castle Shadow Trial Golden Gauntlets Chest" = 0x0000_0020,
            },
        },
        0x0e: GanonsTowerCollapsing {
            region_name: GanonsCastleTower,
        },
        0x0f: InsideGanonsCastleCollapsing {
            region_name: GanonsCastleTower, // rando considers the entire collapse logically part of the tower
        },
        0x10: TreasureChestGame {
            chests: {
                "Market Treasure Chest Game Reward" = 0x0000_0400,
            },
        },
        0x11: GohmasLair {},
        0x12: KingDodongosLair {
            chests: {
                "Dodongos Cavern Boss Room Chest" = 0x0000_0001,
            },
        },
        0x13: BarinadesLair {},
        0x14: PhantomGanonsLair {},
        0x15: VolvagiasLair {},
        0x16: MorphasLair {},
        0x17: TwinrovasLair {},
        0x18: BongoBongosLair {},
        0x19: GanondorfsLair {},
        0x1a: TowerCollapseExterior {},
        0x1b: MarketEntranceChildDay {},
        0x1c: MarketEntranceChildNight {},
        0x1d: MarketEntranceAdult {},
        0x1e: BackAlleyDay {},
        0x1f: BackAlleyNight {},
        0x20: MarketChildDay {},
        0x21: MarketChildNight {},
        0x22: MarketAdult {},
        0x23: TotEntranceChildDay {},
        0x24: TotEntranceChildNight {},
        0x25: TotEntranceAdult {},
        0x26: KnowItAllHouse {},
        0x27: HouseOfTwins {},
        0x28: MidosHouse {
            chests: {
                "KF Midos Bottom Right Chest" = 0x0000_0008,
                "KF Midos Bottom Left Chest" = 0x0000_0004,
                "KF Midos Top Right Chest" = 0x0000_0002,
                "KF Midos Top Left Chest" = 0x0000_0001,
            },
        },
        0x29: SariasHouse {},
        0x2a: CarpenterBossHouse {},
        0x2b: ManInGreenHouse {},
        0x2c: Bazaar {},
        0x2d: KokiriShop {},
        0x2e: GoronShop {},
        0x2f: ZoraShop {},
        0x30: KakPotionShop {},
        0x31: MarketPotionShop {},
        0x32: BombchuShop {},
        0x33: MaskShop {},
        0x34: LinksHouse {},
        0x35: DogLadyHouse {},
        0x36: Stables {},
        0x37: ImpasHouse {
            collectible: {
                "Kak Impas House Freestanding PoH" = 0x0000_0002,
            },
        },
        0x38: Lab {},
        0x39: CarpenterTent {},
        0x3a: DampesHouse {},
        0x3b: GreatFairyFountainUpgrades {
            switches: { //TODO generalize as upgrade Great Fairy rewards depending on knowledge
                "DMT Great Fairy Reward" = 0x0100_0000,
                "DMC Great Fairy Reward" = 0x0001_0000,
                "OGC Great Fairy Reward" = 0x0000_0100,
            },
        },
        0x3c: FairyGrotto {},
        0x3d: GreatFairyFountainSpells {},
        0x3e: Grotto {
            chests: { //TODO generalize as generic grotto chests depending on knowledge
                "DMC Upper Grotto Chest" = 0x0400_0000,
                "DMT Storms Grotto Chest" = 0x0040_0000,
                "LW Near Shortcuts Grotto Chest" = 0x0010_0000,
                "SFM Wolfos Grotto Chest" = 0x0002_0000,
                "KF Storms Grotto Chest" = 0x0000_1000,
                "Kak Redead Grotto Chest" = 0x0000_0400,
                "ZR Open Grotto Chest" = 0x0000_0200,
                "Kak Open Grotto Chest" = 0x0000_0100,
                "HF Open Grotto Chest" = 0x0000_0008,
                "HF Southeast Grotto Chest" = 0x0000_0004,
                "HF Near Market Grotto Chest" = 0x0000_0001,
            },
            collectible: {
                "HF Tektite Grotto Freestanding PoH" = 0x0000_0002,
            },
        },
        0x3f: HeartPieceGrave {
            chests: {
                "Graveyard Heart Piece Grave Chest" = 0x0000_0001,
            },
        },
        0x40: ShieldGrave {
            chests: {
                "Graveyard Shield Grave Chest" = 0x0000_0001,
            },
        },
        0x41: RoyalFamilysTomb {
            chests: {
                "Graveyard Composers Grave Chest" = 0x0000_0001,
            },
        },
        0x42: ShootingGallery {},
        0x43: TempleOfTime {},
        0x44: ChamberOfTheSages {},
        0x45: CastleHedgeMazeDay {},
        0x46: CastleHedgeMazeNight {},
        0x47: CutsceneMap {},
        0x48: WindmillAndDampesGrave {
            chests: {
                "Graveyard Hookshot Chest" = 0x0000_0001,
            },
            collectible: {
                "Graveyard Dampe Race Freestanding PoH" = 0x0000_0080,
                "Kak Windmill Freestanding PoH" = 0x0000_0002,
            },
            unused: {
                TRIFORCE_PIECES = 0xffff_ffff,
            },
        },
        0x49: FishingHole {},
        0x4a: HcGarden {},
        0x4b: BombchuBowling {},
        0x4c: LlrHouseAndTower {
            collectible: {
                "LLR Freestanding PoH" = 0x0000_0002,
            },
        },
        0x4d: GuardHouse {},
        0x4e: OddMedicineBuilding {},
        0x4f: GanonsLair {},
        0x50: HouseOfSkulltula {},
        0x51: HyruleField {
            switches: {
                0 for HyruleField -> HfFairyGrotto = 0x0001_0000,
                0 for HyruleField -> HfNearMarketGrotto = 0x0000_4000,
                0 for HyruleField -> HfSoutheastGrotto = 0x0000_0100,
            },
        },
        0x52: KakarikoVillage {},
        0x53: Graveyard {
            collectible: {
                "Graveyard Dampe Gravedigging Tour" = 0x0000_0100,
                "Graveyard Freestanding PoH" = 0x0000_0010,
            },
        },
        0x54: ZoraRiver {
            switches: {
                0 for ZoraRiver -> ZrFairyGrotto = 0x0000_0020,
            },
            collectible: {
                "ZR Near Domain Freestanding PoH" = 0x0000_0800,
                "ZR Near Open Grotto Freestanding PoH" = 0x0000_0010,
            },
        },
        0x55: KokiriForest {
            chests: {
                "KF Kokiri Sword Chest" = 0x0000_0001,
            },
        },
        0x56: SacredForestMeadow {},
        0x57: LakeHylia {
            chests: {
                "LH Sun" = 0x0000_0001,
            },
            collectible: {
                "LH Freestanding PoH" = 0x4000_0000,
            },
        },
        0x58: ZorasDomain {
            chests: {
                "ZD Chest" = 0x0000_0001,
            },
        },
        0x59: ZorasFountain {
            collectible: {
                "ZF Bottom Freestanding PoH" = 0x0010_0000,
                "ZF Iceberg Freestanding PoH" = 0x0000_0002,
            },
        },
        0x5a: GerudoValley {
            chests: {
                "GV Chest" = 0x0000_0001,
            },
            collectible: {
                "GV Crate Freestanding PoH" = 0x0000_0004,
                "GV Waterfall Freestanding PoH" = 0x0000_0002,
            },
        },
        0x5b: LostWoods {
            switches: {
                0 for LwBeyondMido -> LwScrubsGrotto = 0x8000_0000,
                0 for LostWoods -> LwNearShortcutsGrotto = 0x0002_0000,
            },
        },
        0x5c: DesertColossus {
            chests: {
                "Spirit Temple Silver Gauntlets Chest" = 0x0000_0800,
                "Spirit Temple Mirror Shield Chest" = 0x0000_0200,
            },
            collectible: {
                "Colossus Freestanding PoH" = 0x0000_2000,
            },
        },
        0x5d: GerudoFortress {
            chests: {
                "GF Chest" = 0x0000_0001,
            },
            switches: {
                event "GF Gate Open" = 0x0000_0008,
            },
        },
        0x5e: HauntedWasteland {
            chests: {
                "Wasteland Chest" = 0x0000_0001,
            },
        },
        0x5f: HyruleCastle {},
        0x60: DeathMountainTrail {
            chests: {
                "DMT Chest" = 0x0000_0002,
            },
            switches: {
                0 for DeathMountainSummit -> DmtCowGrotto = 0x8000_0000,
                DMT_TO_SUMMIT_SECOND_BOULDER = 0x0000_0400,
                DMT_TO_SUMMIT_FIRST_BOULDER = 0x0000_0100,
                PLANT_BEAN = 0x0000_0040,
                BLOW_UP_DC_ENTRANCE = 0x0000_0010,
                0 for DeathMountainSummit -> DmtGreatFairyFountain = 0x0000_0008,
            },
            collectible: {
                "DMT Freestanding PoH" = 0x4000_0000,
            },
        },
        0x61: DeathMountainCrater {
            collectible: {
                "DMC Volcano Freestanding PoH" = 0x0000_0100,
                "DMC Wall Freestanding PoH" = 0x0000_0004,
            },
        },
        0x62: GoronCity {
            chests: {
                "GC Maze Center Chest" = 0x0000_0004,
                "GC Maze Right Chest" = 0x0000_0002,
                "GC Maze Left Chest" = 0x0000_0001,
            },
            switches: {
                event "Goron City Child Fire" = 0x1000_0000,
                LW_LEFT_BOULDER = 0x0000_1000,
                LW_MIDDLE_BOULDER = 0x0000_0800,
                LW_RIGHT_BOULDER = 0x0000_0100,
            },
            collectible: {
                "GC Pot Freestanding PoH" = 0x8000_0000,
            },
        },
        0x63: LonLonRanch {},
        0x64: GanonsCastleGrounds {},
    }
}

impl Scene {
    pub(crate) fn current(ram: &Ram) -> Result<Self, u8> {
        Self::from_id(ram.current_scene_id).ok_or(ram.current_scene_id)
    }

    /*
    pub(crate) fn regions(&self) -> Result<Vec<Region>, RegionLookupError> {
        let name = self.0;
        Ok(
            Region::into_enum_iter()
                .filter(move |region| region.scene().0 == name)
                .collect()
        )
    }
    */ //TODO
}
