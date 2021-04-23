use {
    std::{
        any::TypeId,
        collections::{
            BTreeMap,
            hash_map::DefaultHasher,
        },
        convert::{
            TryFrom as _,
            TryInto as _,
        },
        fmt,
        hash::{
            Hash,
            Hasher,
        },
        iter,
        pin::Pin,
        sync::Arc,
    },
    async_stream::try_stream,
    collect_mac::collect,
    futures::stream::{
        Stream,
        TryStreamExt as _,
    },
    pin_utils::pin_mut,
    serde::{
        Deserialize,
        de::DeserializeOwned,
        ser::Serialize,
    },
    serde_json::{
        json,
        Value as Json,
    },
    tokio::sync::Mutex,
    wheel::FromArc,
    ootr::{
        check::Check,
        model::{
            DungeonReward,
            DungeonRewardLocation,
            MainDungeon,
        },
        region::Mq,
    },
    crate::{
        ModelState,
        checks::CheckExt as _,
        ui::{
            TrackerCellId,
            TrackerCellKind::{
                self,
                *,
            },
        },
    },
};

// to obtain a Firebase web tracker's API key, open a room in the tracker and copy the element `apiKey` from the local storage entry starting with `firebase:authUser`.
include!("../../../assets/firebase-api-keys.rs");

trait TrackerCellKindExt {
    fn render(&self, state: &ModelState) -> Json;
    fn set(&self, state: &mut ModelState, value: Json) -> Result<(), Json>;
}

