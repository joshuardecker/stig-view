use iced::color;
use iced::keyboard;
use iced::keyboard::key;
use iced::theme;
use iced::theme::Custom;
use iced::widget::Id;
use iced::widget::text_editor;
use iced::window;
use iced::window::icon::*;
use iced::{Subscription, Task, Theme};
use image::ImageFormat;
use regex::Regex;
use rfd::AsyncFileDialog;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::db::{DB, Data, Pinned};
use crate::preload_assets::Assets;
use crate::stig::Stig;

/// This applications state.
#[derive(Debug, Clone)]
pub struct App {
    pub db: DB,
    pub displayed: Option<String>,
    pub content: [text_editor::Content; 6],
    pub popup: Option<Popup>,
    pub cmd_input: String,
    pub assets: Assets,
    // Not known at when creating state.
    pub window_id: Option<window::Id>,
}

/// Every way the state of the application can change.
#[derive(Debug, Clone)]
pub enum Message {
    KeyPressed(keyboard::Event),

    OpenFileSelect,
    OpenFolderSelect,
    OpenFile(Option<PathBuf>),
    OpenFolder(Option<PathBuf>),
    PushStigToContent,
    SetDisplayed(String),

    SelectContent(text_editor::Action, usize),

    SwitchNext,

    ToggleCmdInput,
    FocusCmdInput(Id),
    ChangeCmdInput(String),
    SubmitCmdInput,

    ChangePopup(Option<Popup>),

    UserPin(String),

    // Used when an async task finishes with no return value.
    Done,

    GetWindowId(Option<window::Id>),
    CloseApp,
    MinimizeApp,
    ToggleFullscreenApp,
    MoveWindow,
}

// A popup that appears over the main content of the application.
#[derive(Debug, Clone)]
pub enum Popup {
    CommandPrompt,
}

