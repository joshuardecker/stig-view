mod app;
mod stig;
mod ui;

use ui::{new, subscription, update, view};

fn main() -> iced::Result {
    iced::application(new, update, view)
        .subscription(subscription)
        .run()
}