impl TrackerCellKindExt for TrackerCellKind {
    fn render(&self, state: &ModelState) -> Json {
        match self {
            BossKey { active, .. } => json!(active(&state.ram.save.boss_keys)),
            Composite { active, .. } => json!(match active(state) {
                (false, false) => 0,
                (true, false) => 1,
                (false, true) => 2,
                (true, true) => 3,
            }),
            Count { get, max, step, .. } => json!(get(state).min(*max) / step),
            FortressMq => json!(state.knowledge.string_settings.get("gerudo_fortress").map_or(false, |values| values.iter().eq(iter::once("normal")))),
            Medallion(med) => json!(state.ram.save.quest_items.has(med)),
            MedallionLocation(med) => json!(match state.knowledge.dungeon_reward_locations.get(&DungeonReward::Medallion(*med)) {
                None => 0,
                Some(DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)) => 1,
                Some(DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)) => 2,
                Some(DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)) => 3,
                Some(DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)) => 4,
                Some(DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)) => 5,
                Some(DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)) => 6,
                Some(DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)) => 7,
                Some(DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)) => 8,
                Some(DungeonRewardLocation::LinksPocket) => 9,
            }),
            Mq(dungeon) => json!(state.knowledge.mq.get(dungeon) == Some(&Mq::Mq)),
            OptionalOverlay { active, .. } | Overlay { active, .. } => json!(active(state).0),
            Sequence { idx, .. } => json!(idx(state)),
            Simple { active, .. } => json!(active(state)),
            SmallKeys { get, .. } => json!(get(&state.ram.save.small_keys)),
            Song { song, .. } => json!(state.ram.save.quest_items.contains(*song)),
            SongCheck { check, .. } => json!(Check::<ootr_static::Rando>::Location(check.to_string()).checked(state).unwrap_or(false)), //TODO allow ootr_dynamic::Rando
            Stone(stone) => json!(state.ram.save.quest_items.has(stone)),
            StoneLocation(stone) => json!(match state.knowledge.dungeon_reward_locations.get(&DungeonReward::Stone(*stone)) {
                None => 0,
                Some(DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)) => 1,
                Some(DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)) => 2,
                Some(DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)) => 3,
                Some(DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)) => 4,
                Some(DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)) => 5,
                Some(DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)) => 6,
                Some(DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)) => 7,
                Some(DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)) => 8,
                Some(DungeonRewardLocation::LinksPocket) => 9,
            }),
            BigPoeTriforce | CompositeKeys { .. } | FreeReward => unimplemented!(),
        }
    }

    fn set(&self, state: &mut ModelState, value: Json) -> Result<(), Json> {
        match self {
            BossKey { active, toggle } => if active(&state.ram.save.boss_keys) != value.as_bool().ok_or_else(|| value.clone())? {
                toggle(&mut state.ram.save.boss_keys)
            },
            Composite { active, toggle_left, toggle_right, .. } => {
                let (active_left, active_right) = active(state);
                let (value_left, value_right) = match value.as_u64().ok_or_else(|| value.clone())? {
                    0 => (false, false),
                    1 => (true, false),
                    2 => (false, true),
                    3 => (true, true),
                    _ => return Err(value),
                };
                if active_left != value_left { toggle_left(state) }
                if active_right != value_right { toggle_right(state) }
            }
            Count { get, set, max, step, .. } => {
                let value = u8::try_from(value.as_u64().ok_or_else(|| value.clone())?).map_err(|_| value)?;
                // only update if the local value doesn't fit into the window received
                // so that e.g. decrementing skulls from 40 to 39 doesn't immediately set them to 30
                if get(state).min(*max) / step != value {
                    set(state, value * step);
                }
            }
            FortressMq => if value.as_bool().ok_or_else(|| value.clone())? {
                state.knowledge.string_settings.insert(format!("gerudo_fortress"), collect![format!("normal")]);
            } else {
                // don't override local state that's consistent with the value received
                if state.knowledge.string_settings.get("gerudo_fortress").map_or(false, |fort| fort.iter().eq(iter::once("normal"))) {
                    state.knowledge.string_settings.remove("gerudo_fortress");
                }
            },
            Medallion(med) => if value.as_bool().ok_or_else(|| value.clone())? {
                state.ram.save.quest_items.insert(med.into());
            } else {
                state.ram.save.quest_items.remove(med.into());
            },
            MedallionLocation(med) => {
                match value.as_u64().ok_or_else(|| value.clone())? {
                    0 => state.knowledge.dungeon_reward_locations.remove(&DungeonReward::Medallion(*med)),
                    1 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Medallion(*med), DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)),
                    2 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Medallion(*med), DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)),
                    3 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Medallion(*med), DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)),
                    4 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Medallion(*med), DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)),
                    5 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Medallion(*med), DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)),
                    6 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Medallion(*med), DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)),
                    7 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Medallion(*med), DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)),
                    8 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Medallion(*med), DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)),
                    9 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Medallion(*med), DungeonRewardLocation::LinksPocket),
                    _ => return Err(value),
                };
            }
            Mq(dungeon) => if value.as_bool().ok_or_else(|| value.clone())? {
                state.knowledge.mq.insert(*dungeon, Mq::Mq);
            } else {
                // don't override local state that's consistent with the value received
                if state.knowledge.mq.get(dungeon).map_or(false, |&mq| mq == Mq::Mq) {
                    state.knowledge.mq.remove(dungeon);
                }
            },
            OptionalOverlay { active, toggle_main, .. } | Overlay { active, toggle_main, .. } => if active(state).0 != value.as_bool().ok_or_else(|| value.clone())? {
                toggle_main(state);
            },
            Sequence { idx, increment, decrement, .. } => {
                let mut old_idx = idx(state);
                let new_idx = value.as_u64().ok_or_else(|| value.clone())?.try_into().map_err(|_| value.clone())?;
                while old_idx < new_idx { increment(state); old_idx += 1 }
                while old_idx > new_idx { decrement(state); old_idx -= 1 }
            }
            Simple { active, toggle, .. } => if active(state) != value.as_bool().ok_or_else(|| value.clone())? {
                toggle(state);
            },
            SmallKeys { set, .. } => set(&mut state.ram.save.small_keys, value.as_u64().ok_or_else(|| value.clone())?.try_into().map_err(|_| value.clone())?),
            Song { song, .. } => if value.as_bool().ok_or(value)? {
                state.ram.save.quest_items.insert(*song);
            } else {
                state.ram.save.quest_items.remove(*song);
            },
            SongCheck { check, toggle_overlay } => if Check::<ootr_static::Rando>::Location(check.to_string()).checked(state).unwrap_or(false) != value.as_bool().ok_or_else(|| value.clone())? { //TODO allow ootr_dynamic::Rando
                toggle_overlay(&mut state.ram.save.event_chk_inf);
            },
            Stone(stone) => if value.as_bool().ok_or_else(|| value.clone())? {
                state.ram.save.quest_items.insert(stone.into());
            } else {
                state.ram.save.quest_items.remove(stone.into());
            },
            StoneLocation(stone) => {
                match value.as_u64().ok_or_else(|| value.clone())? {
                    0 => state.knowledge.dungeon_reward_locations.remove(&DungeonReward::Stone(*stone)),
                    1 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Stone(*stone), DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)),
                    2 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Stone(*stone), DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)),
                    3 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Stone(*stone), DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)),
                    4 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Stone(*stone), DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)),
                    5 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Stone(*stone), DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)),
                    6 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Stone(*stone), DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)),
                    7 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Stone(*stone), DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)),
                    8 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Stone(*stone), DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)),
                    9 => state.knowledge.dungeon_reward_locations.insert(DungeonReward::Stone(*stone), DungeonRewardLocation::LinksPocket),
                    _ => return Err(value),
                };
            }
            BigPoeTriforce | CompositeKeys { .. } | FreeReward => unimplemented!(),
        }
        Ok(())
    }
}

