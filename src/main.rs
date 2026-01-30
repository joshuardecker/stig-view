mod app;
mod stig;
mod ui;

use ui::{new, update, view};

fn main() -> iced::Result {
    iced::application(new, update, view).run()
}