/// Commands that can be sent by the user from the cmd prompt popup.
#[derive(Debug, Clone)]
pub enum UserCommand {
    SearchForName(String),
    SearchForKeyword(String),
    Reset,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            App {
                db: DB::new(),
                displayed: None,
                content: [
                    text_editor::Content::new(),
                    text_editor::Content::new(),
                    text_editor::Content::new(),
                    text_editor::Content::new(),
                    text_editor::Content::new(),
                    text_editor::Content::new(),
                ],
                popup: None,
                cmd_input: String::new(),
                assets: Assets::new(),
                window_id: None,
            },
            window::oldest().map(Message::GetWindowId),
        )
    }

    /// Updates the state of the application given the message.
    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            // Handle keys being pressed, looking for keybind combinations.
            // I dont like how iced handles this, it seems verbose.
            Message::KeyPressed(event) => match &event {
                keyboard::Event::KeyPressed {
                    key: key::Key::Character(key_smolstr),
                    modifiers,
                    ..
                } => match key_smolstr.as_str() {
                    "q" if modifiers.control() => return iced::exit(),
                    "i" if modifiers.control() => return Task::done(Message::OpenFileSelect),
                    "o" if modifiers.control() => return Task::done(Message::OpenFolderSelect),
                    "p" if modifiers.control() => return Task::done(Message::ToggleCmdInput),
                    _ => Task::none(),
                },
                keyboard::Event::KeyPressed {
                    key: key::Key::Named(key_name),
                    modifiers,
                    ..
                } => match key_name {
                    key::Named::Tab if modifiers.control() => Task::done(Message::SwitchNext),
                    _ => Task::none(),
                },
                _ => Task::none(),
            },

            // Open the single file select system menu.
            Message::OpenFileSelect => Task::perform(
                async {
                    let home_dir = std::env::home_dir().unwrap_or(PathBuf::from("/"));

                    let file_handle = AsyncFileDialog::new()
                        .add_filter("text", &["txt"])
                        .set_directory(home_dir)
                        .set_title("Stig View - Select File")
                        .pick_file()
                        .await;

                    if let Some(file_handle) = file_handle {
                        Some(file_handle.path().to_path_buf())
                    } else {
                        None
                    }
                },
                Message::OpenFile,
            ),
            // Open the folder select system menu.
            Message::OpenFolderSelect => Task::perform(
                async {
                    let home_dir = std::env::home_dir().unwrap_or(PathBuf::from("/"));

                    let folder_handle = AsyncFileDialog::new()
                        .add_filter("text", &["txt"])
                        .set_directory(home_dir)
                        .set_title("Stig View - Select Folder")
                        .pick_folder()
                        .await;

                    if let Some(folder_handle) = folder_handle {
                        Some(folder_handle.path().to_path_buf())
                    } else {
                        None
                    }
                },
                Message::OpenFolder,
            ),
            // Add the file into the program for the user to see.
            Message::OpenFile(path) => {
                let db = self.db.clone();

                if let Some(path) = path {
                    Task::future(async move {
                        let stig = Stig::from_xylok_txt(path);

                        if let Some(stig) = stig {
                            let name = stig.version.clone();
                            let stig = Arc::new(stig);
                            let data = Data::new(stig);

                            db.insert(name.clone(), data).await;

                            return Message::SetDisplayed(name);
                        }

                        Message::Done
                    })
                } else {
                    Task::none()
                }
            }
            // Open a folder and search all of its contents, including subfolders for
            // stigs to add.
            Message::OpenFolder(path) => {
                let mut db = self.db.clone();

                if let Some(path) = path {
                    Task::future(async move {
                        db.clean().await;

                        load_dir(path, db.clone()).await;

                        let snapshot = db.snapshot().await;

                        let first = snapshot.first_key_value();

                        if let Some(first) = first {
                            return Message::SetDisplayed(first.0.clone());
                        }

                        Message::SetDisplayed("".to_string())
                    })
                } else {
                    Task::none()
                }
            }
            // Push contents of the selected stig to the screen for the user to see.
            Message::PushStigToContent => {
                if let Some(name) = &self.displayed {
                    let rt = Runtime::new().unwrap();

                    let displayed_data = rt.block_on(self.db.get(&name));

                    if let Some(displayed_data) = displayed_data {
                        let stig = displayed_data.get_stig();

                        self.content[0] = text_editor::Content::with_text(&stig.version);
                        self.content[1] = text_editor::Content::with_text(&stig.intro);
                        self.content[2] = text_editor::Content::with_text(&stig.desc);
                        self.content[3] = text_editor::Content::with_text(&stig.check_text);
                        self.content[4] = text_editor::Content::with_text(&stig.fix_text);
                        self.content[5] = text_editor::Content::with_text(&stig.similar_checks);
                    }
                }

                Task::none()
            }
            Message::SetDisplayed(name) => {
                self.displayed = Some(name);

                Task::done(Message::PushStigToContent)
            }

            // Handle the user selecting text to copy and paste.
            // Block the user from adding / removing text.
            Message::SelectContent(action, index) => {
                if let text_editor::Action::Edit(_) = action {
                    return Task::none();
                }

                self.content[index.to_owned()].perform(action.clone());

                Task::none()
            }

            // Switch to the next stig.
            // Used for the keybind control + tab.
            Message::SwitchNext => {
                let db = self.db.clone();
                let displayed_name = self.displayed.clone();

                if let Some(displayed_name) = displayed_name {
                    Task::future(async move {
                        let snapshot = db.snapshot().await;

                        let mut iter = snapshot.iter();

                        let _ = iter.find(|entry| *entry.0 == displayed_name);

                        let entry: Option<(&String, &Data)> = iter.next();

                        if let Some(entry) = entry {
                            return Message::SetDisplayed(entry.0.to_owned());
                        }

                        let first = snapshot.first_key_value();

                        if let Some(first) = first {
                            return Message::SetDisplayed(first.0.clone());
                        }

                        Message::SetDisplayed("".to_string())
                    })
                } else {
                    Task::none()
                }
            }

            // Toggle whether the cmd prompt is open or closed.
            Message::ToggleCmdInput => match &self.popup {
                Some(Popup::CommandPrompt) => {
                    return Task::done(Message::ChangePopup(None));
                }
                None => {
                    return Task::done(Message::ChangePopup(Some(Popup::CommandPrompt)));
                }
            },
            // Automatically have the cmd prompt text box selected, that way the user does not
            // have to manually click into it every time.
            Message::FocusCmdInput(id) => iced::widget::operation::focus(id),
            Message::ChangeCmdInput(input) => {
                self.cmd_input = input;
                Task::none()
            }
            // When the user presses enter in the cmd prompt.
            Message::SubmitCmdInput => {
                let user_command = parse_command(&self.cmd_input);

                if let Some(user_command) = user_command {
                    let db = self.db.clone();

                    Task::future(async move {
                        run_search_cmd(user_command, db).await;

                        Message::Done
                    })
                } else {
                    Task::none()
                }
            }

            // Switch the popup displayed.
            Message::ChangePopup(new_popup) => {
                self.popup = new_popup;
                Task::none()
            }
            // User manually pins a stig.
            Message::UserPin(name) => {
                let db = self.db.clone();

                Task::future(async move {
                    let stig = db.get(&name).await;

                    if let Some(mut stig) = stig {
                        match stig.get_pin() {
                            Pinned::Not => stig.set_pin(Pinned::ByUser),
                            Pinned::ByUser => stig.set_pin(Pinned::Not),
                            Pinned::ByFilter => return Message::Done,
                        }

                        db.insert(name, stig).await;
                    }

                    Message::Done
                })
            }

            Message::Done => Task::none(),

            // Run at the beginning of the program.
            Message::GetWindowId(id) => {
                self.window_id = id;

                Task::batch(vec![
                    window::toggle_decorations(self.window_id.unwrap()),
                    window::set_icon(
                        self.window_id.unwrap(),
                        from_file_data(&self.assets.app_icon, Some(ImageFormat::Png))
                            .expect("Could not load app icon!"),
                    ),
                ])
            }
            Message::CloseApp => iced::exit(),
            Message::ToggleFullscreenApp => window::toggle_maximize(self.window_id.unwrap()),
            Message::MinimizeApp => window::minimize(self.window_id.unwrap(), true),
            // Allow the user to drag the window by grabbing the top.
            Message::MoveWindow => window::drag(self.window_id.unwrap()),
        }
    }

    /// Listen for all keyboard inputs.
    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(|event| Some(Message::KeyPressed(event)))
    }

    /// The theme of the application.
    pub fn theme(&self) -> Theme {
        // Dark theme:
        let palette = theme::Palette {
            background: color!(0x1B1C1C),
            text: color!(0xE6E6E6),
            primary: color!(0xA2A2D0),
            success: color!(0x188B6C),
            warning: color!(0xffc14e),
            danger: color!(0xc3423f),
        };

        // Light theme
        /*let palette = theme::Palette {
            background: color!(0xDFD7D5),
            text: color!(0x1B1C1C),
            primary: color!(0x444488),
            success: color!(0x188B6C),
            warning: color!(0xffc14e),
            danger: color!(0xc3423f),
        };*/

        Theme::Custom(Arc::new(Custom::new(String::from("Custom Dark"), palette)))
    }
}

