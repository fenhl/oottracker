#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use {
    std::{
        collections::HashMap,
        convert::Infallible as Never,
        env,
        fmt,
        io,
        sync::Arc,
    },
    derivative::Derivative,
    derive_more::From,
    enum_iterator::IntoEnumIterator,
    iced::{
        Application,
        Background,
        Color,
        Command,
        Element,
        HorizontalAlignment,
        Length,
        Settings,
        widget::{
            Column,
            Image,
            Row,
            Space,
            Text,
            button::{
                self,
                Button,
            },
            container::{
                self,
                Container,
            },
            pick_list::{
                self,
                PickList,
            },
            text_input::{
                self,
                TextInput,
            },
        },
        window::{
            self,
            Icon,
        },
    },
    iced_futures::Subscription,
    iced_native::keyboard::Modifiers as KeyboardModifiers,
    image::DynamicImage,
    itertools::Itertools as _,
    semver::{
        SemVerError,
        Version,
    },
    smart_default::SmartDefault,
    structopt::StructOpt,
    tokio::fs,
    url::Url,
    wheel::FromArc,
    ootr::{
        Rando,
        check::Check,
        model::{
            DungeonReward,
            DungeonRewardLocation,
            MainDungeon,
            Stone,
        },
    },
    oottracker::{
        ModelState,
        checks::{
            self,
            CheckExt as _,
            CheckStatus,
            CheckStatusError,
        },
        firebase,
        github::Repo,
        net::{
            self,
            Connection,
        },
        proto::Packet,
        save::*,
        ui::{
            self,
            *,
            TrackerCellKind::*,
        },
    },
};
#[cfg(target_os = "macos")] use {
    std::time::Duration,
    futures::stream::TryStreamExt as _,
    tokio::{
        fs::File,
        io::AsyncWriteExt as _,
        time::sleep,
    },
};

mod lang;
mod logic;
mod subscriptions;

const CELL_SIZE: u16 = 50;
const STONE_SIZE: u16 = 30;
const MEDALLION_LOCATION_HEIGHT: u16 = 18;
//const STONE_LOCATION_HEIGHT: u16 = 10;
const WIDTH: u32 = (CELL_SIZE as u32 + 10) * 6; // 6 images, each 50px wide, plus 10px spacing
const HEIGHT: u32 = (MEDALLION_LOCATION_HEIGHT as u32 + 10) + (CELL_SIZE as u32 + 10) * 7; // dungeon reward location text, 18px high, and 7 images, each 50px high, plus 10px spacing

struct ContainerStyle;

impl container::StyleSheet for ContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::BLACK)),
            ..container::Style::default()
        }
    }
}

trait TrackerCellKindExt {
    fn render(&self, state: &ModelState) -> Image;
    #[must_use] fn left_click(&self, can_change_state: bool, keyboard_modifiers: KeyboardModifiers, state: &mut ModelState) -> bool;
    #[must_use] fn right_click(&self, can_change_state: bool, state: &mut ModelState) -> bool;
}

