use {
    std::{
        fmt,
        path::PathBuf,
    },
    derivative::Derivative,
    enum_iterator::{
        Sequence,
        all,
    },
    iced::{
        Color,
        Command,
        Element,
        Length,
        alignment,
        widget::{
            Column,
            Row,
            Text,
            button::{
                self,
                Button,
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
    },
    itertools::Itertools as _,
    ootr::Rando,
};

#[derive(Derivative, Debug)]
#[derivative(Default)]
enum SettingsInfo {
    #[derivative(Default)]
    String(String),
    Plando(PathBuf),
    Weights(PathBuf),
}

impl SettingsInfo {
    fn kind(&self) -> SettingsInfoKind {
        match self {
            SettingsInfo::String(_) => SettingsInfoKind::String,
            SettingsInfo::Plando(_) => SettingsInfoKind::Plando,
            SettingsInfo::Weights(_) => SettingsInfoKind::Weights,
        }
    }

    fn set_kind(&mut self, new_kind: SettingsInfoKind) {
        if self.kind() == new_kind { return }
        *self = match new_kind {
            SettingsInfoKind::String => SettingsInfo::String(String::default()),
            SettingsInfoKind::Plando => SettingsInfo::Plando(PathBuf::default()),
            SettingsInfoKind::Weights => SettingsInfo::Weights(PathBuf::default()),
        }
    }
}

#[derive(Derivative, Debug, Sequence, Clone, Copy, PartialEq, Eq)]
#[derivative(Default)]
pub(crate) enum SettingsInfoKind {
    #[derivative(Default)]
    String,
    Plando,
    Weights,
}

impl fmt::Display for SettingsInfoKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsInfoKind::String => write!(f, "Settings String"),
            SettingsInfoKind::Plando => write!(f, "Plandomizer"),
            SettingsInfoKind::Weights => write!(f, "Random Weights"),
        }
    }
}

struct TextInputStyle;

impl text_input::StyleSheet for TextInputStyle {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            border_radius: 0.0,
            border_width: 1.0,
            border_color: Color::BLACK,
            ..text_input::Style::default()
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            border_radius: 0.0,
            border_width: 1.0,
            border_color: Color::BLACK,
            ..text_input::Style::default()
        }
    }

    fn hovered(&self) -> text_input::Style {
        text_input::Style {
            border_radius: 0.0,
            border_width: 1.0,
            border_color: Color::BLACK,
            ..text_input::Style::default()
        }
    }

    fn placeholder_color(&self) -> Color { Color::from_rgb(0.5, 0.5, 0.5) }
    fn value_color(&self) -> Color { Color::BLACK }
    fn selection_color(&self) -> Color { Color::from_rgb8(0x0d, 0x7a, 0xff) }
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""), Clone(bound = ""))]
pub(crate) enum Message<R: Rando> {
    EditPlandoPath(String),
    EditSettingsString(String),
    EditWeightsPath(String),
    PickRegion(R::RegionName),
    PickSettingsInfo(SettingsInfoKind),
}

#[derive(Derivative, Debug)]
#[derivative(Default)]
pub(crate) struct State<R: Rando + 'static> {
    //TODO store in model state
    #[derivative(Default(value = "R::root()"))]
    current_region: R::RegionName,
    region_pick: pick_list::State<R::RegionName>,
    save_btn: button::State,
    reset_btn: button::State,
    //TODO store in knowledge
    settings_info: SettingsInfo,
    settings_pick: pick_list::State<SettingsInfoKind>,
    settings_text: text_input::State,
    starting_inv_btns_row0: [button::State; 11],
    starting_inv_btns_row1: [button::State; 11],
    starting_inv_btns_row2: [button::State; 11],
    starting_inv_btns_row3: [button::State; 7],
    starting_inv_btns_row4: [button::State; 9],
    starting_inv_btns_row5: [button::State; 9],
    starting_inv_btns_row6: [button::State; 4],
}

