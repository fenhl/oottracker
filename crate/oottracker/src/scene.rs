//! This module contains types representing the “permanent scene flags” and “gold skulltulas” sections of [save data](crate::save::Save).
//!
//! The entry points are the types [`SceneFlags`] and [`GoldSkulltulas`]. All other types appear in their fields.

use {
    std::{
        fmt,
        sync::Arc,
    },
    ootr::{
        Rando,
        region::Region,
    },
    oottracker_derive::scene_flags,
    crate::{
        Ram,
        region::{
            RegionExt as _,
            RegionLookupError,
        },
    },
};

pub(crate) struct Scene(pub(crate) &'static str);

pub(crate) trait FlagsScene {
    fn set_chests(&mut self, chests: u32);
    fn set_switches(&mut self, switches: u32);
    fn set_room_clear(&mut self, room_clear: u32);
}

scene_flags! {
    pub struct SceneFlags {
        0x00: "Deku Tree" {
            region_name: |_| "Deku Tree Lobby", //TODO return region name based on room number and/or coords, missing regions: Deku Tree Slingshot Room, Deku Tree Basement Backroom, Deku Tree Boss Room
            //TODO region_name_mq
            switches: {
                1 for "Deku Tree GS Basement Backroom" = 0x0004_0000,
                BASEMENT_PUSHED_BLOCK /*vanilla*/ = 0x0001_0000,
                BASEMENT_BURN_FIRST_WEB_TO_BACK_ROOM /*vanilla*/ = 0x0000_0200,
                0 for "Deku Tree GS Basement Backroom" = 0x0000_0100,
                0 for "Deku Tree Lobby" /*vanilla*/ -> "Deku Tree Boss Room" = 0x0000_0040,
                1 for "Deku Tree Lobby" /*vanilla*/ -> "Deku Tree Basement Backroom" = 0x0000_0010,
                LIGHT_TORCHES_AFTER_WATER_ROOM /*vanilla*/ = 0x0000_0008,
            },
            room_clear: {
                SCRUBS_231_PUZZLE = 0x0000_0200,
                0 for "Deku Tree Lobby" /*vanilla*/ -> "Deku Tree Slingshot Room" = 0x0000_0002,
            },
        },
        0x01: "Dodongos Cavern" {
            switches: {
                0 for "Dodongos Cavern Lobby" /*mq*/ -> "Dodongos Cavern Lower Right Side" = 0x8000_0000,
                0 for "Dodongos Cavern Lower Right Side" /*mq*/ -> "Dodongos Cavern Bomb Bag Area" = 0x0800_0000, //TODO confirm, logic says this entrance needs slingshot but that doesn't seem to be the case
                0 for "Dodongos Cavern Lobby" /*vanilla*/ -> "Dodongos Cavern Staircase Room" = 0x0200_0000,
                0 for "Dodongos Cavern Lobby" /*vanilla*/ -> "Dodongos Cavern Far Bridge" = 0x0000_0400,
                0 for "Dodongos Cavern Beginning" /*either*/ -> "Dodongos Cavern Lobby" = 0x0000_0080,
            },
        },
        0x02: "Jabu Jabus Belly" {},
        0x03: "Forest Temple" {
            chests: {
                "Forest Temple Raised Island Courtyard Chest" = 0x0000_0020,
            },
            switches: {
                BETH_DEFEATED /*vanilla*/ = 0x4000_0000,
                JOELLE_DEFEATED /*vanilla*/ = 0x2000_0000,
            },
            room_clear: {
                0 for "Forest Temple NW Outdoors" /*vanilla*/ -> "Forest Temple Outdoors High Balconies" = 0x0000_0400,
            },
            gold_skulltulas: {
                "Forest Temple GS Level Island Courtyard" = 0x04,
            },
        },
        0x04: "Fire Temple" {},
        0x05: "Water Temple" {
            switches: {
                event "Raise Water Level" /*vanilla*/ = 0x4000_0000,
                //WATER_LEVEL_MID /*vanilla*/ = 0x2000_0000,
                //WATER_LEVEL_LOW /*vanilla*/ = 0x1000_0000,
            },
        },
        0x06: "Spirit Temple" {},
        0x07: "Shadow Temple" {},
        0x08: "Bottom of the Well" {},
        0x09: "Ice Cavern" {},
        0x0a: "Ganons Castle Tower" {},
        0x0b: "Gerudo Training Grounds" {
            switches: {
                0 for "Gerudo Training Grounds Lobby" /*vanilla*/ -> "Gerudo Training Grounds Lava Room" = 0x4000_0000,
            },
        },
        0x0c: ThievesHideout {
            region_name: "Gerudo Fortress",
        },
        0x0d: "Ganons Castle" {},
        0x0e: GanonsCastleTowerCollapsing {
            region_name: "Ganons Castle Tower",
        },
        0x0f: GanonsCastleCollapsing {
            region_name: "Ganons Castle Tower", // rando considers the entire collapse logically part of the tower
        },
        0x10: "Market Treasure Chest Game" {
            chests: {
                "Market Treasure Chest Game Reward" = 0x0000_0400,
            },
        },
        0x11: "Deku Tree Boss Room" {},
        //TODO remaining scenes (https://wiki.cloudmodding.com/oot/Scene_Table/NTSC_1.0)
        0x28: "KF Midos House" {
            chests: {
                "KF Midos Bottom Right Chest" = 0x0000_0008,
                "KF Midos Bottom Left Chest" = 0x0000_0004,
                "KF Midos Top Right Chest" = 0x0000_0002,
                "KF Midos Top Left Chest" = 0x0000_0001,
            },
        },
        0x3b: UpgradeFairyFountain {
            switches: {
                "DMT Great Fairy Reward" = 0x0100_0000,
                "DMC Great Fairy Reward" = 0x0001_0000,
                "OGC Great Fairy Reward" = 0x0000_0100,
            },
        },
        0x3e: Grottos {
            chests: {
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
        },
        0x3f: "Graveyard Heart Piece Grave" {
            chests: {
                "Graveyard Heart Piece Grave Chest" = 0x0000_0001,
            },
        },
        0x40: "Graveyard Shield Grave" {
            chests: {
                "Graveyard Shield Grave Chest" = 0x0000_0001,
            },
        },
        0x41: "Graveyard Composers Grave" {
            chests: {
                "Graveyard Composers Grave Chest" = 0x0000_0001,
            },
        },
        0x48: "Windmill and Dampes Grave" {
            chests: {
                "Graveyard Hookshot Chest" = 0x0000_0001,
            },
            unused: {
                TRIFORCE_PIECES = 0xffff_ffff,
            },
        },
        0x51: "Hyrule Field" {
            switches: {
                0 for "Hyrule Field" -> "HF Fairy Grotto" = 0x0001_0000,
                0 for "Hyrule Field" -> "HF Near Market Grotto" = 0x0000_4000,
                0 for "Hyrule Field" -> "HF Southeast Grotto" = 0x0000_0100,
            },
        },
        0x52: "Kakariko Village" {},
        0x53: "Zora River" {
            switches: {
                0 for "Zora River" -> "ZR Fairy Grotto" = 0x0000_0020,
            },
        },
        0x55: "Kokiri Forest" {
            chests: {
                "KF Kokiri Sword Chest" = 0x0000_0001,
            },
        },
        0x58: "Zoras Domain" {
            chests: {
                "ZD Chest" = 0x0000_0001,
            },
        },
        0x5a: "Gerudo Valley" {
            chests: {
                "GV Chest" = 0x0000_0001,
            },
        },
        0x5b: "Lost Woods" {
            switches: {
                0 for "LW Beyond Mido" -> "LW Scrubs Grotto" = 0x8000_0000,
                0 for "Lost Woods" -> "LW Near Shortcuts Grotto" = 0x0002_0000,
            },
        },
        0x5d: "Gerudo Fortress" {
            chests: {
                "GF Chest" = 0x0000_0001,
            },
            switches: {
                event "GF Gate Open" = 0x0000_0008,
            },
        },
        0x5e: "Haunted Wasteland" {
            chests: {
                "Wasteland Chest" = 0x0000_0001,
            },
        },
        0x60: "Death Mountain" {
            chests: {
                "DMT Chest" = 0x0000_0002,
            },
            switches: {
                0 for "Death Mountain Summit" -> "DMT Cow Grotto" = 0x8000_0000,
                DMT_TO_SUMMIT_SECOND_BOULDER = 0x0000_0400,
                DMT_TO_SUMMIT_FIRST_BOULDER = 0x0000_0100,
                PLANT_BEAN = 0x0000_0040,
                BLOW_UP_DC_ENTRANCE = 0x0000_0010,
                0 for "Death Mountain Summit" -> "DMT Great Fairy Fountain" = 0x0000_0008,
            },
        },
        0x62: "Goron City" {
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
        },
    }
}

impl Scene {
    pub(crate) fn current(ram: &Ram) -> Scene {
        Scene::from_id(ram.current_scene_id)
    }

    pub(crate) fn regions<'a, R: Rando>(&self, rando: &'a R) -> Result<Vec<Arc<Region<R>>>, RegionLookupError<R>> {
        let name = self.0;
        Ok(
            Region::all(rando)?
                .iter()
                .filter(move |region| region.scene.as_ref().map_or(false, |scene| scene == name) || region.dungeon.as_ref().map_or(false, |(dungeon, _)| dungeon.to_string() == name))
                .cloned()
                .collect()
        )
    }
}

impl fmt::Display for Scene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
