pub mod app;
mod assets;
mod async_fns;
mod command;

// Re-exports:
pub use crate::app::assets::Assets;

use iced::keyboard;
use iced::widget::Id;
use iced::widget::text_editor::{Action, Content};
use iced::window;
use iced::window::Direction;
use iced::{Task, task::Handle};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stig_view_core::db::DB;
use stig_view_core::stig_dep::Stig;

#[derive(Debug, Clone)]
pub struct App {
    pub db: DB,
    pub displayed: Option<Arc<Stig>>,
    pub contents: [Content; 6],
    pub filter_input: String,
    pub popup: Popup,
    pub err_notif: ErrNotif,
    pub assets: Assets,
    pub window_id: Option<window::Id>,
    pub settings: AppSettings,
    pub load_handle: Option<Handle>,
}

#[derive(Debug, Clone)]
pub enum Popup {
    Filter,
    Settings,
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
    OpenFolder,

    SelectContent(Action, ContentSlot),

    Switch(String),
    SwitchWithError(String, &'static str),
    SwitchNext,
    Display(Arc<Stig>),

    SwitchPopup(Popup),

    SendErrNotif(&'static str),
    ClearErrNotif,

    Pin(String),

    FocusWidget(Id),

    TypeCmd(String),
    ProcessCmd(String),

    KeyPressed(keyboard::Event),

    SaveSettings,

    DoNothing,
}

#[derive(Debug, Clone, Copy)]
pub enum ContentSlot {
    Version = 0,
    Intro = 1,
    Desc = 2,
    CheckText = 3,
    FixText = 4,
    SimilarChecks = 5,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppSettings {
    pub theme: AppTheme,
}

#[derive(Debug, Clone)]
pub enum AppSettingsErr {
    CantSave(&'static str),
}

impl AppSettings {
    pub fn default() -> Self {
        Self {
            theme: AppTheme::Dark,
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
