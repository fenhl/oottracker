#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use {
    std::{
        convert::Infallible as Never,
        env,
        fmt,
        io,
        sync::Arc,
    },
    derivative::Derivative,
    derive_more::From,
    enum_iterator::{
        Sequence,
        all,
    },
    futures::future::FutureExt as _,
    iced::{
        Application,
        Background,
        Color,
        Command,
        Element,
        Length,
        Settings,
        alignment,
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
    iced_native::{
        command::Action,
        keyboard::Modifiers as KeyboardModifiers,
    },
    image::DynamicImage,
    itertools::Itertools as _,
    semver::Version,
    tokio::fs,
    url::Url,
    wheel::FromArc,
    ootr::Rando,
    oottracker::{
        ModelState,
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

fn cell_image(cell: &TrackerCellId, state: &ModelState) -> Image {
    let kind = cell.kind();
    let CellRender { img, style, overlay } = kind.render(state);
    match (style, overlay) {
        (CellStyle::Normal, CellOverlay::None) => img.embedded::<Image>(ImageDirContext::Normal),
        (CellStyle::Normal, CellOverlay::Count { count, count_img }) => count_img.embedded(ImageDirContext::Count(count)),
        (CellStyle::Normal, CellOverlay::Image(overlay)) => img.with_overlay(&overlay).embedded(true),
        (CellStyle::Dimmed, CellOverlay::None) => img.embedded(ImageDirContext::Dimmed),
        (CellStyle::Dimmed, CellOverlay::Image(overlay)) => img.with_overlay(&overlay).embedded(false),
        (_, CellOverlay::Location { loc, style }) => loc.embedded(match style {
            LocationStyle::Normal => ImageDirContext::Normal,
            LocationStyle::Dimmed => ImageDirContext::Dimmed,
            LocationStyle::Mq => unimplemented!(),
        }),
        (CellStyle::Dimmed, CellOverlay::Count { .. }) | (CellStyle::LeftDimmed | CellStyle::RightDimmed, _) => unimplemented!(),
    }.width(Length::Units(match kind {
        TrackerCellKind::Stone(_) | TrackerCellKind::StoneLocation(_) => STONE_SIZE,
        _ => CELL_SIZE,
    }))
}

trait TrackerCellIdExt {
    fn view<'a>(&self, state: &ModelState, cell_button: &'a mut button::State) -> Element<'a, Message<ootr_static::Rando>>; //TODO allow ootr_dynamic::Rando
}

impl TrackerCellIdExt for TrackerCellId {
    fn view<'a>(&self, state: &ModelState, cell_button: &'a mut button::State) -> Element<'a, Message<ootr_static::Rando>> { //TODO allow ootr_dynamic::Rando
        Button::new(cell_button, cell_image(self, state))
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
    UpdateCheck,
    UpdateCheckComplete(Option<Version>),
    UpdateCheckError(UpdateCheckError),
}

impl<R: Rando> fmt::Display for Message<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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

#[derive(Derivative, Debug, Sequence, Clone, Copy, PartialEq, Eq)]
#[derivative(Default)]
enum ConnectionKind {
    TcpListener,
    #[derivative(Default)]
    RetroArch,
    Web,
}

impl fmt::Display for ConnectionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionKind::TcpListener => write!(f, "Project64"),
            ConnectionKind::RetroArch => write!(f, "RetroArch"),
            ConnectionKind::Web => write!(f, "web"),
        }
    }
}