macro_rules! cells {
    ($($cell_name:literal: $id:ident),*$(,)?) => {
        fn cell_id(&self, cell_id: &str) -> Option<TrackerCellId> {
            match cell_id {
                $(
                    $cell_name => Some(TrackerCellId::$id),
                )*
                _ => None,
            }
        }

        fn serialize_state(&self, state: &ModelState) -> serde_json::Result<BTreeMap<&'static str, Json>> {
            let mut map = BTreeMap::default();
            $(
                map.insert($cell_name, serde_json::to_value(TrackerCellId::$id.kind().render(state))?);
            )*
            Ok(map)
        }
    };
}

#[derive(Debug, FromArc, Clone)]
pub enum Error {
    Cancelled,
    CellId,
    EventSource(String),
    #[from_arc]
    Json(Arc<serde_json::Error>),
    MissingData,
    PathPrefix,
    #[from_arc]
    Reqwest(Arc<reqwest::Error>),
    UnexpectedEndOfStream,
    UnknownEvent(String),
}

impl From<eventsource_client::Error> for Error {
    fn from(e: eventsource_client::Error) -> Error {
        Error::EventSource(format!("{:?}", e))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Cancelled => write!(f, "event source was cancelled"),
            Error::CellId => write!(f, "received data for unknown cell"),
            Error::EventSource(debug) => write!(f, "error in event source: {}", debug),
            Error::Json(e) => write!(f, "JSON error: {}", e),
            Error::MissingData => write!(f, "event source did not send any data"),
            Error::PathPrefix => write!(f, "event source sent an incorrect path"),
            Error::Reqwest(e) => if let Some(url) = e.url() {
                write!(f, "HTTP error at {}: {}", url, e)
            } else {
                write!(f, "HTTP error: {}", e)
            },
            Error::UnexpectedEndOfStream => write!(f, "unexpected end of event stream"),
            Error::UnknownEvent(event) => write!(f, "event source sent unknown event: {:?}", event),
        }
    }
}

pub trait App: fmt::Debug + Send + Sync + 'static {
    fn base_url(&self) -> &'static str;
    fn api_key(&self) -> &'static str;
    fn cell_id(&self, cell_id: &str) -> Option<TrackerCellId>;
    fn serialize_state(&self, state: &ModelState) -> serde_json::Result<BTreeMap<&'static str, Json>>;

    fn set_cell(&self, state: &mut ModelState, cell_id: TrackerCellId, value: Json) -> Result<(), Json> {
        cell_id.kind().set(state, value)
    }
}

impl App for Box<dyn App> {
    fn base_url(&self) -> &'static str { (**self).base_url() }
    fn api_key(&self) -> &'static str { (**self).api_key() }
    fn cell_id(&self, cell_id: &str) -> Option<TrackerCellId> { (**self).cell_id(cell_id) }
    fn serialize_state(&self, state: &ModelState) -> serde_json::Result<BTreeMap<&'static str, Json>> { (**self).serialize_state(state) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct OldRestreamTracker;

impl App for OldRestreamTracker {
    fn base_url(&self) -> &'static str { "https://oot-tracker.firebaseio.com" }
    fn api_key(&self) -> &'static str { OLD_RESTREAM_API_KEY }