impl TrackerCellKindExt for TrackerCellKind {
    fn render(&self, state: &ModelState) -> Image {
        match self {
            BigPoeTriforce => if state.ram.save.triforce_pieces() > 0 {
                images::xopar_images_count(&format!("force_{}", state.ram.save.triforce_pieces()))
            } else if state.ram.save.big_poes > 0 { //TODO show dimmed Triforce icon if it's known that it's TH
                images::extra_images_count(&format!("poes_{}", state.ram.save.big_poes))
            } else {
                images::extra_images_dimmed("big_poe")
            },
            Composite { left_img, right_img, both_img, active, .. } => match active(state) {
                (false, false) => both_img.embedded(ImageDirContext::Dimmed),
                (false, true) => right_img.embedded(ImageDirContext::Normal),
                (true, false) => left_img.embedded(ImageDirContext::Normal),
                (true, true) => both_img.embedded(ImageDirContext::Normal),
            },
            Count { dimmed_img, img, get, .. } => {
                let count = get(state);
                if count == 0 {
                    dimmed_img.embedded(ImageDirContext::Dimmed)
                } else {
                    img.embedded(ImageDirContext::Count(count))
                }
            }
            Medallion(med) => {
                let med_filename = format!("{}_medallion", med.element().to_ascii_lowercase());
                if state.ram.save.quest_items.has(*med) {
                    images::xopar_images::<Image>(&med_filename)
                } else {
                    images::xopar_images_dimmed(&med_filename)
                }
            }
            MedallionLocation(med) => match state.knowledge.dungeon_reward_locations.get(&DungeonReward::Medallion(*med)) {
                None => images::xopar_images_dimmed::<Image>("unknown_text"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)) => images::xopar_images("deku_text"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)) => images::xopar_images("dc_text"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)) => images::xopar_images("jabu_text"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)) => images::xopar_images("forest_text"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)) => images::xopar_images("fire_text"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)) => images::xopar_images("water_text"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)) => images::xopar_images("shadow_text"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)) => images::xopar_images("spirit_text"),
                Some(DungeonRewardLocation::LinksPocket) => images::xopar_images("free_text"),
            }.width(Length::Units(CELL_SIZE)),
            OptionalOverlay { main_img, overlay_img, active, .. } | Overlay { main_img, overlay_img, active, .. } => match active(state) {
                (false, false) => main_img.embedded(ImageDirContext::Dimmed),
                (true, false) => main_img.embedded(ImageDirContext::Normal),
                (main_active, true) => main_img.with_overlay(overlay_img).embedded(main_active),
            },
            Sequence { img, .. } => match img(state) {
                (false, img) => img.embedded(ImageDirContext::Dimmed),
                (true, img) => img.embedded(ImageDirContext::Normal),
            },
            Simple { img, active, .. } => if active(state) {
                img.embedded(ImageDirContext::Normal)
            } else {
                img.embedded(ImageDirContext::Dimmed)
            },
            Song { song, check, .. } => {
                let song_filename = match *song {
                    QuestItems::ZELDAS_LULLABY => "lullaby",
                    QuestItems::EPONAS_SONG => "epona",
                    QuestItems::SARIAS_SONG => "saria",
                    QuestItems::SUNS_SONG => "sun",
                    QuestItems::SONG_OF_TIME => "time",
                    QuestItems::SONG_OF_STORMS => "storms",
                    QuestItems::MINUET_OF_FOREST => "minuet",
                    QuestItems::BOLERO_OF_FIRE => "bolero",
                    QuestItems::SERENADE_OF_WATER => "serenade",
                    QuestItems::NOCTURNE_OF_SHADOW => "nocturne",
                    QuestItems::REQUIEM_OF_SPIRIT => "requiem",
                    QuestItems::PRELUDE_OF_LIGHT => "prelude",
                    _ => unreachable!(),
                };
                match (state.ram.save.quest_items.contains(*song), Check::<ootr_static::Rando>::Location(check.to_string()).checked(state).unwrap_or(false)) { //TODO allow ootr_dynamic::Rando
                    (false, false) => images::xopar_images_dimmed(song_filename),
                    (false, true) => images::xopar_images_overlay_dimmed(&format!("{}_check", song_filename)),
                    (true, false) => images::xopar_images(song_filename),
                    (true, true) => images::xopar_images_overlay(&format!("{}_check", song_filename)),
                }
            }
            Stone(stone) => {
                let stone_filename = match *stone {
                    Stone::KokiriEmerald => "kokiri_emerald",
                    Stone::GoronRuby => "goron_ruby",
                    Stone::ZoraSapphire => "zora_sapphire",
                };
                if state.ram.save.quest_items.has(*stone) {
                    images::xopar_images::<Image>(stone_filename)
                } else {
                    images::xopar_images_dimmed(stone_filename)
                }.width(Length::Units(STONE_SIZE))
            }
            StoneLocation(stone) => match state.knowledge.dungeon_reward_locations.get(&DungeonReward::Stone(*stone)) {
                None => images::xopar_images_dimmed::<Image>("unknown_text"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::DekuTree)) => images::xopar_images("deku_text"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::DodongosCavern)) => images::xopar_images("dc_text"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::JabuJabu)) => images::xopar_images("jabu_text"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::ForestTemple)) => images::xopar_images("forest_text"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::FireTemple)) => images::xopar_images("fire_text"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::WaterTemple)) => images::xopar_images("water_text"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::ShadowTemple)) => images::xopar_images("shadow_text"),
                Some(DungeonRewardLocation::Dungeon(MainDungeon::SpiritTemple)) => images::xopar_images("spirit_text"),
                Some(DungeonRewardLocation::LinksPocket) => images::xopar_images("free_text"),
            }.width(Length::Units(STONE_SIZE)),
            BossKey { .. } | CompositeKeys { .. } | FortressMq | FreeReward | Mq(_) | TrackerCellKind::SmallKeys { .. } | SongCheck { .. } => unimplemented!(),
        }
    }

    #[must_use]
    /// Returns `true` if the menu should be opened.
    fn left_click(&self, can_change_state: bool, #[cfg_attr(not(target_os = "macos"), allow(unused))] keyboard_modifiers: KeyboardModifiers, state: &mut ModelState) -> bool {
        #[cfg(target_os = "macos")] if keyboard_modifiers.control {
            return self.right_click(can_change_state, state)
        }
        if can_change_state {
            match self {
                Composite { toggle_left: toggle, .. } | OptionalOverlay { toggle_main: toggle, .. } | Overlay { toggle_main: toggle, .. } | Simple { toggle, .. } => toggle(state),
                Count { get, set, max, step, .. } => {
                    let current = get(state);
                    if current == *max { set(state, 0) } else { set(state, current + step) }
                }
                Medallion(med) => state.ram.save.quest_items.toggle(QuestItems::from(med)),
                MedallionLocation(med) => state.knowledge.dungeon_reward_locations.increment(DungeonReward::Medallion(*med)),
                Sequence { increment, .. } => increment(state),
                Song { song: quest_item, .. } => state.ram.save.quest_items.toggle(*quest_item),
                Stone(stone) => state.ram.save.quest_items.toggle(QuestItems::from(stone)),
                StoneLocation(stone) => state.knowledge.dungeon_reward_locations.increment(DungeonReward::Stone(*stone)),
                BigPoeTriforce | BossKey { .. } | CompositeKeys { .. } | FortressMq | FreeReward | Mq(_) | TrackerCellKind::SmallKeys { .. } | SongCheck { .. } => unimplemented!(),
            }
        }
        false
    }

    #[must_use]
    /// Returns `true` if the menu should be opened.
    fn right_click(&self, can_change_state: bool, state: &mut ModelState) -> bool {
        if let Medallion(_) = self { return true }
        if can_change_state {
            match self {
                Composite { toggle_right: toggle, .. } | OptionalOverlay { toggle_overlay: toggle, .. } | Overlay { toggle_overlay: toggle, .. } => toggle(state),
                Count { get, set, max, step, .. } => {
                    let current = get(state);
                    if current == 0 { set(state, *max) } else { set(state, current - step) }
                }
                Medallion(_) => unreachable!("already handled above"),
                MedallionLocation(med) => state.knowledge.dungeon_reward_locations.decrement(DungeonReward::Medallion(*med)),
                Sequence { decrement, .. } => decrement(state),
                Simple { .. } | Stone(_) => {}
                Song { toggle_overlay, .. } => toggle_overlay(&mut state.ram.save.event_chk_inf),
                StoneLocation(stone) => state.knowledge.dungeon_reward_locations.decrement(DungeonReward::Stone(*stone)),
                BigPoeTriforce | BossKey { .. } | CompositeKeys { .. } | FortressMq | FreeReward | Mq(_) | TrackerCellKind::SmallKeys { .. } | SongCheck { .. } => unimplemented!(),
            }
        }
        false
    }
}

trait TrackerCellIdExt {
    fn view<'a>(&self, state: &ModelState, cell_button: &'a mut button::State) -> Element<'a, Message<ootr_static::Rando>>; //TODO allow ootr_dynamic::Rando
}

