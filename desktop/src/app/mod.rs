pub mod app;
mod command;

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
use std::time::Instant;
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};
use stig_view_core::{Benchmark, Rule};

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
    pub saved_when: SavedWhen,
    pub display_type: DisplayType,

    // Fields that have to due with animation.
    pub main_col_opacity: f32,
    pub main_col_last_tick: Option<Instant>,
    pub popup_opacity: f32,
    pub popup_last_tick: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Popup {
    Filter,
    Settings,
    Save,
    None,
}

#[derive(Debug, Clone)]
pub enum Command {
    Phrase(String),
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
    DeleteCachedBenchmark(std::path::PathBuf),

    SwitchDisplayType(DisplayType),
    SaveDisplayType(DisplayType),

    SaveAnimate(bool),

    ReturnHome,

    Tick(Instant),

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

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct AppSettings {
    pub theme: AppTheme,
    pub default_display_type: DisplayType,
    pub animate: bool,
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
#[derive(Debug, Clone, Copy, PartialEq, Hash, Deserialize, Serialize)]
pub enum DisplayType {
    GroupId,
    RuleId,
    STIGId,
}

/// A struct that remembers when the user last opened a benchmark.
/// Used for the home screen to sort by most recently opened.
/// Will be saved to disk.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SavedWhen {
    benchmarks: HashMap<String, u64>, // (Benchmark name, unix time).
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
            animate: true,
        }
    }

    /// Save app settings in the users config directory.
    pub fn save(&self) -> Result<(), AppSettingsErr> {
        use std::fs::File;
        use std::io::Write;

        let mut save_dir = dirs::config_local_dir().ok_or(AppSettingsErr::CantSave(
            "Couldn't locate config directory.",
        ))?;

        save_dir.push("stig-view-settings.toml");

        let settings_str = toml::to_string(self)
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

impl SavedWhen {
    pub fn new() -> Self {
        Self {
            benchmarks: HashMap::new(),
        }
    }

    /// Returns when the the given benchmark was last accessed.
    /// Defaults to the current time if a value is not found to be saved on disk.
    pub fn get_time_used(&self, benchmark_id: &str) -> u64 {
        use std::time::SystemTime;

        match self.benchmarks.get(benchmark_id) {
            Some(time) => time.to_owned(),
            None => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Insert the current time with a benchmark id.
    pub fn insert(&mut self, benchmark_id: String) {
        self.benchmarks.insert(
            benchmark_id,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );

        self.save();
    }

    /// Load the saved_when data from disk. Returns None if failed.
    pub fn load() -> Option<Self> {
        use std::fs::read_to_string;

        let mut save_dir = dirs::data_local_dir()?;
        save_dir.push("stig-view");
        save_dir.push("saved_when.toml");

        let saved_when_str = read_to_string(save_dir).ok()?;

        let saved_when: SavedWhen = toml::from_str(&saved_when_str).ok()?;

        Some(saved_when)
    }

    /// Saves the SavedWhen to disk. If errors occur, they are silent.
    /// Not ideal if this has errors, but it doesnt really matter if it does.
    fn save(&self) {
        use std::fs::{File, create_dir_all};
        use std::io::Write;

        let mut save_dir = match dirs::data_local_dir() {
            Some(dir) => dir,
            None => return,
        };

        // Create the dir if it does not exist.
        save_dir.push("stig-view");
        let _ = create_dir_all(&save_dir);

        save_dir.push("saved_when.toml");

        let saved_when_str = match toml::to_string(self) {
            Ok(string) => string,
            Err(_) => return,
        };

        let mut file = match File::create(save_dir) {
            Ok(file) => file,
            Err(_) => return,
        };

        let _ = write!(file, "{}", saved_when_str);
    }
}
