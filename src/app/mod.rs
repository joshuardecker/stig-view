/// Contains animation helping logic.
mod animation;
/// Contains the app internal code.
mod app;
/// Contains the logic for remembering when benchmarks were last opened, and saving this to the disk.
mod last_opened;
/// Detect whether the user is running the latest release.
mod latest_release;
/// Contains search logic.
mod search;
/// Contains settings logic, like saving to the disk.
mod settings;

use std::{collections::HashMap, time::Instant};

use disa_stig::Rule;
use iced::{Task, widget::Id, window, window::Direction};
use serde::{Deserialize, Serialize};

use crate::app::{animation::Animations, last_opened::LastOpened, settings::Settings};

pub use crate::app::app::{App, AppTheme, DisplayType, Message, Pinned, Popup};
