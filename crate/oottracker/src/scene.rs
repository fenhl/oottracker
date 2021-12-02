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
            chests: {
                "Dodongos Cavern End of Bridge Chest" = 0x0000_0400,
                "Dodongos Cavern Map Chest" = 0x0000_0100,
                "Dodongos Cavern Bomb Flower Platform Chest" = 0x0000_0040,
                "Dodongos Cavern Compass Chest" = 0x0000_0020,
                "Dodongos Cavern Bomb Bag Chest" = 0x0000_0010,
            },
            switches: {
                0 for "Dodongos Cavern Lobby" /*mq*/ -> "Dodongos Cavern Lower Right Side" = 0x8000_0000,
                0 for "Dodongos Cavern Lower Right Side" /*mq*/ -> "Dodongos Cavern Bomb Bag Area" = 0x0800_0000, //TODO confirm, logic says this entrance needs slingshot but that doesn't seem to be the case
                0 for "Dodongos Cavern Lobby" /*vanilla*/ -> "Dodongos Cavern Staircase Room" = 0x0200_0000,
                0 for "Dodongos Cavern Lobby" /*vanilla*/ -> "Dodongos Cavern Far Bridge" = 0x0000_0400,
                0 for "Dodongos Cavern Beginning" /*either*/ -> "Dodongos Cavern Lobby" = 0x0000_0080,
            },
        },
        0x02: "Jabu Jabus Belly" {
            chests: {
                "Jabu Jabus Belly Compass Chest" = 0x0000_0010,
                "Jabu Jabus Belly Map Chest" = 0x0000_0004,
                "Jabu Jabus Belly Boomerang Chest" = 0x0000_0002,
            },
        },
        0x03: "Forest Temple" {
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
                0 for "Forest Temple NW Outdoors" /*vanilla*/ -> "Forest Temple Outdoors High Balconies" = 0x0000_0400,
            },
            gold_skulltulas: {
                "Forest Temple GS Level Island Courtyard" = 0x04,
            },
        },
        0x04: "Fire Temple" {
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
        0x05: "Water Temple" {
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
        0x06: "Spirit Temple" {
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
        0x07: "Shadow Temple" {
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
        0x08: "Bottom of the Well" {
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
        0x09: "Ice Cavern" {
            chests: {
                "Ice Cavern Iron Boots Chest" = 0x0000_0004,
                "Ice Cavern Compass Chest" = 0x0000_0002,
                "Ice Cavern Map Chest" = 0x0000_0001,
            },
            collectible: {
                "Ice Cavern Freestanding PoH" = 0x0000_0002,
            },
        },
        0x0a: "Ganons Castle Tower" {
            chests: {
                "Ganons Tower Boss Key Chest" = 0x0000_0800,
            },
        },
        0x0b: "Gerudo Training Ground" {
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
                0 for "Gerudo Training Ground Lobby" /*vanilla*/ -> "Gerudo Training Ground Lava Room" = 0x4000_0000,
            },
            collectible: {
                "Gerudo Training Ground Freestanding Key" = 0x0000_0002,
            },
        },
        0x0c: ThievesHideout {
            region_name: "Gerudo Fortress",
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
        0x0d: "Ganons Castle" {
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
        0x12: "Dodongos Cavern Boss Area" {
            chests: {
                "Dodongos Cavern Boss Room Chest" = 0x0000_0001,
            },
        },
        //TODO remaining scenes (https://wiki.cloudmodding.com/oot/Scene_Table/NTSC_1.0)
        0x28: "KF Midos House" {
            chests: {
                "KF Midos Bottom Right Chest" = 0x0000_0008,
                "KF Midos Bottom Left Chest" = 0x0000_0004,
                "KF Midos Top Right Chest" = 0x0000_0002,
                "KF Midos Top Left Chest" = 0x0000_0001,
            },
        },
        0x37: "Kak Impas House" {
            collectible: {
                "Kak Impas House Freestanding PoH" = 0x0000_0002,
            },
        },
        0x3b: UpgradeFairyFountain {
            switches: { //TODO generalize as upgrade Great Fairy rewards depending on knowledge
                "DMT Great Fairy Reward" = 0x0100_0000,
                "DMC Great Fairy Reward" = 0x0001_0000,
                "OGC Great Fairy Reward" = 0x0000_0100,
            },
        },
        0x3e: Grottos {
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
            collectible: {
                "Graveyard Dampe Race Freestanding PoH" = 0x0000_0080,
                "Kak Windmill Freestanding PoH" = 0x0000_0002,
            },
            unused: {
                TRIFORCE_PIECES = 0xffff_ffff,
            },
        },
        0x4c: "LLR Tower" {
            collectible: {
                "LLR Freestanding PoH" = 0x0000_0002,
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
        0x53: "Graveyard" {
            collectible: {
                "Graveyard Dampe Gravedigging Tour" = 0x0000_0100,
                "Graveyard Freestanding PoH" = 0x0000_0010,
            },
        },
        0x54: "Zora River" {
            switches: {
                0 for "Zora River" -> "ZR Fairy Grotto" = 0x0000_0020,
            },
            collectible: {
                "ZR Near Domain Freestanding PoH" = 0x0000_0800,
                "ZR Near Open Grotto Freestanding PoH" = 0x0000_0010,
            },
        },
        0x55: "Kokiri Forest" {
            chests: {
                "KF Kokiri Sword Chest" = 0x0000_0001,
            },
        },
        0x57: "Lake Hylia" {
            chests: {
                "LH Sun" = 0x0000_0001,
            },
            collectible: {
                "LH Freestanding PoH" = 0x4000_0000,
            },
        },
        0x58: "Zoras Domain" {
            chests: {
                "ZD Chest" = 0x0000_0001,
            },
        },
        0x59: "Zoras Fountain" {
            collectible: {
                "ZF Bottom Freestanding PoH" = 0x0010_0000,
                "ZF Iceberg Freestanding PoH" = 0x0000_0002,
            },
        },
        0x5a: "Gerudo Valley" {
            chests: {
                "GV Chest" = 0x0000_0001,
            },
            collectible: {
                "GV Crate Freestanding PoH" = 0x0000_0004,
                "GV Waterfall Freestanding PoH" = 0x0000_0002,
            },
        },
        0x5b: "Lost Woods" {
            switches: {
                0 for "LW Beyond Mido" -> "LW Scrubs Grotto" = 0x8000_0000,
                0 for "Lost Woods" -> "LW Near Shortcuts Grotto" = 0x0002_0000,
            },
        },
        0x5c: "Desert Colossus" {
            chests: {
                "Spirit Temple Silver Gauntlets Chest" = 0x0000_0800,
                "Spirit Temple Mirror Shield Chest" = 0x0000_0200,
            },
            collectible: {
                "Colossus Freestanding PoH" = 0x0000_2000,
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
            collectible: {
                "DMT Freestanding PoH" = 0x4000_0000,
            },
        },
        0x61: "Death Mountain Crater" {
            collectible: {
                "DMC Volcano Freestanding PoH" = 0x0000_0100,
                "DMC Wall Freestanding PoH" = 0x0000_0004,
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
            collectible: {
                "GC Pot Freestanding PoH" = 0x8000_0000,
            },
        },
    }
}

impl Scene {
    pub(crate) fn current(ram: &Ram) -> Result<Scene, u8> {
        Scene::from_id(ram.current_scene_id).ok_or(ram.current_scene_id)
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
