use crate::Save;

pub fn checked(save: &Save, loc: &str) -> Option<bool> { //TODO change return type to bool once all used checks are implemented
    if let Some(checked) = save.event_chk_inf.checked(loc) { return Some(checked) }
    if let Some(checked) = save.scene_flags.checked(loc) { return Some(checked) }
    // location list taken from Dev-R as of commit 478587c7c33cac7305a15a1cb9673861ae712213
    match loc {
        "KF Midos Top Left Chest" => None, //TODO
        "KF Midos Top Right Chest" => None, //TODO
        "KF Midos Bottom Left Chest" => None, //TODO
        "KF Midos Bottom Right Chest" => None, //TODO
        "Graveyard Shield Grave Chest" => None, //TODO
        "Graveyard Heart Piece Grave Chest" => None, //TODO
        "Graveyard Composers Grave Chest" => None, //TODO
        "DMT Chest" => None, //TODO
        "GC Maze Left Chest" => None, //TODO
        "GC Maze Right Chest" => None, //TODO
        "GC Maze Center Chest" => None, //TODO
        "ZD Chest" => None, //TODO
        "Graveyard Hookshot Chest" => None, //TODO
        "GV Chest" => None, //TODO
        "GF Chest" => None, //TODO
        "Wasteland Chest" => None, //TODO
        "Kak Redead Grotto Chest" => None, //TODO
        "SFM Wolfos Grotto Chest" => None, //TODO
        "HF Near Market Grotto Chest" => None, //TODO
        "HF Southeast Grotto Chest" => None, //TODO
        "HF Open Grotto Chest" => None, //TODO
        "Kak Open Grotto Chest" => None, //TODO
        "ZR Open Grotto Chest" => None, //TODO
        "KF Storms Grotto Chest" => None, //TODO
        "LW Near Shortcuts Grotto Chest" => None, //TODO
        "DMT Storms Grotto Chest" => None, //TODO
        "DMC Upper Grotto Chest" => None, //TODO
        "Market Treasure Chest Game Reward" => None, //TODO

        "ToT Light Arrows Cutscene" => None, //TODO
        "LW Gift from Saria" => None, //TODO
        "ZF Great Fairy Reward" => None, //TODO
        "HC Great Fairy Reward" => None, //TODO
        "Colossus Great Fairy Reward" => None, //TODO
        "DMT Great Fairy Reward" => None, //TODO
        "DMC Great Fairy Reward" => None, //TODO
        "OGC Great Fairy Reward" => None, //TODO

        "HC Malon Egg" => None, //TODO
        "HC Zeldas Letter" => None, //TODO
        "GC Darunias Joy" => None, //TODO
        "ZD Diving Minigame" => None, //TODO
        "LH Child Fishing" => None, //TODO
        "LH Adult Fishing" => None, //TODO
        "LH Lab Dive" => None, //TODO
        "GC Rolling Goron as Adult" => None, //TODO
        "ZD King Zora Thawed" => None, //TODO
        "Market Bombchu Bowling First Prize" => None, //TODO
        "Market Bombchu Bowling Second Prize" => None, //TODO
        "Market Bombchu Bowling Bombchus" => None, //TODO
        "Market Lost Dog" => None, //TODO
        "LW Skull Kid" => None, //TODO
        "LW Ocarina Memory Game" => None, //TODO
        "Kak 10 Gold Skulltula Reward" => None, //TODO
        "Kak 20 Gold Skulltula Reward" => None, //TODO
        "Kak 30 Gold Skulltula Reward" => None, //TODO
        "Kak 40 Gold Skulltula Reward" => None, //TODO
        "Kak 50 Gold Skulltula Reward" => None, //TODO
        "Kak Man on Roof" => None, //TODO
        "ZR Magic Bean Salesman" => None, //TODO
        "ZR Frogs Ocarina Game" => None, //TODO
        "ZR Frogs in the Rain" => None, //TODO
        "GF HBA 1000 Points" => None, //TODO
        "GF HBA 1500 Points" => None, //TODO
        "Market Shooting Gallery Reward" => None, //TODO
        "Kak Shooting Gallery Reward" => None, //TODO
        "LW Target in Woods" => None, //TODO
        "Deku Theater Skull Mask" => None, //TODO
        "Deku Theater Mask of Truth" => None, //TODO
        "Kak Anju as Adult" => None, //TODO
        "DMT Biggoron" => None, //TODO
        "Kak Anju as Child" => None, //TODO
        "LLR Talons Chickens" => None, //TODO
        "Market 10 Big Poes" => None, //TODO
        "GC Rolling Goron as Child" => None, //TODO
        "LH Underwater Item" => None, //TODO
        "LH Sun" => None, //TODO
        "GF Gerudo Membership Card" => None, //TODO
        "HF Ocarina of Time Item" => None, //TODO
        "Wasteland Bombchu Salesman" => None, //TODO
        "GC Medigoron" => None, //TODO

        "Kak Impas House Freestanding PoH" => None, //TODO
        "HF Tektite Grotto Freestanding PoH" => None, //TODO
        "Kak Windmill Freestanding PoH" => None, //TODO
        "Graveyard Dampe Race Freestanding PoH" => None, //TODO
        "LLR Freestanding PoH" => None, //TODO
        "Graveyard Freestanding PoH" => None, //TODO
        "Graveyard Dampe Gravedigging Tour" => None, //TODO
        "ZR Near Open Grotto Freestanding PoH" => None, //TODO
        "ZR Near Domain Freestanding PoH" => None, //TODO
        "LH Freestanding PoH" => None, //TODO
        "ZF Iceberg Freestanding PoH" => None, //TODO
        "ZF Bottom Freestanding PoH" => None, //TODO
        "GV Waterfall Freestanding PoH" => None, //TODO
        "GV Crate Freestanding PoH" => None, //TODO
        "Colossus Freestanding PoH" => None, //TODO
        "DMT Freestanding PoH" => None, //TODO
        "DMC Wall Freestanding PoH" => None, //TODO
        "DMC Volcano Freestanding PoH" => None, //TODO
        "GC Pot Freestanding PoH" => None, //TODO
        "GF North F1 Carpenter" => None, //TODO
        "GF North F2 Carpenter" => None, //TODO
        "GF South F1 Carpenter" => None, //TODO
        "GF South F2 Carpenter" => None, //TODO

        "Pierre" => None, //TODO
        "Deliver Rutos Letter" => None, //TODO
        "Master Sword Pedestal" => None, //TODO

        "Deku Baba Sticks" => None, //TODO
        "Deku Baba Nuts" => None, //TODO
        "Stick Pot" => None, //TODO
        "Nut Pot" => None, //TODO
        "Nut Crate" => None, //TODO
        "Blue Fire" => None, //TODO
        "Lone Fish" => None, //TODO
        "Fish Group" => None, //TODO
        "Bug Rock" => None, //TODO
        "Bug Shrub" => None, //TODO
        "Wandering Bugs" => None, //TODO
        "Fairy Pot" => None, //TODO
        "Free Fairies" => None, //TODO
        "Wall Fairy" => None, //TODO
        "Butterfly Fairy" => None, //TODO
        "Gossip Stone Fairy" => None, //TODO
        "Bean Plant Fairy" => None, //TODO
        "Fairy Pond" => None, //TODO
        "Big Poe Kill" => None, //TODO

        // Deku Tree vanilla
        "Deku Tree Map Chest" => None, //TODO
        "Deku Tree Slingshot Chest" => None, //TODO
        "Deku Tree Slingshot Room Side Chest" => None, //TODO
        "Deku Tree Compass Chest" => None, //TODO
        "Deku Tree Compass Room Side Chest" => None, //TODO
        "Deku Tree Basement Chest" => None, //TODO
        // Deku Tree MQ
        "Deku Tree MQ Map Chest" => None, //TODO
        "Deku Tree MQ Compass Chest" => None, //TODO
        "Deku Tree MQ Slingshot Chest" => None, //TODO
        "Deku Tree MQ Slingshot Room Back Chest" => None, //TODO
        "Deku Tree MQ Basement Chest" => None, //TODO
        "Deku Tree MQ Before Spinning Log Chest" => None, //TODO
        "Deku Tree MQ After Spinning Log Chest" => None, //TODO

        // Dodongo's Cavern shared
        "Dodongos Cavern Boss Room Chest" => None, //TODO
        // Dodongo's Cavern vanilla
        "Dodongos Cavern Map Chest" => None, //TODO
        "Dodongos Cavern Compass Chest" => None, //TODO
        "Dodongos Cavern Bomb Flower Platform Chest" => None, //TODO
        "Dodongos Cavern Bomb Bag Chest" => None, //TODO
        "Dodongos Cavern End of Bridge Chest" => None, //TODO
        // Dodongo's Cavern MQ
        "Dodongos Cavern MQ Map Chest" => None, //TODO
        "Dodongos Cavern MQ Bomb Bag Chest" => None, //TODO
        "Dodongos Cavern MQ Compass Chest" => None, //TODO
        "Dodongos Cavern MQ Larvae Room Chest" => None, //TODO
        "Dodongos Cavern MQ Torch Puzzle Room Chest" => None, //TODO
        "Dodongos Cavern MQ Under Grave Chest" => None, //TODO

        // Jabu Jabu's Belly vanilla
        "Jabu Jabus Belly Boomerang Chest" => None, //TODO
        "Jabu Jabus Belly Map Chest" => None, //TODO
        "Jabu Jabus Belly Compass Chest" => None, //TODO
        // Jabu Jabu's Belly MQ
        "Jabu Jabus Belly MQ First Room Side Chest" => None, //TODO
        "Jabu Jabus Belly MQ Map Chest" => None, //TODO
        "Jabu Jabus Belly MQ Second Room Lower Chest" => None, //TODO
        "Jabu Jabus Belly MQ Compass Chest" => None, //TODO
        "Jabu Jabus Belly MQ Second Room Upper Chest" => None, //TODO
        "Jabu Jabus Belly MQ Basement Near Switches Chest" => None, //TODO
        "Jabu Jabus Belly MQ Basement Near Vines Chest" => None, //TODO
        "Jabu Jabus Belly MQ Near Boss Chest" => None, //TODO
        "Jabu Jabus Belly MQ Falling Like Like Room Chest" => None, //TODO
        "Jabu Jabus Belly MQ Boomerang Room Small Chest" => None, //TODO
        "Jabu Jabus Belly MQ Boomerang Chest" => None, //TODO
        "Jabu Jabus Belly MQ Cow" => None, //TODO

        // Forest Temple vanilla
        "Forest Temple First Room Chest" => None, //TODO
        "Forest Temple First Stalfos Chest" => None, //TODO
        "Forest Temple Well Chest" => None, //TODO
        "Forest Temple Map Chest" => None, //TODO
        "Forest Temple Raised Island Courtyard Chest" => None, //TODO
        "Forest Temple Falling Ceiling Room Chest" => None, //TODO
        "Forest Temple Eye Switch Chest" => None, //TODO
        "Forest Temple Boss Key Chest" => None, //TODO
        "Forest Temple Floormaster Chest" => None, //TODO
        "Forest Temple Bow Chest" => None, //TODO
        "Forest Temple Red Poe Chest" => None, //TODO
        "Forest Temple Blue Poe Chest" => None, //TODO
        "Forest Temple Basement Chest" => None, //TODO
        // Forest Temple MQ
        "Forest Temple MQ First Room Chest" => None, //TODO
        "Forest Temple MQ Wolfos Chest" => None, //TODO
        "Forest Temple MQ Bow Chest" => None, //TODO
        "Forest Temple MQ Raised Island Courtyard Lower Chest" => None, //TODO
        "Forest Temple MQ Raised Island Courtyard Upper Chest" => None, //TODO
        "Forest Temple MQ Well Chest" => None, //TODO
        "Forest Temple MQ Map Chest" => None, //TODO
        "Forest Temple MQ Compass Chest" => None, //TODO
        "Forest Temple MQ Falling Ceiling Room Chest" => None, //TODO
        "Forest Temple MQ Basement Chest" => None, //TODO
        "Forest Temple MQ Redead Chest" => None, //TODO
        "Forest Temple MQ Boss Key Chest" => None, //TODO

        // Fire Temple vanilla
        "Fire Temple Near Boss Chest" => None, //TODO
        "Fire Temple Flare Dancer Chest" => None, //TODO
        "Fire Temple Boss Key Chest" => None, //TODO
        "Fire Temple Big Lava Room Blocked Door Chest" => None, //TODO
        "Fire Temple Big Lava Room Lower Open Door Chest" => None, //TODO
        "Fire Temple Boulder Maze Lower Chest" => None, //TODO
        "Fire Temple Boulder Maze Upper Chest" => None, //TODO
        "Fire Temple Boulder Maze Side Room Chest" => None, //TODO
        "Fire Temple Boulder Maze Shortcut Chest" => None, //TODO
        "Fire Temple Scarecrow Chest" => None, //TODO
        "Fire Temple Map Chest" => None, //TODO
        "Fire Temple Compass Chest" => None, //TODO
        "Fire Temple Highest Goron Chest" => None, //TODO
        "Fire Temple Megaton Hammer Chest" => None, //TODO
        // Fire Temple MQ
        "Fire Temple MQ Near Boss Chest" => None, //TODO
        "Fire Temple MQ Megaton Hammer Chest" => None, //TODO
        "Fire Temple MQ Compass Chest" => None, //TODO
        "Fire Temple MQ Lizalfos Maze Lower Chest" => None, //TODO
        "Fire Temple MQ Lizalfos Maze Upper Chest" => None, //TODO
        "Fire Temple MQ Chest On Fire" => None, //TODO
        "Fire Temple MQ Map Room Side Chest" => None, //TODO
        "Fire Temple MQ Map Chest" => None, //TODO
        "Fire Temple MQ Boss Key Chest" => None, //TODO
        "Fire Temple MQ Big Lava Room Blocked Door Chest" => None, //TODO
        "Fire Temple MQ Lizalfos Maze Side Room Chest" => None, //TODO
        "Fire Temple MQ Freestanding Key" => None, //TODO

        // Water Temple vanilla
        "Water Temple Map Chest" => None, //TODO
        "Water Temple Compass Chest" => None, //TODO
        "Water Temple Torches Chest" => None, //TODO
        "Water Temple Dragon Chest" => None, //TODO
        "Water Temple Central Bow Target Chest" => None, //TODO
        "Water Temple Central Pillar Chest" => None, //TODO
        "Water Temple Cracked Wall Chest" => None, //TODO
        "Water Temple Boss Key Chest" => None, //TODO
        "Water Temple Longshot Chest" => None, //TODO
        "Water Temple River Chest" => None, //TODO
        // Water Temple MQ
        "Water Temple MQ Central Pillar Chest" => None, //TODO
        "Water Temple MQ Boss Key Chest" => None, //TODO
        "Water Temple MQ Longshot Chest" => None, //TODO
        "Water Temple MQ Compass Chest" => None, //TODO
        "Water Temple MQ Map Chest" => None, //TODO
        "Water Temple MQ Freestanding Key" => None, //TODO

        // Spirit Temple shared
        "Spirit Temple Silver Gauntlets Chest" => None, //TODO
        "Spirit Temple Mirror Shield Chest" => None, //TODO
        // Spirit Temple vanilla
        "Spirit Temple Child Bridge Chest" => None, //TODO
        "Spirit Temple Child Early Torches Chest" => None, //TODO
        "Spirit Temple Compass Chest" => None, //TODO
        "Spirit Temple Early Adult Right Chest" => None, //TODO
        "Spirit Temple First Mirror Left Chest" => None, //TODO
        "Spirit Temple First Mirror Right Chest" => None, //TODO
        "Spirit Temple Map Chest" => None, //TODO
        "Spirit Temple Child Climb North Chest" => None, //TODO
        "Spirit Temple Child Climb East Chest" => None, //TODO
        "Spirit Temple Sun Block Room Chest" => None, //TODO
        "Spirit Temple Statue Room Hand Chest" => None, //TODO
        "Spirit Temple Statue Room Northeast Chest" => None, //TODO
        "Spirit Temple Near Four Armos Chest" => None, //TODO
        "Spirit Temple Hallway Right Invisible Chest" => None, //TODO
        "Spirit Temple Hallway Left Invisible Chest" => None, //TODO
        "Spirit Temple Boss Key Chest" => None, //TODO
        "Spirit Temple Topmost Chest" => None, //TODO
        // Spirit Temple MQ
        "Spirit Temple MQ Entrance Front Left Chest" => None, //TODO
        "Spirit Temple MQ Entrance Back Right Chest" => None, //TODO
        "Spirit Temple MQ Entrance Front Right Chest" => None, //TODO
        "Spirit Temple MQ Entrance Back Left Chest" => None, //TODO
        "Spirit Temple MQ Child Hammer Switch Chest" => None, //TODO
        "Spirit Temple MQ Map Chest" => None, //TODO
        "Spirit Temple MQ Map Room Enemy Chest" => None, //TODO
        "Spirit Temple MQ Child Climb North Chest" => None, //TODO
        "Spirit Temple MQ Child Climb South Chest" => None, //TODO
        "Spirit Temple MQ Compass Chest" => None, //TODO
        "Spirit Temple MQ Statue Room Lullaby Chest" => None, //TODO
        "Spirit Temple MQ Statue Room Invisible Chest" => None, //TODO
        "Spirit Temple MQ Silver Block Hallway Chest" => None, //TODO
        "Spirit Temple MQ Sun Block Room Chest" => None, //TODO
        "Spirit Temple MQ Symphony Room Chest" => None, //TODO
        "Spirit Temple MQ Leever Room Chest" => None, //TODO
        "Spirit Temple MQ Beamos Room Chest" => None, //TODO
        "Spirit Temple MQ Chest Switch Chest" => None, //TODO
        "Spirit Temple MQ Boss Key Chest" => None, //TODO
        "Spirit Temple MQ Mirror Puzzle Invisible Chest" => None, //TODO

        // Shadow Temple vanilla
        "Shadow Temple Map Chest" => None, //TODO
        "Shadow Temple Hover Boots Chest" => None, //TODO
        "Shadow Temple Compass Chest" => None, //TODO
        "Shadow Temple Early Silver Rupee Chest" => None, //TODO
        "Shadow Temple Invisible Blades Visible Chest" => None, //TODO
        "Shadow Temple Invisible Blades Invisible Chest" => None, //TODO
        "Shadow Temple Falling Spikes Lower Chest" => None, //TODO
        "Shadow Temple Falling Spikes Upper Chest" => None, //TODO
        "Shadow Temple Falling Spikes Switch Chest" => None, //TODO
        "Shadow Temple Invisible Spikes Chest" => None, //TODO
        "Shadow Temple Wind Hint Chest" => None, //TODO
        "Shadow Temple After Wind Enemy Chest" => None, //TODO
        "Shadow Temple After Wind Hidden Chest" => None, //TODO
        "Shadow Temple Spike Walls Left Chest" => None, //TODO
        "Shadow Temple Boss Key Chest" => None, //TODO
        "Shadow Temple Invisible Floormaster Chest" => None, //TODO
        "Shadow Temple Freestanding Key" => None, //TODO
        // Shadow Temple MQ
        "Shadow Temple MQ Compass Chest" => None, //TODO
        "Shadow Temple MQ Hover Boots Chest" => None, //TODO
        "Shadow Temple MQ Early Gibdos Chest" => None, //TODO
        "Shadow Temple MQ Map Chest" => None, //TODO
        "Shadow Temple MQ Beamos Silver Rupees Chest" => None, //TODO
        "Shadow Temple MQ Falling Spikes Switch Chest" => None, //TODO
        "Shadow Temple MQ Falling Spikes Lower Chest" => None, //TODO
        "Shadow Temple MQ Falling Spikes Upper Chest" => None, //TODO
        "Shadow Temple MQ Invisible Spikes Chest" => None, //TODO
        "Shadow Temple MQ Boss Key Chest" => None, //TODO
        "Shadow Temple MQ Spike Walls Left Chest" => None, //TODO
        "Shadow Temple MQ Stalfos Room Chest" => None, //TODO
        "Shadow Temple MQ Invisible Blades Invisible Chest" => None, //TODO
        "Shadow Temple MQ Invisible Blades Visible Chest" => None, //TODO
        "Shadow Temple MQ Bomb Flower Chest" => None, //TODO
        "Shadow Temple MQ Wind Hint Chest" => None, //TODO
        "Shadow Temple MQ After Wind Hidden Chest" => None, //TODO
        "Shadow Temple MQ After Wind Enemy Chest" => None, //TODO
        "Shadow Temple MQ Near Ship Invisible Chest" => None, //TODO
        "Shadow Temple MQ Freestanding Key" => None, //TODO

        // Bottom of the Well vanilla
        "Bottom of the Well Front Left Fake Wall Chest" => None, //TODO
        "Bottom of the Well Front Center Bombable Chest" => None, //TODO
        "Bottom of the Well Right Bottom Fake Wall Chest" => None, //TODO
        "Bottom of the Well Compass Chest" => None, //TODO
        "Bottom of the Well Center Skulltula Chest" => None, //TODO
        "Bottom of the Well Back Left Bombable Chest" => None, //TODO
        "Bottom of the Well Lens of Truth Chest" => None, //TODO
        "Bottom of the Well Invisible Chest" => None, //TODO
        "Bottom of the Well Underwater Front Chest" => None, //TODO
        "Bottom of the Well Underwater Left Chest" => None, //TODO
        "Bottom of the Well Map Chest" => None, //TODO
        "Bottom of the Well Fire Keese Chest" => None, //TODO
        "Bottom of the Well Like Like Chest" => None, //TODO
        "Bottom of the Well Freestanding Key" => None, //TODO
        // Bottom of the Well MQ
        "Bottom of the Well MQ Map Chest" => None, //TODO
        "Bottom of the Well MQ Lens of Truth Chest" => None, //TODO
        "Bottom of the Well MQ Compass Chest" => None, //TODO
        "Bottom of the Well MQ Dead Hand Freestanding Key" => None, //TODO
        "Bottom of the Well MQ East Inner Room Freestanding Key" => None, //TODO

        // Ice Cavern vanilla
        "Ice Cavern Map Chest" => None, //TODO
        "Ice Cavern Compass Chest" => None, //TODO
        "Ice Cavern Iron Boots Chest" => None, //TODO
        "Ice Cavern Freestanding PoH" => None, //TODO
        // Ice Cavern MQ
        "Ice Cavern MQ Iron Boots Chest" => None, //TODO
        "Ice Cavern MQ Compass Chest" => None, //TODO
        "Ice Cavern MQ Map Chest" => None, //TODO
        "Ice Cavern MQ Freestanding PoH" => None, //TODO

        // Gerudo Training Grounds vanilla
        "Gerudo Training Grounds Lobby Left Chest" => None, //TODO
        "Gerudo Training Grounds Lobby Right Chest" => None, //TODO
        "Gerudo Training Grounds Stalfos Chest" => None, //TODO
        "Gerudo Training Grounds Beamos Chest" => None, //TODO
        "Gerudo Training Grounds Hidden Ceiling Chest" => None, //TODO
        "Gerudo Training Grounds Maze Path First Chest" => None, //TODO
        "Gerudo Training Grounds Maze Path Second Chest" => None, //TODO
        "Gerudo Training Grounds Maze Path Third Chest" => None, //TODO
        "Gerudo Training Grounds Maze Path Final Chest" => None, //TODO
        "Gerudo Training Grounds Maze Right Central Chest" => None, //TODO
        "Gerudo Training Grounds Maze Right Side Chest" => None, //TODO
        "Gerudo Training Grounds Underwater Silver Rupee Chest" => None, //TODO
        "Gerudo Training Grounds Hammer Room Clear Chest" => None, //TODO
        "Gerudo Training Grounds Hammer Room Switch Chest" => None, //TODO
        "Gerudo Training Grounds Eye Statue Chest" => None, //TODO
        "Gerudo Training Grounds Near Scarecrow Chest" => None, //TODO
        "Gerudo Training Grounds Before Heavy Block Chest" => None, //TODO
        "Gerudo Training Grounds Heavy Block First Chest" => None, //TODO
        "Gerudo Training Grounds Heavy Block Second Chest" => None, //TODO
        "Gerudo Training Grounds Heavy Block Third Chest" => None, //TODO
        "Gerudo Training Grounds Heavy Block Fourth Chest" => None, //TODO
        "Gerudo Training Grounds Freestanding Key" => None, //TODO
        // Gerudo Training Grounds MQ                             
        "Gerudo Training Grounds MQ Lobby Right Chest" => None, //TODO
        "Gerudo Training Grounds MQ Lobby Left Chest" => None, //TODO
        "Gerudo Training Grounds MQ First Iron Knuckle Chest" => None, //TODO
        "Gerudo Training Grounds MQ Before Heavy Block Chest" => None, //TODO
        "Gerudo Training Grounds MQ Eye Statue Chest" => None, //TODO
        "Gerudo Training Grounds MQ Flame Circle Chest" => None, //TODO
        "Gerudo Training Grounds MQ Second Iron Knuckle Chest" => None, //TODO
        "Gerudo Training Grounds MQ Dinolfos Chest" => None, //TODO
        "Gerudo Training Grounds MQ Ice Arrows Chest" => None, //TODO
        "Gerudo Training Grounds MQ Maze Right Central Chest" => None, //TODO
        "Gerudo Training Grounds MQ Maze Path First Chest" => None, //TODO
        "Gerudo Training Grounds MQ Maze Right Side Chest" => None, //TODO
        "Gerudo Training Grounds MQ Maze Path Third Chest" => None, //TODO
        "Gerudo Training Grounds MQ Maze Path Second Chest" => None, //TODO
        "Gerudo Training Grounds MQ Hidden Ceiling Chest" => None, //TODO
        "Gerudo Training Grounds MQ Underwater Silver Rupee Chest" => None, //TODO
        "Gerudo Training Grounds MQ Heavy Block Chest" => None, //TODO

        // Ganon's Castle shared
        "Ganons Tower Boss Key Chest" => None, //TODO
        // Ganon's Castle vanilla                                
        "Ganons Castle Forest Trial Chest" => None, //TODO
        "Ganons Castle Water Trial Left Chest" => None, //TODO
        "Ganons Castle Water Trial Right Chest" => None, //TODO
        "Ganons Castle Shadow Trial Front Chest" => None, //TODO
        "Ganons Castle Shadow Trial Golden Gauntlets Chest" => None, //TODO
        "Ganons Castle Spirit Trial Crystal Switch Chest" => None, //TODO
        "Ganons Castle Spirit Trial Invisible Chest" => None, //TODO
        "Ganons Castle Light Trial First Left Chest" => None, //TODO
        "Ganons Castle Light Trial Second Left Chest" => None, //TODO
        "Ganons Castle Light Trial Third Left Chest" => None, //TODO
        "Ganons Castle Light Trial First Right Chest" => None, //TODO
        "Ganons Castle Light Trial Second Right Chest" => None, //TODO
        "Ganons Castle Light Trial Third Right Chest" => None, //TODO
        "Ganons Castle Light Trial Invisible Enemies Chest" => None, //TODO
        "Ganons Castle Light Trial Lullaby Chest" => None, //TODO
        // Ganon's Castle MQ
        "Ganons Castle MQ Water Trial Chest" => None, //TODO
        "Ganons Castle MQ Forest Trial Eye Switch Chest" => None, //TODO
        "Ganons Castle MQ Forest Trial Frozen Eye Switch Chest" => None, //TODO
        "Ganons Castle MQ Light Trial Lullaby Chest" => None, //TODO
        "Ganons Castle MQ Shadow Trial Bomb Flower Chest" => None, //TODO
        "Ganons Castle MQ Shadow Trial Eye Switch Chest" => None, //TODO
        "Ganons Castle MQ Spirit Trial Golden Gauntlets Chest" => None, //TODO
        "Ganons Castle MQ Spirit Trial Sun Back Right Chest" => None, //TODO
        "Ganons Castle MQ Spirit Trial Sun Back Left Chest" => None, //TODO
        "Ganons Castle MQ Spirit Trial Sun Front Left Chest" => None, //TODO
        "Ganons Castle MQ Spirit Trial First Chest" => None, //TODO
        "Ganons Castle MQ Spirit Trial Invisible Chest" => None, //TODO
        "Ganons Castle MQ Forest Trial Freestanding Key" => None, //TODO

        "Links Pocket" => None, //TODO check if vanilla or rando, if vanilla check for appropriate flag
        "Queen Gohma" => None, //TODO
        "King Dodongo" => None, //TODO
        "Barinade" => None, //TODO
        "Phantom Ganon" => None, //TODO
        "Volvagia" => None, //TODO
        "Morpha" => None, //TODO
        "Twinrova" => None, //TODO
        "Bongo Bongo" => None, //TODO
        "Ganon" => None, //TODO

        "Deku Tree Queen Gohma Heart" => None, //TODO
        "Dodongos Cavern King Dodongo Heart" => None, //TODO
        "Jabu Jabus Belly Barinade Heart" => None, //TODO
        "Forest Temple Phantom Ganon Heart" => None, //TODO
        "Fire Temple Volvagia Heart" => None, //TODO
        "Water Temple Morpha Heart" => None, //TODO
        "Spirit Temple Twinrova Heart" => None, //TODO
        "Shadow Temple Bongo Bongo Heart" => None, //TODO

        // Dungeon Skulls
        "Deku Tree GS Basement Back Room" => None, //TODO
        "Deku Tree GS Basement Gate" => None, //TODO
        "Deku Tree GS Basement Vines" => None, //TODO
        "Deku Tree GS Compass Room" => None, //TODO

        "Deku Tree MQ GS Lobby" => None, //TODO
        "Deku Tree MQ GS Compass Room" => None, //TODO
        "Deku Tree MQ GS Basement Graves Room" => None, //TODO
        "Deku Tree MQ GS Basement Back Room" => None, //TODO

        "Dodongos Cavern GS Vines Above Stairs" => None, //TODO
        "Dodongos Cavern GS Scarecrow" => None, //TODO
        "Dodongos Cavern GS Alcove Above Stairs" => None, //TODO
        "Dodongos Cavern GS Back Room" => None, //TODO
        "Dodongos Cavern GS Side Room Near Lower Lizalfos" => None, //TODO

        "Dodongos Cavern MQ GS Scrub Room" => None, //TODO
        "Dodongos Cavern MQ GS Song of Time Block Room" => None, //TODO
        "Dodongos Cavern MQ GS Lizalfos Room" => None, //TODO
        "Dodongos Cavern MQ GS Larvae Room" => None, //TODO
        "Dodongos Cavern MQ GS Back Area" => None, //TODO

        "Jabu Jabus Belly GS Lobby Basement Lower" => None, //TODO
        "Jabu Jabus Belly GS Lobby Basement Upper" => None, //TODO
        "Jabu Jabus Belly GS Near Boss" => None, //TODO
        "Jabu Jabus Belly GS Water Switch Room" => None, //TODO

        "Jabu Jabus Belly MQ GS Tailpasaran Room" => None, //TODO
        "Jabu Jabus Belly MQ GS Invisible Enemies Room" => None, //TODO
        "Jabu Jabus Belly MQ GS Boomerang Chest Room" => None, //TODO
        "Jabu Jabus Belly MQ GS Near Boss" => None, //TODO

        "Forest Temple GS Raised Island Courtyard" => None, //TODO
        "Forest Temple GS First Room" => None, //TODO
        "Forest Temple GS Level Island Courtyard" => None, //TODO
        "Forest Temple GS Lobby" => None, //TODO
        "Forest Temple GS Basement" => None, //TODO

        "Forest Temple MQ GS First Hallway" => None, //TODO
        "Forest Temple MQ GS Block Push Room" => None, //TODO
        "Forest Temple MQ GS Raised Island Courtyard" => None, //TODO
        "Forest Temple MQ GS Level Island Courtyard" => None, //TODO
        "Forest Temple MQ GS Well" => None, //TODO

        "Fire Temple GS Song of Time Room" => None, //TODO
        "Fire Temple GS Boss Key Loop" => None, //TODO
        "Fire Temple GS Boulder Maze" => None, //TODO
        "Fire Temple GS Scarecrow Top" => None, //TODO
        "Fire Temple GS Scarecrow Climb" => None, //TODO

        "Fire Temple MQ GS Above Fire Wall Maze" => None, //TODO
        "Fire Temple MQ GS Fire Wall Maze Center" => None, //TODO
        "Fire Temple MQ GS Big Lava Room Open Door" => None, //TODO
        "Fire Temple MQ GS Fire Wall Maze Side Room" => None, //TODO
        "Fire Temple MQ GS Skull On Fire" => None, //TODO

        "Water Temple GS Behind Gate" => None, //TODO
        "Water Temple GS Falling Platform Room" => None, //TODO
        "Water Temple GS Central Pillar" => None, //TODO
        "Water Temple GS Near Boss Key Chest" => None, //TODO
        "Water Temple GS River" => None, //TODO

        "Water Temple MQ GS Before Upper Water Switch" => None, //TODO
        "Water Temple MQ GS Freestanding Key Area" => None, //TODO
        "Water Temple MQ GS Lizalfos Hallway" => None, //TODO
        "Water Temple MQ GS River" => None, //TODO
        "Water Temple MQ GS Triple Wall Torch" => None, //TODO

        "Spirit Temple GS Hall After Sun Block Room" => None, //TODO
        "Spirit Temple GS Boulder Room" => None, //TODO
        "Spirit Temple GS Lobby" => None, //TODO
        "Spirit Temple GS Sun on Floor Room" => None, //TODO
        "Spirit Temple GS Metal Fence" => None, //TODO

        "Spirit Temple MQ GS Symphony Room" => None, //TODO
        "Spirit Temple MQ GS Leever Room" => None, //TODO
        "Spirit Temple MQ GS Nine Thrones Room West" => None, //TODO
        "Spirit Temple MQ GS Nine Thrones Room North" => None, //TODO
        "Spirit Temple MQ GS Sun Block Room" => None, //TODO

        "Shadow Temple GS Single Giant Pot" => None, //TODO
        "Shadow Temple GS Falling Spikes Room" => None, //TODO
        "Shadow Temple GS Triple Giant Pot" => None, //TODO
        "Shadow Temple GS Like Like Room" => None, //TODO
        "Shadow Temple GS Near Ship" => None, //TODO

        "Shadow Temple MQ GS Falling Spikes Room" => None, //TODO
        "Shadow Temple MQ GS Wind Hint Room" => None, //TODO
        "Shadow Temple MQ GS After Wind" => None, //TODO
        "Shadow Temple MQ GS After Ship" => None, //TODO
        "Shadow Temple MQ GS Near Boss" => None, //TODO

        // Mini Dungeon Skulls
        "Bottom of the Well GS Like Like Cage" => None, //TODO
        "Bottom of the Well GS East Inner Room" => None, //TODO
        "Bottom of the Well GS West Inner Room" => None, //TODO

        "Bottom of the Well MQ GS Basement" => None, //TODO
        "Bottom of the Well MQ GS Coffin Room" => None, //TODO
        "Bottom of the Well MQ GS West Inner Room" => None, //TODO

        "Ice Cavern GS Push Block Room" => None, //TODO
        "Ice Cavern GS Spinning Scythe Room" => None, //TODO
        "Ice Cavern GS Heart Piece Room" => None, //TODO

        "Ice Cavern MQ GS Scarecrow" => None, //TODO
        "Ice Cavern MQ GS Ice Block" => None, //TODO
        "Ice Cavern MQ GS Red Ice" => None, //TODO

        // Overworld Skulls
        "HF GS Cow Grotto" => None, //TODO
        "HF GS Near Kak Grotto" => None, //TODO

        "LLR GS Back Wall" => None, //TODO
        "LLR GS Rain Shed" => None, //TODO
        "LLR GS House Window" => None, //TODO
        "LLR GS Tree" => None, //TODO

        "KF GS Bean Patch" => None, //TODO
        "KF GS Know It All House" => None, //TODO
        "KF GS House of Twins" => None, //TODO

        "LW GS Bean Patch Near Bridge" => None, //TODO
        "LW GS Bean Patch Near Theater" => None, //TODO
        "LW GS Above Theater" => None, //TODO
        "SFM GS" => None, //TODO

        "OGC GS" => None, //TODO
        "HC GS Storms Grotto" => None, //TODO
        "HC GS Tree" => None, //TODO
        "Market GS Guard House" => None, //TODO

        "DMC GS Bean Patch" => None, //TODO
        "DMC GS Crate" => None, //TODO

        "DMT GS Bean Patch" => None, //TODO
        "DMT GS Near Kak" => None, //TODO
        "DMT GS Above Dodongos Cavern" => None, //TODO
        "DMT GS Falling Rocks Path" => None, //TODO

        "GC GS Center Platform" => None, //TODO
        "GC GS Boulder Maze" => None, //TODO
        
        "Kak GS House Under Construction" => None, //TODO
        "Kak GS Skulltula House" => None, //TODO
        "Kak GS Guards House" => None, //TODO
        "Kak GS Tree" => None, //TODO
        "Kak GS Watchtower" => None, //TODO
        "Kak GS Above Impas House" => None, //TODO

        "Graveyard GS Wall" => None, //TODO
        "Graveyard GS Bean Patch" => None, //TODO

        "ZR GS Ladder" => None, //TODO
        "ZR GS Tree" => None, //TODO
        "ZR GS Above Bridge" => None, //TODO
        "ZR GS Near Raised Grottos" => None, //TODO

        "ZD GS Frozen Waterfall" => None, //TODO
        "ZF GS Above the Log" => None, //TODO
        "ZF GS Hidden Cave" => None, //TODO
        "ZF GS Tree" => None, //TODO

        "LH GS Bean Patch" => None, //TODO
        "LH GS Small Island" => None, //TODO
        "LH GS Lab Wall" => None, //TODO
        "LH GS Lab Crate" => None, //TODO
        "LH GS Tree" => None, //TODO

        "GV GS Bean Patch" => None, //TODO
        "GV GS Small Bridge" => None, //TODO
        "GV GS Pillar" => None, //TODO
        "GV GS Behind Tent" => None, //TODO

        "GF GS Archery Range" => None, //TODO
        "GF GS Top Floor" => None, //TODO

        "Wasteland GS" => None, //TODO
        "Colossus GS Bean Patch" => None, //TODO
        "Colossus GS Hill" => None, //TODO
        "Colossus GS Tree" => None, //TODO

        // Shops
        "KF Shop Item 1" => None, //TODO
        "KF Shop Item 2" => None, //TODO
        "KF Shop Item 3" => None, //TODO
        "KF Shop Item 4" => None, //TODO
        "KF Shop Item 5" => None, //TODO
        "KF Shop Item 6" => None, //TODO
        "KF Shop Item 7" => None, //TODO
        "KF Shop Item 8" => None, //TODO

        "Kak Potion Shop Item 1" => None, //TODO
        "Kak Potion Shop Item 2" => None, //TODO
        "Kak Potion Shop Item 3" => None, //TODO
        "Kak Potion Shop Item 4" => None, //TODO
        "Kak Potion Shop Item 5" => None, //TODO
        "Kak Potion Shop Item 6" => None, //TODO
        "Kak Potion Shop Item 7" => None, //TODO
        "Kak Potion Shop Item 8" => None, //TODO

        "Market Bombchu Shop Item 1" => None, //TODO
        "Market Bombchu Shop Item 2" => None, //TODO
        "Market Bombchu Shop Item 3" => None, //TODO
        "Market Bombchu Shop Item 4" => None, //TODO
        "Market Bombchu Shop Item 5" => None, //TODO
        "Market Bombchu Shop Item 6" => None, //TODO
        "Market Bombchu Shop Item 7" => None, //TODO
        "Market Bombchu Shop Item 8" => None, //TODO

        "Market Potion Shop Item 1" => None, //TODO
        "Market Potion Shop Item 2" => None, //TODO
        "Market Potion Shop Item 3" => None, //TODO
        "Market Potion Shop Item 4" => None, //TODO
        "Market Potion Shop Item 5" => None, //TODO
        "Market Potion Shop Item 6" => None, //TODO
        "Market Potion Shop Item 7" => None, //TODO
        "Market Potion Shop Item 8" => None, //TODO

        "Market Bazaar Item 1" => None, //TODO
        "Market Bazaar Item 2" => None, //TODO
        "Market Bazaar Item 3" => None, //TODO
        "Market Bazaar Item 4" => None, //TODO
        "Market Bazaar Item 5" => None, //TODO
        "Market Bazaar Item 6" => None, //TODO
        "Market Bazaar Item 7" => None, //TODO
        "Market Bazaar Item 8" => None, //TODO

        "Kak Bazaar Item 1" => None, //TODO
        "Kak Bazaar Item 2" => None, //TODO
        "Kak Bazaar Item 3" => None, //TODO
        "Kak Bazaar Item 4" => None, //TODO
        "Kak Bazaar Item 5" => None, //TODO
        "Kak Bazaar Item 6" => None, //TODO
        "Kak Bazaar Item 7" => None, //TODO
        "Kak Bazaar Item 8" => None, //TODO

        "ZD Shop Item 1" => None, //TODO
        "ZD Shop Item 2" => None, //TODO
        "ZD Shop Item 3" => None, //TODO
        "ZD Shop Item 4" => None, //TODO
        "ZD Shop Item 5" => None, //TODO
        "ZD Shop Item 6" => None, //TODO
        "ZD Shop Item 7" => None, //TODO
        "ZD Shop Item 8" => None, //TODO

        "GC Shop Item 1" => None, //TODO
        "GC Shop Item 2" => None, //TODO
        "GC Shop Item 3" => None, //TODO
        "GC Shop Item 4" => None, //TODO
        "GC Shop Item 5" => None, //TODO
        "GC Shop Item 6" => None, //TODO
        "GC Shop Item 7" => None, //TODO
        "GC Shop Item 8" => None, //TODO

        // NPC Scrubs are on the overworld, while GrottoNPC is a special handler for Grottos
        // Grottos scrubs are the same scene and actor, so we use a unique grotto ID for the scene

        "Deku Tree MQ Deku Scrub" => None, //TODO

        "HF Deku Scrub Grotto" => None, //TODO
        "LLR Deku Scrub Grotto Left" => None, //TODO
        "LLR Deku Scrub Grotto Right" => None, //TODO
        "LLR Deku Scrub Grotto Center" => None, //TODO

        "LW Deku Scrub Near Deku Theater Right" => None, //TODO
        "LW Deku Scrub Near Deku Theater Left" => None, //TODO
        "LW Deku Scrub Near Bridge" => None, //TODO
        "LW Deku Scrub Grotto Rear" => None, //TODO
        "LW Deku Scrub Grotto Front" => None, //TODO

        "SFM Deku Scrub Grotto Rear" => None, //TODO
        "SFM Deku Scrub Grotto Front" => None, //TODO

        "GC Deku Scrub Grotto Left" => None, //TODO
        "GC Deku Scrub Grotto Right" => None, //TODO
        "GC Deku Scrub Grotto Center" => None, //TODO
        
        "Dodongos Cavern Deku Scrub Near Bomb Bag Left" => None, //TODO
        "Dodongos Cavern Deku Scrub Side Room Near Dodongos" => None, //TODO
        "Dodongos Cavern Deku Scrub Near Bomb Bag Right" => None, //TODO
        "Dodongos Cavern Deku Scrub Lobby" => None, //TODO

        "Dodongos Cavern MQ Deku Scrub Lobby Rear" => None, //TODO
        "Dodongos Cavern MQ Deku Scrub Lobby Front" => None, //TODO
        "Dodongos Cavern MQ Deku Scrub Staircase" => None, //TODO
        "Dodongos Cavern MQ Deku Scrub Side Room Near Lower Lizalfos" => None, //TODO

        "DMC Deku Scrub" => None, //TODO
        "DMC Deku Scrub Grotto Left" => None, //TODO
        "DMC Deku Scrub Grotto Right" => None, //TODO
        "DMC Deku Scrub Grotto Center" => None, //TODO

        "ZR Deku Scrub Grotto Rear" => None, //TODO
        "ZR Deku Scrub Grotto Front" => None, //TODO

        "Jabu Jabus Belly Deku Scrub" => None, //TODO

        "LH Deku Scrub Grotto Left" => None, //TODO
        "LH Deku Scrub Grotto Right" => None, //TODO
        "LH Deku Scrub Grotto Center" => None, //TODO

        "GV Deku Scrub Grotto Rear" => None, //TODO
        "GV Deku Scrub Grotto Front" => None, //TODO

        "Colossus Deku Scrub Grotto Rear" => None, //TODO
        "Colossus Deku Scrub Grotto Front" => None, //TODO

        "Ganons Castle Deku Scrub Center-Left" => None, //TODO
        "Ganons Castle Deku Scrub Center-Right" => None, //TODO
        "Ganons Castle Deku Scrub Right" => None, //TODO
        "Ganons Castle Deku Scrub Left" => None, //TODO

        "Ganons Castle MQ Deku Scrub Right" => None, //TODO
        "Ganons Castle MQ Deku Scrub Center-Left" => None, //TODO
        "Ganons Castle MQ Deku Scrub Center" => None, //TODO
        "Ganons Castle MQ Deku Scrub Center-Right" => None, //TODO
        "Ganons Castle MQ Deku Scrub Left" => None, //TODO

        "LLR Stables Left Cow" => None, //TODO
        "LLR Stables Right Cow" => None, //TODO
        "LLR Tower Right Cow" => None, //TODO
        "LLR Tower Left Cow" => None, //TODO
        "KF Links House Cow" => None, //TODO
        "Kak Impas House Cow" => None, //TODO
        "GV Cow" => None, //TODO
        "DMT Cow Grotto Cow" => None, //TODO
        "HF Cow Grotto Cow" => None, //TODO

        // These are not actual locations, but are filler spots used for hint reachability
        "DMC Gossip Stone" => None, //TODO
        "DMT Gossip Stone" => None, //TODO
        "Colossus Gossip Stone" => None, //TODO
        "Dodongos Cavern Gossip Stone" => None, //TODO
        "GV Gossip Stone" => None, //TODO
        "GC Maze Gossip Stone" => None, //TODO
        "GC Medigoron Gossip Stone" => None, //TODO
        "Graveyard Gossip Stone" => None, //TODO
        "HC Malon Gossip Stone" => None, //TODO
        "HC Rock Wall Gossip Stone" => None, //TODO
        "HC Storms Grotto Gossip Stone" => None, //TODO
        "HF Cow Grotto Gossip Stone" => None, //TODO
        "KF Deku Tree Gossip Stone (Left)" => None, //TODO
        "KF Deku Tree Gossip Stone (Right)" => None, //TODO
        "KF Gossip Stone" => None, //TODO
        "LH Lab Gossip Stone" => None, //TODO
        "LH Gossip Stone (Southeast)" => None, //TODO
        "LH Gossip Stone (Southwest)" => None, //TODO
        "LW Gossip Stone" => None, //TODO
        "SFM Maze Gossip Stone (Lower)" => None, //TODO
        "SFM Maze Gossip Stone (Upper)" => None, //TODO
        "SFM Saria Gossip Stone" => None, //TODO
        "ToT Gossip Stone (Left)" => None, //TODO
        "ToT Gossip Stone (Left-Center)" => None, //TODO
        "ToT Gossip Stone (Right)" => None, //TODO
        "ToT Gossip Stone (Right-Center)" => None, //TODO
        "ZD Gossip Stone" => None, //TODO
        "ZF Fairy Gossip Stone" => None, //TODO
        "ZF Jabu Gossip Stone" => None, //TODO
        "ZR Near Grottos Gossip Stone" => None, //TODO
        "ZR Near Domain Gossip Stone" => None, //TODO

        "HF Near Market Grotto Gossip Stone" => None, //TODO
        "HF Southeast Grotto Gossip Stone" => None, //TODO
        "HF Open Grotto Gossip Stone" => None, //TODO
        "Kak Open Grotto Gossip Stone" => None, //TODO
        "ZR Open Grotto Gossip Stone" => None, //TODO
        "KF Storms Grotto Gossip Stone" => None, //TODO
        "LW Near Shortcuts Grotto Gossip Stone" => None, //TODO
        "DMT Storms Grotto Gossip Stone" => None, //TODO
        "DMC Upper Grotto Gossip Stone" => None, //TODO

        "Ganondorf Hint" => None, //TODO

        _ => panic!("unknown check name: {}", loc),
    }
}