impl TrackerCellIdExt for TrackerCellId {
    fn view<'a>(&self, state: &ModelState, cell_button: &'a mut button::State) -> Element<'a, Message<ootr_static::Rando>> { //TODO allow ootr_dynamic::Rando
        Button::new(cell_button, self.kind().render(state))
            .on_press(Message::LeftClick(*self))
            .padding(0)
            .style(DefaultButtonStyle)
            .into()
    }
}

struct DefaultButtonStyle;

impl button::StyleSheet for DefaultButtonStyle {
    fn active(&self) -> button::Style { button::Style::default() }
}

trait TrackerLayoutExt {
    fn cell_at(&self, pos: [f32; 2], include_songs: bool) -> Option<TrackerCellId>;
}

impl TrackerLayoutExt for TrackerLayout {
    fn cell_at(&self, [x, y]: [f32; 2], include_songs: bool) -> Option<TrackerCellId> {
        if !include_songs && y >= (MEDALLION_LOCATION_HEIGHT as f32 + 10.0) + (CELL_SIZE as f32 + 10.0) * 5.0 { return None }
        self.cells().into_iter()
            .find(|CellLayout { pos: [pos_x, pos_y], size: [size_x, size_y], .. }| (*pos_x..pos_x + size_x).contains(&(x as u16)) && (*pos_y..pos_y + size_y).contains(&(y as u16)))
            .map(|CellLayout { id, .. }| id)
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""), Clone(bound = ""))]
enum Message<R: Rando> {
    CheckStatusErrorStatic(CheckStatusError<R>),
    ClientDisconnected,
    CloseMenu,
    ConfigError(ui::Error),
    ConnectionError(ConnectionError),
    Connect,
    DismissNotification,
    DismissWelcomeScreen,
    InstallUpdate,
    KeyboardModifiers(KeyboardModifiers),
    LeftClick(TrackerCellId),
    LoadConfig(Config),
    Logic(logic::Message<R>),
    MouseMoved([f32; 2]),
    Nop,
    Packet(Packet),
    ResetUpdateState,
    RightClick,
    SetAutoUpdateCheck(bool),
    SetMedOrder(ElementOrder),
    SetPasscode(String),
    SetConnection(Arc<dyn Connection>),
    SetConnectionKind(ConnectionKind),
    SetPort(String),
    SetUrl(String),
    SetWarpSongOrder(ElementOrder),
    UpdateAvailableChecks(HashMap<Check<R>, CheckStatus>),
    UpdateCheck,
    UpdateCheckComplete(Option<Version>),
    UpdateCheckError(UpdateCheckError),
}

impl<R: Rando> fmt::Display for Message<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::CheckStatusErrorStatic(e) => write!(f, "error calculating checks: {}", e),
            Message::ClientDisconnected => write!(f, "connection lost"),
            Message::ConfigError(e) => write!(f, "error loading/saving preferences: {}", e),
            Message::ConnectionError(e) => write!(f, "connection error: {}", e),
            _ => write!(f, "{:?}", self), // these messages are not notifications so just fall back to Debug
        }
    }
}

#[derive(Debug, Default)]
struct MenuState {
    dismiss_btn: button::State,
    med_order: pick_list::State<ElementOrder>,
    warp_song_order: pick_list::State<ElementOrder>,
    connection_kind: pick_list::State<ConnectionKind>,
    connection_params: ConnectionParams,
    connect_btn: button::State,
}

#[derive(Debug, SmartDefault, IntoEnumIterator, Clone, Copy, PartialEq, Eq)]
enum ConnectionKind {
    #[default]
    RetroArch,
    Web,
}

impl fmt::Display for ConnectionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionKind::RetroArch => write!(f, "RetroArch"),
            ConnectionKind::Web => write!(f, "web"),
        }
    }
}

#[derive(Debug, SmartDefault, Clone)]
enum ConnectionParams {
    #[default]
    RetroArch {
        #[default = 55355]
        port: u16,
        port_state: text_input::State,
    },
    Web {
        url: String,
        url_state: text_input::State,
        passcode: String,
        passcode_state: text_input::State,
    },
}

impl ConnectionParams {
    fn kind(&self) -> ConnectionKind {
        match self {
            ConnectionParams::RetroArch { .. } => ConnectionKind::RetroArch,
            ConnectionParams::Web { .. } => ConnectionKind::Web,
        }
    }

    fn set_kind(&mut self, kind: ConnectionKind) {
        if kind == self.kind() { return }
        *self = match kind {
            ConnectionKind::RetroArch => ConnectionParams::RetroArch {
                port: 55355,
                port_state: text_input::State::default(),
            },
            ConnectionKind::Web => ConnectionParams::Web {
                url: String::default(),
                url_state: text_input::State::default(),
                passcode: String::default(),
                passcode_state: text_input::State::default(),
            },
        };
    }

    fn view<R: Rando + 'static>(&mut self) -> Element<'_, Message<R>> {
        match self {
            ConnectionParams::RetroArch { port, port_state } => Row::new()
                .push(Text::new("Port: "))
                .push(TextInput::new(port_state, "", &port.to_string(), Message::SetPort))
                .into(),
            ConnectionParams::Web { url, url_state, passcode, passcode_state } => Column::new()
                .push(TextInput::new(url_state, "URL", url, Message::SetUrl))
                .push(TextInput::new(passcode_state, "passcode", passcode, Message::SetPasscode).password())
                .into(),
        }
    }
}

#[derive(Debug)]
struct State<R: Rando + 'static> {
    flags: Args,
    config: Option<Config>,
    http_client: reqwest::Client,
    update_check: UpdateCheckState,
    connection: Option<Arc<dyn Connection>>,
    keyboard_modifiers: KeyboardModifiers,
    last_cursor_pos: [f32; 2],
    dismiss_welcome_screen_button: button::State,
    enable_update_checks_button: button::State,
    disable_update_checks_button: button::State,
    cell_buttons: [button::State; 52],
    rando: Arc<R>,
    model: ModelState,
    checks: HashMap<Check<R>, CheckStatus>,
    logic: logic::State<R>,
    notification: Option<(bool, Message<R>)>,
    dismiss_notification_button: button::State,
    menu_state: Option<MenuState>,
}

