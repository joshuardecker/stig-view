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
use std::sync::Arc;
use stig_view_core::db::DB;
use stig_view_core::stig::Stig;

#[derive(Debug, Clone)]
pub struct App {
    pub db: DB,
    pub displayed: Option<Arc<Stig>>,
    pub contents: [Content; 6],
    pub filter_input: String,
    pub filter_valid: bool,
    pub popup: Popup,
    pub err_notif: ErrNotif,
    pub assets: Assets,
    pub window_id: Option<window::Id>,
    pub current_theme: Option<AppTheme>,
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

#[derive(Debug, Clone, Copy, PartialEq)]
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
