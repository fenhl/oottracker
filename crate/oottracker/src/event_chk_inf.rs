use oottracker_derive::flags_list;

flags_list! {
    pub struct EventChkInf: [u16; 14] {
        5: {
            "Song from Windmill" = 0x0800,
            "Song from Composers Grave" = 0x0400,
            "Song from Impa" = 0x0200,
            "Song from Malon" = 0x0100,
            "Song from Saria" = 0x0080,
            "Sheik at Temple" = 0x0020, // unsure, documented at CloudModding as “Sheik Moved From Sword Pedestal”
            "Sheik in Kakariko" = 0x0010,
            "Sheik in Ice Cavern" = 0x0004,
            "Sheik in Crater" = 0x0002,
            "Sheik in Forest" = 0x0001,
        },
        9: {
            SCARECROW_SONG = 0x1000,
        },
        10: {
            "Sheik at Colossus" = 0x1000,
            "Song from Ocarina of Time" = 0x0200,
        },
    }
}
