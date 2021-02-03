#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use {
    std::{
        collections::{
            BTreeMap,
            BTreeSet,
        },
        env,
        fmt,
        io,
        path::PathBuf,
        sync::Arc,
    },
    enum_iterator::IntoEnumIterator,
    iced::{
        Application,
        Color,
        Command,
        Element,
        Length,
        Settings,
        widget::{
            Checkbox,
            Column,
            Radio,
            Row,
            Rule,
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
            scrollable::{
                self,
                Scrollable,
            },
            slider::{
                self,
                Slider,
            },
            text_input::{
                self,
                TextInput,
            },
        },
        window,
    },
    itertools::Itertools as _,
    rfd::AsyncFileDialog,
    serde_json::{
        Value as Json,
        json,
    },
    smart_default::SmartDefault,
    structopt::StructOpt,
    tokio::fs,
    rsl::{
        Conditional,
        Distribution,
        GenError,
        GenOptions,
        HashIcon,
        Preset,
        PresetOptions,
        Weights,
        WeightsRule,
    },
    crate::{
        config::Config,
        file::{
            FilePicker,
            Kind as _,
        },
    },
};
#[cfg(not(any(feature = "self-update", windows)))] use tokio_stream as _;
#[cfg(feature = "self-update")] use {
    std::convert::Infallible as Never,
    derive_more::From,
    semver::{
        SemVerError,
        Version,
    },
    rsl::github::Repo,
};
#[cfg(windows)] use rsl::cache_dir;
#[cfg(any(feature = "self-update", windows))] use {
    std::time::Duration,
    tokio::{
        fs::File,
        io::AsyncWriteExt as _,
        time::sleep,
    },
    tokio_stream::StreamExt as _,
    rsl::from_arc,
};

mod config;
mod file;

ootr::uses!();

#[cfg(feature = "self-update")]
#[cfg(target_os = "macos")]
const PLATFORM_SUFFIX: &str = "-mac.app";

#[derive(Debug, Clone)]
enum Message {
    AddCondition(usize, usize),
    AddConditional(usize),
    AddSetting,
    AddSettingValue(usize, Option<usize>),
    AllowedTricks(SetViewMessage),
    BrowseBaseRom,
    BrowseOutputDir,
    ChangeBaseRom(String),
    ChangeCondition(usize, usize, usize, String),
    ChangeConditionalSetting(usize, usize, String),
    ChangeOutputDir(String),
    ChangeRangeMax(usize, String),
    ChangeRangeMin(usize, String),
    ChangeSettingKind(usize, WeightsRuleKind),
    ChangeSettingName(usize, String),
    ChangeSettingValue(usize, Option<usize>, String, String),
    ChangeSettingWeight(usize, Option<usize>, String, String),
    DisabledLocations(SetViewMessage),
    #[cfg(feature = "self-update")]
    DismissUpdateError,
    GenError(GenError),
    Generate,
    #[cfg(windows)]
    InstallPython,
    #[cfg(feature = "self-update")]
    InstallUpdate,
    LoadConfig(Config),
    LoadFile,
    LoadFileError(Arc<io::Error>),
    LoadWeights(Weights),
    Nop,
    #[allow(unused)] OpenLoadPresetMenu, //TODO
    #[allow(unused)] OpenSavePresetMenu, //TODO
    #[cfg(windows)]
    PyInstallError(PyInstallError),
    RemoveCondition(usize, usize, usize),
    RemoveConditional(usize, usize),
    RemoveSettingValue(usize, Option<usize>, String),
    SaveFile,
    SaveFileError(Arc<io::Error>),
    SeedDone,
    #[cfg(feature = "self-update")]
    SetAutoUpdateCheck(bool),
    SetBaseRom(PathBuf),
    SetHashIcon0(HashIcon),
    SetHashIcon1(HashIcon),
    SetOutputDir(PathBuf),
    SetWorldCount(u8),
    SetWorldCountStr(String),
    StartingEquipment(SetViewMessage),
    StartingItems(SetViewMessage),
    StartingSongs(SetViewMessage),
    Tab(Tab),
    ToggleRandomStartingItems(bool),
    ToggleRslTricks(bool),
    ToggleStandardTricks(bool),
    #[cfg(feature = "self-update")]
    UpdateCheck,
    #[cfg(feature = "self-update")]
    UpdateCheckComplete(Option<Version>),
    #[cfg(feature = "self-update")]
    UpdateCheckError(UpdateCheckError),
}

#[cfg(feature = "self-update")]
#[derive(SmartDefault)]
enum UpdateCheckState {
    #[default]
    AskSetting {
        yes_btn: button::State,
        no_btn: button::State,
    },
    Unknown(button::State),
    Checking,
    Error {
        e: UpdateCheckError,
        reset_btn: button::State,
    },
    UpdateAvailable {
        new_ver: Version,
        update_btn: button::State,
    },
    NoUpdateAvailable,
    Installing,
}

#[cfg(feature = "self-update")]
impl UpdateCheckState {
    fn view(&mut self) -> Element<'_, Message> {
        match self {
            UpdateCheckState::AskSetting { yes_btn, no_btn } => Row::new()
                .push(Text::new("Check for updates on launch?"))
                .push(Button::new(yes_btn, Text::new("Yes")).on_press(Message::SetAutoUpdateCheck(true)))
                .push(Button::new(no_btn, Text::new("No")).on_press(Message::SetAutoUpdateCheck(false)))
                .spacing(16)
                .into(),
            UpdateCheckState::Unknown(check_btn) => Row::new()
                .push(Text::new(concat!("version ", env!("CARGO_PKG_VERSION"))))
                .push(Button::new(check_btn, Text::new("Check for Updates")).on_press(Message::UpdateCheck))
                .spacing(16)
                .into(),
            UpdateCheckState::Checking => Text::new(concat!("version ", env!("CARGO_PKG_VERSION"), " — checking for updates…")).into(),
            UpdateCheckState::Error { e, reset_btn } => Row::new()
                .push(Text::new(format!("error checking for updates: {}", e)))
                .push(Button::new(reset_btn, Text::new("Dismiss")).on_press(Message::DismissUpdateError))
                .spacing(16)
                .into(),
            UpdateCheckState::UpdateAvailable { new_ver, update_btn } => Row::new()
                .push(Text::new(format!("{} is available — you have {}", new_ver, env!("CARGO_PKG_VERSION"))))
                .push(Button::new(update_btn, Text::new("Update")).on_press(Message::InstallUpdate))
                .spacing(16)
                .into(),
            UpdateCheckState::NoUpdateAvailable => Text::new(concat!("version ", env!("CARGO_PKG_VERSION"), " — up to date")).into(),
            UpdateCheckState::Installing => Text::new(concat!("version ", env!("CARGO_PKG_VERSION"), " — Installing update…")).into(),
        }
    }
}