impl<R: Rando + 'static> State<R> {
    fn layout(&self) -> TrackerLayout {
        if self.connection.as_ref().map_or(true, |connection| connection.can_change_state()) {
            TrackerLayout::from(&self.config)
        } else {
            if let Some(ref config) = self.config {
                TrackerLayout::new_auto(config)
            } else {
                TrackerLayout::default_auto()
            }
        }
    }

    /// Adds a visible notification/alert/log message.
    ///
    /// Implemented as a separate method in case the way this is displayed is changed later, e.g. to allow multiple notifications.
    #[must_use]
    fn notify(&mut self, message: Message<R>) -> Command<Message<R>> {
        self.notification = Some((false, message));
        Command::none()
    }

    fn save_config(&self) -> Command<Message<R>> {
        if let Some(ref config) = self.config {
            let config = config.clone();
            async move {
                match config.save().await {
                    Ok(()) => Message::Nop,
                    Err(e) => Message::ConfigError(e),
                }
            }.into()
        } else {
            Command::none()
        }
    }
}

impl Default for State<ootr_static::Rando> {
    fn default() -> State<ootr_static::Rando> {
        State {
            flags: Args::default(),
            config: None,
            http_client: reqwest::Client::builder()
                .user_agent(concat!("oottracker/", env!("CARGO_PKG_VERSION")))
                .http2_prior_knowledge()
                .use_rustls_tls()
                .https_only(true)
                .build()
                .expect("failed to build HTTP client"),
            update_check: UpdateCheckState::Unknown(button::State::default()),
            connection: None,
            keyboard_modifiers: KeyboardModifiers::default(),
            last_cursor_pos: [0.0, 0.0],
            dismiss_welcome_screen_button: button::State::default(),
            enable_update_checks_button: button::State::default(),
            disable_update_checks_button: button::State::default(),
            cell_buttons: [
                button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(),
                button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(),
                button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(),
                button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(),
                button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(),
                button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(),
                button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(),
                button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(), button::State::default(),
            ],
            rando: Arc::new(ootr_static::Rando),
            model: ModelState::default(),
            checks: HashMap::default(),
            logic: logic::State::default(),
            notification: None,
            dismiss_notification_button: button::State::default(),
            menu_state: None,
        }
    }
}

impl From<Args> for State<ootr_static::Rando> { //TODO include Rando in flags and make this impl generic
    fn from(flags: Args) -> State<ootr_static::Rando> {
        State {
            flags,
            ..State::default()
        }
    }
}

impl Application for State<ootr_static::Rando> { //TODO include Rando in flags and make this impl generic
    type Executor = iced::executor::Default;
    type Message = Message<ootr_static::Rando>;
    type Flags = Args;

    fn new(flags: Args) -> (State<ootr_static::Rando>, Command<Message<ootr_static::Rando>>) {
        (State::from(flags), async {
            match Config::new().await {
                Ok(Some(config)) => Message::LoadConfig(config),
                Ok(None) => Message::Nop,
                Err(e) => Message::ConfigError(e),
            }
        }.into())
    }

    fn title(&self) -> String {
        if let Some(ref connection) = self.connection {
            format!("OoT Tracker ({} connected)", connection.display_kind())
        } else {
            format!("OoT Tracker")
        }
    }

