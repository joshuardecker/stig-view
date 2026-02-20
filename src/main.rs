mod app;
mod db;
mod preload_assets;
mod stig;
mod styles;
mod ui;

use crate::app::App;

#[cfg(target_os = "linux")]
fn main() -> iced::Result {
    use iced::window::settings::{PlatformSpecific, Settings};

    iced::application(App::new, App::update, App::get_view)
        .subscription(App::subscription)
        .theme(App::theme)
        .title("Stig View")
        .default_font(iced::font::Font::MONOSPACE)
        .window(Settings {
            platform_specific: PlatformSpecific {
                application_id: String::from("io.github.joshuardecker.stig-view"),
                override_redirect: false,
            },
            ..Settings::default()
        })
        .run()
}

#[cfg(not(target_os = "linux"))]
fn main() -> iced::Result {
    iced::application(App::new, App::update, App::get_view)
        .subscription(App::subscription)
        .theme(App::theme)
        .title("Stig View")
        .default_font(iced::font::Font::MONOSPACE)
        .run()
}
