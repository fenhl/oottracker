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

fn eat_any_color_str(s: &mut &[u8], base_prefix: &[u8]) -> bool {
    (0x40..0x48).any(|color| {
        let mut prefix = base_prefix.to_owned();
        for i in 1..prefix.len() {
            if &prefix[i - 1..=i] == b"\x05\x00" {
                prefix[i] = color;
            }
        }
        eat_str(s, &prefix)
    })
}

trait DungeonExt: Sized {
    fn eat_compass_hint_text(s: &mut &[u8]) -> Option<Self>;
}

impl DungeonExt for Dungeon {
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

trait DungeonRewardExt: Sized {
    fn eat_altar_hint_text(s: &mut &[u8]) -> Option<Self>;
    fn eat_compass_hint_text(s: &mut &[u8]) -> Option<Self>;
    fn eat_ruto_hint_text(s: &mut &[u8]) -> Option<Self>;
}

impl DungeonRewardExt for DungeonReward {
    fn eat_altar_hint_text(s: &mut &[u8]) -> Option<Self> {
        if eat_str(s, b"\x08\x13\x6c") {
            Some(Self::Stone(Stone::KokiriEmerald))
        } else if eat_str(s, b"\x08\x13\x6d") {
            Some(Self::Stone(Stone::GoronRuby))
        } else if eat_str(s, b"\x08\x13\x6e") {
            Some(Self::Stone(Stone::ZoraSapphire))
        } else if eat_str(s, b"\x08\x13\x6b") {
            Some(Self::Medallion(Medallion::Light))
        } else if eat_str(s, b"\x08\x13\x66") {
            Some(Self::Medallion(Medallion::Forest))
        } else if eat_str(s, b"\x08\x13\x67") {
            Some(Self::Medallion(Medallion::Fire))
        } else if eat_str(s, b"\x08\x13\x68") {
            Some(Self::Medallion(Medallion::Water))
        } else if eat_str(s, b"\x08\x13\x6a") {
            Some(Self::Medallion(Medallion::Shadow))
        } else if eat_str(s, b"\x08\x13\x69") {
            Some(Self::Medallion(Medallion::Spirit))
        } else {
            None
        }
    }

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

    fn eat_ruto_hint_text(s: &mut &[u8]) -> Option<Self> {
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

trait DungeonRewardLocationExt: Sized {
    fn eat_altar_hint_text(s: &mut &[u8]) -> Option<Self>;
}

impl DungeonRewardLocationExt for DungeonRewardLocation {
    fn eat_altar_hint_text(s: &mut &[u8]) -> Option<Self> {
        if eat_any_color_str(s, b"One inside an \x05\x00ancient tree\x05\x0f...") || eat_any_color_str(s, b"One in the \x05\x00Deku Tree\x05\x0f...") {
            Some(Self::Dungeon(MainDungeon::DekuTree))
        } else if eat_any_color_str(s, b"One within an \x05\x00immense cavern\x05\x0f...") || eat_any_color_str(s, b"One in \x05\x00Dodongo's Cavern\x05\x0f...") {
            Some(Self::Dungeon(MainDungeon::DodongosCavern))
        } else if eat_any_color_str(s, b"One in the \x05\x00belly of a deity\x05\x0f...") || eat_any_color_str(s, b"One in \x05\x00Jabu Jabu's Belly\x05\x0f...") {
            Some(Self::Dungeon(MainDungeon::JabuJabu))
        } else if eat_any_color_str(s, b"One in a \x05\x00deep forest\x05\x0f...") || eat_any_color_str(s, b"One in the \x05\x00Forest Temple\x05\x0f...") {
            Some(Self::Dungeon(MainDungeon::ForestTemple))
        } else if eat_any_color_str(s, b"One on a \x05\x00high mountain\x05\x0f...") || eat_any_color_str(s, b"One in the \x05\x00Fire Temple\x05\x0f...") {
            Some(Self::Dungeon(MainDungeon::FireTemple))
        } else if eat_any_color_str(s, b"One under a \x05\x00vast lake\x05\x0f...") || eat_any_color_str(s, b"One in the \x05\x00Water Temple\x05\x0f...") {
            Some(Self::Dungeon(MainDungeon::WaterTemple))
        } else if eat_any_color_str(s, b"One within the \x05\x00house of the dead\x05\x0f...") || eat_any_color_str(s, b"One in the \x05\x00Shadow Temple\x05\x0f...") {
            Some(Self::Dungeon(MainDungeon::ShadowTemple))
        } else if eat_any_color_str(s, b"One inside a \x05\x00goddess of the sand\x05\x0f...") || eat_any_color_str(s, b"One in the \x05\x00Spirit Temple\x05\x0f...") {
            Some(Self::Dungeon(MainDungeon::SpiritTemple))
        } else if eat_any_color_str(s, b"One in \x05\x00@'s pocket\x05\x0f...") || eat_any_color_str(s, b"One \x05\x00@ already has\x05\x0f...") { //TODO check for the current player name instead
            Some(Self::LinksPocket)
        } else {
            None
        }
    }
}

pub(crate) fn read_knowledge(mut text: &[u8]) -> Knowledge {
    let mut knowledge = Knowledge::default();
    if eat_str(&mut text, b"\x08Princess Ruto got the \x01") {
        if let Some(reward) = DungeonReward::eat_ruto_hint_text(&mut text) {
            knowledge.dungeon_reward_locations.insert(reward, DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu));
        }
    } else if eat_str(&mut text, b"\x08\x13\x75You found the \x05\x41Compass\x05\x40\x01for ") {
        if let Some(Dungeon::Main(dungeon)) = Dungeon::eat_compass_hint_text(&mut text) {
            if eat_str(&mut text, b"\x05\x40!\x01It holds the ") {
                if let Some(reward) = DungeonReward::eat_compass_hint_text(&mut text) {
                    knowledge.dungeon_reward_locations.insert(reward, DungeonRewardLocation::Dungeon(dungeon));
                }
            }
        }
    } else if let Some(reward) = DungeonReward::eat_altar_hint_text(&mut text) {
        if let Some(loc) = DungeonRewardLocation::eat_altar_hint_text(&mut text) {
            knowledge.dungeon_reward_locations.insert(reward, loc);
        }
    }
    //TODO other info (e.g. Jabu dungeon reward)
    knowledge
}
