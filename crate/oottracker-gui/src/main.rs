#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use {
    std::{
        collections::HashMap,
        fmt,
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
    smart_default::SmartDefault,
    structopt::StructOpt,
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
        net::{
            self,
            Connection,
        },
        proto::Packet,
        save::*,
        ui::{
            *,
            TrackerCellKind::*,
        },
    },
};

mod lang;
mod logic;
mod subscriptions;

const CELL_SIZE: u16 = 50;
const STONE_SIZE: u16 = 33;
const MEDALLION_LOCATION_HEIGHT: u16 = 18;
const STONE_LOCATION_HEIGHT: u16 = 12;
const WIDTH: u32 = CELL_SIZE as u32 * 6 + 7; // 6 images, each 50px wide, plus 1px spacing
const HEIGHT: u32 = MEDALLION_LOCATION_HEIGHT as u32 + CELL_SIZE as u32 * 7 + 9; // dungeon reward location text, 18px high, and 7 images, each 50px high, plus 1px spacing

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
            BossKey { .. } | FortressMq | Mq(_) | TrackerCellKind::SmallKeys { .. } | SongCheck { .. } => unimplemented!(),
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
                BigPoeTriforce | BossKey { .. } | FortressMq | Mq(_) | TrackerCellKind::SmallKeys { .. } | SongCheck { .. } => unimplemented!(),
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
                BigPoeTriforce | BossKey { .. } | FortressMq | Mq(_) | TrackerCellKind::SmallKeys { .. } | SongCheck { .. } => unimplemented!(),
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
        if y <= f32::from(MEDALLION_LOCATION_HEIGHT) + 1.0 {
            for (i, med) in self.meds.into_iter().enumerate() {
                if x <= (f32::from(CELL_SIZE) + 1.0) * (i as f32 + 1.0) {
                    return Some(TrackerCellId::med_location(med))
                }
            }
            return None
        }
        if y <= f32::from(MEDALLION_LOCATION_HEIGHT) + f32::from(CELL_SIZE) + 2.0 {
            for (i, med) in self.meds.into_iter().enumerate() {
                if x <= (f32::from(CELL_SIZE) + 1.0) * (i as f32 + 1.0) {
                    return Some(TrackerCellId::from(med))
                }
            }
            return None
        }
        if y <= f32::from(MEDALLION_LOCATION_HEIGHT) + f32::from(CELL_SIZE) * 2.0 + 3.0 {
            return if x <= f32::from(CELL_SIZE) + 1.0 { Some(self.row2[0]) }
            else if x <= f32::from(CELL_SIZE) * 2.0 + 2.0 { Some(self.row2[1]) }
            else if x <= f32::from(CELL_SIZE) * 2.0 + f32::from(STONE_SIZE) + 3.0 {
                Some(if y <= f32::from(MEDALLION_LOCATION_HEIGHT) + f32::from(CELL_SIZE) + f32::from(STONE_LOCATION_HEIGHT) + 3.0 {
                    TrackerCellId::KokiriEmeraldLocation
                } else {
                    TrackerCellId::KokiriEmerald
                })
            } else if x <= f32::from(CELL_SIZE) * 2.0 + f32::from(STONE_SIZE) * 2.0 + 4.0 {
                Some(if y <= f32::from(MEDALLION_LOCATION_HEIGHT) + f32::from(CELL_SIZE) + f32::from(STONE_LOCATION_HEIGHT) + 3.0 {
                    TrackerCellId::GoronRubyLocation
                } else {
                    TrackerCellId::GoronRuby
                })
            } else if x <= f32::from(CELL_SIZE) * 2.0 + f32::from(STONE_SIZE) * 3.0 + 5.0 {
                Some(if y <= f32::from(MEDALLION_LOCATION_HEIGHT) + f32::from(CELL_SIZE) + f32::from(STONE_LOCATION_HEIGHT) + 3.0 {
                    TrackerCellId::ZoraSapphireLocation
                } else {
                    TrackerCellId::ZoraSapphire
                })
            }
            else if x <= f32::from(CELL_SIZE) * 3.0 + f32::from(STONE_SIZE) * 3.0 + 6.0 { Some(self.row2[2]) }
            else if x <= f32::from(CELL_SIZE) * 4.0 + f32::from(STONE_SIZE) * 3.0 + 7.0 { Some(self.row2[3]) }
            else { None }
        }
        for (row_idx, row) in self.rest.iter().enumerate() {
            if !include_songs && row_idx == 3 { return None }
            if y <= f32::from(MEDALLION_LOCATION_HEIGHT) + f32::from(CELL_SIZE) * (row_idx as f32 + 3.0) + row_idx as f32 + 4.0 {
                for (cell_idx, &cell) in row.iter().enumerate() {
                    if x <= (f32::from(CELL_SIZE) + 1.0) * (cell_idx as f32 + 1.0) { return Some(cell) }
                }
                return None
            }
        }
        if y <= f32::from(MEDALLION_LOCATION_HEIGHT) + f32::from(CELL_SIZE) * 7.0 + 8.0 {
            for (i, med) in self.warp_songs.into_iter().enumerate() {
                if x <= (f32::from(CELL_SIZE) + 1.0) * (i as f32 + 1.0) {
                    return Some(TrackerCellId::warp_song(med))
                }
            }
            return None
        }
        None
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""), Clone(bound = ""))]
enum Message<R: Rando> {
    CheckStatusErrorStatic(CheckStatusError<R>),
    ClientDisconnected,
    CloseMenu,
    ConfigError(oottracker::ui::Error),
    ConnectionError(ConnectionError),
    Connect,
    DismissNotification,
    DismissWelcomeScreen,
    KeyboardModifiers(KeyboardModifiers),
    LeftClick(TrackerCellId),
    LoadConfig(Config),
    Logic(logic::Message<R>),
    MissingConfig,
    MouseMoved([f32; 2]),
    Nop,
    Packet(Packet),
    RightClick,
    SetMedOrder(ElementOrder),
    SetPasscode(String),
    SetConnection(Arc<dyn Connection>),
    SetConnectionKind(ConnectionKind),
    SetPort(String),
    SetUrl(String),
    SetWarpSongOrder(ElementOrder),
    UpdateAvailableChecks(HashMap<Check<R>, CheckStatus>),
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
    config: Config,
    connection: Option<Arc<dyn Connection>>,
    keyboard_modifiers: KeyboardModifiers,
    last_cursor_pos: [f32; 2],
    dismiss_welcome_screen_button: Option<button::State>,
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
            TrackerLayout::new_auto(&self.config)
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
        let config = self.config.clone();
        async move {
            match config.save().await {
                Ok(()) => Message::Nop,
                Err(e) => Message::ConfigError(e),
            }
        }.into()
    }
}