#[derive(Debug, SmartDefault, Clone, Copy, IntoEnumIterator, PartialEq, Eq)]
enum Tab {
    #[default]
    League,
    Solo,
    CoOp,
    Multiworld,
    Custom,
}

impl Tab {
    fn view(&self) -> Element<'_, Message> {
        Row::with_children(Tab::into_enum_iter().map(|tab|
            Radio::new(tab, tab.to_string(), Some(*self), Message::Tab).into()
        ).collect()).spacing(16).into()
    }
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tab::League => write!(f, "League"),
            Tab::Solo => write!(f, "Solo"),
            Tab::CoOp => write!(f, "Co-Op"),
            Tab::Multiworld => write!(f, "Multiworld"),
            Tab::Custom => write!(f, "Custom"),
        }
    }
}

struct WeightsState {
    data: Weights,
    //load_preset_btn: button::State,
    //save_preset_btn: button::State,
    load_file_btn: button::State,
    save_file_btn: button::State,
    hash_icon0: pick_list::State<HashIcon>,
    hash_icon1: pick_list::State<HashIcon>,
    disabled_locations: SetView,
    allowed_tricks: SetView,
    starting_items: SetView,
    starting_songs: SetView,
    starting_equipment: SetView,
    weights: Vec<WeightsRuleState>,
    add_btn: button::State,
}

impl WeightsState {
    fn set_rule_kind(&mut self, idx: usize, kind: WeightsRuleKind) {
        match (&mut self.data.weights[idx], kind) {
            (WeightsRule::Custom { .. }, WeightsRuleKind::Custom) => {}
            (rule, WeightsRuleKind::Custom) => {
                let setting = rule.setting().to_owned();
                *rule = WeightsRule::Custom { setting, values: BTreeMap::default(), conditionals: Vec::default() }; //TODO populate values from rando data (each possible value with weight 0)
            }
            (WeightsRule::Range { distribution, .. }, WeightsRuleKind::Range(dist)) => *distribution = dist,
            (rule, WeightsRuleKind::Range(distribution)) => {
                let setting = rule.setting().to_owned();
                *rule = WeightsRule::Range { setting, distribution, min: 1, max: 100 }; //TODO populate min/max from rando data (each possible value with weight 0)
            }
            (_, WeightsRuleKind::Remove) => {
                self.data.weights.remove(idx);
                self.weights.remove(idx);
            }
        }
    }

    fn view<'a>(&'a mut self, scroll: &'a mut scrollable::State, worlds_slider: &'a mut slider::State, worlds_text: &'a mut text_input::State) -> Scrollable<'a, Message> {
        let mut col = Scrollable::new(scroll)
            .push(Row::new()
                //.push(Button::new(&mut self.load_preset_btn, Text::new("Load Preset")).on_press(Message::OpenLoadPresetMenu)) //TODO
                //.push(Button::new(&mut self.save_preset_btn, Text::new("Save Preset")).on_press(Message::OpenSavePresetMenu)) //TODO
                .push(Button::new(&mut self.load_file_btn, Text::new("Load File")).on_press(Message::LoadFile))
                .push(Button::new(&mut self.save_file_btn, Text::new("Save File")).on_press(Message::SaveFile))
                .spacing(16)
            )
            .push(Rule::horizontal(16))
            .push(Row::new()
                .push(Text::new("Hash Prefix:"))
                .push(PickList::new(&mut self.hash_icon0, HashIcon::into_enum_iter().collect_vec(), Some(self.data.hash[0]), Message::SetHashIcon0))
                .push(PickList::new(&mut self.hash_icon1, HashIcon::into_enum_iter().collect_vec(), Some(self.data.hash[1]), Message::SetHashIcon1))
                .spacing(16)
            )
            .push(Rule::horizontal(16))
            .push(Row::new()
                .push(Text::new("Player Count:"))
                .push(Slider::new(worlds_slider, 1..=MAX_WORLDS, self.data.world_count, Message::SetWorldCount))
                .push(TextInput::new(worlds_text, "", &self.data.world_count.to_string(), Message::SetWorldCountStr).width(Length::Units(32)).padding(5).style(TextInputStyle))
                .push(Space::with_width(Length::Shrink)) // to avoid overlap with the scrollbar
                .spacing(16)
            )
            .push(self.disabled_locations.view(&self.data.disabled_locations))
            .push(self.allowed_tricks.view(&self.data.allowed_tricks))
            .push(Rule::horizontal(16))
            .push(Checkbox::new(self.data.random_starting_items, "Randomize Starting Items", Message::ToggleRandomStartingItems))
            .push(self.starting_items.view(&self.data.starting_items))
            .push(self.starting_songs.view(&self.data.starting_songs))
            .push(self.starting_equipment.view(&self.data.starting_equipment));
        for (idx, (rule, state)) in self.data.weights.iter().zip(&mut self.weights).enumerate() {
            col = col.push(state.view(idx, rule));
        }
        col
            .push(Rule::horizontal(16))
            .push(Button::new(&mut self.add_btn, Text::new("Add Setting")).on_press(Message::AddSetting))
    }
}

#[derive(Debug, Clone)]
enum SetViewMessage {
    Add,
    Pick(&'static str),
    Remove(String),
}

struct SetView {
    label: &'static str,
    all: &'static [&'static str],
    message: fn(SetViewMessage) -> Message,
    remove_btns: Vec<button::State>,
    new_item: Option<&'static str>,
    pick: pick_list::State<&'static str>,
    add_btn: button::State,
}

