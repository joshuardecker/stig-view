use crate::xylok_stig::*;

use iced::{
    Element,
    Length::Fill,
    widget::{button, column, scrollable, text},
};

#[derive(Debug, Clone)]
pub enum Message {
    LoadStig,
}

#[derive(Default)]
pub struct State {
    stig: Stig, // Current stig on the screen.
}

pub fn update(state: &mut State, message: Message) {
    match message {
        Message::LoadStig => {
            state.stig = load_stig("info.txt").unwrap();
        }
    }
}

pub fn view(state: &State) -> Element<'_, Message> {
    let col = column![
        text("Version: ".to_string() + &state.stig.version),
        text("Introduction: ".to_string() + &state.stig.introduction),
        text("Description: ".to_string() + &state.stig.description),
        text("Check Text: ".to_string() + &state.stig.check_text),
        text("Fix Text: ".to_string() + &state.stig.fix_text),
        button(text("Load Stig")).on_press(Message::LoadStig),
    ]
    .width(Fill);

    scrollable(col).into()
}
