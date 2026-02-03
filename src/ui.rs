use crate::app::{FilePickScreen, FileSelectScreen, MainScreen, Message, Screen};
use crate::stig::Stig;
use iced::{
    Element, Subscription, Task,
    keyboard::{self, key},
};

// Current state of the application.
pub struct State {
    screen: Screen,
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ChangeScreen(screen) => {
            state.screen = screen;
            Task::none()
        }

        Message::SwitchStig(new_stig) => {
            if let Screen::MainScreen(ref mut current_screen) = state.screen {
                current_screen.switch_displayed(new_stig);
            }

            Task::none()
        }

        Message::LoadStigs(stigs) => {
            if let Screen::FileSelectScreen(ref screen) = state.screen {
                let mut main_screen = MainScreen::new();

                main_screen.stig_list = stigs;
                main_screen.switch_displayed(main_screen.stig_list[0].clone());

                return Task::done(Message::ChangeScreen(Screen::MainScreen(main_screen)));
            }

            Task::none()
        }

        Message::PressEnter => {
            // If the file pick screen is the current screen.
            if let Screen::FilePickScreen(ref mut current_screen) = state.screen {
                match current_screen.change_filepath() {
                    Ok(_) => {
                        if let Ok(stigs) = current_screen.get_stigs() {
                            return Task::batch(vec![
                                Task::done(Message::ChangeScreen(Screen::MainScreen(
                                    MainScreen::new(),
                                ))),
                                Task::done(Message::SwitchStig(stigs[0].clone())),
                                Task::done(Message::LoadStigs(stigs)),
                            ]);
                        }
                    }
                    // todo: Handle Error differently later.
                    Err(_) => (),
                }
            }

            if let Screen::FileSelectScreen(ref mut screen) = state.screen {
                // todo: care about Ok() or Err() from this function.
                let _ = screen.switch_dir();
            }

            Task::none()
        }

        Message::TextInput(input) => {
            if let Screen::FilePickScreen(ref mut screen) = state.screen {
                screen.path_string = input.clone();
            }

            if let Screen::FileSelectScreen(ref mut screen) = state.screen {
                screen.user_input_dir = input.clone();
            }

            Task::none()
        }

        Message::DisplayNewFiles(new_path) => {
            if let Screen::FileSelectScreen(ref mut screen) = state.screen {
                screen.user_input_dir = new_path
                    .into_os_string()
                    .into_string()
                    .unwrap_or(String::from("oops!")); // todo: better error handling.
            }

            // Send the press enter signal to update what is displayed in the file selector.
            Task::done(Message::PressEnter)
        }
    }
}

pub fn view(state: &State) -> Element<'_, Message> {
    match &state.screen {
        Screen::MainScreen(main_screen) => return main_screen.get_view(),
        Screen::FilePickScreen(file_pick_screen) => return file_pick_screen.get_view(),
        Screen::FileSelectScreen(file_select_screen) => return file_select_screen.get_view(),
    }
}

pub fn new() -> State {
    return State {
        screen: Screen::FileSelectScreen(FileSelectScreen::new().unwrap()),
    };
}

pub fn subscription(_state: &State) -> Subscription<Message> {
    keyboard::listen().filter_map(|event| match event {
        keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key::Named::Enter),
            ..
        } => Some(Message::PressEnter),
        _ => None,
    })
}