    fn update(&mut self, message: Message<ootr_static::Rando>) -> Command<Message<ootr_static::Rando>> {
        match message {
            Message::CheckStatusErrorStatic(_) => return self.notify(message),
            Message::ClientDisconnected => return self.notify(message),
            Message::CloseMenu => self.menu_state = None,
            Message::ConfigError(_) => return self.notify(message),
            Message::Connect => if self.connection.is_some() {
                self.connection = None;
            } else {
                if let Some(ref menu_state) = self.menu_state {
                    let params = menu_state.connection_params.clone();
                    let model = self.model.clone();
                    return async move {
                        match connect(params, model).await {
                            Ok(connection) => Message::SetConnection(connection),
                            Err(e) => Message::ConnectionError(e),
                        }
                    }.into()
                }
            },
            Message::ConnectionError(_) => return self.notify(message),
            Message::DismissNotification => self.notification = None,
            Message::DismissWelcomeScreen => {
                self.config = Some(Config::default());
                return self.save_config()
            }
            Message::InstallUpdate => {
                self.update_check = UpdateCheckState::Installing;
                let client = self.http_client.clone();
                return async move {
                    match run_updater(&client).await {
                        Ok(never) => match never {},
                        Err(e) => Message::UpdateCheckError(e),
                    }
                }.into()
            }
            Message::KeyboardModifiers(modifiers) => self.keyboard_modifiers = modifiers,
            Message::LeftClick(cell) => if cell.kind().left_click(self.connection.as_ref().map_or(true, |connection| connection.can_change_state()), self.keyboard_modifiers, &mut self.model) {
                self.menu_state = Some(MenuState::default());
            } else if let Some(ref connection) = self.connection {
                if connection.can_change_state() {
                    let send_fut = connection.set_state(&self.model);
                    return async move {
                        match send_fut.await {
                            Ok(()) => Message::Nop,
                            Err(e) => Message::ConnectionError(e.into()),
                        }
                    }.into()
                }
            },
            Message::LoadConfig(config) => match config.version {
                0 => {
                    let auto_update_check = config.auto_update_check;
                    self.config = Some(config);
                    if auto_update_check == Some(true) {
                        return async { Message::UpdateCheck }.into()
                    }
                }
                v => unimplemented!("config version from the future: {}", v),
            },
            Message::Logic(msg) => return self.logic.update(msg),
            Message::MouseMoved(pos) => self.last_cursor_pos = pos,
            Message::Nop => {}
            Message::Packet(packet) => {
                match packet {
                    Packet::Goodbye => unreachable!(), // Goodbye is not yielded from proto::read
                    Packet::SaveDelta(delta) => {
                        self.model.ram.save = &self.model.ram.save + &delta;
                        self.model.update_knowledge();
                    }
                    Packet::SaveInit(save) => {
                        self.model.ram.save = save;
                        self.model.update_knowledge();
                    }
                    Packet::KnowledgeInit(knowledge) => self.model.knowledge = knowledge,
                    Packet::RamInit(ram) => {
                        self.model.ram = ram;
                        self.model.update_knowledge();
                    }
                    Packet::UpdateCell(cell_id, value) => if let Some(ref connection) = self.connection {
                        if let Some(app) = connection.firebase_app() {
                            app.set_cell(&mut self.model, cell_id, value).expect("failed to apply state change from Firebase"); //TODO show error message instead of panicking?
                        }
                    },
                    Packet::ModelInit(model) => {
                        self.model = model;
                        self.model.update_knowledge();
                    }
                    Packet::ModelDelta(delta) => {
                        self.model += delta;
                        self.model.update_knowledge();
                    }
                }
                if self.flags.show_available_checks {
                    let rando = self.rando.clone();
                    let model = self.model.clone();
                    return async move {
                        tokio::task::spawn_blocking(move || match checks::status(&*rando, &model) {
                            Ok(status) => Message::UpdateAvailableChecks(status),
                            Err(e) => Message::CheckStatusErrorStatic(e),
                        }).await.expect("status checks task panicked")
                    }.into()
                }
            }
            Message::ResetUpdateState => self.update_check = UpdateCheckState::Unknown(button::State::default()),
            Message::RightClick => {
                if self.menu_state.is_none() {
                    if let Some(cell) = self.layout().cell_at(self.last_cursor_pos, self.notification.is_none()) {
                        if cell.kind().right_click(self.connection.as_ref().map_or(true, |connection| connection.can_change_state()), &mut self.model) {
                            self.menu_state = Some(MenuState::default());
                        } else if let Some(ref connection) = self.connection {
                            if connection.can_change_state() {
                                let send_fut = connection.set_state(&self.model);
                                return async move {
                                    match send_fut.await {
                                        Ok(()) => Message::Nop,
                                        Err(e) => Message::ConnectionError(e.into()),
                                    }
                                }.into()
                            }
                        }
                    }
                }
            }
            Message::SetAutoUpdateCheck(enable) => {
                self.config.as_mut().expect("config not yet loaded").auto_update_check = Some(enable);
                return self.save_config()
            }
            Message::SetConnection(connection) => self.connection = Some(connection),
            Message::SetConnectionKind(kind) => if let Some(MenuState { ref mut connection_params, .. }) = self.menu_state {
                connection_params.set_kind(kind);
            }
            Message::SetMedOrder(med_order) => {
                self.config.as_mut().expect("config not yet loaded").med_order = med_order;
                return self.save_config()
            }
            Message::SetPasscode(new_passcode) => if let Some(MenuState { connection_params: ConnectionParams::Web { ref mut passcode, .. }, .. }) = self.menu_state {
                *passcode = new_passcode;
            },
            Message::SetPort(new_port) => if let Some(MenuState { connection_params: ConnectionParams::RetroArch { ref mut port, .. }, .. }) = self.menu_state {
                if let Ok(new_port) = new_port.parse() {
                    *port = new_port;
                }
            },
            Message::SetUrl(new_url) => if let Some(MenuState { connection_params: ConnectionParams::Web { ref mut url, .. }, .. }) = self.menu_state {
                *url = new_url;
            },
            Message::SetWarpSongOrder(warp_song_order) => {
                self.config.as_mut().expect("config not yet loaded").warp_song_order = warp_song_order;
                return self.save_config()
            }
            Message::UpdateAvailableChecks(checks) => self.checks = checks,
            Message::UpdateCheck => {
                self.update_check = UpdateCheckState::Checking;
                let client = self.http_client.clone();
                return async move {
                    match check_for_updates(&client).await {
                        Ok(update_available) => Message::UpdateCheckComplete(update_available),
                        Err(e) => Message::UpdateCheckError(e),
                    }
                }.into()
            }
            Message::UpdateCheckComplete(Some(new_ver)) => self.update_check = UpdateCheckState::UpdateAvailable {
                new_ver,
                update_btn: button::State::default(),
                reset_btn: button::State::default(),
            },
            Message::UpdateCheckComplete(None) => self.update_check = UpdateCheckState::NoUpdateAvailable,
            Message::UpdateCheckError(e) => self.update_check = UpdateCheckState::Error {
                e,
                reset_btn: button::State::default(),
            },
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Message<ootr_static::Rando>> {
        let layout = self.layout();
        let mut cell_buttons = self.cell_buttons.iter_mut();

        macro_rules! cell {
            ($cell:expr) => {{
                $cell.id.view(&self.model, cell_buttons.next().expect("not enough cell button states"))
            }}
        }

        if let Some(ref mut menu_state) = self.menu_state {
            return Column::new()
                .push(Row::new()
                    .push(Button::new(&mut menu_state.dismiss_btn, Text::new("Back")).on_press(Message::CloseMenu))
                    .push(Space::with_width(Length::Fill))
                    .push(self.update_check.view())
                )
                .push(Text::new("Preferences").size(24).width(Length::Fill).horizontal_alignment(HorizontalAlignment::Center))
                .push(Text::new("Medallion order:"))
                .push(PickList::new(&mut menu_state.med_order, ElementOrder::into_enum_iter().collect_vec(), self.config.as_ref().map(|cfg| cfg.med_order), Message::SetMedOrder))
                .push(Text::new("Warp song order:"))
                .push(PickList::new(&mut menu_state.warp_song_order, ElementOrder::into_enum_iter().collect_vec(), self.config.as_ref().map(|cfg| cfg.warp_song_order), Message::SetWarpSongOrder))
                .push(Text::new("Connect").size(24).width(Length::Fill).horizontal_alignment(HorizontalAlignment::Center))
                //TODO replace connection options with “current connection” info when connected
                .push(PickList::new(&mut menu_state.connection_kind, ConnectionKind::into_enum_iter().collect_vec(), Some(menu_state.connection_params.kind()), Message::SetConnectionKind))
                .push(menu_state.connection_params.view())
                .push(Button::new(&mut menu_state.connect_btn, Text::new(if self.connection.is_some() { "Disconnect" } else { "Connect" })).on_press(Message::Connect))
                .padding(5)
                .into()
        }
        let mut cells = layout.cells().into_iter();
        let view = Column::new()
            .push(Row::new()
                .push(cell!(cells.next().unwrap()))
                .push(cell!(cells.next().unwrap()))
                .push(cell!(cells.next().unwrap()))
                .push(cell!(cells.next().unwrap()))
                .push(cell!(cells.next().unwrap()))
                .push(cell!(cells.next().unwrap()))
                .spacing(10)
            )
            .push(Row::new()
                .push(cell!(cells.next().unwrap()))
                .push(cell!(cells.next().unwrap()))
                .push(cell!(cells.next().unwrap()))
                .push(cell!(cells.next().unwrap()))
                .push(cell!(cells.next().unwrap()))
                .push(cell!(cells.next().unwrap()))
                .spacing(10)
            );
        let view = if let Some(ref config) = self.config {
            if let UpdateCheckState::UpdateAvailable { ref new_ver, ref mut update_btn, ref mut reset_btn } = self.update_check {
                view.push(Text::new(format!("OoT Tracker {} is available — you have {}", new_ver, env!("CARGO_PKG_VERSION")))
                    .color([1.0, 1.0, 1.0])
                    .width(Length::Fill)
                    .horizontal_alignment(HorizontalAlignment::Center)
                )
                .push(Row::new()
                    .push(Button::new(update_btn, Text::new("Update")).on_press(Message::InstallUpdate))
                    .push(Button::new(reset_btn, Text::new("Dismiss")).on_press(Message::ResetUpdateState))
                    .spacing(5)
                )
            } else if config.auto_update_check.is_some() {
                let mut row2 = Vec::with_capacity(4);
                let mut stone_locs = Vec::with_capacity(3);
                row2.push(cells.next().unwrap());
                row2.push(cells.next().unwrap());
                stone_locs.push(cells.next().unwrap());
                stone_locs.push(cells.next().unwrap());
                stone_locs.push(cells.next().unwrap());
                row2.push(cells.next().unwrap());
                row2.push(cells.next().unwrap());
                let mut view = view.push(Row::new()
                        .push(cell!(row2[0]))
                        .push(cell!(row2[1]))
                        .push(Column::new()
                            .push(cell!(stone_locs[0]))
                            .push(cell!(cells.next().unwrap()))
                            .spacing(10)
                        )
                        .push(Column::new()
                            .push(cell!(stone_locs[1]))
                            .push(cell!(cells.next().unwrap()))
                            .spacing(10)
                        )
                        .push(Column::new()
                            .push(cell!(stone_locs[2]))
                            .push(cell!(cells.next().unwrap()))
                            .spacing(10)
                        )
                        .push(cell!(row2[2]))
                        .push(cell!(row2[3]))
                        .spacing(10)
                    );
                for i in 0..5 {
                    if i == 3 && self.notification.is_some() { break }
                    view = view.push(Row::new()
                        .push(cell!(cells.next().unwrap()))
                        .push(cell!(cells.next().unwrap()))
                        .push(cell!(cells.next().unwrap()))
                        .push(cell!(cells.next().unwrap()))
                        .push(cell!(cells.next().unwrap()))
                        .push(cell!(cells.next().unwrap()))
                        .spacing(10)
                    );
                }
                if let Some((is_temp, ref notification)) = self.notification {
                    let mut row = Row::new()
                        .push(Text::new(format!("{}", notification)).color([1.0, 1.0, 1.0]).width(Length::Fill));
                    if !is_temp {
                        row = row.push(Button::new(&mut self.dismiss_notification_button, Text::new("X").color([1.0, 0.0, 0.0])).on_press(Message::DismissNotification));
                    }
                    view.push(row.height(Length::Units(101)))
                } else {
                    view
                }
            } else {
                view.push(Text::new("Check for updates on startup?")
                    .color([1.0, 1.0, 1.0])
                    .width(Length::Fill)
                    .horizontal_alignment(HorizontalAlignment::Center)
                )
                .push(Row::new()
                    .push(Button::new(&mut self.enable_update_checks_button, Text::new("Yes")).on_press(Message::SetAutoUpdateCheck(true)))
                    .push(Button::new(&mut self.disable_update_checks_button, Text::new("No")).on_press(Message::SetAutoUpdateCheck(false)))
                    .spacing(5)
                )
            }
        } else {
            view.push(Text::new("Welcome to the OoT tracker!\nTo change settings, right-click a Medallion.")
                    .color([1.0, 1.0, 1.0])
                    .width(Length::Fill)
                    .horizontal_alignment(HorizontalAlignment::Center)
                )
                .push(Button::new(&mut self.dismiss_welcome_screen_button, Text::new("OK")).on_press(Message::DismissWelcomeScreen))
        };
        let items_container = Container::new(Container::new(view.spacing(10).padding(5))
                .width(Length::Units(WIDTH as u16))
                .height(Length::Units(HEIGHT as u16))
            )
            .width(Length::Fill)
            .style(ContainerStyle)
            .width(if self.flags.show_logic_tracker { Length::Units(WIDTH as u16 + 2) } else { Length::Fill })
            .height(if self.flags.show_available_checks { Length::Units(HEIGHT as u16 + 2) } else { Length::Fill })
            .into();
        let left_column = if self.flags.show_available_checks {
            let check_status_map = self.checks.iter().map(|(check, status)| (status, check)).into_group_map();
            let mut col = Column::new()
                .push(Text::new(format!("{} checked", lang::plural(check_status_map.get(&CheckStatus::Checked).map_or(0, Vec::len), "location"))))
                .push(Text::new(format!("{} currently inaccessible", lang::plural(check_status_map.get(&CheckStatus::NotYetReachable).map_or(0, Vec::len), "location"))))
                .push(Text::new(format!("{} accessible:", lang::plural(check_status_map.get(&CheckStatus::Reachable).map_or(0, Vec::len), "location"))));
            for check in check_status_map.get(&CheckStatus::Reachable).into_iter().flatten() {
                col = col.push(Text::new(format!("{}", check)));
            }
            Column::new()
                .push(items_container)
                .push(col)
                .into()
        } else {
            items_container
        };
        if self.flags.show_logic_tracker {
            Row::new()
                .push(left_column)
                .push(self.logic.view(&self.rando).map(Message::Logic))
                .width(Length::Fill)
                .into()
        } else {
            left_column
        }
    }

    fn subscription(&self) -> iced::Subscription<Message<ootr_static::Rando>> {
        Subscription::batch(vec![
            iced_native::subscription::events_with(|event, status| match (event, status) {
                (iced_native::Event::Keyboard(iced_native::keyboard::Event::ModifiersChanged(modifiers)), _) => Some(Message::KeyboardModifiers(modifiers)),
                (iced_native::Event::Mouse(iced_native::mouse::Event::CursorMoved { position }), _) => Some(Message::MouseMoved(position.into())),
                (iced_native::Event::Mouse(iced_native::mouse::Event::ButtonReleased(iced_native::mouse::Button::Right)), iced_native::event::Status::Ignored) => Some(Message::RightClick),
                _ => None,
            }),
            Subscription::from_recipe(subscriptions::Subscription::new(self.connection.clone().unwrap_or_else(|| Arc::new(net::NullConnection)))),
        ])
    }
}

#[derive(Debug, From, FromArc, Clone)]
enum ConnectionError {
    ExtraPathSegments,
    MissingRoomName,
    #[from]
    Net(net::Error),
    #[from_arc]
    Reqwest(Arc<reqwest::Error>),
    UnsupportedHost(Option<url::Host<String>>),
    UnsupportedRoomKind(String),
    #[from]
    UrlParse(url::ParseError),
    #[from]
    Write(async_proto::WriteError),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::ExtraPathSegments => write!(f, "too many path segments in URL"),
            ConnectionError::MissingRoomName => write!(f, "missing room name"),
            ConnectionError::Net(e) => e.fmt(f),
            ConnectionError::Reqwest(e) => if let Some(url) = e.url() {
                write!(f, "HTTP error at {}: {}", url, e)
            } else {
                write!(f, "HTTP error: {}", e)
            },
            ConnectionError::UnsupportedHost(Some(host)) => write!(f, "the tracker at {} is not (yet) supported", host),
            ConnectionError::UnsupportedHost(None) => write!(f, "this kind of connection is not supported"),
            ConnectionError::UnsupportedRoomKind(kind) => write!(f, "“{}” rooms are not (yet) supported", kind),
            ConnectionError::UrlParse(e) => e.fmt(f),
            ConnectionError::Write(e) => e.fmt(f),
        }
    }
}

