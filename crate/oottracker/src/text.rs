use {
    ootr::model::{
        Dungeon,
        DungeonReward,
        DungeonRewardLocation,
        MainDungeon,
        Medallion,
        Stone,
    },
    crate::knowledge::Knowledge,
};

fn eat_str(s: &mut &[u8], prefix: &[u8]) -> bool {
    if s.starts_with(prefix) {
        *s = &s[prefix.len()..];
        true
    } else {
        false
    }
}

trait CompassHintExt: Sized {
    fn eat_compass_hint_text(s: &mut &[u8]) -> Option<Self>;
}

impl CompassHintExt for Dungeon {
    fn eat_compass_hint_text(s: &mut &[u8]) -> Option<Self> {
        if eat_str(s, b"the \x05\x42Deku Tree") {
            Some(Self::Main(MainDungeon::DekuTree))
        } else if eat_str(s, b"\x05\x41Dodongo\'s Cavern") {
            Some(Self::Main(MainDungeon::DodongosCavern))
        } else if eat_str(s, b"\x05\x43Jabu Jabu\'s Belly") {
            Some(Self::Main(MainDungeon::JabuJabu))
        } else if eat_str(s, b"the \x05\x42Forest Temple") {
            Some(Self::Main(MainDungeon::ForestTemple))
        } else if eat_str(s, b"the \x05\x41Fire Temple") {
            Some(Self::Main(MainDungeon::FireTemple))
        } else if eat_str(s, b"the \x05\x43Water Temple") {
            Some(Self::Main(MainDungeon::WaterTemple))
        } else if eat_str(s, b"the \x05\x46Spirit Temple") {
            Some(Self::Main(MainDungeon::SpiritTemple))
        } else if eat_str(s, b"the \x05\x44Ice Cavern") {
            Some(Self::IceCavern)
        } else if eat_str(s, b"the \x05\x45Bottom of the Well") {
            Some(Self::BottomOfTheWell)
        } else if eat_str(s, b"the \x05\x45Shadow Temple") {
            Some(Self::Main(MainDungeon::ShadowTemple))
        } else {
            None
        }
    }
}

impl CompassHintExt for DungeonReward {
    fn eat_compass_hint_text(s: &mut &[u8]) -> Option<Self> {
        if eat_str(s, b"\x05\x42Kokiri Emerald\x05\x40") {
            Some(Self::Stone(Stone::KokiriEmerald))
        } else if eat_str(s, b"\x05\x41Goron Ruby\x05\x40") {
            Some(Self::Stone(Stone::GoronRuby))
        } else if eat_str(s, b"\x05\x43Zora Sapphire\x05\x40") {
            Some(Self::Stone(Stone::ZoraSapphire))
        } else if eat_str(s, b"\x05\x42Forest Medallion\x05\x40") {
            Some(Self::Medallion(Medallion::Forest))
        } else if eat_str(s, b"\x05\x41Fire Medallion\x05\x40") {
            Some(Self::Medallion(Medallion::Fire))
        } else if eat_str(s, b"\x05\x43Water Medallion\x05\x40") {
            Some(Self::Medallion(Medallion::Water))
        } else if eat_str(s, b"\x05\x46Spirit Medallion\x05\x40") {
            Some(Self::Medallion(Medallion::Spirit))
        } else if eat_str(s, b"\x05\x45Shadow Medallion\x05\x40") {
            Some(Self::Medallion(Medallion::Shadow))
        } else if eat_str(s, b"\x05\x44Light Medallion\x05\x40") {
            Some(Self::Medallion(Medallion::Light))
        } else {
            None
        }
    }
}

pub(crate) fn read_knowledge(mut text: &[u8]) -> Knowledge {
    let mut knowledge = Knowledge::default();
    if eat_str(&mut text, b"\x08\x13\x75You found the \x05\x41Compass\x05\x40\x01for ") {
        if let Some(Dungeon::Main(dungeon)) = Dungeon::eat_compass_hint_text(&mut text) {
            if eat_str(&mut text, b"\x05\x40!\x01It holds the ") {
                if let Some(reward) = DungeonReward::eat_compass_hint_text(&mut text) {
                    knowledge.dungeon_reward_locations.insert(reward, DungeonRewardLocation::Dungeon(dungeon));
                }
            }
        }
    }
    //TODO other info (e.g. dungeon rewards from Temple of Time pedestal)
    knowledge
}