impl Default for State<ootr_static::Rando> {
    fn default() -> State<ootr_static::Rando> {
        State {
            flags: Args::default(),
            config: Config::default(),
            connection: None,
            keyboard_modifiers: KeyboardModifiers::default(),
            last_cursor_pos: [0.0, 0.0],
            dismiss_welcome_screen_button: None,
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
                Ok(None) => Message::MissingConfig,
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
                self.dismiss_welcome_screen_button = None;
                return self.save_config()
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
                0 => self.config = config,
                v => unimplemented!("config version from the future: {}", v),
            },
            Message::Logic(msg) => return self.logic.update(msg),
            Message::MissingConfig => self.dismiss_welcome_screen_button = Some(button::State::default()),
            Message::MouseMoved(pos) => self.last_cursor_pos = pos,
            Message::Nop => {}
            Message::Packet(packet) => {
                match packet {
                    Packet::Goodbye => unreachable!(), // Goodbye is not yielded from proto::read
                    Packet::RamInit(ram) => {
                        self.model.ram = ram;
                        self.model.update_knowledge();
                    }
                    Packet::SaveDelta(delta) => {
                        self.model.ram.save = &self.model.ram.save + &delta;
                        self.model.update_knowledge();
                    }
                    Packet::SaveInit(save) => {
                        self.model.ram.save = save;
                        self.model.update_knowledge();
                    }
                    Packet::KnowledgeInit(knowledge) => self.model.knowledge = knowledge,
                    Packet::UpdateCell(cell_id, value) => if let Some(ref connection) = self.connection {
                        if let Some(app) = connection.firebase_app() {
                            app.set_cell(&mut self.model, cell_id, value).expect("failed to apply state change from Firebase"); //TODO show error message instead of panicking?
                        }
                    },
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
            Message::SetConnection(connection) => self.connection = Some(connection),
            Message::SetConnectionKind(kind) => if let Some(MenuState { ref mut connection_params, .. }) = self.menu_state {
                connection_params.set_kind(kind);
            }
            Message::SetMedOrder(med_order) => {
                self.config.med_order = med_order;
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
                self.config.warp_song_order = warp_song_order;
                return self.save_config()
            }
            Message::UpdateAvailableChecks(checks) => self.checks = checks,
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Message<ootr_static::Rando>> {
        let layout = self.layout();
        let mut cell_buttons = self.cell_buttons.iter_mut();

        macro_rules! cell {
            ($cell:expr) => {{
                $cell.view(&self.model, cell_buttons.next().expect("not enough cell button states"))
            }}
        }

        if let Some(ref mut menu_state) = self.menu_state {
            return Column::new()
                .push(Row::new()
                    .push(Button::new(&mut menu_state.dismiss_btn, Text::new("Back")).on_press(Message::CloseMenu))
                    .push(Text::new(concat!("version ", env!("CARGO_PKG_VERSION"))).horizontal_alignment(HorizontalAlignment::Right).width(Length::Fill))
                )
                .push(Text::new("Preferences").size(24).width(Length::Fill).horizontal_alignment(HorizontalAlignment::Center))
                .push(Text::new("Medallion order:"))
                .push(PickList::new(&mut menu_state.med_order, ElementOrder::into_enum_iter().collect_vec(), Some(self.config.med_order), Message::SetMedOrder))
                .push(Text::new("Warp song order:"))
                .push(PickList::new(&mut menu_state.warp_song_order, ElementOrder::into_enum_iter().collect_vec(), Some(self.config.warp_song_order), Message::SetWarpSongOrder))
                .push(Text::new("Connect").size(24).width(Length::Fill).horizontal_alignment(HorizontalAlignment::Center))
                .push(PickList::new(&mut menu_state.connection_kind, ConnectionKind::into_enum_iter().collect_vec(), Some(menu_state.connection_params.kind()), Message::SetConnectionKind))
                .push(menu_state.connection_params.view())
                .push(Button::new(&mut menu_state.connect_btn, Text::new(if self.connection.is_some() { "Disconnect" } else { "Connect" })).on_press(Message::Connect))
                .into()
        }
        let mut med_locations = Row::new();
        let mut meds = Row::new();
        for med in layout.meds {
            med_locations = med_locations.push(cell!(TrackerCellId::med_location(med)));
            meds = meds.push(cell!(TrackerCellId::from(med)));
        }
        let view = Column::new()
            .push(med_locations.spacing(1))
            .push(meds.spacing(1));
        let view = if let Some(ref mut dismiss_button) = self.dismiss_welcome_screen_button {
            view.push(Text::new("Welcome to the OoT tracker!\nTo change settings, right-click a Medallion.")
                    .color([1.0, 1.0, 1.0])
                    .width(Length::Fill)
                    .horizontal_alignment(HorizontalAlignment::Center)
                )
                .push(Button::new(dismiss_button, Text::new("OK")).on_press(Message::DismissWelcomeScreen))
        } else {
            let mut view = view.push(Row::new()
                    .push(cell!(layout.row2[0]))
                    .push(cell!(layout.row2[1]))
                    .push(Column::new()
                        .push(cell!(TrackerCellId::KokiriEmeraldLocation))
                        .push(cell!(TrackerCellId::KokiriEmerald))
                        .spacing(1)
                    )
                    .push(Column::new()
                        .push(cell!(TrackerCellId::GoronRubyLocation))
                        .push(cell!(TrackerCellId::GoronRuby))
                        .spacing(1)
                    )
                    .push(Column::new()
                        .push(cell!(TrackerCellId::ZoraSapphireLocation))
                        .push(cell!(TrackerCellId::ZoraSapphire))
                        .spacing(1)
                    )
                    .push(cell!(layout.row2[2]))
                    .push(cell!(layout.row2[3]))
                    .spacing(1)
                );
            for (i, layout_row) in layout.rest.iter().enumerate() {
                if i == 3 && self.notification.is_some() { break }
                let mut row = Row::new();
                for cell in layout_row {
                    row = row.push(cell!(cell));
                }
                view = view.push(row.spacing(1));
            }
            if let Some((is_temp, ref notification)) = self.notification {
                let mut row = Row::new()
                    .push(Text::new(format!("{}", notification)).color([1.0, 1.0, 1.0]).width(Length::Fill));
                if !is_temp {
                    row = row.push(Button::new(&mut self.dismiss_notification_button, Text::new("X").color([1.0, 0.0, 0.0])).on_press(Message::DismissNotification));
                }
                view.push(row.height(Length::Units(101)))
            } else {
                let mut row = Row::new();
                for med in layout.warp_songs {
                    row = row.push(cell!(TrackerCellId::warp_song(med)));
                }
                view.push(row.spacing(1))
            }
        };
        let items_container = Container::new(Container::new(view.spacing(1).padding(1))
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
    #[from]
    UrlParse(url::ParseError),
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
            ConnectionError::UrlParse(e) => e.fmt(f),
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
                //TODO support for oottracker.fenhl.net
                host => return Err(ConnectionError::UnsupportedHost(host.map(|host| host.to_owned()))),
            }
        }
    };
    if connection.can_change_state() {
        connection.set_state(&state).await?;
    }
    Ok(connection)
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
