mod app;
mod preload_assets;
mod sgroup;
mod stig;
mod styles;
mod ui;

use crate::app::App;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::get_view)
        .subscription(App::subscription)
        .theme(App::theme)
        .title("Stig View")
        .default_font(iced::font::Font::MONOSPACE)
        .run()
}