#[derive(Derivative, Debug, Clone)]
#[derivative(Default)]
enum ConnectionParams {
    TcpListener,
    #[derivative(Default)]
    RetroArch {
        #[derivative(Default(value = "55355"))]
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
            ConnectionParams::TcpListener => ConnectionKind::TcpListener,
            ConnectionParams::RetroArch { .. } => ConnectionKind::RetroArch,
            ConnectionParams::Web { .. } => ConnectionKind::Web,
        }
    }

    fn set_kind(&mut self, kind: ConnectionKind) {
        if kind == self.kind() { return }
        *self = match kind {
            ConnectionKind::TcpListener => ConnectionParams::TcpListener,
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
            ConnectionParams::TcpListener => Row::new().into(),
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
            Command::single(Action::Future(async move {
                match config.save().await {
                    Ok(()) => Message::Nop,
                    Err(e) => Message::ConfigError(e),
                }
            }.boxed()))
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
        (State::from(flags), Command::single(Action::Future(async {
            match Config::new().await {
                Ok(Some(config)) => Message::LoadConfig(config),
                Ok(None) => Message::Nop,
                Err(e) => Message::ConfigError(e),
            }
        }.boxed())))
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
            Message::ClientDisconnected => if self.notification.as_ref().map_or(true, |&(is_temp, _)| is_temp) { // don't override an existing, probably more descriptive error message
                return self.notify(message)
            },
            Message::CloseMenu => self.menu_state = None,
            Message::ConfigError(_) => return self.notify(message),
            Message::Connect => if self.connection.is_some() {
                self.connection = None;
            } else {
                if let Some(ref menu_state) = self.menu_state {
                    let params = menu_state.connection_params.clone();
                    let model = self.model.clone();
                    return Command::single(Action::Future(async move {
                        match connect(params, model).await {
                            Ok(connection) => Message::SetConnection(connection),
                            Err(e) => Message::ConnectionError(e),
                        }
                    }.boxed()))
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
                return Command::single(Action::Future(async move {
                    match run_updater(&client).await {
                        Ok(never) => match never {},
                        Err(e) => Message::UpdateCheckError(e),
                    }
                }.boxed()))
            }
            Message::KeyboardModifiers(modifiers) => self.keyboard_modifiers = modifiers,
            Message::LeftClick(cell) => if cell.kind().left_click(self.connection.as_ref().map_or(true, |connection| connection.can_change_state()), self.keyboard_modifiers, &mut self.model) {
                self.menu_state = Some(MenuState::default());
            } else if let Some(ref connection) = self.connection {
                if connection.can_change_state() {
                    let send_fut = connection.set_state(&self.model);
                    return Command::single(Action::Future(async move {
                        match send_fut.await {
                            Ok(()) => Message::Nop,
                            Err(e) => Message::ConnectionError(e.into()),
                        }
                    }.boxed()))
                }
            },
            Message::LoadConfig(config) => match config.version {
                0 => {
                    let auto_update_check = config.auto_update_check;
                    self.config = Some(config);
                    if auto_update_check == Some(true) {
                        return Command::single(Action::Future(async { Message::UpdateCheck }.boxed()))
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
                        if ram.save.game_mode == GameMode::Gameplay { self.model.ram = ram }
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
            }
            Message::ResetUpdateState => self.update_check = UpdateCheckState::Unknown(button::State::default()),
            Message::RightClick => {
                if self.menu_state.is_none() {
                    if let Some(cell) = self.layout().cell_at(self.last_cursor_pos, self.notification.is_none()) {
                        if cell.kind().right_click(self.connection.as_ref().map_or(true, |connection| connection.can_change_state()), self.keyboard_modifiers, &mut self.model) {
                            self.menu_state = Some(MenuState::default());
                        } else if let Some(ref connection) = self.connection {
                            if connection.can_change_state() {
                                let send_fut = connection.set_state(&self.model);
                                return Command::single(Action::Future(async move {
                                    match send_fut.await {
                                        Ok(()) => Message::Nop,
                                        Err(e) => Message::ConnectionError(e.into()),
                                    }
                                }.boxed()))
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
            Message::UpdateCheck => {
                self.update_check = UpdateCheckState::Checking;
                let client = self.http_client.clone();
                return Command::single(Action::Future(async move {
                    match check_for_updates(&client).await {
                        Ok(update_available) => Message::UpdateCheckComplete(update_available),
                        Err(e) => Message::UpdateCheckError(e),
                    }
                }.boxed()))
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
                .push(Text::new("Preferences").size(24).width(Length::Fill).horizontal_alignment(alignment::Horizontal::Center))
                .push(Text::new("Medallion order:"))
                .push(PickList::new(&mut menu_state.med_order, all().collect_vec(), self.config.as_ref().map(|cfg| cfg.med_order), Message::SetMedOrder))
                .push(Text::new("Warp song order:"))
                .push(PickList::new(&mut menu_state.warp_song_order, all().collect_vec(), self.config.as_ref().map(|cfg| cfg.warp_song_order), Message::SetWarpSongOrder))
                .push(Text::new("Connect").size(24).width(Length::Fill).horizontal_alignment(alignment::Horizontal::Center))
                //TODO replace connection options with “current connection” info when connected
                .push(PickList::new(&mut menu_state.connection_kind, all().collect_vec(), Some(menu_state.connection_params.kind()), Message::SetConnectionKind))
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
                    .horizontal_alignment(alignment::Horizontal::Center)
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
                    .horizontal_alignment(alignment::Horizontal::Center)
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
                    .horizontal_alignment(alignment::Horizontal::Center)
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
            .height(Length::Fill)
            .into();
        if self.flags.show_logic_tracker {
            Row::new()
                .push(items_container)
                .push(self.logic.view(&self.rando).map(Message::Logic))
                .width(Length::Fill)
                .into()
        } else {
            items_container
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
    #[from_arc]
    Write(Arc<async_proto::WriteError>),
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
        ConnectionParams::TcpListener => Arc::new(net::TcpConnection) as Arc<dyn Connection>,
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
    #[from_arc]
    SemVer(Arc<semver::Error>),
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

#[derive(Debug, Default, clap::Parser)]
#[clap(version)]
struct Args {
    #[clap(long = "logic")]
    show_logic_tracker: bool,
}

#[derive(Debug, From)]
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
            size: (WIDTH + if args.show_logic_tracker { 800 } else { 0 }, HEIGHT + if args.show_logic_tracker { 400 } else { 0 }),
            min_size: Some((WIDTH, HEIGHT)),
            max_size: if args.show_logic_tracker {
                None
            } else {
                Some((WIDTH, HEIGHT))
            },
            resizable: args.show_logic_tracker,
            icon: Some(Icon::from_rgba(icon.as_flat_samples().as_slice().to_owned(), icon.width(), icon.height())?),
            ..window::Settings::default()
        },
        ..Settings::with_flags(args)
    })?;
    Ok(())
}
