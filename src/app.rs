use crate::stig::Stig;
use directories::UserDirs;
use iced::Element;
use iced::Length::{Fill, FillPortion};
use iced::widget::text::Alignment::Center;
use iced::widget::{
    Button, Container, Row, button, column, container, row, scrollable, space, text, text_input,
};
use std::ffi::OsString;
use std::fs::{DirEntry, ReadDir};
use std::path::{Path, PathBuf};
use std::ptr::read;

// Current Screen the application is displaying.
#[derive(Clone)]
pub enum Screen {
    MainScreen(MainScreen),
    FilePickScreen(FilePickScreen),
    FileSelectScreen(FileSelectScreen),
}

#[derive(Clone)]
pub enum Message {
    ChangeScreen(Screen),
    LoadStigs(Vec<Box<Stig>>),
    SwitchStig(Box<Stig>),
    PressEnter,
    TextInput(String),
    DisplayNewFiles(PathBuf),
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

    pub fn get_view(&self) -> Element<'_, Message> {
        let mut buttons_vec = Vec::new();
        let displayed: Container<'_, Message>;

        for stig in &self.stig_list {
            buttons_vec.push(Box::new(self.get_stig_button(stig.clone())));
        }

        let mut button_col = column![text("Versions:")].align_x(Center);

        for button in buttons_vec {
            button_col = button_col.push(*button);
        }

        if let Some(displayed_stig) = &self.displayed_stig {
            displayed = self.get_displayed_stig(displayed_stig.clone());
        } else {
            displayed = self.get_no_stig_displayed();
        }

        container(row![
            scrollable(button_col).spacing(10).width(FillPortion(1)),
            displayed.width(FillPortion(5)) // Already a scrollable.
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
            text("Version:").size(32),
            text(stig.version.clone()),
            space().height(20),
            text("Introduction:").size(32),
            text(stig.intro.clone()),
            space().height(20),
            text("Description:").size(32),
            text(stig.desc.clone()),
            space().height(20),
            text("Check Text / Commands:").size(32),
            text(stig.check_text.clone()),
            space().height(20),
            text("Fix Text / Commands:").size(32),
            text(stig.fix_text.clone()),
            space().height(20),
        ];

        return container(scrollable(col).spacing(100));
    }

    // Get a nice button with the stigs version on it.
    // Used to display selectable stigs in the application.
    fn get_stig_button(&self, stig: Box<Stig>) -> Button<'_, Message> {
        return button(text(stig.version.clone())).on_press(Message::SwitchStig(stig));
    }

    // What should be displayed when no stig is selected or found to be displayed.
    // This container is what will be displayed.
    fn get_no_stig_displayed(&self) -> Container<'_, Message> {
        return container(text("Nothing to display!"));
    }
}

/// Depricated for FileSelectScreen.
/// todo: remove depricated code.
#[derive(Clone)]
pub struct FilePickScreen {
    pub path_string: String,
    path: Option<PathBuf>,

    pub stig_list: Vec<Box<Stig>>,
}

#[derive(Debug)]
pub enum FilePickError {
    DoesntExist,
    NoStigs,
    BadTxtFile,
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
            return Err(FilePickError::DoesntExist);
        }

        self.path = Some(path.to_owned());

        return Ok(());
    }

    pub fn get_stigs(&self) -> Result<Vec<Box<Stig>>, FilePickError> {
        let path = self.path.clone().ok_or(FilePickError::NoStigs)?;

        if path.extension().ok_or(FilePickError::DoesntExist)? == "txt" {
            let stig = Stig::from_xylok(path).map_err(|_| FilePickError::BadTxtFile)?;

            return Ok(vec![Box::new(stig)]);
        }

        Err(FilePickError::NoStigs)
    }

    pub fn get_view(&self) -> Element<'_, Message> {
        container(text_input("Type path here...", &self.path_string).on_input(Message::TextInput))
            .center(Fill)
            .padding(100)
            .into()
    }
}

