#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod ui;

use crate::app::App;

#[cfg(target_os = "windows")]
fn main() -> iced::Result {
    use iced::Font;

    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .title("Stig View")
        .font(include_bytes!("../../assets/fonts/font.ttf"))
        .default_font(Font::with_name("CMU Sans Serif"))
        .run()
}

#[cfg(target_os = "linux")]
fn main() -> iced::Result {
    use iced::{
        Font,
        window::settings::{PlatformSpecific, Settings},
    };

    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .title("Stig View")
        .font(include_bytes!("../../assets/fonts/font.ttf"))
        .default_font(Font::with_name("CMU Sans Serif"))
        .window(Settings {
            platform_specific: PlatformSpecific {
                application_id: String::from("io.github.joshuardecker.stig-view"),
                override_redirect: false,
            },
            ..Settings::default()
        })
        .run()
}
