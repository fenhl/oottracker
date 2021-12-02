use oottracker_derive::flags_list;

flags_list! {
    pub struct EventChkInf: [u16; 14] {
        0: {
            //USED_BLUE_WARP_IN_GOHMAS_LAIR = 0x0200, //TODO check for parity with Deku Tree Clear event
            event "Deku Tree Clear" = 0x0080,
            event "Showed Mido Sword & Shield" = 0x0010,
        },
        1: {
            "HC Malon Egg" = 0x0004, // unsure, documented at CloudModding as “Obtained Pocket Egg”, not Weird Egg
        },
        2: {
            "King Dodongo" = 0x0020,
        },
        3: {
            "ZD Diving Minigame" = 0x0100,
            "Barinade" = 0x0080,
            "GC Darunias Joy" = 0x0040,
            DELIVER_RUTOS_LETTER = 0x0008,
            "LH Underwater Item" = 0x0002,
        },
        4: {
            "Morpha" = 0x0400,
            "Volvagia" = 0x0200,
            "Phantom Ganon" = 0x0100,
            "HF Ocarina of Time Item" = 0x0008,
            "HC Zeldas Letter" = 0x0001,
        },
        5: {
            "Song from Windmill" = 0x0800,
            "Song from Composers Grave" = 0x0400,
            "Song from Impa" = 0x0200,
            "Song from Malon" = 0x0100,
            "Song from Saria" = 0x0080,
            "Sheik at Temple" = 0x0020,
            "Sheik in Kakariko" = 0x0010,
            "Sheik in Ice Cavern" = 0x0004,
            "Sheik in Crater" = 0x0002,
            "Sheik in Forest" = 0x0001,
        },
        6: {
            event "Drain Well" = 0x0080,
        },
        9: {
            SCARECROW_SONG = 0x1000,
        },
        10: {
            event "Spirit Trial Clear" = 0x2000, //TODO only consider when known by settings knowledge or visual confirmation
            "Sheik at Colossus" = 0x1000,
            "Song from Ocarina of Time" = 0x0200,
        },
        11: {
            event "Light Trial Clear" = 0x8000, //TODO only consider when known by settings knowledge or visual confirmation
            event "Fire Trial Clear" = 0x4000, //TODO only consider when known by settings knowledge or visual confirmation
            event "Shadow Trial Clear" = 0x2000, //TODO only consider when known by settings knowledge or visual confirmation
            event "Water Trial Clear" = 0x1000, //TODO only consider when known by settings knowledge or visual confirmation
            event "Forest Trial Clear" = 0x0800, //TODO only consider when known by settings knowledge or visual confirmation
        },
        12: {
            "ToT Light Arrows Cutscene" = 0x0010,
            "LW Gift from Saria" = 0x0002,
        },
        13: {
            "Kak 50 Gold Skulltula Reward" = 0x4000,
            "Kak 40 Gold Skulltula Reward" = 0x2000,
            "Kak 30 Gold Skulltula Reward" = 0x1000,
            "Kak 20 Gold Skulltula Reward" = 0x0800,
            "Kak 10 Gold Skulltula Reward" = 0x0400,
            "ZR Frogs in the Rain" = 0x0040,
            "ZR Frogs Ocarina Game" = 0x0001,
        },
    }
}

flags_list! {
    pub struct ItemGetInf: [u8; 8] {
        0: {
            "GF HBA 1500 Points" = 0x80,
            "Kak Shooting Gallery Reward" = 0x40,
            "Market Shooting Gallery Reward" = 0x20,
            "Kak Anju as Child" = 0x10,
        },
        1: {
            "LLR Talons Chickens" = 0x04,
        },
        2: {
            "Deku Theater Mask of Truth" = 0x80,
            "Deku Theater Skull Mask" = 0x40,
            "LW Target in Woods" = 0x20,
            "Colossus Great Fairy Reward" = 0x04,
            "HC Great Fairy Reward" = 0x02,
            "ZF Great Fairy Reward" = 0x01,
        },
        3: {
            "LW Ocarina Memory Game" = 0x80,
            "LW Skull Kid" = 0x40,
            "Kak Man on Roof" = 0x20,
            "Market Bombchu Bowling Second Prize" = 0x04,
            "Market Bombchu Bowling First Prize" = 0x02,
            "LH Lab Dive" = 0x01,
        },
        4: {
            "Kak Anju as Adult" = 0x10,
        },
    }
}

flags_list! {
    pub struct InfTable: [u8; 60] {
        15: {
            event "Kakariko Village Gate Open" = 0x40, //TODO only consider when known by settings knowledge or visual confirmation
        },
        32: {
            "GC Rolling Goron as Adult" = 0x02,
        },
        34: {
            "GC Rolling Goron as Child" = 0x40,
        },
        38: {
            "ZD King Zora Thawed" = 0x02,
        },
        51: {
            "LW Deku Scrub Near Bridge" = 0x04,
            "Market Lost Dog" = 0x02,
            "GF HBA 1000 Points" = 0x01,
        },
        55: {
            TOT_ALTAR_READ_STONE_LOCATIONS = 0x02,
            TOT_ALTAR_READ_MEDALLION_LOCATIONS = 0x01,
        },
    }
}
