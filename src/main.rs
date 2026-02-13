mod app;
mod stig;
mod ui;

use crate::app::App;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::get_view)
        .subscription(App::subscription)
        .run()
}
