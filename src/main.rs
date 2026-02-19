mod app;
mod preload_assets;
mod sgroup;
mod stig;
mod styles;
mod ui;

use iced::window::settings::{PlatformSpecific, Settings};

use crate::app::App;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::get_view)
        .subscription(App::subscription)
        .theme(App::theme)
        .title("Stig View")
        .default_font(iced::font::Font::MONOSPACE)
        /*.settings(Settings {
            id: Some(String::from("io.github.joshuardecker.stig-view")),
            ..Settings::default()
        })*/
        .window(Settings {
            platform_specific: PlatformSpecific {
                application_id: String::from("io.github.joshuardecker.stig-view"),
                override_redirect: false,
            },
            ..Settings::default()
        })
        .run()
}
