pub mod app;
mod assets;
mod async_fns;

// Re-exports:
pub use crate::app::assets::Assets;

use iced::keyboard;
use iced::widget::text_editor::Action;
use iced::window;
use iced::window::Direction;
use stig_view_core::db::DB;
use stig_view_core::stig::Stig;

#[derive(Debug, Clone)]
pub struct App {
    db: DB,
    displayed: Option<Stig>,
    popup: Popup,
    assets: Assets,
    window_id: Option<window::Id>,
    current_theme: AppTheme,
}

#[derive(Debug, Clone)]
pub enum Popup {
    Filter,
    Settings,
    None,
}

#[derive(Debug, Clone)]
pub enum Message {
    InitWindow(Option<window::Id>),
    WindowClose,
    WindowMin,
    WindowFullscreenToggle,
    WindowMove,
    WindowDragResize(Direction),

    OpenFile,
    OpenFolder,

    SelectContent(Action, usize),

    Switch(String),
    SwitchWithError(String, &'static str),
    SwitchNext,

    SwitchPopup(Popup),

    SendErrNotif(&'static str),

    Pin(String),

    KeyPressed(keyboard::Event),

    DoNothing,
}

#[derive(Debug, Clone)]
pub enum AppTheme {
    Dark,
    Light,
}