async fn connect(params: ConnectionParams, state: ModelState) -> Result<Arc<dyn Connection>, ConnectionError> {
    let connection = match params {
        ConnectionParams::RetroArch { port, .. } => Arc::new(net::RetroArchConnection { port }),
        ConnectionParams::Web { url, passcode, .. } => {
            let url = url.parse::<Url>()?;

            macro_rules! firebase_host {
                ($ty:ident) => {{
                    let mut path_segments = url.path_segments().into_iter().flatten().fuse();
                    let name = match (path_segments.next(), path_segments.next(), path_segments.next()) {
                        (None, _, _) => return Err(ConnectionError::MissingRoomName),
                        (Some(room_name), None, _) |
                        (Some(_), Some(room_name), None) => room_name.to_owned(),
                        (Some(_), Some(_), Some(_)) => return Err(ConnectionError::ExtraPathSegments),
                    };
                    let session = firebase::Session::new(firebase::$ty).await?;
                    Arc::new(net::FirebaseConnection::new(firebase::Room { session, name, passcode })) as Arc<dyn Connection>
                }};
            }

            match url.host() {
                Some(url::Host::Domain("oot-tracker.web.app")) | Some(url::Host::Domain("oot-tracker.firebaseapp.com")) => firebase_host!(OldRestreamTracker),
                Some(url::Host::Domain("ootr-tracker.web.app")) | Some(url::Host::Domain("ootr-tracker.firebaseapp.com")) => firebase_host!(RestreamTracker),
                Some(url::Host::Domain("ootr-random-settings-tracker.web.app")) | Some(url::Host::Domain("ootr-random-settings-tracker.firebaseapp.com")) => firebase_host!(RslItemTracker),
                //TODO support for rsl-settings-tracker.web.app
                Some(url::Host::Domain("oottracker.fenhl.net")) => {
                    let mut path_segments = url.path_segments().into_iter().flatten().fuse();
                    match path_segments.next() {
                        None => return Err(ConnectionError::MissingRoomName),
                        Some("room") => Arc::new(net::WebConnection::new(path_segments.next().ok_or(ConnectionError::MissingRoomName)?).await?),
                        Some("restream") => return Err(ConnectionError::UnsupportedRoomKind(format!("restream"))), //TODO support for single-player restream room connections
                        Some(room_kind) => return Err(ConnectionError::UnsupportedRoomKind(room_kind.to_owned())),
                    }
                }
                host => return Err(ConnectionError::UnsupportedHost(host.map(|host| host.to_owned()))),
            }
        }
    };
    if connection.can_change_state() {
        connection.set_state(&state).await?;
    }
    Ok(connection)
}

