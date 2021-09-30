use {
    std::collections::HashMap,
    enum_iterator::IntoEnumIterator as _,
    crate::{
        ModelState,
        check::Check,
        region::Region,
    },
};

pub trait CheckExt {
    fn checked(&self, model: &ModelState) -> Option<bool>; //TODO implement all checks, then change return type to bool
}

impl CheckExt for Check {
    fn checked(&self, model: &ModelState) -> Option<bool> {
        // event and location lists from Dev-R as of commit b670183e9aff520c20ac2ee65aa55e3740c5f4b4
        if let Some(checked) = model.ram.save.gold_skulltulas.checked(self) { return Some(checked) }
        if let Some(checked) = model.ram.scene_flags().checked(self) { return Some(checked) }
        if let Some(checked) = model.ram.save.event_chk_inf.checked(self) { return Some(checked) }
        if let Some(checked) = model.ram.save.item_get_inf.checked(self) { return Some(checked) }
        if let Some(checked) = model.ram.save.inf_table.checked(self) { return Some(checked) }
        match self {
            /*
            Check::AnonymousEvent(at_check, id) => match (&**at_check, id) {
                (Check::Event(event), 0) if *event == "Deku Tree Clear" /*vanilla*/ => Some(
                    model.ram.scene_flags().deku_tree.room_clear.contains(
                        crate::scene::DekuTreeRoomClear::SCRUBS_231_PUZZLE
                    )
                ),
                (Check::Exit { from: Region::DeathMountain, to: Region::DeathMountainSummit }, 0) => Some(
                    model.ram.scene_flags().death_mountain.switches.contains(
                        crate::scene::DeathMountainSwitches::DMT_TO_SUMMIT_FIRST_BOULDER
                        | crate::scene::DeathMountainSwitches::DMT_TO_SUMMIT_SECOND_BOULDER
                    )
                ),
                (Check::Exit { from: Region::DeathMountain, to: Region::DeathMountainSummit }, 1) => Some(
                    model.ram.scene_flags().death_mountain.switches.contains(
                        crate::scene::DeathMountainSwitches::BLOW_UP_DC_ENTRANCE
                        | crate::scene::DeathMountainSwitches::PLANT_BEAN
                    )
                ),
                (Check::Exit { from: Region::DekuTreeLobby, to: Region::DekuTreeBasementBackroom }, 0) => Some(
                    model.ram.scene_flags().deku_tree.switches.contains(
                        crate::scene::DekuTreeSwitches::BASEMENT_BURN_FIRST_WEB_TO_BACK_ROOM
                        | crate::scene::DekuTreeSwitches::LIGHT_TORCHES_AFTER_WATER_ROOM
                    )
                ),
                (Check::Exit { from: Region::DekuTreeLobby, to: Region::DekuTreeBasementBackroom }, 2) => Some(
                    model.ram.scene_flags().deku_tree.switches.contains(
                        crate::scene::DekuTreeSwitches::BASEMENT_PUSHED_BLOCK
                    )
                ),
                (Check::Exit { from: Region::DekuTreeLobby, to: Region::DekuTreeBossRoom }, 1) => Some(
                    model.ram.scene_flags().deku_tree.switches.contains(
                        crate::scene::DekuTreeSwitches::BASEMENT_PUSHED_BLOCK
                    )
                ),
                (Check::Location(loc), 0) if *loc == "Deku Tree Queen Gohma Heart" => Some(
                    model.ram.scene_flags().deku_tree.room_clear.contains(
                        crate::scene::DekuTreeRoomClear::SCRUBS_231_PUZZLE
                    )
                ),
                (Check::Location(loc), 0) if *loc == "Queen Gohma" => Some(
                    model.ram.scene_flags().deku_tree.room_clear.contains(
                        crate::scene::DekuTreeRoomClear::SCRUBS_231_PUZZLE
                    )
                ),
                // the anonymous event for this skulltula is really just collecting it from a different region with different item requirements
                (Check::Location(loc), 0) if *loc == "Forest Temple GS Level Island Courtyard" => Some(
                    model.ram.save.gold_skulltulas.forest_temple.contains(
                        crate::scene::ForestTempleGoldSkulltulas::FOREST_TEMPLE_GS_LEVEL_ISLAND_COURTYARD
                    )
                ),
                // the anonymous events for this chest are really just opening it from different regions with different item requirements
                (Check::Location(loc), 0) | (Check::Location(loc), 1) if *loc == "Forest Temple Raised Island Courtyard Chest" => Some(
                    model.ram.scene_flags().forest_temple.chests.contains(
                        crate::scene::ForestTempleChests::FOREST_TEMPLE_RAISED_ISLAND_COURTYARD_CHEST
                    )
                ),
                (_, _) => None, //TODO make a list of all anonymous events
            },
            Check::Event(event) => match &event[..] {
                // Overworld
                "Showed Mido Sword & Shield" => None,
                "Bonooru" => None,
                "Carpenter Rescue" => None,
                "GF Gate Open" => None,
                "Sell Big Poe" => None,
                "Skull Mask" => None,
                "Mask of Truth" => None,
                "Drain Well" => None,
                "GC Woods Warp Open" => Some(
                    model.ram.scene_flags().goron_city.switches.intersects(
                        crate::scene::GoronCitySwitches::LW_LEFT_BOULDER
                        | crate::scene::GoronCitySwitches::LW_MIDDLE_BOULDER
                        | crate::scene::GoronCitySwitches::LW_RIGHT_BOULDER
                    )
                ),
                "Epona" => None,
                "Links Cow" => None,
                "Odd Mushroom Access" => None,
                "Poachers Saw Access" => None,
                "Eyedrops Access" => None,
                "Broken Sword Access" => None,
                "Cojiro Access" => None,
                "Wake Up Adult Talon" => None,
                "Odd Potion Access" => None,
                "Dampes Windmill Access" => None,
                "Prescription Access" => None,
                "Stop GC Rolling Goron as Adult" => None,
                "King Zora Thawed" => None,
                "Eyeball Frog Access" => None,

                // Forest Temple
                "Forest Temple Jo and Beth" => Some(
                    model.ram.scene_flags().forest_temple.switches.contains(
                        crate::scene::ForestTempleSwitches::JOELLE_DEFEATED
                        | crate::scene::ForestTempleSwitches::BETH_DEFEATED
                    )
                ),
                "Forest Temple Amy and Meg" => None,

                // Water Temple
                "Child Water Temple" => None,
                "Water Temple Clear" => None,

                _ => panic!("unknown event name: {}", event),
            },
            Check::Exit { from, to, .. } => Some(model.knowledge.entrances.contains_key(&Entrance { from: from.clone(), to: to.clone() })), //TODO check if value is a single entrance
            Check::Location(loc) => match &loc[..] {
                "LH Child Fishing" => Some(model.ram.save.fishing_context.contains(crate::save::FishingContext::CHILD_PRIZE_OBTAINED)),
                "LH Adult Fishing" => Some(model.ram.save.fishing_context.contains(crate::save::FishingContext::ADULT_PRIZE_OBTAINED)),
                "Market Bombchu Bowling Bombchus" => None, // repeatable check
                "ZR Magic Bean Salesman" => None, //TODO make sure this is handled correctly both with and without bean shuffle
                "DMT Biggoron" => Some(model.ram.save.dmt_biggoron_checked),
                "Market 10 Big Poes" => None, //TODO figure out how to read point target count from ROM, or read it from the text box
                "Wasteland Bombchu Salesman" => None, //TODO make sure this is handled correctly both with and without medi/carp shuffle (and according to knowledge)
                "GC Medigoron" => None, //TODO make sure this is handled correctly both with and without medi/carp shuffle (and according to knowledge)

                "Pierre" => None,
                "Deliver Rutos Letter" => None,
                "Master Sword Pedestal" => None, // repeatable check

                "Deku Baba Sticks" => None, // repeatable check
                "Deku Baba Nuts" => None, // repeatable check
                "Stick Pot" => None, // repeatable check
                "Nut Pot" => None, // repeatable check
                "Nut Crate" => None, // repeatable check
                "Blue Fire" => None, // repeatable check
                "Lone Fish" => None, // repeatable check
                "Fish Group" => None, // repeatable check
                "Bug Rock" => None, // repeatable check
                "Bug Shrub" => None, // repeatable check
                "Wandering Bugs" => None, // repeatable check
                "Fairy Pot" => None, // repeatable check
                "Free Fairies" => None, // repeatable check
                "Wall Fairy" => None, // repeatable check
                "Butterfly Fairy" => None, // repeatable check
                "Gossip Stone Fairy" => None, // repeatable check
                "Bean Plant Fairy" => None, // repeatable check
                "Fairy Pond" => None, // repeatable check
                "Big Poe Kill" => None, //TODO mark as checked when enough big Poes are collected (sold + in current bottles)

                // Deku Tree MQ
                "Deku Tree MQ Map Chest" => None,
                "Deku Tree MQ Compass Chest" => None,
                "Deku Tree MQ Slingshot Chest" => None,
                "Deku Tree MQ Slingshot Room Back Chest" => None,
                "Deku Tree MQ Basement Chest" => None,
                "Deku Tree MQ Before Spinning Log Chest" => None,
                "Deku Tree MQ After Spinning Log Chest" => None,

                // Dodongo's Cavern MQ
                "Dodongos Cavern MQ Map Chest" => None,
                "Dodongos Cavern MQ Bomb Bag Chest" => None,
                "Dodongos Cavern MQ Compass Chest" => None,
                "Dodongos Cavern MQ Larvae Room Chest" => None,
                "Dodongos Cavern MQ Torch Puzzle Room Chest" => None,
                "Dodongos Cavern MQ Under Grave Chest" => None,

                // Jabu Jabu's Belly MQ
                "Jabu Jabus Belly MQ First Room Side Chest" => None,
                "Jabu Jabus Belly MQ Map Chest" => None,
                "Jabu Jabus Belly MQ Second Room Lower Chest" => None,
                "Jabu Jabus Belly MQ Compass Chest" => None,
                "Jabu Jabus Belly MQ Second Room Upper Chest" => None,
                "Jabu Jabus Belly MQ Basement Near Switches Chest" => None,
                "Jabu Jabus Belly MQ Basement Near Vines Chest" => None,
                "Jabu Jabus Belly MQ Near Boss Chest" => None,
                "Jabu Jabus Belly MQ Falling Like Like Room Chest" => None,
                "Jabu Jabus Belly MQ Boomerang Room Small Chest" => None,
                "Jabu Jabus Belly MQ Boomerang Chest" => None,
                "Jabu Jabus Belly MQ Cow" => None,

                // Forest Temple MQ
                "Forest Temple MQ First Room Chest" => None,
                "Forest Temple MQ Wolfos Chest" => None,
                "Forest Temple MQ Bow Chest" => None,
                "Forest Temple MQ Raised Island Courtyard Lower Chest" => None,
                "Forest Temple MQ Raised Island Courtyard Upper Chest" => None,
                "Forest Temple MQ Well Chest" => None,
                "Forest Temple MQ Map Chest" => None,
                "Forest Temple MQ Compass Chest" => None,
                "Forest Temple MQ Falling Ceiling Room Chest" => None,
                "Forest Temple MQ Basement Chest" => None,
                "Forest Temple MQ Redead Chest" => None,
                "Forest Temple MQ Boss Key Chest" => None,

                // Fire Temple MQ
                "Fire Temple MQ Near Boss Chest" => None,
                "Fire Temple MQ Megaton Hammer Chest" => None,
                "Fire Temple MQ Compass Chest" => None,
                "Fire Temple MQ Lizalfos Maze Lower Chest" => None,
                "Fire Temple MQ Lizalfos Maze Upper Chest" => None,
                "Fire Temple MQ Chest On Fire" => None,
                "Fire Temple MQ Map Room Side Chest" => None,
                "Fire Temple MQ Map Chest" => None,
                "Fire Temple MQ Boss Key Chest" => None,
                "Fire Temple MQ Big Lava Room Blocked Door Chest" => None,
                "Fire Temple MQ Lizalfos Maze Side Room Chest" => None,
                "Fire Temple MQ Freestanding Key" => None,

                // Water Temple MQ
                "Water Temple MQ Central Pillar Chest" => None,
                "Water Temple MQ Boss Key Chest" => None,
                "Water Temple MQ Longshot Chest" => None,
                "Water Temple MQ Compass Chest" => None,
                "Water Temple MQ Map Chest" => None,
                "Water Temple MQ Freestanding Key" => None,

                // Spirit Temple MQ
                "Spirit Temple MQ Entrance Front Left Chest" => None,
                "Spirit Temple MQ Entrance Back Right Chest" => None,
                "Spirit Temple MQ Entrance Front Right Chest" => None,
                "Spirit Temple MQ Entrance Back Left Chest" => None,
                "Spirit Temple MQ Child Hammer Switch Chest" => None,
                "Spirit Temple MQ Map Chest" => None,
                "Spirit Temple MQ Map Room Enemy Chest" => None,
                "Spirit Temple MQ Child Climb North Chest" => None,
                "Spirit Temple MQ Child Climb South Chest" => None,
                "Spirit Temple MQ Compass Chest" => None,
                "Spirit Temple MQ Statue Room Lullaby Chest" => None,
                "Spirit Temple MQ Statue Room Invisible Chest" => None,
                "Spirit Temple MQ Silver Block Hallway Chest" => None,
                "Spirit Temple MQ Sun Block Room Chest" => None,
                "Spirit Temple MQ Symphony Room Chest" => None,
                "Spirit Temple MQ Leever Room Chest" => None,
                "Spirit Temple MQ Beamos Room Chest" => None,
                "Spirit Temple MQ Chest Switch Chest" => None,
                "Spirit Temple MQ Boss Key Chest" => None,
                "Spirit Temple MQ Mirror Puzzle Invisible Chest" => None,

                // Shadow Temple MQ
                "Shadow Temple MQ Compass Chest" => None,
                "Shadow Temple MQ Hover Boots Chest" => None,
                "Shadow Temple MQ Early Gibdos Chest" => None,
                "Shadow Temple MQ Map Chest" => None,
                "Shadow Temple MQ Beamos Silver Rupees Chest" => None,
                "Shadow Temple MQ Falling Spikes Switch Chest" => None,
                "Shadow Temple MQ Falling Spikes Lower Chest" => None,
                "Shadow Temple MQ Falling Spikes Upper Chest" => None,
                "Shadow Temple MQ Invisible Spikes Chest" => None,
                "Shadow Temple MQ Boss Key Chest" => None,
                "Shadow Temple MQ Spike Walls Left Chest" => None,
                "Shadow Temple MQ Stalfos Room Chest" => None,
                "Shadow Temple MQ Invisible Blades Invisible Chest" => None,
                "Shadow Temple MQ Invisible Blades Visible Chest" => None,
                "Shadow Temple MQ Bomb Flower Chest" => None,
                "Shadow Temple MQ Wind Hint Chest" => None,
                "Shadow Temple MQ After Wind Hidden Chest" => None,
                "Shadow Temple MQ After Wind Enemy Chest" => None,
                "Shadow Temple MQ Near Ship Invisible Chest" => None,
                "Shadow Temple MQ Freestanding Key" => None,

                // Bottom of the Well MQ
                "Bottom of the Well MQ Map Chest" => None,
                "Bottom of the Well MQ Lens of Truth Chest" => None,
                "Bottom of the Well MQ Compass Chest" => None,
                "Bottom of the Well MQ Dead Hand Freestanding Key" => None,
                "Bottom of the Well MQ East Inner Room Freestanding Key" => None,

                // Ice Cavern MQ
                "Ice Cavern MQ Iron Boots Chest" => None,
                "Ice Cavern MQ Compass Chest" => None,
                "Ice Cavern MQ Map Chest" => None,
                "Ice Cavern MQ Freestanding PoH" => None,

                // Gerudo Training Ground MQ
                "Gerudo Training Ground MQ Lobby Right Chest" => None,
                "Gerudo Training Ground MQ Lobby Left Chest" => None,
                "Gerudo Training Ground MQ First Iron Knuckle Chest" => None,
                "Gerudo Training Ground MQ Before Heavy Block Chest" => None,
                "Gerudo Training Ground MQ Eye Statue Chest" => None,
                "Gerudo Training Ground MQ Flame Circle Chest" => None,
                "Gerudo Training Ground MQ Second Iron Knuckle Chest" => None,
                "Gerudo Training Ground MQ Dinolfos Chest" => None,
                "Gerudo Training Ground MQ Ice Arrows Chest" => None,
                "Gerudo Training Ground MQ Maze Right Central Chest" => None,
                "Gerudo Training Ground MQ Maze Path First Chest" => None,
                "Gerudo Training Ground MQ Maze Right Side Chest" => None,
                "Gerudo Training Ground MQ Maze Path Third Chest" => None,
                "Gerudo Training Ground MQ Maze Path Second Chest" => None,
                "Gerudo Training Ground MQ Hidden Ceiling Chest" => None,
                "Gerudo Training Ground MQ Underwater Silver Rupee Chest" => None,
                "Gerudo Training Ground MQ Heavy Block Chest" => None,

                // Ganon's Castle MQ
                "Ganons Castle MQ Water Trial Chest" => None,
                "Ganons Castle MQ Forest Trial Eye Switch Chest" => None,
                "Ganons Castle MQ Forest Trial Frozen Eye Switch Chest" => None,
                "Ganons Castle MQ Light Trial Lullaby Chest" => None,
                "Ganons Castle MQ Shadow Trial Bomb Flower Chest" => None,
                "Ganons Castle MQ Shadow Trial Eye Switch Chest" => None,
                "Ganons Castle MQ Spirit Trial Golden Gauntlets Chest" => None,
                "Ganons Castle MQ Spirit Trial Sun Back Right Chest" => None,
                "Ganons Castle MQ Spirit Trial Sun Back Left Chest" => None,
                "Ganons Castle MQ Spirit Trial Sun Front Left Chest" => None,
                "Ganons Castle MQ Spirit Trial First Chest" => None,
                "Ganons Castle MQ Spirit Trial Invisible Chest" => None,
                "Ganons Castle MQ Forest Trial Freestanding Key" => None,

                "Links Pocket" => Some(true), //TODO check if vanilla or rando, if vanilla check for appropriate flag
                "Queen Gohma" => None,
                "Twinrova" => None,
                "Bongo Bongo" => None,
                "Ganon" => Some(false), //TODO remember if game has been beaten (relevant for multiworld and go mode)

                "Deku Tree Queen Gohma Heart" => None,
                "Dodongos Cavern King Dodongo Heart" => None,
                "Jabu Jabus Belly Barinade Heart" => None,
                "Forest Temple Phantom Ganon Heart" => None,
                "Fire Temple Volvagia Heart" => None,
                "Water Temple Morpha Heart" => None,
                "Spirit Temple Twinrova Heart" => None,
                "Shadow Temple Bongo Bongo Heart" => None,

                // Dungeon Skulls
                "Deku Tree GS Basement Back Room" => None,
                "Deku Tree GS Basement Gate" => None,
                "Deku Tree GS Basement Vines" => None,
                "Deku Tree GS Compass Room" => None,

                "Deku Tree MQ GS Lobby" => None,
                "Deku Tree MQ GS Compass Room" => None,
                "Deku Tree MQ GS Basement Graves Room" => None,
                "Deku Tree MQ GS Basement Back Room" => None,

                "Dodongos Cavern GS Vines Above Stairs" => None,
                "Dodongos Cavern GS Scarecrow" => None,
                "Dodongos Cavern GS Alcove Above Stairs" => None,
                "Dodongos Cavern GS Back Room" => None,
                "Dodongos Cavern GS Side Room Near Lower Lizalfos" => None,

                "Dodongos Cavern MQ GS Scrub Room" => None,
                "Dodongos Cavern MQ GS Song of Time Block Room" => None,
                "Dodongos Cavern MQ GS Lizalfos Room" => None,
                "Dodongos Cavern MQ GS Larvae Room" => None,
                "Dodongos Cavern MQ GS Back Area" => None,

                "Jabu Jabus Belly GS Lobby Basement Lower" => None,
                "Jabu Jabus Belly GS Lobby Basement Upper" => None,
                "Jabu Jabus Belly GS Near Boss" => None,
                "Jabu Jabus Belly GS Water Switch Room" => None,

                "Jabu Jabus Belly MQ GS Tailpasaran Room" => None,
                "Jabu Jabus Belly MQ GS Invisible Enemies Room" => None,
                "Jabu Jabus Belly MQ GS Boomerang Chest Room" => None,
                "Jabu Jabus Belly MQ GS Near Boss" => None,

                "Forest Temple GS Raised Island Courtyard" => None,
                "Forest Temple GS First Room" => None,
                "Forest Temple GS Lobby" => None,
                "Forest Temple GS Basement" => None,

                "Forest Temple MQ GS First Hallway" => None,
                "Forest Temple MQ GS Block Push Room" => None,
                "Forest Temple MQ GS Raised Island Courtyard" => None,
                "Forest Temple MQ GS Level Island Courtyard" => None,
                "Forest Temple MQ GS Well" => None,

                "Fire Temple GS Song of Time Room" => None,
                "Fire Temple GS Boss Key Loop" => None,
                "Fire Temple GS Boulder Maze" => None,
                "Fire Temple GS Scarecrow Top" => None,
                "Fire Temple GS Scarecrow Climb" => None,

                "Fire Temple MQ GS Above Fire Wall Maze" => None,
                "Fire Temple MQ GS Fire Wall Maze Center" => None,
                "Fire Temple MQ GS Big Lava Room Open Door" => None,
                "Fire Temple MQ GS Fire Wall Maze Side Room" => None,
                "Fire Temple MQ GS Skull On Fire" => None,

                "Water Temple GS Behind Gate" => None,
                "Water Temple GS Falling Platform Room" => None,
                "Water Temple GS Central Pillar" => None,
                "Water Temple GS Near Boss Key Chest" => None,
                "Water Temple GS River" => None,

                "Water Temple MQ GS Before Upper Water Switch" => None,
                "Water Temple MQ GS Freestanding Key Area" => None,
                "Water Temple MQ GS Lizalfos Hallway" => None,
                "Water Temple MQ GS River" => None,
                "Water Temple MQ GS Triple Wall Torch" => None,

                "Spirit Temple GS Hall After Sun Block Room" => None,
                "Spirit Temple GS Boulder Room" => None,
                "Spirit Temple GS Lobby" => None,
                "Spirit Temple GS Sun on Floor Room" => None,
                "Spirit Temple GS Metal Fence" => None,

                "Spirit Temple MQ GS Symphony Room" => None,
                "Spirit Temple MQ GS Leever Room" => None,
                "Spirit Temple MQ GS Nine Thrones Room West" => None,
                "Spirit Temple MQ GS Nine Thrones Room North" => None,
                "Spirit Temple MQ GS Sun Block Room" => None,

                "Shadow Temple GS Single Giant Pot" => None,
                "Shadow Temple GS Falling Spikes Room" => None,
                "Shadow Temple GS Triple Giant Pot" => None,
                "Shadow Temple GS Like Like Room" => None,
                "Shadow Temple GS Near Ship" => None,

                "Shadow Temple MQ GS Falling Spikes Room" => None,
                "Shadow Temple MQ GS Wind Hint Room" => None,
                "Shadow Temple MQ GS After Wind" => None,
                "Shadow Temple MQ GS After Ship" => None,
                "Shadow Temple MQ GS Near Boss" => None,

                // Mini Dungeon Skulls
                "Bottom of the Well GS Like Like Cage" => None,
                "Bottom of the Well GS East Inner Room" => None,
                "Bottom of the Well GS West Inner Room" => None,

                "Bottom of the Well MQ GS Basement" => None,
                "Bottom of the Well MQ GS Coffin Room" => None,
                "Bottom of the Well MQ GS West Inner Room" => None,

                "Ice Cavern GS Push Block Room" => None,
                "Ice Cavern GS Spinning Scythe Room" => None,
                "Ice Cavern GS Heart Piece Room" => None,

                "Ice Cavern MQ GS Scarecrow" => None,
                "Ice Cavern MQ GS Ice Block" => None,
                "Ice Cavern MQ GS Red Ice" => None,

                // Overworld Skulls
                "HF GS Cow Grotto" => None,
                "HF GS Near Kak Grotto" => None,

                "LLR GS Back Wall" => None,
                "LLR GS Rain Shed" => None,
                "LLR GS House Window" => None,
                "LLR GS Tree" => None,

                "KF GS Bean Patch" => None,
                "KF GS Know It All House" => None,
                "KF GS House of Twins" => None,

                "LW GS Bean Patch Near Bridge" => None,
                "LW GS Bean Patch Near Theater" => None,
                "LW GS Above Theater" => None,
                "SFM GS" => None,

                "OGC GS" => None,
                "HC GS Storms Grotto" => None,
                "HC GS Tree" => None,
                "Market GS Guard House" => None,

                "DMC GS Bean Patch" => None,
                "DMC GS Crate" => None,

                "DMT GS Bean Patch" => None,
                "DMT GS Near Kak" => None,
                "DMT GS Above Dodongos Cavern" => None,
                "DMT GS Falling Rocks Path" => None,

                "GC GS Center Platform" => None,
                "GC GS Boulder Maze" => None,

                "Kak GS House Under Construction" => None,
                "Kak GS Skulltula House" => None,
                "Kak GS Guards House" => None,
                "Kak GS Tree" => None,
                "Kak GS Watchtower" => None,
                "Kak GS Above Impas House" => None,

                "Graveyard GS Wall" => None,
                "Graveyard GS Bean Patch" => None,

                "ZR GS Ladder" => None,
                "ZR GS Tree" => None,
                "ZR GS Above Bridge" => None,
                "ZR GS Near Raised Grottos" => None,

                "ZD GS Frozen Waterfall" => None,
                "ZF GS Above the Log" => None,
                "ZF GS Hidden Cave" => None,
                "ZF GS Tree" => None,

                "LH GS Bean Patch" => None,
                "LH GS Small Island" => None,
                "LH GS Lab Wall" => None,
                "LH GS Lab Crate" => None,
                "LH GS Tree" => None,

                "GV GS Bean Patch" => None,
                "GV GS Small Bridge" => None,
                "GV GS Pillar" => None,
                "GV GS Behind Tent" => None,

                "GF GS Archery Range" => None,
                "GF GS Top Floor" => None,

                "Wasteland GS" => None,
                "Colossus GS Bean Patch" => None,
                "Colossus GS Hill" => None,
                "Colossus GS Tree" => None,

                // Shops
                "KF Shop Item 1" => None,
                "KF Shop Item 2" => None,
                "KF Shop Item 3" => None,
                "KF Shop Item 4" => None,
                "KF Shop Item 5" => None,
                "KF Shop Item 6" => None,
                "KF Shop Item 7" => None,
                "KF Shop Item 8" => None,

                "Kak Potion Shop Item 1" => None,
                "Kak Potion Shop Item 2" => None,
                "Kak Potion Shop Item 3" => None,
                "Kak Potion Shop Item 4" => None,
                "Kak Potion Shop Item 5" => None,
                "Kak Potion Shop Item 6" => None,
                "Kak Potion Shop Item 7" => None,
                "Kak Potion Shop Item 8" => None,

                "Market Bombchu Shop Item 1" => None,
                "Market Bombchu Shop Item 2" => None,
                "Market Bombchu Shop Item 3" => None,
                "Market Bombchu Shop Item 4" => None,
                "Market Bombchu Shop Item 5" => None,
                "Market Bombchu Shop Item 6" => None,
                "Market Bombchu Shop Item 7" => None,
                "Market Bombchu Shop Item 8" => None,

                "Market Potion Shop Item 1" => None,
                "Market Potion Shop Item 2" => None,
                "Market Potion Shop Item 3" => None,
                "Market Potion Shop Item 4" => None,
                "Market Potion Shop Item 5" => None,
                "Market Potion Shop Item 6" => None,
                "Market Potion Shop Item 7" => None,
                "Market Potion Shop Item 8" => None,

                "Market Bazaar Item 1" => None,
                "Market Bazaar Item 2" => None,
                "Market Bazaar Item 3" => None,
                "Market Bazaar Item 4" => None,
                "Market Bazaar Item 5" => None,
                "Market Bazaar Item 6" => None,
                "Market Bazaar Item 7" => None,
                "Market Bazaar Item 8" => None,

                "Kak Bazaar Item 1" => None,
                "Kak Bazaar Item 2" => None,
                "Kak Bazaar Item 3" => None,
                "Kak Bazaar Item 4" => None,
                "Kak Bazaar Item 5" => None,
                "Kak Bazaar Item 6" => None,
                "Kak Bazaar Item 7" => None,
                "Kak Bazaar Item 8" => None,

                "ZD Shop Item 1" => None,
                "ZD Shop Item 2" => None,
                "ZD Shop Item 3" => None,
                "ZD Shop Item 4" => None,
                "ZD Shop Item 5" => None,
                "ZD Shop Item 6" => None,
                "ZD Shop Item 7" => None,
                "ZD Shop Item 8" => None,

                "GC Shop Item 1" => None,
                "GC Shop Item 2" => None,
                "GC Shop Item 3" => None,
                "GC Shop Item 4" => None,
                "GC Shop Item 5" => None,
                "GC Shop Item 6" => None,
                "GC Shop Item 7" => None,
                "GC Shop Item 8" => None,

                // NPC Scrubs are on the overworld, while GrottoNPC is a special handler for Grottos
                // Grottos scrubs are the same scene and actor, so we use a unique grotto ID for the scene

                "Deku Tree MQ Deku Scrub" => None,

                "HF Deku Scrub Grotto" => None,
                "LLR Deku Scrub Grotto Left" => None,
                "LLR Deku Scrub Grotto Right" => None,
                "LLR Deku Scrub Grotto Center" => None,

                "LW Deku Scrub Near Deku Theater Right" => None,
                "LW Deku Scrub Near Deku Theater Left" => None,
                "LW Deku Scrub Grotto Rear" => None,
                "LW Deku Scrub Grotto Front" => None,

                "SFM Deku Scrub Grotto Rear" => None,
                "SFM Deku Scrub Grotto Front" => None,

                "GC Deku Scrub Grotto Left" => None,
                "GC Deku Scrub Grotto Right" => None,
                "GC Deku Scrub Grotto Center" => None,

                "Dodongos Cavern Deku Scrub Near Bomb Bag Left" => None,
                "Dodongos Cavern Deku Scrub Side Room Near Dodongos" => None,
                "Dodongos Cavern Deku Scrub Near Bomb Bag Right" => None,
                "Dodongos Cavern Deku Scrub Lobby" => None,

                "Dodongos Cavern MQ Deku Scrub Lobby Rear" => None,
                "Dodongos Cavern MQ Deku Scrub Lobby Front" => None,
                "Dodongos Cavern MQ Deku Scrub Staircase" => None,
                "Dodongos Cavern MQ Deku Scrub Side Room Near Lower Lizalfos" => None,

                "DMC Deku Scrub" => None,
                "DMC Deku Scrub Grotto Left" => None,
                "DMC Deku Scrub Grotto Right" => None,
                "DMC Deku Scrub Grotto Center" => None,

                "ZR Deku Scrub Grotto Rear" => None,
                "ZR Deku Scrub Grotto Front" => None,

                "Jabu Jabus Belly Deku Scrub" => None,

                "LH Deku Scrub Grotto Left" => None,
                "LH Deku Scrub Grotto Right" => None,
                "LH Deku Scrub Grotto Center" => None,

                "GV Deku Scrub Grotto Rear" => None,
                "GV Deku Scrub Grotto Front" => None,

                "Colossus Deku Scrub Grotto Rear" => None,
                "Colossus Deku Scrub Grotto Front" => None,

                "Ganons Castle Deku Scrub Center-Left" => None,
                "Ganons Castle Deku Scrub Center-Right" => None,
                "Ganons Castle Deku Scrub Right" => None,
                "Ganons Castle Deku Scrub Left" => None,

                "Ganons Castle MQ Deku Scrub Right" => None,
                "Ganons Castle MQ Deku Scrub Center-Left" => None,
                "Ganons Castle MQ Deku Scrub Center" => None,
                "Ganons Castle MQ Deku Scrub Center-Right" => None,
                "Ganons Castle MQ Deku Scrub Left" => None,

                "LLR Stables Left Cow" => None,
                "LLR Stables Right Cow" => None,
                "LLR Tower Right Cow" => None,
                "LLR Tower Left Cow" => None,
                "KF Links House Cow" => None,
                "Kak Impas House Cow" => None,
                "GV Cow" => None,
                "DMT Cow Grotto Cow" => None,
                "HF Cow Grotto Cow" => None,

                // These are not actual locations, but are filler spots used for hint reachability
                "DMC Gossip Stone" => None, //TODO check knowledge
                "DMT Gossip Stone" => None, //TODO check knowledge
                "Colossus Gossip Stone" => None, //TODO check knowledge
                "Dodongos Cavern Gossip Stone" => None, //TODO check knowledge
                "GV Gossip Stone" => None, //TODO check knowledge
                "GC Maze Gossip Stone" => None, //TODO check knowledge
                "GC Medigoron Gossip Stone" => None, //TODO check knowledge
                "Graveyard Gossip Stone" => None, //TODO check knowledge
                "HC Malon Gossip Stone" => None, //TODO check knowledge
                "HC Rock Wall Gossip Stone" => None, //TODO check knowledge
                "HC Storms Grotto Gossip Stone" => None, //TODO check knowledge
                "HF Cow Grotto Gossip Stone" => None, //TODO check knowledge
                "KF Deku Tree Gossip Stone (Left)" => None, //TODO check knowledge
                "KF Deku Tree Gossip Stone (Right)" => None, //TODO check knowledge
                "KF Gossip Stone" => None, //TODO check knowledge
                "LH Lab Gossip Stone" => None, //TODO check knowledge
                "LH Gossip Stone (Southeast)" => None, //TODO check knowledge
                "LH Gossip Stone (Southwest)" => None, //TODO check knowledge
                "LW Gossip Stone" => None, //TODO check knowledge
                "SFM Maze Gossip Stone (Lower)" => None, //TODO check knowledge
                "SFM Maze Gossip Stone (Upper)" => None, //TODO check knowledge
                "SFM Saria Gossip Stone" => None, //TODO check knowledge
                "ToT Gossip Stone (Left)" => None, //TODO check knowledge
                "ToT Gossip Stone (Left-Center)" => None, //TODO check knowledge
                "ToT Gossip Stone (Right)" => None, //TODO check knowledge
                "ToT Gossip Stone (Right-Center)" => None, //TODO check knowledge
                "ZD Gossip Stone" => None, //TODO check knowledge
                "ZF Fairy Gossip Stone" => None, //TODO check knowledge
                "ZF Jabu Gossip Stone" => None, //TODO check knowledge
                "ZR Near Grottos Gossip Stone" => None, //TODO check knowledge
                "ZR Near Domain Gossip Stone" => None, //TODO check knowledge

                "HF Near Market Grotto Gossip Stone" => None, //TODO check knowledge
                "HF Southeast Grotto Gossip Stone" => None, //TODO check knowledge
                "HF Open Grotto Gossip Stone" => None, //TODO check knowledge
                "Kak Open Grotto Gossip Stone" => None, //TODO check knowledge
                "ZR Open Grotto Gossip Stone" => None, //TODO check knowledge
                "KF Storms Grotto Gossip Stone" => None, //TODO check knowledge
                "LW Near Shortcuts Grotto Gossip Stone" => None, //TODO check knowledge
                "DMT Storms Grotto Gossip Stone" => None, //TODO check knowledge
                "DMC Upper Grotto Gossip Stone" => None, //TODO check knowledge

                "Ganondorf Hint" => None, //TODO check knowledge

                _ => panic!("unknown location name: {}", loc),
            },
            Check::LogicHelper(_) => panic!("logic helpers can't be checked"),
            Check::Mq(_) => Some(false), //TODO disambiguate MQ-ness here instead?
            Check::Setting(_) => panic!("setting checks not implemented"), //TODO check knowledge
            Check::TrialActive(_) => panic!("trial-active checks not implemented"), //TODO check knowledge
            Check::Trick(_) => panic!("trick checks not implemented"), //TODO check knowledge, allow the player to decide their own tricks if unknown
            */
            _ => None, //TODO error?
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckStatus {
    Checked,
    Reachable,
    NotYetReachable, //TODO split into definitely/possibly/not reachable later in order to determine Reachable Locations setting
}

/*
#[derive(Debug, Clone, From, FromArc)]
pub enum CheckStatusError {
    #[from_arc]
    Io(Arc<io::Error>),
    RegionLookup(RegionLookupError),
}

impl fmt::Display for CheckStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckStatusError::Io(e) => write!(f, "I/O error: {}", e),
            CheckStatusError::RegionLookup(e) => e.fmt(f),
        }
    }
}
*/ //TODO

/// WIP Algorithm to determine which checks are reachable
///
/// # Knowledge checks
///
/// A *knowledge check* is a kind of check introduced by the tracker to handle random/mystery settings scenarios correctly, among other use-cases.
/// Most knowledge checks are hardcoded, but there are also exit checks which simply exist for every exit.
/// A knowledge check is considered *checked* if the current knowledge about the seed is already a subset of one of the possible answers of the check.
/// A knowledge check is considered *dead* (the equivalent to a location check that can no longer logically advance progression) if there are no unchecked access expressions where it can make a difference.
/// Reachability of knowledge checks is calculated first, before any other checks.
/// Some knowledge checks are special in that they require *not* having certain items. For example, there are different knowledge checks for LACS and rainbow bridge conditions depending on the current inventory.
///
/// # Reachability of access expressions
///
/// At compile time:
///
/// 1. Detemine the partition of the knowledge space that is relevant for evaluating the expression. If the expression has subexpressions, this is the set of intersections of members of the Cartesian product of the partitions yielded by the subexpressions.
/// 2. Ignore all contradictions in the partition.
/// 3. Then for each remaining possibility in the partition, determine the required items and events, assuming that knowledge, in sum of products form. If a possibility is unreachable even with a complete inventory, remove it from the partition.
///
/// At runtime:
///
/// 1. Remove all possibilities that contradict current knowledge or require the other age. If no possibilities remain, consider the expression unreachable.
/// 2. Based on current items, sort the possibilities into ones where the access is possible and ones where it isn't. If only one of these sets is nonempty, return accordingly.
/// 3. For each possible combination of results of the currently accessible knowledge checks that isn't a contradiction, check if it can confirm or deconfirm access by contradicting all possibilities in one of the sets. If so, consider access *impossible* (until performing knowledge checks reevaluates the runtime portion of this algorithm). Otherwise, assume it *is* reachable, since there should never be a situation where you can't know whether you are or aren't allowed to do something but actually aren't.
///
/// All events in an access expression, including anonymous events, should be evaluated based on whether they *have been* checked, not whether they *can be* checked. Instead, access for events is simply handled like access for locations.
///
/// # Reachability of regions
///
/// * If the starting age is unknown, all checks other than the starting age knowledge check are unreachable.
/// * First, determine which regions are reachable as the current age from the current region
/// * If the current age's spawn point is known, add all regions reachable from there. Otherwise, add a knowledge check for that entrance as reachable
/// * If Time Travel is reachable, add all regions reachable from the Temple of Time as the other age, and handle that age's spawn accordingly
///
/// Then determine which locations, events, and anonymous events within those regions are reachable, taking into account the age(s) as which the region is reachable.
///
/// Potential issue with this algorithm: A knowledge check may be locked behind other checks that depend on it. May have to iteratively determine which knowledge checks are reachable and thus which other checks should default to being assumed reachable, but this may also result in an infinite loop.
pub fn status(model: &ModelState) -> HashMap<Check, bool> {
    let mut child_region_reachability = Region::into_enum_iter().map(|region| (region, false)).collect::<HashMap<_, _>>();
    let mut adult_region_reachability = Region::into_enum_iter().map(|region| (region, false)).collect();
    let current_region_reachability = if model.ram.save.is_adult { &mut adult_region_reachability } else { &mut child_region_reachability }; //TODO check against effective starting age setting in knowledge to ensure this is correct and not just uninitialized. If unknown, add knowledge check for starting age.
    let mut unhandled_current_reachable_regions = vec![Region::Root]; //TODO insert current region as well
    while let Some(region) = unhandled_current_reachable_regions.pop() {
        current_region_reachability.insert(region, true);
        for (_ /*vanilla_target*/, can_access) in region.exits() {
            if can_access(model) {
                //TODO check exit knowledge
                //TODO if the exit can be shuffled, add an exit check, marked as checked if exit is known, and as reachable otherwise
                //TODO if known, also add the region behind the exit to unhandled_current_reachable_regions (if not already marked reachable or in there)
            }
        }
    }
    //TODO repeat the same for the other age
    //TODO add unreachable exit knowledge checks for all exits that can be shuffled but haven't already been visited
    //TODO handle locations, events, anonymous events
    //TODO handle knowledge checks
    HashMap::default() //TODO
}

/*
pub fn status(model: &ModelState) -> Result<HashMap<Check, CheckStatus>, CheckStatusError> {
    let mut map = HashMap::default();
    let all_regions = Region::into_enum_iter();
    let mut reachable_regions = iter::once(Region::Root).collect::<HashSet<_>>();
    let mut unhandled_reachable_checks = Vec::default();
    let current_region = model.current_region()?;
    //TODO run separate logic check using knowledge only and not considering current region
    if let Some(dungeon) = current_region.mq_check() {
        unhandled_reachable_checks.push((Check::Mq(dungeon), |_| true));
    } else {
        reachable_regions.insert(current_region);
    }
    let mut unhandled_reachable_regions = reachable_regions.iter().cloned().collect_vec();
    let mut unhandled_unreachable_regions = all_regions.iter().filter(|region_info| !reachable_regions.contains(*region_info)).collect::<HashSet<_>>();
    let mut unhandled_unreachable_checks = Vec::<(_, _ /*access::Expr*/ /*TODO access function type */)>::default();
    loop {
        if let Some(region) = unhandled_reachable_regions.pop() {
            for (exit, rule) in &region.exits() {
                unhandled_reachable_checks.push((Check::Exit { from: region, to: exit }, rule.clone()));
            }
            //TODO events, locations, setting checks
        } else if let Some((check, rule)) = unhandled_reachable_checks.pop() {
            let status = if check.checked(model).expect(&format!("checked unimplemented for {}", check)) {
                if let Check::Exit { to, .. } = check {
                    let region_behind_exit = to; //TODO entrance rando support (look up exit knowledge)
                    if !reachable_regions.iter().any(|region| region == region_behind_exit) {
                        if model.can_access(&rule) == Ok(true) {
                            // exit is checked (i.e. we know what's behind it) and reachable (i.e. we can actually use it), so the region behind it becomes reachable
                            let region_info = match Region::new(region_behind_exit)? {
                                RegionLookup::Overworld(region)
                                | RegionLookup::Dungeon(EitherOrBoth::Left(region))
                                | RegionLookup::Dungeon(EitherOrBoth::Right(region)) => region,
                                RegionLookup::Dungeon(EitherOrBoth::Both(_, _)) => unimplemented!(), //TODO disambiguate MQ-ness based on knowledge, add MQ-ness check if unknown
                            };
                            unhandled_unreachable_regions.remove(&region_info);
                            reachable_regions.insert(region_info);
                            unhandled_reachable_regions.push(region_info);
                        }
                    }
                }
                CheckStatus::Checked
            } else {
                match model.can_access(&rule) {
                    Ok(true) => CheckStatus::Reachable,
                    Ok(false) => CheckStatus::NotYetReachable,
                    Err(deps) => {
                        map.extend(deps.into_iter().map(|dep| (dep, CheckStatus::Reachable))); //TODO check reachability of dependency
                        CheckStatus::NotYetReachable
                    }
                }
            };
            map.insert(check, status);
        } else if !unhandled_unreachable_regions.is_empty() {
            for region in unhandled_unreachable_regions.drain() {
                for (exit, cond) in &region.exits {
                    unhandled_unreachable_checks.push((Check::Exit { from_mq: region.dungeon.map(|(_, mq)| mq), from: region.name.clone(), to: exit.clone() }, cond.to_owned()));
                }
                //TODO events, locations, setting checks
            }
        } else if let Some((check, rule)) = unhandled_unreachable_checks.pop() {
            let status = if check.checked(model).expect(&format!("checked unimplemented for {}", check)) {
                CheckStatus::Checked
            } else {
                match model.can_access(&rule) {
                    Ok(_) => CheckStatus::NotYetReachable,
                    Err(deps) => {
                        map.extend(deps.into_iter().map(|dep| (dep, CheckStatus::NotYetReachable))); //TODO check reachability of dependency
                        CheckStatus::NotYetReachable
                    }
                }
            };
            map.insert(check, status);
        } else {
            break
        }
    }
    Ok(map)
}
*/ //TODO rewrite into above

#[test]
fn default_status() {
    /*
    if let Err(e) = status(&ModelState::default()) {
        eprintln!("{:?}", e);
        panic!("{}", e) // for better error message
    }
    */ //TODO
    eprintln!("{:?}", status(&ModelState::default()));
}