impl SetView {
    fn new(label: &'static str, all: &'static [&'static str], message: fn(SetViewMessage) -> Message, data: &BTreeSet<String>) -> SetView {
        SetView {
            new_item: all.iter().find(|&&elt| !data.contains(elt)).copied(),
            label, all, message,
            pick: pick_list::State::default(),
            add_btn: button::State::default(),
            remove_btns: vec![button::State::default(); data.len()],
        }
    }

    fn update(&mut self, data: &mut BTreeSet<String>, msg: SetViewMessage) {
        match msg {
            SetViewMessage::Add => if let Some(elt) = self.new_item {
                data.insert(elt.to_owned());
                self.new_item = self.all.iter().find(|&&elt| !data.contains(elt)).copied();
                self.remove_btns.push(button::State::default());
            },
            SetViewMessage::Pick(elt) => self.new_item = Some(elt),
            SetViewMessage::Remove(elt) => {
                data.remove(&elt);
                self.remove_btns.pop();
            }
        }
    }

    fn view(&mut self, data: &BTreeSet<String>) -> Column<'_, Message> {
        let available = self.all.iter().filter(|&&elt| !data.contains(elt)).copied().collect_vec();
        let msg = self.message;
        Column::new()
            .push(Rule::horizontal(16))
            .push(Row::new()
                .push(Text::new(self.label))
                .push(Text::new(if data.is_empty() { "(none)" } else { "" }))
                .spacing(16)
            )
            .push(Column::with_children(data.iter().zip(&mut self.remove_btns).map(|(elt, btn)| Row::new()
                .push(Text::new(elt.clone())) //TODO remove this clone
                .push(Button::new(btn, Text::new('-')).on_press(msg(SetViewMessage::Remove(elt.clone())))) //TODO remove this clone
                .spacing(16)
                .into()
            ).collect()).spacing(16))
            .push(Row::new()
                .push(PickList::new(&mut self.pick, available, self.new_item, move |elt| msg(SetViewMessage::Pick(elt))).width(Length::Fill))
                .push(Button::new(&mut self.add_btn, Text::new('+')).on_press(msg(SetViewMessage::Add)).width(Length::Units(21)))
                .push(Space::with_width(Length::Shrink)) // to avoid overlap with the scrollbar
                .spacing(16)
            )
            .spacing(16)
    }
}

impl<T: Into<Weights>> From<T> for WeightsState {
    fn from(x: T) -> WeightsState {
        let data = x.into();
        WeightsState {
            //load_preset_btn: button::State::default(),
            //save_preset_btn: button::State::default(),
            load_file_btn: button::State::default(),
            save_file_btn: button::State::default(),
            hash_icon0: pick_list::State::default(),
            hash_icon1: pick_list::State::default(),
            disabled_locations: SetView::new("Disabled Locations:", LOCATIONS, Message::DisabledLocations, &data.disabled_locations),
            allowed_tricks: SetView::new("Allowed Tricks:", TRICKS, Message::AllowedTricks, &data.allowed_tricks), //TODO show display names?
            starting_items: SetView::new("Starting Items:", INVENTORY, Message::StartingItems, &data.starting_items),
            starting_songs: SetView::new("Starting Songs:", SONGS, Message::StartingSongs, &data.starting_songs),
            starting_equipment: SetView::new("Starting Equipment:", EQUIPMENT, Message::StartingEquipment, &data.starting_equipment),
            weights: data.weights.iter().map(WeightsRuleState::from).collect(),
            add_btn: button::State::default(),
            data,
        }
    }
}

#[derive(Debug, Clone, IntoEnumIterator, PartialEq, Eq)]
enum WeightsRuleKind {
    Custom,
    Range(Distribution),
    Remove,
}

impl<'a> From<&'a WeightsRule> for WeightsRuleKind {
    fn from(rule: &WeightsRule) -> WeightsRuleKind {
        match rule {
            WeightsRule::Custom { .. } => WeightsRuleKind::Custom,
            WeightsRule::Range { distribution, .. } => WeightsRuleKind::Range(*distribution),
        }
    }
}

impl fmt::Display for WeightsRuleKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WeightsRuleKind::Custom => write!(f, "Custom"),
            WeightsRuleKind::Range(distribution) => distribution.fmt(f),
            WeightsRuleKind::Remove => write!(f, "(Remove)"),
        }
    }
}

#[derive(Clone)]
struct WeightsRuleState {
    setting: text_input::State,
    kind: pick_list::State<WeightsRuleKind>,
    conditionals: Vec<(text_input::State, button::State, button::State, Vec<(text_input::State, button::State)>, button::State, Vec<(text_input::State, text_input::State, button::State)>)>,
    values: Vec<(text_input::State, text_input::State, button::State)>,
    add_btn: button::State,
    add_conditional_btn: button::State,
    min: text_input::State,
    max: text_input::State,
}

