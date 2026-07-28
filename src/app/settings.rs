use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::app::app::{AppTheme, DisplayType};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Settings {
    pub theme: AppTheme,
    pub default_display_type: DisplayType,
    pub animate: bool,
    pub notify_if_update: bool,
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Could not access save directory")]
    DirError,
    #[error("Error accessing or creating the benchmark time file")]
    FileError(#[from] std::io::Error),
    #[error("Error serializing the benchmark time file")]
    SerializationError(#[from] toml::ser::Error),
    #[error("Error deserializing the benchmark time file")]
    DeserializationError(#[from] toml::de::Error),
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: AppTheme::Dark,
            default_display_type: DisplayType::GroupId,
            animate: true,
            notify_if_update: true,
        }
    }
}

impl Settings {
    /// Save app settings in the users config directory.
    pub fn save(&self) -> Result<(), SettingsError> {
        use std::fs::File;
        use std::io::Write;

        let mut save_dir = dirs::config_local_dir().ok_or(SettingsError::DirError)?;

        save_dir.push("xylok-view-settings.toml");

        let settings_str = toml::to_string(self)?;

        let mut file = File::create(save_dir)?;

        file.write_all(settings_str.as_bytes())?;

        Ok(())
    }

    /// Load app settings. No errors, just returns None if it could not find the settings.
    pub fn load() -> Option<Self> {
        use std::fs::read_to_string;

        let mut save_dir = dirs::config_local_dir()?;

        save_dir.push("xylok-view-settings.toml");

        let settings_str = read_to_string(save_dir).ok()?;

        let settings: Settings = toml::from_str(&settings_str).ok()?;

        Some(settings)
    }
}