#[derive(Debug)]
enum UpdateCheckState {
    Unknown(button::State),
    Checking,
    Error {
        e: UpdateCheckError,
        reset_btn: button::State,
    },
    UpdateAvailable {
        new_ver: Version,
        update_btn: button::State,
        reset_btn: button::State,
    },
    NoUpdateAvailable,
    Installing,
}

impl UpdateCheckState {
    fn view(&mut self) -> Element<'_, Message<ootr_static::Rando>> {
        match self {
            UpdateCheckState::Unknown(check_btn) => Row::new()
                .push(Text::new(concat!("version ", env!("CARGO_PKG_VERSION"))))
                .push(Button::new(check_btn, Text::new("Check for Updates")).on_press(Message::UpdateCheck))
                .into(),
            UpdateCheckState::Checking => Text::new(concat!("version ", env!("CARGO_PKG_VERSION"), " — checking for updates…")).into(),
            UpdateCheckState::Error { e, reset_btn } => Row::new()
                .push(Text::new(format!("error checking for updates: {}", e)))
                .push(Button::new(reset_btn, Text::new("Dismiss")).on_press(Message::ResetUpdateState))
                .into(),
            UpdateCheckState::UpdateAvailable { new_ver, update_btn, .. } => Row::new()
                .push(Text::new(format!("{} is available — you have {}", new_ver, env!("CARGO_PKG_VERSION"))))
                .push(Button::new(update_btn, Text::new("Update")).on_press(Message::InstallUpdate))
                .into(),
            UpdateCheckState::NoUpdateAvailable => Text::new(concat!("version ", env!("CARGO_PKG_VERSION"), " — up to date")).into(),
            UpdateCheckState::Installing => Text::new(concat!("version ", env!("CARGO_PKG_VERSION"), " — Installing update…")).into(),
        }
    }
}