impl WeightsRuleState {
    fn view(&mut self, idx: usize, rule: &WeightsRule) -> Element<'_, Message> {
        let col = Column::new()
            .push(Rule::horizontal(16))
            .push(Row::new()
                .push(TextInput::new(&mut self.setting, "Setting", rule.setting(), move |new_val| Message::ChangeSettingName(idx, new_val)).padding(5).style(TextInputStyle))
                .push(PickList::new(&mut self.kind, WeightsRuleKind::into_enum_iter().collect::<Vec<_>>(), Some(rule.into()), move |new_val| Message::ChangeSettingKind(idx, new_val)))
                .push(Space::with_width(Length::Shrink)) // to avoid overlap with the scrollbar
                .spacing(16)
            );
        match rule {
            WeightsRule::Custom { conditionals, values, .. } => {
                let mut col = col;
                for (cond_idx, (Conditional { setting, conditions, values }, (setting_state, remove_btn, add_cond_btn, cond_states, add_val_btn, val_states))) in conditionals.iter().zip(&mut self.conditionals).enumerate() {
                    col = col.push(Row::new()
                        .push(Text::new("If the setting:"))
                        .push(TextInput::new(setting_state, "Setting", setting, move |new_val| Message::ChangeConditionalSetting(idx, cond_idx, new_val)).padding(5).style(TextInputStyle))
                        .push(Button::new(remove_btn, Text::new('-')).on_press(Message::RemoveConditional(idx, cond_idx)))
                        .push(Space::with_width(Length::Shrink)) // to avoid overlap with the scrollbar
                        .spacing(16)
                    );
                    col = col.push(Row::new()
                        .push(Text::new("Has one of these values:"))
                        .push(Button::new(add_cond_btn, Text::new("Add Value")).on_press(Message::AddCondition(idx, cond_idx)))
                        .spacing(16)
                    );
                    for (val_idx, (val, (val_text, remove_btn))) in conditions.iter().zip(cond_states).enumerate() {
                        col = col.push(Row::new()
                            .push(TextInput::new(val_text, "Value", &match val {
                                Json::Bool(val) => val.to_string(),
                                Json::Number(val) => val.to_string(),
                                Json::String(val) => val.to_owned(),
                                _ => unimplemented!("null/array/object setting values not implemented"),
                            }, move |new_val| Message::ChangeCondition(idx, cond_idx, val_idx, new_val)).padding(5).style(TextInputStyle))
                            .push(Button::new(remove_btn, Text::new('-')).on_press(Message::RemoveCondition(idx, cond_idx, val_idx)))
                            .push(Space::with_width(Length::Shrink)) // to avoid overlap with the scrollbar
                            .spacing(16)
                        );
                    }
                    col = col.push(Row::new()
                        .push(Text::new("Use these weights:"))
                        .push(Button::new(add_val_btn, Text::new("Add Value")).on_press(Message::AddSettingValue(idx, Some(cond_idx))))
                        .spacing(16)
                    );
                    let total = values.values().sum::<u64>().to_string();
                    for ((value, weight), (value_state, weight_state, del_btn_state)) in values.iter().zip(val_states) {
                        //TODO remove these clone calls (work with indices instead?)
                        let val_clone1 = value.clone();
                        let val_clone2 = value.clone();
                        let val_clone3 = value.clone();
                        col = col.push(Row::new()
                            .push(TextInput::new(value_state, "value", value, move |new_val| Message::ChangeSettingValue(idx, Some(cond_idx), val_clone1.clone(), new_val)).padding(5).style(TextInputStyle))
                            .push(Text::new(':'))
                            .push(TextInput::new(weight_state, "weight", &weight.to_string(), move |new_val| Message::ChangeSettingWeight(idx, Some(cond_idx), val_clone2.clone(), new_val)).padding(5).style(TextInputStyle))
                            .push(Text::new('/'))
                            .push(Text::new(&total))
                            .push(Button::new(del_btn_state, Text::new('-')).on_press(Message::RemoveSettingValue(idx, Some(cond_idx), val_clone3)))
                            .push(Space::with_width(Length::Shrink)) // to avoid overlap with the scrollbar
                            .spacing(16)
                        );
                    }
                }
                if !conditionals.is_empty() {
                    col = col.push(Text::new("Otherwise, use these weights:"));
                }
                let total = values.values().sum::<u64>().to_string();
                for ((value, weight), (value_state, weight_state, del_btn_state)) in values.iter().zip(&mut self.values) {
                    //TODO remove these clone calls (work with indices instead?)
                    let val_clone1 = value.clone();
                    let val_clone2 = value.clone();
                    let val_clone3 = value.clone();
                    col = col.push(Row::new()
                        .push(TextInput::new(value_state, "value", value, move |new_val| Message::ChangeSettingValue(idx, None, val_clone1.clone(), new_val)).padding(5).style(TextInputStyle))
                        .push(Text::new(':'))
                        .push(TextInput::new(weight_state, "weight", &weight.to_string(), move |new_val| Message::ChangeSettingWeight(idx, None, val_clone2.clone(), new_val)).padding(5).style(TextInputStyle))
                        .push(Text::new('/'))
                        .push(Text::new(&total))
                        .push(Button::new(del_btn_state, Text::new('-')).on_press(Message::RemoveSettingValue(idx, None, val_clone3)))
                        .push(Space::with_width(Length::Shrink)) // to avoid overlap with the scrollbar
                        .spacing(16)
                    );
                }
                col.push(Row::new()
                    .push(Button::new(&mut self.add_btn, Text::new("Add Setting Value")).on_press(Message::AddSettingValue(idx, None)))
                    .push(Button::new(&mut self.add_conditional_btn, Text::new("Add Conditional")).on_press(Message::AddConditional(idx)))
                    .spacing(16)
                )
            }
            WeightsRule::Range { min, max, .. } => {
                col.push(Row::new()
                    .push(Text::new("Range:"))
                    .push(TextInput::new(&mut self.min, "min", &min.to_string(), move |new_val| Message::ChangeRangeMin(idx, new_val)).padding(5).style(TextInputStyle))
                    .push(Text::new('–'))
                    .push(TextInput::new(&mut self.max, "max", &max.to_string(), move |new_val| Message::ChangeRangeMax(idx, new_val)).padding(5).style(TextInputStyle))
                    .push(Space::with_width(Length::Shrink)) // to avoid overlap with the scrollbar
                    .spacing(16)
                )
            }
        }.spacing(16).into()
    }
}

impl<'a> From<&'a WeightsRule> for WeightsRuleState {
    fn from(rule: &WeightsRule) -> WeightsRuleState {
        WeightsRuleState {
            setting: text_input::State::default(),
            kind: pick_list::State::default(),
            conditionals: if let WeightsRule::Custom { conditionals, .. } = rule {
                conditionals.iter()
                    .map(|Conditional { conditions, values, .. }| (
                        text_input::State::default(),
                        button::State::default(),
                        button::State::default(),
                        vec![<_>::default(); conditions.len()],
                        button::State::default(),
                        vec![<_>::default(); values.len()],
                    ))
                    .collect()
            } else {
                Vec::default()
            },
            values: if let WeightsRule::Custom { values, .. } = rule {
                vec![<_>::default(); values.len()]
            } else {
                Vec::default()
            },
            add_btn: button::State::default(),
            add_conditional_btn: button::State::default(),
            min: text_input::State::default(),
            max: text_input::State::default(),
        }
    }
}

#[derive(SmartDefault)]
enum GenState {
    #[default]
    Idle(button::State),
    Generating,
    Error {
        e: GenError,
        reset_btn: button::State,
    },
    PyNotFound {
        install_btn: button::State,
        reset_btn: button::State,
    },
    #[cfg(windows)]
    InstallingPython,
    #[cfg(windows)]
    PyInstallError {
        e: PyInstallError,
        reset_btn: button::State,
    },
}

