use iced::color;
use iced::keyboard;
use iced::keyboard::key;
use iced::theme;
use iced::theme::Custom;
use iced::widget::Id;
use iced::widget::text_editor;
use iced::window;
use iced::window::icon::*;
use iced::{Element, Subscription, Task, Theme};
use image::ImageFormat;
use regex::Regex;
use rfd::AsyncFileDialog;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::preload_assets::Assets;
use crate::sgroup::{Pinned, SGroup, StigWrapper};
use crate::stig::Stig;

/// This applications state.
#[derive(Debug, Clone)]
pub struct App {
    pub list: Arc<RwLock<SGroup>>,
    pub displayed: Arc<RwLock<Option<Box<StigWrapper>>>>,
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
    SetStigVec(Vec<Box<Stig>>),
    PushStigToContent,
    StigsSorted(Option<SGroup>),

    SelectContent(text_editor::Action, usize),

    SwitchDisplayed(Uuid),
    SwitchNext,

    ToggleCmdInput,
    FocusCmdInput(Id),
    ChangeCmdInput(String),
    SubmitCmdInput,

    ChangePopup(Option<Popup>),

    UserPin(Uuid),

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
    // Not an error value, a popup displaying there was an error.
    Error,
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
                list: Arc::new(RwLock::new(SGroup::new())),
                displayed: Arc::new(RwLock::new(None)),
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
                let sgroup_lock = self.list.clone();
                let displayed_lock = self.displayed.clone();

                if let Some(path) = path {
                    Task::future(async move {
                        let stig = Stig::from_xylok_txt(path);

                        if let Some(stig) = stig {
                            let stig = Box::new(stig);
                            sgroup_lock.write().unwrap().add(stig.clone());
                            *displayed_lock.write().unwrap() =
                                Some(sgroup_lock.read().unwrap().first());
                        }

                        Message::PushStigToContent
                    })
                } else {
                    Task::none()
                }
            }
            // Open a folder and search all of its contents, including subfolders for
            // stigs to add.
            Message::OpenFolder(path) => {
                if let Some(path) = path {
                    Task::perform(
                        async move { load_dir(path.clone()).await },
                        Message::SetStigVec,
                    )
                } else {
                    Task::none()
                }
            }
            // Given a vector of stigs, set the application to have those open.
            // This is different than adding to the already loaded stigs.
            Message::SetStigVec(stigs) => {
                if stigs.len() == 0 {
                    return Task::none();
                }

                let sgroup_lock = self.list.clone();
                let displayed_lock = self.displayed.clone();

                Task::future(async move {
                    sgroup_lock.write().unwrap().set_group(stigs);
                    *displayed_lock.write().unwrap() = Some(sgroup_lock.read().unwrap().first());

                    Message::PushStigToContent
                })
            }
            // Push contents of the selected stig to the screen for the user to see.
            Message::PushStigToContent => {
                let displayed_lock = self.displayed.clone();

                if let Some(stig) = &*displayed_lock.read().unwrap() {
                    self.content[0] = text_editor::Content::with_text(&stig.stig.version);
                    self.content[1] = text_editor::Content::with_text(&stig.stig.intro);
                    self.content[2] = text_editor::Content::with_text(&stig.stig.desc);
                    self.content[3] = text_editor::Content::with_text(&stig.stig.check_text);
                    self.content[4] = text_editor::Content::with_text(&stig.stig.fix_text);
                    self.content[5] = text_editor::Content::with_text(&stig.stig.similar_checks);
                }

                Task::none()
            }
            // When the stigs have been sorted, save this order to the app state here.
            Message::StigsSorted(sorted_stigs) => {
                let sgroup_lock = self.list.clone();

                Task::future(async move {
                    if let Some(sorted_stigs) = sorted_stigs {
                        *sgroup_lock.write().unwrap() = sorted_stigs;
                    }

                    Message::Done
                })
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

            // Switch which stig is being displayed.
            Message::SwitchDisplayed(uuid) => {
                let sgroup_lock = self.list.clone();
                let displayed_lock = self.displayed.clone();

                Task::future(async move {
                    let new_stig = sgroup_lock.read().unwrap().get_by_uuid(uuid);

                    if let Some(new_stig) = new_stig {
                        *displayed_lock.write().unwrap() = Some(new_stig);
                    }

                    Message::PushStigToContent
                })
            }
            // Switch to the next stig.
            // Used for the keybind control + tab.
            Message::SwitchNext => {
                let sgroup_lock = self.list.clone();
                let displayed_lock = self.displayed.clone();

                // Dead async code.
                // For some reason, switching this code to run async makes the RwLock get stuck on itself.
                // I cant get it to do that in sync land, so we will keep it there I guess.
                /*Task::future(async move {
                    let mut displayed = displayed_lock.read().unwrap();

                    let stig = displayed.clone();

                    if let Some(stig) = stig {
                        let next_stig = sgroup_lock.read().unwrap().get_next_wrapping(stig.uuid);

                        if let Some(next_stig) = next_stig {
                            Message::SwitchDisplayed(next_stig.uuid)
                        } else {
                            Message::Done
                        }
                    } else {
                        Message::Done
                    }
                })*/

                let mut displayed_stig = displayed_lock.write().unwrap();

                if let Some(stig) = displayed_stig.clone() {
                    let next_stig = sgroup_lock
                        .read()
                        .unwrap()
                        .get_next_wrapping(stig.uuid.clone());

                    *displayed_stig = next_stig;

                    Task::done(Message::PushStigToContent)
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
                _ => return Task::none(),
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
                    let list_clone = self.list.clone();
                    Task::perform(
                        async move { run_search_cmd(user_command, list_clone).await },
                        Message::StigsSorted,
                    )
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
            Message::UserPin(uuid) => {
                let sgroup_lock = self.list.clone();

                Task::perform(
                    async move {
                        let mut sgroup = sgroup_lock.read().unwrap().clone();
                        let stig_wrapper = sgroup.get_by_uuid(uuid);

                        if let Some(stig_wrapper) = stig_wrapper {
                            match stig_wrapper.pinned {
                                Pinned::ByUser => {
                                    sgroup.unpin(uuid);
                                    sgroup.sort_by_version();
                                    return Some(sgroup);
                                }
                                Pinned::Not => {
                                    sgroup.pin(uuid, Pinned::ByUser);
                                    sgroup.sort_by_version();
                                    return Some(sgroup);
                                }
                                Pinned::ByCmd => (),
                            }
                        }
                        Some(sgroup)
                    },
                    Message::StigsSorted,
                )
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

    /// Get the view that should be rendered to the screen.
    pub fn get_view(&self) -> Element<'_, Message> {
        if let Some(_) = *self.displayed.read().unwrap() {
            self.get_view_displayed()
        } else {
            self.get_view_none_displayed()
        }
    }

    /// Listen for all keyboard inputs.
    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(|event| Some(Message::KeyPressed(event)))
    }

    /// The theme of the application.
    pub fn theme(&self) -> Theme {
        let palette = theme::Palette {
            background: color!(0x1B1C1C),
            text: color!(0xE6E6E6),
            primary: color!(0xA2A2D0),
            success: color!(0x188B6C),
            warning: color!(0xffc14e),
            danger: color!(0xc3423f),
        };

        Theme::Custom(Arc::new(Custom::new(String::from("Custom Dark"), palette)))
    }
}

/// Load all found stigs from a given directory.
/// Searches all subfolders as well.
/// If I had to rewrite this, I would return stig wrappers,
/// rather than raw stigs.
async fn load_dir(path: PathBuf) -> Vec<Box<Stig>> {
    let mut dirs: Vec<Box<PathBuf>> = vec![Box::new(path)];
    let mut next_dirs: Vec<Box<PathBuf>> = vec![];

    let mut txts = Vec::new();
    let mut stigs = Vec::new();

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
            stigs.push(Box::new(stig));
        }
    }

    stigs
}