/// Load all found stigs from a given directory.
/// Searches all subfolders as well.
/// If I had to rewrite this, I would return stig wrappers,
/// rather than raw stigs.
async fn load_dir(path: PathBuf, db: DB) {
    let mut dirs: Vec<Box<PathBuf>> = vec![Box::new(path)];
    let mut next_dirs: Vec<Box<PathBuf>> = vec![];

    let mut txts = Vec::new();

    loop {
        for dir in dirs.iter() {
            for entry in dir.read_dir().expect("read_dir io failed!") {
                if let Ok(entry) = entry {
                    if entry.path().is_dir() {
                        next_dirs.push(Box::new(entry.path()));
                        continue;
                    }

                    if let Some(extension) = entry.path().as_path().extension() {
                        if extension
                            .to_str()
                            .expect("Could not convert file extension to string!")
                            == "txt"
                        {
                            txts.push(Box::new(entry.path()));
                        }
                    }
                }
            }
        }

        // To make rust happy, we save subfolders found in a new variable.
        // At the end of the loop, iterate through all of the new subfolders.
        // Repeat until no more sub folders.
        dirs = next_dirs;
        next_dirs = Vec::new();

        if dirs.len() == 0 {
            break;
        }
    }

    // Loop through all text files found.
    // If it is a xylok generated txt stig file, save it.
    for txt in txts {
        if let Some(stig) = Stig::from_xylok_txt(&*txt) {
            let name = stig.version.clone();
            let stig = Arc::new(stig);
            let data = Data::new(stig);

            db.insert(name, data).await;
        }
    }
}