impl GenState {
    fn view(&mut self, disabled_reason: Option<&str>) -> Element<'_, Message> {
        match self {
            GenState::Idle(gen_btn) => if let Some(disabled_reason) = disabled_reason {
                Row::new()
                    .push(Button::new(gen_btn, Text::new("Generate Seed")))
                    .push(Text::new(format!("({})", disabled_reason)))
                    .spacing(16)
                    .into()
            } else {
                Button::new(gen_btn, Text::new("Generate Seed")).on_press(Message::Generate).into()
            },
            GenState::Generating => Text::new("Generating…").into(),
            GenState::Error { e, reset_btn } => Row::new()
                .push(Text::new(format!("error generating seed: {}", e)))
                .push(Button::new(reset_btn, Text::new("Dismiss")).on_press(Message::SeedDone))
                .spacing(16)
                .into(),
            #[cfg_attr(not(windows), allow(unused))]
            GenState::PyNotFound { install_btn, reset_btn } => {
                let mut row = Row::new().push(Text::new("Python not found"));
                #[cfg(windows)] {
                    row = row.push(Button::new(install_btn, Text::new("Install")).on_press(Message::InstallPython));
                }
                row = row.push(Button::new(reset_btn, Text::new("Dismiss")).on_press(Message::SeedDone));
                row.spacing(16).into()
            }
            #[cfg(windows)]
            GenState::InstallingPython => Text::new("Installing Python…").into(),
            #[cfg(windows)]
            GenState::PyInstallError { e, reset_btn } => Row::new()
                .push(Text::new(format!("error installing Python: {}", e)))
                .push(Button::new(reset_btn, Text::new("Dismiss")).on_press(Message::SeedDone))
                .spacing(16)
                .into(),
        }
    }
}

struct TabContainerStyle;

impl container::StyleSheet for TabContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            border_width: 1.0,
            border_color: Color::BLACK,
            ..container::Style::default()
        }
    }
}

pub(crate) struct TextInputStyle;

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

#[derive(SmartDefault)]
struct App {
    #[default(reqwest::Client::builder().user_agent(concat!("rsl/", env!("CARGO_PKG_VERSION"))).build().expect("failed to create reqwest client"))]
    client: reqwest::Client,
    #[cfg(feature = "self-update")]
    update_check: UpdateCheckState,
    #[default(FilePicker::new(format!("Base ROM"), Message::ChangeBaseRom, Message::BrowseBaseRom))]
    base_rom: FilePicker<file::File, Message>,
    #[default(FilePicker::new(format!("Output Directory"), Message::ChangeOutputDir, Message::BrowseOutputDir))]
    output_dir: FilePicker<file::Folder, Message>,
    tab: Tab,
    scroll: scrollable::State,
    #[default(PresetOptions { world_count: 2, ..PresetOptions::default() })]
    options: PresetOptions,
    worlds_slider: slider::State,
    worlds_text: text_input::State,
    #[default(WeightsState::from(GenOptions::League))]
    weights: WeightsState,
    gen: GenState,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new((): ()) -> (App, Command<Message>) {
        (App::default(), async { Message::LoadConfig(Config::new().await) }.into())
    }

