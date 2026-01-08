mod window;
mod xylok_stig;

fn main() -> iced::Result {
    iced::run(window::update, window::view)
}
