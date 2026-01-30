use iced::{Element, Subscription, event, keyboard, window};

use crate::app::{MainScreen, Message};

// Current Screen the application is displaying.
enum Screen {
    MainScreen(MainScreen),
}

// Current state of the application.
pub struct State {
    screen: Screen,
}

pub fn update(state: &mut State, message: Message) {
    match message {
        Message::DisplayMainScreen(main_screen) => state.screen = Screen::MainScreen(main_screen),
        Message::ChangeDisplayedStig(stig) => {
            if let Screen::MainScreen(ref mut main_screen) = state.screen {
                main_screen.switch_displayed(stig);
            }
        }
    }
}

pub fn view(state: &State) -> Element<'_, Message> {
    match &state.screen {
        Screen::MainScreen(main_screen) => return main_screen.get_container().into(),
    }
}

pub fn new() -> State {
    return State {
        screen: Screen::MainScreen(MainScreen::new()),
    };
}
