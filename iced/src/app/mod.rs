pub mod app;
mod assets;

// Re-exports:
pub use crate::app::assets::Assets;

use iced::keyboard;
use iced::widget::text_editor::Content;
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

    SelectContent(Content, usize),

    Switch(String),
    SwitchNext,

    SwitchPopup(Popup),

    Pin(String),

    KeyPressed(keyboard::Event),
}