impl<R: Rando + 'static> State<R> {
    pub(crate) fn update(&mut self, msg: Message<R>) -> Command<crate::Message<R>> {
        match msg {
            Message::EditPlandoPath(new_path) => if let Ok(new_path) = new_path.parse() {
                if let SettingsInfo::Plando(ref mut path) = self.settings_info {
                    *path = new_path;
                }
            },
            Message::EditSettingsString(new_string) => if let SettingsInfo::String(ref mut string) = self.settings_info {
                *string = new_string;
            },
            Message::EditWeightsPath(new_path) => if let Ok(new_path) = new_path.parse() {
                if let SettingsInfo::Weights(ref mut path) = self.settings_info {
                    *path = new_path;
                }
            },
            Message::PickRegion(new_region) => self.current_region = new_region,
            Message::PickSettingsInfo(new_info) => self.settings_info.set_kind(new_info),
        }
        Command::none()
    }

    pub(crate) fn view(&mut self, rando: &R) -> Element<'_, Message<R>> {
        let mut col = Column::new().push(Row::new()
            .push(PickList::new(
                &mut self.region_pick,
                rando.regions().expect("failed to load regions" /*TODO better error handling */).iter().map(|region| region.name.clone()).collect_vec(),
                Some(self.current_region.clone()),
                Message::PickRegion,
            ))
            .push(Button::new(&mut self.save_btn, Text::new("Save Game"))) //TODO on_press
            .push(Button::new(&mut self.reset_btn, Text::new("Reset N64"))) //TODO on_press
            .spacing(16)
        );
        match self.current_region.as_ref() {
            "Root" => col = col
                .push(Text::new("External knowledge:"))
                //TODO randomizer version (support latest release, latest Dev, latest Dev-R, and any version currently used in a major tournament or the RSL)
                .push(Row::new()
                    .push(Text::new("Settings:").height(Length::Units(30)).vertical_alignment(alignment::Vertical::Center))
                    .push(PickList::new(&mut self.settings_pick, all().collect_vec(), Some(self.settings_info.kind()), Message::PickSettingsInfo))
                    .push(match self.settings_info {
                        SettingsInfo::String(ref s) => TextInput::new(&mut self.settings_text, "Enter settings string", s, Message::EditSettingsString).padding(5).style(TextInputStyle),
                        SettingsInfo::Plando(ref path) => TextInput::new(&mut self.settings_text, "Path to plando file", &path.display().to_string(), Message::EditPlandoPath).padding(5).style(TextInputStyle), //TODO file select
                        SettingsInfo::Weights(ref path) => TextInput::new(&mut self.settings_text, "Path to weights file", &path.display().to_string(), Message::EditWeightsPath).padding(5).style(TextInputStyle), //TODO file select/preset support
                    })
                    .spacing(16)
                )
                .push(Text::new("Starting inventory:")) //TODO on_press for each row
                .push(Row::with_children(self.starting_inv_btns_row0.iter_mut().zip(vec!["Hearts", "Shadow Medallion", "Light Medallion", "Forest Medallion", "Deku Sticks", "Deku Nuts", "Bombs", "Bow", "Fire Arrows", "Din's Fire", "Bottle"]).map(|(state, label)| Button::new(state, Text::new(label)).into()).collect()))
                .push(Row::with_children(self.starting_inv_btns_row1.iter_mut().zip(vec!["Wallet/Rupees", "Spirit Medallion", "Water Medallion", "Fire Medallion", "Slingshot", "Ocarina", "Bombchus", "Hookshot", "Ice Arrows", "Farore's Wind", "Child Trade"]).map(|(state, label)| Button::new(state, Text::new(label)).into()).collect()))
                .push(Row::with_children(self.starting_inv_btns_row2.iter_mut().zip(vec!["Tokens", "Kokiri Emerald", "Goron Ruby", "Zora Sapphire", "Boomerang", "Lens of Truth", "Beans", "Hammer", "Light Arrows", "Nayru's Love", "Adult Trade"]).map(|(state, label)| Button::new(state, Text::new(label)).into()).collect()))
                .push(Row::with_children(self.starting_inv_btns_row3.iter_mut().zip(vec!["Strength", "Scale", "Magic", "Gerudo Card", "Kokiri Sword", "Adult Start", "Giant's Knife/Biggoron's Sword"]).map(|(state, label)| Button::new(state, Text::new(label)).into()).collect()))
                .push(Row::with_children(self.starting_inv_btns_row4.iter_mut().zip(vec!["Zelda's Lullaby", "Epona's Song", "Saria's Song", "Sun's Song", "Song of Time", "Song of Storms", "Deku Shield", "Hylian Shield", "Mirror Shield"]).map(|(state, label)| Button::new(state, Text::new(label)).into()).collect()))
                .push(Row::with_children(self.starting_inv_btns_row5.iter_mut().zip(vec!["Minuet", "Bolero", "Serenade", "Requiem", "Nocturne", "Prelude", "Kokiri Tunic", "Goron Tunic", "Zora Tunic"]).map(|(state, label)| Button::new(state, Text::new(label)).into()).collect()))
                .push(Row::with_children(self.starting_inv_btns_row6.iter_mut().zip(vec!["Stone of Agony", "Kokiri Boots", "Iron Boots", "Hover Boots"]).map(|(state, label)| Button::new(state, Text::new(label)).into()).collect())),
            "Temple of Time" => col = col
                .push(Text::new("TODO “read pedestal” UI")),
            "Beyond Door of Time" => col = col
                .push(Text::new("TODO replace Master Sword Pedestal location with big “age change” button (age is considered for which checks are in logic and where savewarp goes)")),
            _ => {}
        }
        col
            .push(Text::new("TODO checks"))
            .spacing(16)
            .padding(16)
            .into()
    }
}