/// Parse the given str from the cmd prompt into a command.
pub fn parse_command(input: &str) -> Option<UserCommand> {
    let cmd_regex = Regex::new(r"(\w+)\s*(.*)").unwrap();
    let captures = cmd_regex.captures(input)?;

    match captures[1].to_string().as_str() {
        "find" => Some(UserCommand::SearchForKeyword(captures[2].to_string())),
        "search" => Some(UserCommand::SearchForKeyword(captures[2].to_string())),
        "name" => Some(UserCommand::SearchForName(captures[2].to_string())),
        "title" => Some(UserCommand::SearchForName(captures[2].to_string())),
        "reset" => Some(UserCommand::Reset),
        _ => None,
    }
}

/// Run the given command, and return the modified group of stigs.
async fn run_search_cmd(cmd: UserCommand, db: DB) {
    match cmd {
        UserCommand::SearchForKeyword(keyword) => {
            let re = Regex::new(&keyword).ok().expect("Bad regex!");

            let snapshot = db.snapshot().await;

            for (name, data) in snapshot.iter() {
                if let Pinned::ByUser = data.get_pin() {
                    continue;
                }

                let mut is_match = false;

                is_match |= re.is_match(&data.get_stig().version);
                is_match |= re.is_match(&data.get_stig().intro);
                is_match |= re.is_match(&data.get_stig().desc);
                is_match |= re.is_match(&data.get_stig().check_text);
                is_match |= re.is_match(&data.get_stig().fix_text);
                is_match |= re.is_match(&data.get_stig().similar_checks);

                if is_match {
                    let mut data = data.to_owned();
                    data.set_pin(Pinned::ByFilter);

                    db.insert(name.to_owned(), data).await;

                    continue;
                }

                if let Pinned::ByFilter = data.get_pin() {
                    let mut data = data.to_owned();
                    data.set_pin(Pinned::Not);

                    db.insert(name.to_owned(), data).await;
                }
            }
        }
        UserCommand::SearchForName(name) => {
            let re = Regex::new(&name).ok().expect("Bad regex!");

            let snapshot = db.snapshot().await;

            for (name, data) in snapshot.iter() {
                if let Pinned::ByUser = data.get_pin() {
                    continue;
                }

                let is_match = re.is_match(&data.get_stig().version);

                if is_match {
                    let mut data = data.to_owned();
                    data.set_pin(Pinned::ByFilter);

                    db.insert(name.to_owned(), data).await;

                    continue;
                }

                if let Pinned::ByFilter = data.get_pin() {
                    let mut data = data.to_owned();
                    data.set_pin(Pinned::Not);

                    db.insert(name.to_owned(), data).await;
                }
            }
        }
        UserCommand::Reset => {
            let snapshot = db.snapshot().await;

            for (name, data) in snapshot.iter() {
                if let Pinned::ByFilter = data.get_pin() {
                    let mut data = data.to_owned();
                    data.set_pin(Pinned::Not);

                    db.insert(name.to_owned(), data).await;
                }
            }
        }
    }
}
