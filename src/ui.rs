use iced::{
    Element, Event, Subscription, event,
    keyboard::{self, key},
    window,
};

use crate::app::{FilePickScreen, MainScreen, Message};

// Current Screen the application is displaying.
enum Screen {
    MainScreen(MainScreen),
    FilePickScreen(FilePickScreen),
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
        Message::ChangedFilePathStr(content) => {
            if let Screen::FilePickScreen(ref mut screen) = state.screen {
                screen.path_string = content;
            }
        }
        Message::Event(event) => match event {
            // Handle pressing the enter key when in the file pick menu.
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Enter),
                modifiers,
                ..
            }) => {
                if let Screen::FilePickScreen(ref mut screen) = state.screen {
                    // todo: add functionality and error handling.
                    let _ = screen.change_filepath();

                    screen.path_string = "Hi there!".to_string();
                }
            }
            // Don't care otherwise.
            _ => {
                return;
            }
        },
    }
}

pub fn view(state: &State) -> Element<'_, Message> {
    match &state.screen {
        Screen::MainScreen(main_screen) => return main_screen.get_container().into(),
        Screen::FilePickScreen(file_pick_screen) => return file_pick_screen.get_container().into(),
    }
}

pub fn new() -> State {
    return State {
        screen: Screen::FilePickScreen(FilePickScreen::new()),
    };
}

pub fn subscription(state: &State) -> Subscription<Message> {
    event::listen().map(Message::Event)
}
