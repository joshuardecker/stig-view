pub mod app;
mod assets;
mod command;

use std::collections::HashMap;

// Re-exports:
pub use crate::app::assets::Assets;

use iced::keyboard;
use iced::widget::Id;
use iced::widget::text_editor::{Action, Content};
use iced::window;
use iced::window::Direction;
use iced::{Task, task::Handle};
use serde::{Deserialize, Serialize};
use stig_view_core::{Benchmark, Rule};

#[derive(Debug, Clone)]
pub struct App {
    pub benchmark: Benchmark,
    // Benchmarks that live in the background, but are not currently displayed.
    pub benchmarks: Vec<Benchmark>,
    pub pins: HashMap<String, Pinned>,
    pub displayed: Option<Rule>,
    pub contents: [Content; 7],
    pub filter_input: String,
    pub popup: Popup,
    pub err_notif: ErrNotif,
    pub assets: Assets,
    pub window_id: Option<window::Id>,
    pub settings: AppSettings,
    pub load_handle: Option<Handle>,
    pub display_type: DisplayType,
}

#[derive(Debug, Clone)]
pub enum Popup {
    Filter,
    Settings,
    Save,
    None,
}

#[derive(Debug, Clone)]
pub enum ErrNotif {
    Err(&'static str),
    None,
}

#[derive(Debug, Clone)]
pub enum Command {
    NameSearch(String),
    KeywordSearch(String),
    Reset,
}

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

    SwitchDisplayType(DisplayType),
    SaveDisplayType(DisplayType),

    ReturnHome,

    DoNothing,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum AppTheme {
    Dark,
    Light,
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AppTheme::Dark => "Dark",
            AppTheme::Light => "Light",
        })
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct AppSettings {
    pub theme: AppTheme,
    pub default_display_type: DisplayType,
}

#[derive(Debug, Clone)]
pub enum AppSettingsErr {
    CantSave(&'static str),
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
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
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

impl AppSettings {
    pub fn default() -> Self {
        Self {
            theme: AppTheme::Dark,
            default_display_type: DisplayType::GroupId,
        }
    }

    /// Save app settings in the users config directory.
    pub fn save(settings: AppSettings) -> Result<(), AppSettingsErr> {
        use std::fs::File;
        use std::io::Write;

        let mut save_dir = dirs::config_local_dir().ok_or(AppSettingsErr::CantSave(
            "Couldn't locate config directory.",
        ))?;

        save_dir.push("stig-view-settings.toml");

        let settings_str = toml::to_string(&settings)
            .map_err(|_| AppSettingsErr::CantSave("Couldn't save user settings."))?;

        let mut file = File::create(save_dir)
            .map_err(|_| AppSettingsErr::CantSave("Error creating settings.toml save file."))?;

        let err = write!(file, "{}", settings_str);

        if err.is_err() {
            return Err(AppSettingsErr::CantSave(
                "Error writing settings to settings.toml",
            ));
        }

        Ok(())
    }

    /// Load app settings. No errors, just returns None if it could not find the settings.
    pub fn load() -> Option<Self> {
        use std::fs::read_to_string;

        let mut save_dir = dirs::config_local_dir()?;

        save_dir.push("stig-view-settings.toml");

        let settings_str = read_to_string(save_dir).ok()?;

        let settings: AppSettings = toml::from_str(&settings_str).ok()?;

        Some(settings)
    }
}
