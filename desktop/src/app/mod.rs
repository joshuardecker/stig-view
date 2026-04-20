/// Contains the app internal code.
mod app;
/// Contains command logic, such as turning a string into a command, and running it.
mod command;
/// Contains settings logic, like saving to the disk.
mod settings;
/// Contains the logic for remembering when benchmarks were last opened, and saving this to the disk.
mod time_opened;

use iced::{
    Task, keyboard,
    widget::{
        Id,
        text_editor::{Action, Content},
    },
    window,
    window::Direction,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Instant};
use stig_view_core::{Benchmark, Rule};

use crate::app::{
    settings::{AppSettings, AppSettingsErr},
    time_opened::TimeLastOpened,
};

/// The overarching state of the application.
#[derive(Debug, Clone)]
pub struct App {
    pub benchmark: Benchmark,
    // Benchmarks that live in the background, but are not currently displayed.
    pub background_benchmarks: Vec<Benchmark>,
    pub pins: HashMap<String, Pinned>,
    pub displayed: Option<Rule>,
    pub contents: [Content; 7],
    pub filter_input: String,
    pub popup: Popup,
    pub err_notif: Option<String>,
    pub window_id: Option<window::Id>,
    pub settings: AppSettings,
    pub saved_when: TimeLastOpened,
    pub home_menu_hash: u64,
    pub stig_list_hash: u64,
    pub display_type: DisplayType,

    // Fields that have to due with animation.
    pub main_col_opacity: f32,
    pub main_col_last_tick: Option<Instant>,
    pub popup_opacity: f32,
    pub popup_last_tick: Option<Instant>,
}

/// Popups that can appear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Popup {
    Filter,
    Settings,
    Save,
    None,
}

/// Every way to change the state.
#[derive(Debug, Clone)]
pub enum Message {
    InitWindow(Option<window::Id>),
    WindowClose,
    WindowMin,
    WindowFullscreenToggle,
    WindowMove,
    WindowDragResize(Direction),

    SwitchTheme(AppTheme),

    OpenFile,

    SelectContent(Action, ContentIndex),

    Switch(String),
    SwitchBenchmark(Benchmark),
    SwitchBenchmarks(Vec<Benchmark>),
    PushBackgroundBenchmark(Benchmark),
    // Switch the current Benchmark to one loaded in the background.
    // Puts the current Benchmark into the background.
    SwitchToBackground,

    SetPins(HashMap<String, Pinned>),
    SwitchNext,
    Display(Rule),

    SwitchPopup(Popup),

    SendErrNotif(&'static str),
    ClearErrNotif,

    Pin(String),

    FocusWidget(Id),

    TypeCmd(String),
    ProcessCmd(String),

    KeyPressed(keyboard::Event),

    SaveSettings,
    SaveBenchmark,
    LoadCachedBenchmark(std::path::PathBuf),
    DeleteCachedBenchmark(std::path::PathBuf),

    SwitchDisplayType(DisplayType),
    SaveDisplayType(DisplayType),

    SaveAnimate(bool),

    ReturnHome,

    Tick(Instant),

    DoNothing,
}

/// An enum that represents an index for accessing the text editors context array
/// in the state.
#[derive(Debug, Clone, Copy)]
pub enum ContentIndex {
    Title,
    Discussion,
    Check,
    Fix,
    CCIRefs,
    FalsePositives,
    FalseNegatives,
}

/// The color theme of the app.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum AppTheme {
    Dark,
    Light,
    HighContrast,
    Coffee,
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AppTheme::Dark => "Dark",
            AppTheme::Light => "Light",
            AppTheme::HighContrast => "High Contrast",
            AppTheme::Coffee => "Coffee",
        })
    }
}

/// Whether the stig has been pinned in the list for any reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pinned {
    Not,
    ByUser,
    ByFilter,
    ByFilterAndUser,
}

/// What name should be displayed on the buttons that switch the displayed STIG.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Deserialize, Serialize)]
pub enum DisplayType {
    GroupId,
    RuleId,
    STIGId,
}

impl std::fmt::Display for DisplayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DisplayType::GroupId => "Group ID",
            DisplayType::RuleId => "Rule ID",
            DisplayType::STIGId => "STIG ID",
        })
    }
}
