use crate::stig::Stig;
use iced::Element;
use iced::Length::{Fill, FillPortion};
use iced::widget::{
    Button, Container, button, column, container, row, scrollable, text, text_input,
};
use std::path::{Path, PathBuf};

// Current Screen the application is displaying.
#[derive(Clone)]
pub enum Screen {
    MainScreen(MainScreen),
    FilePickScreen(FilePickScreen),
}

#[derive(Clone)]
pub enum Message {
    ChangeScreen(Screen),
    LoadStigs(Vec<Box<Stig>>),
    SwitchStig(Box<Stig>),
    PressEnter,
    TextInput(String),
}

/// The main displayed screen of the application.
/// Displays a list of stigs on the left, and the currently being viewed stig on the right.
#[derive(Clone)]
pub struct MainScreen {
    pub stig_list: Vec<Box<Stig>>,
    displayed_stig: Option<Box<Stig>>,
}

impl MainScreen {
    pub const fn new() -> Self {
        return MainScreen {
            stig_list: Vec::new(),
            displayed_stig: None,
        };
    }

    // Return a container of the main screen widgets to be drawn to the screen.
    pub fn get_view(&self) -> Element<'_, Message> {
        let mut buttons_vec = Vec::new();
        let displayed: Container<'_, Message>;

        for stig in &self.stig_list {
            buttons_vec.push(Box::new(self.get_stig_button(stig.clone())));
        }

        let mut button_col = column![];

        for button in buttons_vec {
            button_col = button_col.push(*button);
        }

        if let Some(displayed_stig) = &self.displayed_stig {
            displayed = self.get_displayed_stig(displayed_stig.clone());
        } else {
            displayed = self.get_no_stig_displayed();
        }

        container(row![
            button_col.width(FillPortion(1)),
            displayed.width(FillPortion(5))
        ])
        .height(Fill)
        .into()
    }

    // Switch the main sting being displayed.
    pub fn switch_displayed(&mut self, new_stig: Box<Stig>) {
        self.displayed_stig = Some(new_stig);
    }

    // Purposefully display no stig.
    pub fn display_none(&mut self) {
        self.displayed_stig = None;
    }

    // Get a nice container displaying all the information of a stig.
    // Use to display the selected stig.
    fn get_displayed_stig(&self, stig: Box<Stig>) -> Container<'_, Message> {
        let col = column![
            text(stig.version.clone()),
            text(stig.intro.clone()),
            text(stig.desc.clone()),
            text(stig.check_text.clone()),
            text(stig.fix_text.clone())
        ];

        return container(scrollable(col));
    }

    // Get a nice button with the stigs version on it.
    // Used to display selectable stigs in the application.
    fn get_stig_button(&self, stig: Box<Stig>) -> Button<'_, Message> {
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

    pub stig_list: Vec<Box<Stig>>,
}

#[derive(Debug)]
pub enum FilePickError {
    FileToStrError,
    NoStigsError,
}

impl FilePickScreen {
    pub const fn new() -> Self {
        FilePickScreen {
            path: None,
            path_string: String::new(),
            stig_list: Vec::new(),
        }
    }

    /// Change the filepath saved internally to the path string buffer.
    /// Will return an error if the path does not exist.
    /// Will set the internal path to None when an error occurs, discarding
    /// any path saved there.
    pub fn change_filepath(&mut self) -> Result<(), FilePickError> {
        let path = Path::new(&self.path_string);

        if !path.exists() {
            self.path = None;
            return Err(FilePickError::FileToStrError);
        }

        self.path = Some(path.to_owned());

        return Ok(());
    }

    pub fn get_stigs(&self) -> Result<Vec<Box<Stig>>, FilePickError> {
        let path = self.path.clone().ok_or(FilePickError::NoStigsError)?;

        if path.ends_with("info.txt") {
            // todo: make return more than one stig.
            return Ok(vec![Box::new(Stig::from_xylok(path).unwrap())]);
        }

        Err(FilePickError::NoStigsError)
    }

    /// Return the container of this screen that should be drawn to the users screen.
    pub fn get_view(&self) -> Element<'_, Message> {
        container(text_input("Type path here...", &self.path_string).on_input(Message::TextInput))
            .center(Fill)
            .padding(100)
            .into()
    }
}