#[derive(Debug, Clone, From, FromArc)]
enum UpdateCheckError {
    #[from_arc]
    Io(Arc<io::Error>),
    #[cfg(target_os = "macos")]
    MissingAsset,
    NoReleases,
    #[from_arc]
    Reqwest(Arc<reqwest::Error>),
    #[from]
    SemVer(SemVerError),
    #[from]
    Ui(ui::Error),
}

impl fmt::Display for UpdateCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateCheckError::Io(e) => write!(f, "I/O error: {}", e),
            #[cfg(target_os = "macos")]
            UpdateCheckError::MissingAsset => write!(f, "release does not have a download for this platform"),
            UpdateCheckError::NoReleases => write!(f, "there are no released versions"),
            UpdateCheckError::Reqwest(e) => if let Some(url) = e.url() {
                write!(f, "HTTP error at {}: {}", url, e)
            } else {
                write!(f, "HTTP error: {}", e)
            },
            UpdateCheckError::SemVer(e) => e.fmt(f),
            UpdateCheckError::Ui(e) => e.fmt(f),
        }
    }
}

async fn check_for_updates(client: &reqwest::Client) -> Result<Option<Version>, UpdateCheckError> {
    let repo = Repo::new("fenhl", "oottracker");
    if let Some(release) = repo.latest_release(client).await? {
        let new_ver = release.version()?;
        Ok(if new_ver > Version::parse(env!("CARGO_PKG_VERSION"))? { Some(new_ver) } else { None })
    } else {
        Err(UpdateCheckError::NoReleases)
    }
}

async fn run_updater(#[cfg_attr(windows, allow(unused))] client: &reqwest::Client) -> Result<Never, UpdateCheckError> {
    #[cfg(target_os = "macos")] { //TODO use Sparkle or similar on macOS for automation?
        let release = Repo::new("fenhl", "oottracker").latest_release(&client).await?.ok_or(UpdateCheckError::NoReleases)?;
        let (asset,) = release.assets.into_iter()
            .filter(|asset| asset.name.ends_with("-mac.dmg"))
            .collect_tuple().ok_or(UpdateCheckError::MissingAsset)?;
        let response = client.get(asset.browser_download_url).send().await?.error_for_status()?;
        let project_dirs = dirs()?;
        let cache_dir = project_dirs.cache_dir();
        fs::create_dir_all(cache_dir).await?;
        let dmg_download_path = cache_dir.join(asset.name);
        {
            let mut data = response.bytes_stream();
            let mut dmg_file = File::create(&dmg_download_path).await?;
            while let Some(chunk) = data.try_next().await? {
                dmg_file.write_all(chunk.as_ref()).await?;
            }
        }
        sleep(Duration::from_secs(1)).await; // to make sure the download is closed
        std::process::Command::new("open").arg(dmg_download_path).spawn()?;
        std::process::exit(0)
    }
    #[cfg(target_os = "windows")] {
        let project_dirs = dirs()?;
        let cache_dir = project_dirs.cache_dir();
        fs::create_dir_all(cache_dir).await?;
        let updater_path = cache_dir.join("updater.exe");
        #[cfg(target_arch = "x86_64")] let updater_data = include_bytes!("../../../target/x86_64-pc-windows-msvc/release/oottracker-updater.exe");
        fs::write(&updater_path, updater_data).await?;
        let _ = std::process::Command::new(updater_path).arg(env::current_exe()?).spawn()?;
        std::process::exit(0)
    }
}

#[derive(Debug, Default, StructOpt)]
struct Args {
    #[structopt(long = "checks")]
    show_available_checks: bool,
    #[structopt(long = "logic")]
    show_logic_tracker: bool,
}

#[derive(From)]
enum Error {
    Iced(iced::Error),
    Icon(iced::window::icon::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Iced(e) => e.fmt(f),
            Error::Icon(e) => write!(f, "failed to set app icon: {}", e),
        }
    }
}

#[wheel::main]
fn main(args: Args) -> Result<(), Error> {
    let icon = images::icon::<DynamicImage>().to_rgba8();
    State::run(Settings {
        window: window::Settings {
            size: (WIDTH + if args.show_logic_tracker { 800 } else { 0 }, HEIGHT + if args.show_logic_tracker || args.show_available_checks { 400 } else { 0 }),
            min_size: Some((WIDTH, HEIGHT)),
            max_size: if args.show_logic_tracker {
                None
            } else if args.show_available_checks {
                Some((WIDTH, u32::MAX))
            } else {
                Some((WIDTH, HEIGHT))
            },
            resizable: args.show_logic_tracker || args.show_available_checks,
            icon: Some(Icon::from_rgba(icon.as_flat_samples().as_slice().to_owned(), icon.width(), icon.height())?),
            ..window::Settings::default()
        },
        flags: args,
        ..Settings::default()
    })?;
    Ok(())
}