    fn title(&self) -> String { format!("Ocarina of Time Randomizer — Random Settings Generator") }

    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::AddCondition(idx, cond_idx) => if let WeightsRule::Custom { ref mut conditionals, .. } = self.weights.data.weights[idx] {
                conditionals[cond_idx].conditions.push(json!(""));
                self.weights.weights[idx].conditionals[cond_idx].3.push(<_>::default());
            },
            Message::AddConditional(idx) => if let WeightsRule::Custom { ref mut conditionals, ref values, .. } = self.weights.data.weights[idx] {
                conditionals.push(Conditional {
                    setting: String::default(),
                    conditions: vec![json!("")],
                    values: values.clone(),
                });
                self.weights.weights[idx].conditionals.push(<_>::default());
            },
            Message::AddSetting => {
                let setting = WeightsRule::Custom { setting: String::default(), conditionals: Vec::default(), values: BTreeMap::default() };
                self.weights.weights.push(WeightsRuleState::from(&setting));
                self.weights.data.weights.push(setting);
            }
            Message::AddSettingValue(idx, cond_idx) => if let WeightsRule::Custom { ref mut conditionals, ref mut values, .. } = self.weights.data.weights[idx] {
                let values = if let Some(cond_idx) = cond_idx {
                    self.weights.weights[idx].conditionals[cond_idx].3.push(<_>::default());
                    &mut conditionals[cond_idx].values
                } else {
                    self.weights.weights[idx].values.push(<_>::default());
                    values
                };
                values.insert(String::default(), 0);
            },
            Message::AllowedTricks(msg) => self.weights.allowed_tricks.update(&mut self.weights.data.allowed_tricks, msg),
            Message::BrowseBaseRom => {
                let picker = file::File::pick();
                return async move {
                    if let Some(data) = picker.await {
                        Message::SetBaseRom(data)
                    } else {
                        Message::Nop
                    }
                }.into()
            }
            Message::BrowseOutputDir => {
                let picker = file::Save::pick();
                return async move {
                    if let Some(data) = picker.await {
                        Message::SetOutputDir(data)
                    } else {
                        Message::Nop
                    }
                }.into()
            }
            Message::ChangeBaseRom(path_str) => self.base_rom.set(path_str),
            Message::ChangeCondition(idx, cond_idx, val_idx, new_val) => if let WeightsRule::Custom { ref mut conditionals, .. } = self.weights.data.weights[idx] {
                conditionals[cond_idx].conditions[val_idx] = match &new_val[..] { //TODO determine type based on setting, not value
                    "true" => json!(true),
                    "false" => json!(false),
                    _ => json!(new_val),
                };
            },
            Message::ChangeConditionalSetting(idx, cond_idx, new_val) => if let WeightsRule::Custom { ref mut conditionals, .. } = self.weights.data.weights[idx] {
                conditionals[cond_idx].setting = new_val;
            },
            Message::ChangeOutputDir(path_str) => self.output_dir.set(path_str),
            Message::ChangeRangeMax(idx, new_max) => if let Ok(new_max) = new_max.parse() {
                if let WeightsRule::Range { ref mut max, .. } = self.weights.data.weights[idx] {
                    *max = new_max;
                }
            },
            Message::ChangeRangeMin(idx, new_min) => if let Ok(new_min) = new_min.parse() {
                if let WeightsRule::Range { ref mut min, .. } = self.weights.data.weights[idx] {
                    *min = new_min;
                }
            },
            Message::ChangeSettingKind(idx, new_kind) => self.weights.set_rule_kind(idx, new_kind),
            Message::ChangeSettingName(idx, new_name) => *self.weights.data.weights[idx].setting_mut() = new_name,
            Message::ChangeSettingValue(idx, cond_idx, old_val, new_val) => if let WeightsRule::Custom { ref mut conditionals, ref mut values, .. } = self.weights.data.weights[idx] {
                let values = if let Some(cond_idx) = cond_idx {
                    &mut conditionals[cond_idx].values
                } else {
                    values
                };
                if let Some(weight) = values.remove(&old_val) {
                    if values.insert(new_val, weight).is_some() {
                        self.weights.weights[idx].values.pop();
                    }
                }
            },
            Message::ChangeSettingWeight(idx, cond_idx, value, new_weight) => if let Ok(new_weight) = new_weight.parse() {
                if let WeightsRule::Custom { ref mut conditionals, ref mut values, .. } = self.weights.data.weights[idx] {
                    let values = if let Some(cond_idx) = cond_idx {
                        &mut conditionals[cond_idx].values
                    } else {
                        values
                    };
                    values.insert(value, new_weight);
                }
            },
            Message::DisabledLocations(msg) => self.weights.disabled_locations.update(&mut self.weights.data.disabled_locations, msg),
            #[cfg(feature = "self-update")]
            Message::DismissUpdateError => self.update_check = UpdateCheckState::Unknown(button::State::default()),
            Message::GenError(e) => self.gen = if let GenError::PyNotFound = e {
                GenState::PyNotFound {
                    install_btn: button::State::default(),
                    reset_btn: button::State::default(),
                }
            } else {
                GenState::Error {
                    e,
                    reset_btn: button::State::default(),
                }
            },
            Message::Generate => {
                self.gen = GenState::Generating;
                let client = self.client.clone();
                let base_rom = self.base_rom.data.as_ref().expect("generate button should be disabled if no base rom is given").clone();
                let output_dir = self.output_dir.data.as_ref().expect("generate button should be disabled if no output dir is given").clone();
                let options = match self.tab {
                    Tab::League => GenOptions::League,
                    Tab::Solo => GenOptions::Preset { preset: Preset::Solo, options: PresetOptions { world_count: 1, ..self.options } },
                    Tab::CoOp => GenOptions::Preset { preset: Preset::CoOp, options: PresetOptions { world_count: 1, ..self.options } },
                    Tab::Multiworld => GenOptions::Preset { preset: Preset::Multiworld, options: self.options },
                    Tab::Custom => GenOptions::Custom(self.weights.data.clone()),
                };
                return async move {
                    match rsl::generate(&client, base_rom, output_dir, options).await {
                        Ok(()) => Message::SeedDone, //TODO button to open output dir
                        Err(e) => Message::GenError(e),
                    }
                }.into()
            }
            #[cfg(windows)] //TODO macOS/Linux support?
            Message::InstallPython => {
                self.gen = GenState::InstallingPython;
                return async {
                    match install_python().await {
                        Ok(()) => Message::Generate,
                        Err(e) => Message::PyInstallError(e),
                    }
                }.into()
            }
            #[cfg(feature = "self-update")]
            Message::InstallUpdate => {
                self.update_check = UpdateCheckState::Installing;
                let client = self.client.clone();
                return async move {
                    match run_updater(&client).await {
                        Ok(never) => match never {},
                        Err(e) => Message::UpdateCheckError(e),
                    }
                }.into()
            }
            #[cfg_attr(not(feature = "self-update"), allow(unused))]
            Message::LoadConfig(Config { auto_update_check }) => {
                #[cfg(feature = "self-update")] match auto_update_check {
                    Some(true) => {
                        self.update_check = UpdateCheckState::Checking;
                        return async { Message::UpdateCheck }.into()
                    }
                    Some(false) => self.update_check = UpdateCheckState::Unknown(button::State::default()),
                    None => {}
                }
            }
            Message::LoadFile => {
                let picker = AsyncFileDialog::new().pick_file(); //TODO picker options?
                return async move {
                    if let Some(handle) = picker.await {
                        let buf = match fs::read_to_string(handle.path()).await {
                            Ok(file) => file,
                            Err(e) => return Message::LoadFileError(Arc::new(e)),
                        };
                        match serde_json::from_str(&buf) { //TODO async-json?
                            Ok(weights) => Message::LoadWeights(weights),
                            Err(e) => Message::LoadFileError(Arc::new(e.into())),
                        }
                    } else {
                        Message::Nop
                    }
                }.into()
            }
            Message::LoadFileError(e) => panic!("error loading file: {}", e), //TODO display error message without crashing
            Message::LoadWeights(weights) => self.weights.data = weights,
            Message::Nop => {}
            Message::OpenLoadPresetMenu => unimplemented!(), //TODO
            Message::OpenSavePresetMenu => unimplemented!(), //TODO
            #[cfg(windows)]
            Message::PyInstallError(e) => self.gen = GenState::PyInstallError {
                e,
                reset_btn: button::State::default(),
            },
            Message::RemoveCondition(idx, cond_idx, val_idx) => if let WeightsRule::Custom { ref mut conditionals, .. } = self.weights.data.weights[idx] {
                self.weights.weights[idx].conditionals[cond_idx].3.remove(val_idx);
                conditionals[cond_idx].conditions.remove(val_idx);
            },
            Message::RemoveConditional(idx, cond_idx) => if let WeightsRule::Custom { ref mut conditionals, .. } = self.weights.data.weights[idx] {
                self.weights.weights[idx].conditionals.remove(cond_idx);
                conditionals.remove(cond_idx);
            },
            Message::RemoveSettingValue(idx, cond_idx, value) => if let WeightsRule::Custom { ref mut conditionals, ref mut values, .. } = self.weights.data.weights[idx] {
                let values = if let Some(cond_idx) = cond_idx {
                    &mut conditionals[cond_idx].values
                } else {
                    values
                };
                if let Some(val_idx) = values.keys().position(|val| *val == value) {
                    if let Some(cond_idx) = cond_idx {
                        self.weights.weights[idx].conditionals[cond_idx].5.remove(val_idx);
                    } else {
                        self.weights.weights[idx].values.remove(val_idx);
                    }
                    values.remove(&value);
                }
            },
            Message::SaveFile => {
                let json = serde_json::to_vec_pretty(&self.weights.data) //TODO async-json?
                    .map(|mut json| { json.push(b'\n'); json });
                let picker = AsyncFileDialog::new().save_file(); //TODO picker options?
                return async move {
                    if let Some(handle) = picker.await {
                        let buf = match json {
                            Ok(buf) => buf,
                            Err(e) => return Message::SaveFileError(Arc::new(e.into())),
                        };
                        if let Err(e) = fs::write(handle.path(), buf).await { return Message::SaveFileError(Arc::new(e)) }
                    }
                    Message::Nop
                }.into()
            }
            Message::SaveFileError(e) => panic!("error saving file: {}", e), //TODO display error message without crashing
            Message::SeedDone => self.gen = GenState::default(),
            #[cfg(feature = "self-update")]
            Message::SetAutoUpdateCheck(enable) => {
                self.update_check = if enable { UpdateCheckState::Checking } else { UpdateCheckState::Unknown(button::State::default()) };
                return async move {
                    let mut config = Config::new().await;
                    config.auto_update_check = Some(enable);
                    match config.save().await {
                        Ok(()) => if enable { Message::UpdateCheck } else { Message::Nop },
                        Err(e) => Message::UpdateCheckError(UpdateCheckError::Config(e)),
                    }
                }.into()
            }
            Message::SetBaseRom(path) => self.base_rom.data = Some(path),
            Message::SetHashIcon0(icon) => self.weights.data.hash[0] = icon,
            Message::SetHashIcon1(icon) => self.weights.data.hash[1] = icon,
            Message::SetOutputDir(path) => self.output_dir.data = Some(path),
            Message::SetWorldCount(world_count) => match self.tab {
                Tab::Multiworld => if (2..=MAX_WORLDS).contains(&world_count) { self.options.world_count = world_count },
                Tab::Custom => if (1..=MAX_WORLDS).contains(&world_count) { self.weights.data.world_count = world_count },
                _ => {}
            },
            Message::SetWorldCountStr(world_count_str) => if let Ok(world_count) = world_count_str.parse() {
                return async move { Message::SetWorldCount(world_count) }.into()
            },
            Message::StartingEquipment(msg) => self.weights.starting_equipment.update(&mut self.weights.data.starting_equipment, msg),
            Message::StartingItems(msg) => self.weights.starting_items.update(&mut self.weights.data.starting_items, msg),
            Message::StartingSongs(msg) => self.weights.starting_songs.update(&mut self.weights.data.starting_songs, msg),
            Message::Tab(tab) => self.tab = tab,
            Message::ToggleRandomStartingItems(checked) => if let Tab::Custom = self.tab {
                self.weights.data.random_starting_items = checked;
            } else {
                self.options.random_starting_items = checked;
            },
            Message::ToggleRslTricks(checked) => self.options.rsl_tricks = checked,
            Message::ToggleStandardTricks(checked) => self.options.standard_tricks = checked,
            #[cfg(feature = "self-update")]
            Message::UpdateCheck => {
                self.update_check = UpdateCheckState::Checking;
                let client = self.client.clone();
                return async move {
                    match check_for_updates(&client).await {
                        Ok(update_available) => Message::UpdateCheckComplete(update_available),
                        Err(e) => Message::UpdateCheckError(e),
                    }
                }.into()
            }
            #[cfg(feature = "self-update")]
            Message::UpdateCheckComplete(Some(new_ver)) => self.update_check = UpdateCheckState::UpdateAvailable {
                new_ver,
                update_btn: button::State::default(),
            },
            #[cfg(feature = "self-update")]
            Message::UpdateCheckComplete(None) => self.update_check = UpdateCheckState::NoUpdateAvailable,
            #[cfg(feature = "self-update")]
            Message::UpdateCheckError(e) => self.update_check = UpdateCheckState::Error {
                e,
                reset_btn: button::State::default(),
            },
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Message> {
        let disabled_reason = if self.base_rom.data.is_none() {
            Some("base ROM is required")
        } else if self.output_dir.data.is_none() {
            Some("output directory is required")
        } else {
            //TODO if on custom tab, check to make sure:
            // * for Custom weights: the sum of the weights is greater than 0
            // * for Range weights: the range is non-empty (end >= start, since it's an inclusive range)
            None
        };
        Column::new()
            .push({
                #[cfg(feature = "self-update")] { self.update_check.view() }
                #[cfg(not(feature = "self-update"))] { Text::new(concat!("version ", env!("CARGO_PKG_VERSION"), " — built from source")) }
            })
            .push(self.base_rom.view())
            .push(self.output_dir.view())
            .push(self.tab.view())
            .push(Container::new(match self.tab {
                Tab::League => Scrollable::new(&mut self.scroll).push(Text::new(format!(
                    //TODO after s2, update description (e.g. “season 2 tournament” → “season 3 ladder”?)
                    "This will generate a seed with the Random Settings League's season 2 tournament weights. It will use version {} of the randomizer. You can use the tabs above to switch to the latest version and use different weights.",
                    LEAGUE_VERSION,
                ))),
                Tab::Solo | Tab::CoOp | Tab::Multiworld => {
                    let col = Scrollable::new(&mut self.scroll)
                        .push(Checkbox::new(self.options.standard_tricks, "Standard Tricks", Message::ToggleStandardTricks))
                        .push(Checkbox::new(self.options.rsl_tricks, "RSL Tricks", Message::ToggleRslTricks))
                        //TODO conditionals toggle?
                        .push(Checkbox::new(self.options.random_starting_items, "Randomize Starting Items", Message::ToggleRandomStartingItems));
                    if let Tab::Multiworld = self.tab {
                        col.push(Row::new()
                            .push(Text::new("Player Count:"))
                            .push(Slider::new(&mut self.worlds_slider, 2..=MAX_WORLDS, self.options.world_count, Message::SetWorldCount))
                            .push(TextInput::new(&mut self.worlds_text, "", &self.options.world_count.to_string(), Message::SetWorldCountStr).width(Length::Units(32)).padding(5).style(TextInputStyle))
                            .spacing(16)
                        )
                    } else {
                        col
                    }
                }
                Tab::Custom => self.weights.view(&mut self.scroll, &mut self.worlds_slider, &mut self.worlds_text),
            }.spacing(16).height(Length::Fill)).width(Length::Fill).height(Length::Fill).padding(16).style(TabContainerStyle))
            .push(self.gen.view(disabled_reason))
            .spacing(16)
            .padding(16)
            .into()
    }
}

