//! This module contains types representing the permanent scene flags section of save data. The type `SceneFlags` represents that entire section, and the other types appear in its fields.

use oottracker_derive::scene_flags;

scene_flags! {
    pub struct SceneFlags {
        0x10: "Market Treasure Chest Game" {
            chests: {
                "Market Treasure Chest Game Reward" = 0x0000_0400,
            },
        },
        0x28: "KF Midos House" {
            chests: {
                "KF Midos Bottom Right Chest" = 0x0000_0008,
                "KF Midos Bottom Left Chest" = 0x0000_0004,
                "KF Midos Top Right Chest" = 0x0000_0002,
                "KF Midos Top Left Chest" = 0x0000_0001,
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
        0x5d: "Gerudo Fortress" {
            chests: {
                "GF Chest" = 0x0000_0001,
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
        },
        0x62: "Goron City" {
            chests: {
                "GC Maze Center Chest" = 0x0000_0004,
                "GC Maze Right Chest" = 0x0000_0002,
                "GC Maze Left Chest" = 0x0000_0001,
            },
        },
    }
}