/// Parse the given str from the cmd prompt into a command.
fn parse_command(input: &str) -> Option<UserCommand> {
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
async fn run_search_cmd(cmd: UserCommand, sgroup_lock: Arc<RwLock<SGroup>>) -> Option<SGroup> {
    match cmd {
        UserCommand::SearchForKeyword(keyword) => {
            let mut sgroup = sgroup_lock.read().unwrap().clone();
            let re = Regex::new(&keyword).ok()?;

            sgroup.unpin_all_from_cmd();

            for stig_wrapper in sgroup.get_all().iter() {
                if let Pinned::ByUser = stig_wrapper.pinned {
                    continue;
                }

                let mut is_match = false;

                is_match |= re.is_match(&stig_wrapper.stig.version);
                is_match |= re.is_match(&stig_wrapper.stig.intro);
                is_match |= re.is_match(&stig_wrapper.stig.desc);
                is_match |= re.is_match(&stig_wrapper.stig.check_text);
                is_match |= re.is_match(&stig_wrapper.stig.fix_text);
                is_match |= re.is_match(&stig_wrapper.stig.similar_checks);

                if is_match {
                    sgroup.pin(stig_wrapper.uuid, Pinned::ByCmd);
                }
            }

            sgroup.sort_by_version();

            Some(sgroup)
        }
        UserCommand::SearchForName(name) => {
            let mut sgroup = sgroup_lock.read().unwrap().clone();
            let re = Regex::new(&name).ok()?;

            sgroup.unpin_all_from_cmd();

            for stig_wrapper in sgroup.get_all().iter() {
                if let Pinned::ByUser = stig_wrapper.pinned {
                    continue;
                }

                if re.is_match(&stig_wrapper.stig.version) {
                    sgroup.pin(stig_wrapper.uuid, Pinned::ByCmd);
                }
            }

            sgroup.sort_by_version();

            Some(sgroup)
        }
        UserCommand::Reset => {
            let mut sgroup = sgroup_lock.read().unwrap().clone();

            sgroup.unpin_all_from_cmd();
            sgroup.sort_by_version();

            Some(sgroup)
        }
    }
}