#[cfg(feature = "self-update")]
#[derive(Debug, Clone, From)]
enum UpdateCheckError {
    Config(config::Error),
    Io(Arc<io::Error>),
    #[cfg(unix)]
    MissingAsset,
    #[cfg(windows)]
    MissingHomeDir,
    NoReleases,
    Reqwest(Arc<reqwest::Error>),
    #[from]
    SemVer(SemVerError),
}

#[cfg(feature = "self-update")]
from_arc! {
    io::Error => UpdateCheckError, Io,
    reqwest::Error => UpdateCheckError, Reqwest,
}

#[cfg(feature = "self-update")]
impl fmt::Display for UpdateCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateCheckError::Config(e) => write!(f, "error saving preferences: {}", e),
            UpdateCheckError::Io(e) => write!(f, "I/O error: {}", e),
            #[cfg(unix)]
            UpdateCheckError::MissingAsset => write!(f, "release does not have a download for this platform"),
            #[cfg(windows)]
            UpdateCheckError::MissingHomeDir => write!(f, "failed to locate home directory"),
            UpdateCheckError::NoReleases => write!(f, "there are no released versions"),
            UpdateCheckError::Reqwest(e) => if let Some(url) = e.url() {
                write!(f, "HTTP error at {}: {}", url, e)
            } else {
                write!(f, "HTTP error: {}", e)
            },
            UpdateCheckError::SemVer(e) => e.fmt(f),
        }
    }
}

