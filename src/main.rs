mod window;
mod xylok_stig;

use window::*;

fn main() -> iced::Result {
    iced::application(new, update, view).run()
}