#[derive(Clone)]
pub struct FileSelectScreen {
    pub user_input_dir: String,

    dir: PathBuf,
}

#[derive(Debug)]
pub enum FileSelectError {
    CantGetHome, // Cant get the users home dir.
    CantGetDirItems,
    InvalidDir, // User gave an invalid dir.
}

impl FileSelectScreen {
    /// By default, points to the users home dir.
    /// Returns an error if it cannot determine this.
    pub fn new() -> Result<Self, FileSelectError> {
        if let Some(user_dir) = UserDirs::new() {
            let home_dir = user_dir.home_dir().to_owned();

            // todo: better error handling.
            let home_dir_string = home_dir
                .clone()
                .into_os_string()
                .into_string()
                .unwrap_or(String::from("Error occured!"));

            return Ok(Self {
                user_input_dir: home_dir_string,
                dir: home_dir,
            });
        }

        Err(FileSelectError::CantGetHome)
    }

    pub fn get_view(&self) -> Element<'_, Message> {
        let top_row = row![
            space().width(20),
            button(text("↑")).on_press(Message::DisplayNewFiles(
                self.dir.parent().unwrap_or(&self.dir).to_owned()
            )),
            space().width(50),
            text_input("Path here...", &self.user_input_dir).on_input(Message::TextInput),
            space().width(20),
        ];

        ///
        let mut col = column![top_row, space().height(50)];

        match self.get_dir_items() {
            Ok(read_dir) => {
                for item in read_dir {
                    if let Ok(ok_item) = item {
                        let ok_item_path = ok_item.path();

                        if ok_item_path.is_file()
                            && (ok_item_path.extension().unwrap_or(&OsString::from("magic"))
                                != "txt")
                        {
                            continue;
                        }

                        if ok_item_path
                            .file_name()
                            .unwrap_or(&OsString::from("magic"))
                            .to_str()
                            .unwrap_or("magic")
                            .starts_with(".")
                        {
                            continue;
                        }

                        col = col.push(self.dir_item_to_button(ok_item));
                    }
                }
            }
            Err(_) => {}
        }

        scrollable(col).spacing(10).into()
    }

    fn get_dir_items(&self) -> Result<ReadDir, FileSelectError> {
        if !self.dir.is_dir() {
            return Err(FileSelectError::CantGetDirItems);
        }

        let read_dir = self
            .dir
            .read_dir()
            .map_err(|_| FileSelectError::CantGetDirItems)?;

        Ok(read_dir)
    }

    fn dir_item_to_button(&self, item: DirEntry) -> Button<'_, Message> {
        if item.path().is_dir() {
            return button(text(
                "🗀  ".to_string() +
                // todo: better error handling.
                &item.file_name()
                    .into_string()
                    .unwrap_or(String::from("Error Occured!")),
            ))
            .on_press(Message::DisplayNewFiles(item.path()));
        }

        button(text(
            "🗎  ".to_string() +
            // todo: better error handling.
            &item.file_name()
                .into_string()
                .unwrap_or(String::from("Error Occured!")),
        ))
        .on_press(Message::LoadStigs(vec![Box::new(
            Stig::from_xylok(&item.path()).unwrap(), // todo: better error handling.
        )]))
    }

    /// Attempt to switch the dir given the internal user provided dir string.
    pub fn switch_dir(&mut self) -> Result<(), FileSelectError> {
        let path = PathBuf::from(self.user_input_dir.clone());

        match &path.try_exists() {
            Ok(true) => {
                // Cant switch dir if the given path is to a file, not a dir.
                if path.is_file() {
                    return Err(FileSelectError::InvalidDir);
                }

                self.dir = path;
                return Ok(());
            }
            Ok(false) => return Err(FileSelectError::InvalidDir),
            Err(_) => return Err(FileSelectError::CantGetDirItems),
        }
    }
}
