use {
    std::{
        borrow::Cow,
        collections::hash_map::{
            self,
            HashMap,
        },
    },
    collect_mac::collect,
    pyo3::{
        AsPyPointer as _,
        types::{
            PyBool,
            PyDict,
            PyInt,
            PyList,
            PyString,
            PyType,
        },
    },
    ootr::settings::{
        KnowledgeType,
        KnowledgeTypeError,
        KnowledgeValue
    },
    crate::PyResultExt as _,
};

pub struct Knowledge(pub HashMap<Cow<'static, str>, KnowledgeValue>);

impl<'p> ootr::settings::Knowledge<crate::Rando<'p>> for Knowledge {
    fn default(rando: &crate::Rando<'p>) -> Result<Self, crate::RandoErr> {
        if rando.setting_infos.borrow().is_none() {
            let settings_list = rando.py.import("SettingsList").at("SettingsList")?;
            let mut settings = HashMap::default();
            for setting in settings_list.getattr("setting_infos").at("setting_infos")?.iter().at("setting_infos")? {
                let setting = setting.at("setting_infos")?;
                let name = setting.getattr("name").at("name")?.extract().at("name")?;
                if matches!(name, "bombchu_trail_color_inner" | "mirror_shield_frame_color") // workaround //TODO remove when https://github.com/TestRunnerSRL/OoT-Randomizer/pull/1394 and https://github.com/TestRunnerSRL/OoT-Randomizer/pull/1395 are merged into Dev-R
                || setting.getattr("cosmetic").at("cosmetic")?.extract().at("cosmetic")? { continue } // ignore cosmetic settings for now //TODO use to style items on the GUI?
                if ignore_setting(name) { continue }
                let setting_type = setting.getattr("type").at("type")?;
                settings.insert(Cow::Owned(name.to_owned()), if settings_list.getattr("Combobox").at("Combobox")?.downcast::<PyType>()?.is_instance(setting).at("Combobox")? {
                    KnowledgeValue::String(
                        setting.getattr("choice_list").at("choice_list")?.iter().at("choice_list")?
                            .map(|choice_res| choice_res.and_then(|choice| Ok(Cow::Owned(choice.extract::<String>()?))))
                            .collect::<Result<_, _>>().at("choice_list")?
                    )
                } else if setting_type.as_ptr() == rando.py.get_type::<PyBool>().as_ptr() {
                    KnowledgeValue::Bool(None)
                } else if setting_type.as_ptr() == rando.py.get_type::<PyInt>().as_ptr() {
                    KnowledgeValue::Int(setting.getattr("gui_params").at("gui_params")?.downcast::<PyDict>()?.get_item("min").ok_or(crate::RandoErr::MissingIntSettingBound)?.extract().at("min")?..=setting.getattr("gui_params").at("gui_params")?.downcast::<PyDict>()?.get_item("max").ok_or(crate::RandoErr::MissingIntSettingBound)?.extract().at("max")?)
                } else if setting_type.as_ptr() == rando.py.get_type::<PyString>().as_ptr() {
                    panic!("unknown non-cosmetic string setting: {}", name)
                } else if setting_type.as_ptr() == rando.py.get_type::<PyList>().as_ptr() {
                    KnowledgeValue::List(HashMap::default())
                } else if setting_type.as_ptr() == rando.py.get_type::<PyDict>().as_ptr() {
                    unimplemented!() //TODO hint_dist_user
                } else {
                    panic!("unknown setting type {} for setting {}", setting_type, name)
                });
            }
            *rando.setting_infos.borrow_mut() = Some(settings);
        }
        Ok(Self(rando.setting_infos.borrow().as_ref().expect("just inserted").clone()))
    }

    fn vanilla(_: &crate::Rando<'p>) -> Self {
        Self(collect![
            Cow::Borrowed("world_count") => KnowledgeValue::Int(1..=1),
            Cow::Borrowed("player_num") => KnowledgeValue::Int(1..=1),
            Cow::Borrowed("randomize_settings") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("open_forest") => KnowledgeValue::String(collect![Cow::Borrowed("closed")]),
            Cow::Borrowed("open_kakariko") => KnowledgeValue::String(collect![Cow::Borrowed("closed")]),
            Cow::Borrowed("open_door_of_time") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("zora_fountain") => KnowledgeValue::String(collect![Cow::Borrowed("closed")]),
            Cow::Borrowed("gerudo_fortress") => KnowledgeValue::String(collect![Cow::Borrowed("normal")]),
            Cow::Borrowed("bridge") => KnowledgeValue::String(collect![Cow::Borrowed("vanilla")]),
            Cow::Borrowed("bridge_medallions") => KnowledgeValue::Int(6..=6),
            Cow::Borrowed("bridge_stones") => KnowledgeValue::Int(3..=3),
            Cow::Borrowed("bridge_rewards") => KnowledgeValue::Int(9..=9),
            Cow::Borrowed("bridge_tokens") => KnowledgeValue::Int(100..=100),
            Cow::Borrowed("triforce_hunt") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("triforce_goal_per_world") => KnowledgeValue::Int(20..=20),
            Cow::Borrowed("logic_rules") => KnowledgeValue::String(collect![Cow::Borrowed("glitchless")]),
            Cow::Borrowed("reachable_locations") => KnowledgeValue::String(collect![Cow::Borrowed("all")]),
            Cow::Borrowed("bombchus_in_logic") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("one_item_per_dungeon") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("trials_random") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("trials") => KnowledgeValue::Int(6..=6),
            Cow::Borrowed("skip_child_zelda") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("no_escape_sequence") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("no_guard_stealth") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("no_epona_race") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("skip_some_minigame_phases") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("useful_cutscenes") => KnowledgeValue::Bool(Some(true)),
            Cow::Borrowed("complete_mask_quest") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("fast_chests") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("logic_no_night_tokens_without_suns_song") => KnowledgeValue::Bool(Some(false)),

            Cow::Borrowed("big_poe_count") => KnowledgeValue::Int(10..=10),
            Cow::Borrowed("starting_hearts") => KnowledgeValue::Int(3..=3),
            Cow::Borrowed("free_scarecrow") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("fast_bunny_hood") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("start_with_rupees") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("start_with_consumables") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("chicken_count_random") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("big_poe_count_random") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("shuffle_kokiri_sword") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("shuffle_ocarinas") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("shuffle_weird_egg") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("shuffle_gerudo_card") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("shuffle_cows") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("shuffle_beans") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("shuffle_medigoron_carpet_salesman") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("shuffle_grotto_entrances") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("shuffle_dungeon_entrances") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("shuffle_overworld_entrances") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("decouple_entrances") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("owl_drops") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("warp_songs") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("spawn_positions") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("enhance_map_compass") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("mq_dungeons_random") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("ocarina_songs") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("correct_chest_sizes") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("no_collectible_hearts") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("shuffle_song_items") => KnowledgeValue::String(collect![Cow::Borrowed("song")]),
            Cow::Borrowed("shuffle_interior_entrances") => KnowledgeValue::String(collect![Cow::Borrowed("off")]),
            Cow::Borrowed("mix_entrance_pools") => KnowledgeValue::String(collect![Cow::Borrowed("off")]),
            Cow::Borrowed("shuffle_scrubs") => KnowledgeValue::String(collect![Cow::Borrowed("off")]),
            Cow::Borrowed("shopsanity") => KnowledgeValue::String(collect![Cow::Borrowed("off")]),
            Cow::Borrowed("tokensanity") => KnowledgeValue::String(collect![Cow::Borrowed("off")]),
            Cow::Borrowed("shuffle_mapcompass") => KnowledgeValue::String(collect![Cow::Borrowed("vanilla")]),
            Cow::Borrowed("shuffle_smallkeys") => KnowledgeValue::String(collect![Cow::Borrowed("any_dungeon")]), // logically account for the lock on the Fire Temple boss key loop
            Cow::Borrowed("shuffle_hideoutkeys") => KnowledgeValue::String(collect![Cow::Borrowed("vanilla")]),
            Cow::Borrowed("shuffle_bosskeys") => KnowledgeValue::String(collect![Cow::Borrowed("vanilla")]),
            Cow::Borrowed("shuffle_ganon_bosskey") => KnowledgeValue::String(collect![Cow::Borrowed("vanilla")]),
            Cow::Borrowed("logic_earliest_adult_trade") => KnowledgeValue::String(collect![Cow::Borrowed("pocket_egg")]),
            Cow::Borrowed("logic_latest_adult_trade") => KnowledgeValue::String(collect![Cow::Borrowed("pocket_egg")]),
            Cow::Borrowed("hints") => KnowledgeValue::String(collect![Cow::Borrowed("none")]),
            Cow::Borrowed("misc_hints") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("text_shuffle") => KnowledgeValue::String(collect![Cow::Borrowed("none")]),
            Cow::Borrowed("junk_ice_traps") => KnowledgeValue::String(collect![Cow::Borrowed("normal")]),
            Cow::Borrowed("item_pool_value") => KnowledgeValue::String(collect![Cow::Borrowed("balanced")]),
            Cow::Borrowed("damage_multiplier") => KnowledgeValue::String(collect![Cow::Borrowed("normal")]),
            Cow::Borrowed("starting_tod") => KnowledgeValue::String(collect![Cow::Borrowed("default")]),
            Cow::Borrowed("starting_items") => KnowledgeValue::List(HashMap::default()), //TODO set all to false
            Cow::Borrowed("starting_equipment") => KnowledgeValue::List(HashMap::default()), //TODO set all to false
            Cow::Borrowed("starting_songs") => KnowledgeValue::List(HashMap::default()), //TODO set all to false
            Cow::Borrowed("disabled_locations") => KnowledgeValue::List(HashMap::default()), //TODO set all to false
            Cow::Borrowed("mq_dungeons") => KnowledgeValue::Int(0..=0),
            Cow::Borrowed("lacs_condition") => KnowledgeValue::String(collect![Cow::Borrowed("vanilla")]),
            Cow::Borrowed("starting_age") => KnowledgeValue::String(collect![Cow::Borrowed("child")]),
            Cow::Borrowed("allowed_tricks") => KnowledgeValue::List(collect![
                Cow::Borrowed("logic_vanilla_water_temple") => true,
                Cow::Borrowed("logic_vanilla_spirit_temple") => true,
            ]), //TODO set all others to false
            Cow::Borrowed("ice_trap_appearance") => KnowledgeValue::String(collect![Cow::Borrowed("junk_only")]),
            Cow::Borrowed("hint_dist") => KnowledgeValue::String(collect![Cow::Borrowed("useless")]),
            Cow::Borrowed("clearer_hints") => KnowledgeValue::Bool(Some(false)),
            Cow::Borrowed("lacs_medallions") => KnowledgeValue::Int(6..=6),
            Cow::Borrowed("lacs_stones") => KnowledgeValue::Int(3..=3),
            Cow::Borrowed("lacs_rewards") => KnowledgeValue::Int(9..=9),
            Cow::Borrowed("lacs_tokens") => KnowledgeValue::Int(100..=100),
            Cow::Borrowed("ganon_bosskey_medallions") => KnowledgeValue::Int(6..=6),
            Cow::Borrowed("ganon_bosskey_stones") => KnowledgeValue::Int(3..=3),
            Cow::Borrowed("ganon_bosskey_rewards") => KnowledgeValue::Int(9..=9),
            Cow::Borrowed("ganon_bosskey_tokens") => KnowledgeValue::Int(100..=100),
            Cow::Borrowed("chicken_count") => KnowledgeValue::Int(7..=7),
            //TODO other settings
        ])
    }

    fn get<T: KnowledgeType>(&self, setting: &str) -> Result<Option<T>, KnowledgeTypeError> {
        Ok(if let Some(val) = self.0.get(setting) {
            Some(T::from_any(val)?)
        } else {
            None
        })
    }

    fn update<T: KnowledgeType>(&mut self, setting: &str, value: T) -> Result<(), KnowledgeTypeError> {
        match self.0.entry(Cow::Owned(setting.to_owned())) {
            hash_map::Entry::Occupied(mut entry) => { entry.insert((entry.get().clone() & value.into_any())?); }
            hash_map::Entry::Vacant(entry) => { entry.insert(value.into_any()); }
        }
        Ok(())
    }

    fn remove(&mut self, setting: &str) {
        self.0.remove(setting);
    }
}

pub(crate) fn ignore_setting(name: &str) -> bool {
    match name {
        "web_wad_file"
        | "web_common_key_file"
        | "web_common_key_string"
        | "web_wad_channel_id"
        | "web_wad_channel_title"
        | "web_output_type"
        | "web_persist_in_cache"
        | "cosmetics_only"
        | "check_version"
        | "output_settings"
        | "generate_from_file"
        | "enable_distribution_file"
        | "enable_cosmetic_file"
        | "distribution_file"
        | "cosmetic_file"
        | "checked_version"
        | "rom"
        | "output_dir"
        | "output_file"
        | "seed"
        | "patch_file"
        | "count"
        | "presets"
        | "open_output_dir"
        | "open_python_dir"
        | "repatch_cosmetics"
        | "create_spoiler"
        | "create_cosmetics_log"
        | "compress_rom"
        | "tricks_list_msg" => true, // ignore patching and GUI infrastructure
        "bingosync_url"
        | "item_hints" => true, // ignoring bingo stuff for now //TODO use for hints knowledge and routing goal?
        "hint_dist_user" => true, //TODO handle settings with no display name, handle hint_dist_user structure
        _ => false
    }
}