    //TODO other collections (presumably medallions and chestsopened)
    cells! {
        "Bow": Quiver,
        "Hookshot": Hookshot,
        "Hammer": Hammer,
        "Bombs": BombBag,
        "Scale": Scale,
        "Glove": Strength,
        "KokiriSword": KokiriSword,
        "BiggoronSword": BiggoronSword,
        "MirrorShield": MirrorShield,
        "ZoraTunic": ZoraTunic,
        "GoronTunic": GoronTunic,
        "IronBoots": IronBoots,
        "HoverBoots": HoverBoots,
        "Dins": DinsFire,
        "Farores": FaroresWind,
        "Nayrus": NayrusLove,
        "Magic": MagicCapacity,
        "Fire": FireArrows,
        "Ice": IceArrows,
        "Light": LightArrows,
        "Slingshot": BulletBag,
        "Boomerang": Boomerang,
        "Lens": Lens,
        "Bottle": NumBottles,
        "ZoraLetter": RutosLetter,
        "Wallet": WalletNoTycoon,
        "Skulltula": SkulltulaTens,
        "ZeldasLullaby": ZeldasLullaby,
        "EponasSong": EponasSong,
        "SunsSong": SunsSong,
        "SariasSong": SariasSong,
        "SongofTime": SongOfTime,
        "SongofStorms": SongOfStorms,
        "MinuetofForest": Minuet,
        "BoleroofFire": Bolero,
        "SerenadeofWater": Serenade,
        "NocturneofShadow": Nocturne,
        "RequiemofSpirit": Requiem,
        "PreludeofLight": Prelude,
        "ForestMedallion": ForestMedallion,
        "FireMedallion": FireMedallion,
        "WaterMedallion": WaterMedallion,
        "ShadowMedallion": ShadowMedallion,
        "SpiritMedallion": SpiritMedallion,
        "LightMedallion": LightMedallion,
        "KokiriEmerald": KokiriEmerald,
        "GoronRuby": GoronRuby,
        "ZoraSapphire": ZoraSapphire,
        "StoneofAgony": StoneOfAgony,
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RestreamTracker;

impl App for RestreamTracker {
    fn base_url(&self) -> &'static str { "https://ootr-tracker.firebaseio.com" }
    fn api_key(&self) -> &'static str { RESTREAM_API_KEY }

    //TODO medallions, chestsopened
    cells! {
        "KokiriSword": KokiriSword,
        "Slingshot": BulletBag,
        "GoMode": GoMode,
        "BiggoronSword": BiggoronSword,
        "Bombchu": Bombchus,
        "Mask": ChildTradeSoldOut,
        "Bombs": BombBag,
        "Bow": Quiver,
        "ForestMedallion": ForestMedallion,
        "Skulltula": SkulltulaTens,
        "Wallet": WalletNoTycoon,
        "Trade": AdultTrade, //TODO add Pocket Cucco
        "Boomerang": Boomerang,
        "Hammer": Hammer,
        "FireMedallion": FireMedallion,
        "ZoraTunic": ZoraTunic,
        "GoronTunic": GoronTunic,
        "Triforce": TriforceOneAndFives,
        "Hookshot": Hookshot,
        "Spells": Spells,
        "WaterMedallion": WaterMedallion,
        "Nayrus": NayrusLove,
        "Magic": MagicCapacity,
        "Lens": Lens,
        "ZoraLetter": RutosLetter,
        "Arrows": Arrows,
        "SpiritMedallion": SpiritMedallion,
        "Ice": IceArrows,
        "Bottle": NumBottles,
        "StoneofAgony": StoneOfAgony,
        "MirrorShield": MirrorShield,
        "Glove": Strength,
        "ShadowMedallion": ShadowMedallion,
        "Boots": Boots,
        "Scale": Scale,
        "LightMedallion": LightMedallion,
        "ZeldasLullaby": ZeldasLullaby,
        "EponasSong": EponasSong,
        "SariasSong": SariasSong,
        "KokiriEmerald": KokiriEmerald,
        "GoronRuby": GoronRuby,
        "ZoraSapphire": ZoraSapphire,
        "SunsSong": SunsSong,
        "SongofTime": SongOfTime,
        "SongofStorms": SongOfStorms,
        "MinuetofForest": Minuet,
        "BoleroofFire": Bolero,
        "SerenadeofWater": Serenade,
        "RequiemofSpirit": Requiem,
        "NocturneofShadow": Nocturne,
        "PreludeofLight": Prelude,
        "Fire": FireArrows,
        "Light": LightArrows,
        "Dins": DinsFire,
        "Farores": FaroresWind,
        "IronBoots": IronBoots,
        "HoverBoots": HoverBoots,
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RslItemTracker;

impl App for RslItemTracker {
    fn base_url(&self) -> &'static str { "https://ootr-random-settings-tracker.firebaseio.com" }
    fn api_key(&self) -> &'static str { RSL_API_KEY }

    cells! {
        "forestmed": ForestMedallion,
        "forest_med_text": ForestMedallionLocation,
        "firemed": FireMedallion,
        "fire_med_text": FireMedallionLocation,
        "watermed": WaterMedallion,
        "water_med_text": WaterMedallionLocation,
        "shadowmed": ShadowMedallion,
        "shadow_med_text": ShadowMedallionLocation,
        "spiritmed": SpiritMedallion,
        "spirit_med_text": SpiritMedallionLocation,
        "lightmed": LightMedallion,
        "light_med_text": LightMedallionLocation,
        "atrade_full": AdultTradeNoChicken,
        "gst": Skulltula,
        "kokiri_emerald": KokiriEmerald,
        "kokiri_emerald_text": KokiriEmeraldLocation,
        "goron_ruby": GoronRuby,
        "goron_ruby_text": GoronRubyLocation,
        "zora_sapphire": ZoraSapphire,
        "zora_sapphire_text": ZoraSapphireLocation,
        "bottle_letter_base": Bottle,
        "bottle_letter_badge": RutosLetter,
        "scale": Scale,
        "slingshot": Slingshot,
        "explosives_base": Bombs,
        "explosives_badge": Bombchus,
        "boomerang": Boomerang,
        "strength": Strength,
        "magic_lens_base": Magic,
        "magic_lens_badge": Lens,
        "spells": Spells,
        "hooks": Hookshot,
        "bow": Bow,
        "magicarrows": Arrows,
        "hammer": Hammer,
        "boots": Boots,
        "mirrorshield": MirrorShield,
        "ctrade_full": ChildTradeNoChicken,
        "ocarina": Ocarina,
        "beans": Beans,
        "dungeonopeners": SwordCard,
        "tunics": Tunics,
        "triforce": Triforce,
        "zlsong_base": ZeldasLullaby,
        "zlsong_badge": ZeldasLullabyCheck,
        "eponasong_base": EponasSong,
        "eponasong_badge": EponasSongCheck,
        "sariasong_base": SariasSong,
        "sariasong_badge": SariasSongCheck,
        "sunsong_base": SunsSong,
        "sunsong_badge": SunsSongCheck,
        "timesong_base": SongOfTime,
        "timesong_badge": SongOfTimeCheck,
        "stormssong_base": SongOfStorms,
        "stormssong_badge": SongOfStormsCheck,
        "minuetsong_base": Minuet,
        "minuetsong_badge": MinuetCheck,
        "bolerosong_base": Bolero,
        "bolerosong_badge": BoleroCheck,
        "serenadesong_base": Serenade,
        "serenadesong_badge": SerenadeCheck,
        "nocturnesong_base": Nocturne,
        "nocturnesong_badge": NocturneCheck,
        "requiemsong_base": Requiem,
        "requiemsong_badge": RequiemCheck,
        "preludesong_base": Prelude,
        "preludesong_badge": PreludeCheck,
        "dekutype": DekuMq,
        "dctype": DcMq,
        "jabutype": JabuMq,
        "foresttype": ForestMq,
        "forestsk": ForestSmallKeys,
        "forestbk": ForestBossKey,
        "shadowtype": ShadowMq,
        "shadowsk": ShadowSmallKeys,
        "shadowbk": ShadowBossKey,
        "welltype": WellMq,
        "wellsk": WellSmallKeys,
        "firetype": FireMq,
        "firesk": FireSmallKeys,
        "firebk": FireBossKey,
        "spirittype": SpiritMq,
        "spiritsk": SpiritSmallKeys,
        "spiritbk": SpiritBossKey,
        "forttype": FortressMq,
        "fortsk": FortressSmallKeys,
        "watertype": WaterMq,
        "watersk": WaterSmallKeys,
        "waterbk": WaterBossKey,
        "ganontype": GanonMq,
        "ganonsk": GanonSmallKeys,
        "ganonbk": GanonBossKey,
        "gtgtype": GtgMq,
        "gtgsk": GtgSmallKeys,
        // for tsgmain and tsgsquare layouts:
        "kokirisword": KokiriSword,
        "gomode": GoMode,
        "kokiri_emerald_full": KokiriEmerald,
        "goron_ruby_full": GoronRuby,
        "zora_sapphire_full": ZoraSapphire,
    }
}

#[derive(Deserialize)]
enum SignupNewUserResponse {
    #[serde(rename = "identitytoolkit#SignupNewUserResponse")]
    SignupNewUserResponse,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthResponse {
    kind: SignupNewUserResponse,
    id_token: String,
    local_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PutData {
    path: String,
    data: Json,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PatchData {
    path: String,
    data: BTreeMap<String, Json>,
}

#[derive(Clone)]
pub struct Session<A: App> {
    client: reqwest::Client,
    local_id: String,
    id_token: String,
    app: A,
}

impl<A: App> Session<A> {
    pub async fn new(app: A) -> reqwest::Result<Session<A>> {
        let client = reqwest::Client::builder()
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
            .build()?;
        let mut session = Session {
            client, app,
            local_id: String::default(),
            id_token: String::default(),
        };
        session.base_auth().await?;
        Ok(session)
    }

    async fn base_auth(&mut self) -> reqwest::Result<()> {
        let AuthResponse { kind: SignupNewUserResponse::SignupNewUserResponse, id_token, local_id, .. } = self.client.post("https://identitytoolkit.googleapis.com/v1/accounts:signUp")
            .query(&[("key", self.app.api_key())])
            .json(&json!({"returnSecureToken": true}))
            .send().await?
            .error_for_status()?
            .json().await?;
        self.local_id = local_id;
        self.id_token = id_token;
        Ok(())
    }

    async fn room_auth(&mut self, name: &str, passcode: &str) -> reqwest::Result<()> {
        self.base_auth().await?;
        if self.get::<Option<String>>(&format!("{}/games/{}/owner.json", self.app.base_url(), name)).await?.is_some() {
            self.put(&format!("{}/games/{}/editors/{}.json", self.app.base_url(), name, self.local_id), &json!(passcode)).await?;
        } else {
            self.put(&format!("{}/games/{}.json", self.app.base_url(), name), &json!({"owner": self.local_id, "passcode": passcode, "editors": {&self.local_id: true}})).await?;
        }
        Ok(())
    }

    async fn get<T: DeserializeOwned>(&mut self, url: &str) -> reqwest::Result<T> {
        self.client.get(url)
            .query(&[("auth", &self.id_token)])
            .send().await?
            .error_for_status()?
            .json().await
    }

    async fn put<T: Serialize>(&mut self, url: &str, data: &T) -> reqwest::Result<()> {
        self.client.put(url)
            .query(&[("auth", &self.id_token)])
            .json(data)
            .send().await?
            .error_for_status()?;
        //TODO check to make sure response body is same as request body
        Ok(())
    }

    async fn put_reauth<T: Serialize>(&mut self, name: &str, passcode: &str, url: &str, data: &T) -> reqwest::Result<()> {
        let mut response = self.client.put(url)
            .query(&[("auth", &self.id_token)])
            .json(data)
            .send().await?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            self.room_auth(name, passcode).await?;
            response = self.client.put(url)
                .query(&[("auth", &self.id_token)])
                .json(data)
                .send().await?;
        }
        response.error_for_status()?;
        //TODO check to make sure response body is same as request body
        Ok(())
    }

    fn to_dyn(&self) -> Session<Box<dyn App>>
    where A: Clone {
        Session {
            client: self.client.clone(),
            local_id: self.local_id.clone(),
            id_token: self.id_token.clone(),
            app: Box::new(self.app.clone()),
        }
    }
}

#[derive(Clone)]
pub struct Room<A: App> {
    pub session: Session<A>,
    pub name: String,
    pub passcode: String,
}

impl<A: App> Room<A> {
    pub fn to_dyn(&self) -> DynRoom
    where A: Clone + Send {
        let mut hasher = DefaultHasher::default();
        TypeId::of::<A>().hash(&mut hasher);
        DynRoom {
            app_hash: hasher.finish(),
            session: Arc::new(Mutex::new(self.session.to_dyn())),
            name: self.name.clone(),
            passcode: self.passcode.clone(),
        }
    }
}

#[derive(Clone)]
pub struct DynRoom {
    app_hash: u64,
    session: Arc<Mutex<Session<Box<dyn App>>>>,
    name: String,
    passcode: String,
}

impl DynRoom {
    pub async fn set_state(&self, new_state: &ModelState) -> Result<(), Error> {
        let mut session = self.session.lock().await;
        let url = format!("{}/games/{}/items.json", session.app.base_url(), self.name);
        let state = session.app.serialize_state(new_state)?;
        Ok(session.put_reauth(&self.name, &self.passcode, &url, &state).await?)
    }

    pub fn subscribe(&self) -> Pin<Box<dyn Stream<Item = Result<(TrackerCellId, Json), Error>> + Send>> {
        let session = Arc::clone(&self.session);
        let name = self.name.clone();
        Box::pin(try_stream! {
            'reauth_loop: loop {
                let url = {
                    let session = session.lock().await;
                    format!("{}/games/{}/items.json?auth={}", session.app.base_url(), name, session.id_token)
                };
                let events = eventsource_client::Client::for_url(&url)?
                    .header("Accept", "text/event-stream")?
                    .build()
                    .stream();
                pin_mut!(events);
                while let Some(event) = events.try_next().await? {
                    match &*event.event_type {
                        "put" => {
                            let PutData { path, data } = serde_json::from_slice(event.field("data").ok_or(Error::MissingData)?)?;
                            let session = session.lock().await;
                            if path == "/" {
                                for (item, value) in serde_json::from_value::<BTreeMap<String, Json>>(data)? {
                                    let cell_id = session.app.cell_id(&item).ok_or(Error::CellId)?;
                                    yield (cell_id, value);
                                }
                            } else {
                                let item = path.strip_prefix('/').ok_or(Error::PathPrefix)?;
                                let cell_id = session.app.cell_id(item).ok_or(Error::CellId)?;
                                yield (cell_id, data);
                            }
                        }
                        "patch" => {
                            let PatchData { path, data } = serde_json::from_slice(event.field("data").ok_or(Error::MissingData)?)?;
                            if path != "/" { unimplemented!("patch for path {}", path) }
                            let session = session.lock().await;
                            for (item, value) in data {
                                let cell_id = session.app.cell_id(&item).ok_or(Error::CellId)?;
                                yield (cell_id, value);
                            }
                        }
                        "keep-alive" => {}
                        "cancel" => { Err(Error::Cancelled)?; }
                        "auth_revoked" => {
                            session.lock().await.base_auth().await?;
                            continue 'reauth_loop
                        }
                        _ => { Err(Error::UnknownEvent(event.event_type))?; }
                    }
                }
                Err(Error::UnexpectedEndOfStream)?;
            }
        })
    }
}

impl fmt::Debug for DynRoom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynRoom {{ name: {:?}, session: _ }}", self.name) //TODO use debug_struct with finish_non_exhaustive
    }
}

impl Hash for DynRoom {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.app_hash.hash(state);
        self.name.hash(state);
        self.passcode.hash(state);
    }
}
