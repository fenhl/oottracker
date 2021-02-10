use {
    std::{
        collections::BTreeMap,
        fmt,
        iter,
        sync::Arc,
    },
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

trait TrackerCellKindExt {
    fn render(&self, state: &ModelState) -> Json;
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
            SongCheck { check, .. } => json!(Check::Location(check.to_string()).checked(state).unwrap_or(false)),
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
        }
    }
}

macro_rules! cells {
    ($state:expr, {$($cell_name:literal: $id:ident),*$(,)?}) => {{
        let mut map = BTreeMap::default();
        $(
            map.insert($cell_name, serde_json::to_value(TrackerCellId::$id.kind().render($state))?);
        )*
        Ok(map)
    }};
}

#[derive(Debug, Clone)]
pub enum Error {
    Json(Arc<serde_json::Error>),
    Reqwest(Arc<reqwest::Error>),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Json(Arc::new(e))
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Error {
        Error::Reqwest(Arc::new(e))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Json(e) => write!(f, "JSON error: {}", e),
            Error::Reqwest(e) => if let Some(url) = e.url() {
                write!(f, "HTTP error at {}: {}", url, e)
            } else {
                write!(f, "HTTP error: {}", e)
            },
        }
    }
}

pub trait App: Send + Sync + 'static {
    fn base_url(&self) -> &'static str;
    fn api_key(&self) -> &'static str;
    fn serialize_state(&self, state: &ModelState) -> serde_json::Result<BTreeMap<&'static str, Json>>;
}

impl App for Box<dyn App> {
    fn base_url(&self) -> &'static str { (**self).base_url() }
    fn api_key(&self) -> &'static str { (**self).api_key() }
    fn serialize_state(&self, state: &ModelState) -> serde_json::Result<BTreeMap<&'static str, Json>> { (**self).serialize_state(state) }
}

#[derive(Default, Clone, Copy)]
pub struct RestreamTracker;

impl App for RestreamTracker {
    fn base_url(&self) -> &'static str { "https://ootr-tracker.firebaseio.com" }
    fn api_key(&self) -> &'static str { "AIzaSyDsnur0ixzqAx9uO8Ej_Rc7zhLRHlHPGRE" }

    fn serialize_state(&self, state: &ModelState) -> serde_json::Result<BTreeMap<&'static str, Json>> {
        //TODO medallions, chestsopened
        cells!(state, {
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
        })
    }
}

#[derive(Default, Clone, Copy)]
pub struct RslItemTracker;

impl App for RslItemTracker {
    fn base_url(&self) -> &'static str { "https://ootr-random-settings-tracker.firebaseio.com" }
    fn api_key(&self) -> &'static str { "AIzaSyB9qoaU5aFkIxNUy473FsgU0Oe7SssJDhs" }

    fn serialize_state(&self, state: &ModelState) -> serde_json::Result<BTreeMap<&'static str, Json>> {
        cells!(state, {
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
        })
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
    //refresh_token: String,
    //expires_in: String, //TODO decode to Duration?
    local_id: String,
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

    /*
    async fn get_reauth<T: DeserializeOwned>(&mut self, name: &str, passcode: &str, url: &str) -> reqwest::Result<T> {
        let mut response = self.client.get(url)
            .query(&[("auth", &self.id_token)])
            .send().await?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            self.room_auth(name, passcode).await?;
            response = self.client.get(url)
                .query(&[("auth", &self.id_token)])
                .send().await?;
        }
        response.error_for_status()?.json().await
    }
    */

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
    /*
    async fn state(&mut self) -> reqwest::Result<ModelState> {
        self.session.get_reauth(&self.name, &self.passcode, &format!("{}/games/{}/items.json", self.session.app.base_url(), self.name)).await
    }
    */

    pub async fn set_state(&mut self, new_state: &ModelState) -> Result<(), Error> {
        Ok(self.session.put_reauth(&self.name, &self.passcode, &format!("{}/games/{}/items.json", self.session.app.base_url(), self.name), &self.session.app.serialize_state(new_state)?).await?)
    }

    pub fn to_dyn(&self) -> DynRoom
    where A: Clone + Send {
        DynRoom {
            session: Arc::new(Mutex::new(self.session.to_dyn())),
            name: self.name.clone(),
            passcode: self.passcode.clone(),
        }
    }
}

#[derive(Clone)]
pub struct DynRoom {
    session: Arc<Mutex<Session<Box<dyn App>>>>,
    name: String,
    passcode: String,
}

impl DynRoom {
    /*
    async fn state(&self) -> reqwest::Result<ModelState> {
        let mut session = self.session.lock().await;
        let url = format!("{}/games/{}/items.json", session.app.base_url(), self.name);
        session.get_reauth(&self.name, &self.passcode, &url).await
    }
    */

    pub async fn set_state(&self, new_state: &ModelState) -> Result<(), Error> {
        let mut session = self.session.lock().await;
        let url = format!("{}/games/{}/items.json", session.app.base_url(), self.name);
        let state = session.app.serialize_state(new_state)?;
        Ok(session.put_reauth(&self.name, &self.passcode, &url, &state).await?)
    }
}

impl fmt::Debug for DynRoom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynRoom {{ name: {:?}, session: _ }}", self.name) //TODO use debug_struct with finish_non_exhaustive
    }
}
