use oottracker_derive::scene_flags;

scene_flags! {
    pub struct SceneFlags {
        0x48: "Windmill and Dampes Grave" {
            unused: {
                TRIFORCE_PIECES = 0xffff_ffff,
            },
        },
        0x0055: "Kokiri Forest" {
            chests: {
                "KF Kokiri Sword Chest" = 0x0000_0001,
            },
        },
    }
}