#[cfg(feature = "self-update")]
async fn check_for_updates(client: &reqwest::Client) -> Result<Option<Version>, UpdateCheckError> {
    let repo = Repo::new("matthewkirby", "plando-random-settings");
    if let Some(release) = repo.latest_release(client).await? {
        let new_ver = release.version()?;
        Ok(if new_ver > Version::parse(env!("CARGO_PKG_VERSION"))? { Some(new_ver) } else { None })
    } else {
        Err(UpdateCheckError::NoReleases)
    }
}

#[cfg(feature = "self-update")]
async fn run_updater(#[cfg_attr(windows, allow(unused))] client: &reqwest::Client) -> Result<Never, UpdateCheckError> {
    #[cfg(unix)] { //TODO use Sparkle or similar on macOS? The current code only replaces the executable, not the entire app
        let current_exe = env::current_exe()?;
        fs::remove_file(&current_exe).await?;
        let release = Repo::new("matthewkirby", "plando-random-settings").latest_release(&client).await?.ok_or(UpdateCheckError::NoReleases)?;
        let (asset,) = release.assets.into_iter()
            .filter(|asset| asset.name.ends_with(PLATFORM_SUFFIX))
            .collect_tuple().ok_or(UpdateCheckError::MissingAsset)?;
        let response = client.get(asset.browser_download_url).send().await?.error_for_status()?;
        {
            let mut data = response.bytes_stream();
            let mut exe_file = File::create(&current_exe).await?;
            while let Some(chunk) = data.try_next().await? {
                exe_file.write_all(chunk.as_ref()).await?;
            }
        }
        sleep(Duration::from_secs(1)).await; // to make sure the download is closed
        std::process::Command::new(current_exe).spawn()?;
        std::process::exit(0)
    }
    #[cfg(windows)] {
        let cache_dir = cache_dir().ok_or(UpdateCheckError::MissingHomeDir)?;
        fs::create_dir_all(&cache_dir).await?;
        let updater_path = cache_dir.join("updater.exe");
        #[cfg(target_arch = "x86_64")] let updater_data = include_bytes!("../../../target/x86_64-pc-windows-msvc/release/rsl-updater.exe");
        fs::write(&updater_path, updater_data).await?;
        let _ = std::process::Command::new(updater_path).arg(env::current_exe()?).spawn()?;
        std::process::exit(0)
    }
}

#[cfg(windows)]
#[derive(Debug, Clone)]
enum PyInstallError {
    InstallerExit,
    Io(Arc<io::Error>),
    MissingHomeDir,
    Reqwest(Arc<reqwest::Error>),
}

#[cfg(windows)]
from_arc! {
    io::Error => PyInstallError, Io,
    reqwest::Error => PyInstallError, Reqwest,
}

#[cfg(windows)]
impl fmt::Display for PyInstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PyInstallError::InstallerExit => write!(f, "the installer exited with an error status"),
            PyInstallError::Io(e) => write!(f, "I/O error: {}", e),
            PyInstallError::MissingHomeDir => write!(f, "failed to locate home directory"),
            PyInstallError::Reqwest(e) => if let Some(url) = e.url() {
                write!(f, "HTTP error at {}: {}", url, e)
            } else {
                write!(f, "HTTP error: {}", e)
            },
        }
    }
}

#[cfg(windows)]
async fn install_python() -> Result<(), PyInstallError> {
    #[cfg(target_arch = "x86")] let arch_suffix = "";
    #[cfg(target_arch = "x86_64")] let arch_suffix = "-amd64";
    let response = reqwest::get(&format!("https://www.python.org/ftp/python/{0}/python-{0}{1}.exe", PY_VERSION, arch_suffix)).await?
        .error_for_status()?;
    let installer_path = cache_dir().ok_or(PyInstallError::MissingHomeDir)?.join("python-installer.exe");
    {
        let mut data = response.bytes_stream();
        let mut installer_file = File::create(&installer_path).await?;
        while let Some(chunk) = data.try_next().await? {
            installer_file.write_all(chunk.as_ref()).await?;
        }
    }
    sleep(Duration::from_secs(1)).await; // to make sure the download is closed
    if !tokio::process::Command::new(installer_path).arg("/passive").arg("PrependPath=1").status().await?.success() {
        return Err(PyInstallError::InstallerExit)
    }
    Ok(())
}

#[derive(StructOpt)]
struct Args {}

#[wheel::main]
fn main(Args {}: Args) -> iced::Result {
    let size = (604, 420);
    App::run(Settings {
        window: window::Settings {
            size,
            min_size: Some(size),
            //TODO icon
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}
