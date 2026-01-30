use crate::stig::Stig;
use iced::Element;
use iced::Event;
use iced::Length::{Fill, FillPortion};
use iced::widget::{Button, Container, button, column, container, row, text, text_input};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub enum Message {
    // Changing screens.
    DisplayMainScreen(MainScreen),
    // Changing state.
    ChangeDisplayedStig(Stig),
    // Change the home file path currently being viewed.
    ChangedFilePathStr(String),

    Event(Event),
}

/// The main displayed screen of the application.
/// Displays a list of stigs on the left, and the currently being viewed stig on the right.
#[derive(Clone)]
pub struct MainScreen {
    stig_list: Vec<Stig>,
    displayed_stig: Option<Stig>,
}

impl MainScreen {
    pub const fn new() -> Self {
        return MainScreen {
            stig_list: Vec::new(),
            displayed_stig: None,
        };
    }

    // Return a container of the main screen widgets to be drawn to the screen.
    pub fn get_container(&self) -> Container<'_, Message> {
        let mut buttons_vec = Vec::new();
        let mut displayed: Container<'_, Message>;

        for stig in &self.stig_list {
            buttons_vec.push(Box::new(self.get_stig_button(stig)));
        }

        let mut button_col = column![];

        for button in buttons_vec {
            button_col = button_col.push(*button);
        }

        if let Some(displayed_stig) = &self.displayed_stig {
            displayed = self.get_displayed_stig(displayed_stig);
        } else {
            displayed = self.get_no_stig_displayed();
        }

        return container(row![
            button_col.width(FillPortion(1)),
            displayed.width(FillPortion(5))
        ])
        .height(Fill);
    }

    // Switch the main sting being displayed.
    pub fn switch_displayed(&mut self, new_stig: Stig) {
        self.displayed_stig = Some(new_stig);
    }

    // Purposefully display no stig.
    pub fn display_none(&mut self) {
        self.displayed_stig = None;
    }

    // Get a nice container displaying all the information of a stig.
    // Use to display the selected stig.
    fn get_displayed_stig(&self, stig: &Stig) -> Container<'_, Message> {
        let col = column![
            text(stig.version.clone()),
            text(stig.intro.clone()),
            text(stig.desc.clone()),
            text(stig.check_text.clone()),
            text(stig.fix_text.clone())
        ];

        return container(col);
    }

    // Get a nice button with the stigs version on it.
    // Used to display selectable stigs in the application.
    fn get_stig_button(&self, stig: &Stig) -> Button<'_, Message> {
        return button(text(stig.version.clone()));
    }

    // What should be displayed when no stig is selected or found to be displayed.
    // This container is what will be displayed.
    fn get_no_stig_displayed(&self) -> Container<'_, Message> {
        return container(text("Nothing to display!"));
    }
}

/// The screen where the user chooses a base directory to load stigs from.
/// A local home directory where any child stig file is loaded.
#[derive(Clone)]
pub struct FilePickScreen {
    pub path_string: String,

    path: Option<PathBuf>,
}

pub enum FilePickError {
    FileToStrError,
}

impl FilePickScreen {
    pub const fn new() -> Self {
        FilePickScreen {
            path: None,
            path_string: String::new(),
        }
    }

    /// Change the filepath saved internally to the path string buffer.
    /// Will return an error if the path does not exist.
    /// Will set the internal path to None when an error occurs, discarding
    /// any path saved there.
    pub fn change_filepath(&mut self) -> Result<(), FilePickError> {
        let path = Path::new(&self.path_string);

        if path.exists() {
            self.path = Some(path.to_owned());

            return Ok(());
        } else {
            self.path = None;

            return Err(FilePickError::FileToStrError);
        }
    }

    /// Return the container of this screen that should be drawn to the users screen.
    pub fn get_container(&self) -> Container<'_, Message> {
        container(
            text_input("Type path here...", &self.path_string)
                .on_input(Message::ChangedFilePathStr),
        )
        .center(Fill)
        .padding(100)
    }
}
