use iced::keyboard;
use iced::keyboard::key;
use iced::theme;
use iced::theme::Custom;
use iced::widget::Id;
use iced::widget::text_editor;
use iced::{Color, color};
use iced::{Element, Subscription, Task, Theme};
use regex::Regex;
use rfd::AsyncFileDialog;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::sgroup::SGroup;
use crate::stig::Stig;

/// This applications state.
#[derive(Debug, Clone)]
pub struct App {
    pub list: Arc<RwLock<SGroup>>,
    pub displayed: Arc<RwLock<Option<Box<(Uuid, Stig)>>>>,
    pub content: [text_editor::Content; 6],
    pub popup: Option<Popup>,
    pub cmd_input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    KeyPressed(keyboard::Event),

    OpenFileSelect,
    OpenFolderSelect,
    OpenFile(Option<PathBuf>),
    OpenFolder(Option<PathBuf>),
    SetStigVec(Vec<Box<Stig>>),
    PushStigToContent,

    SelectContent(text_editor::Action, usize),

    SwitchDisplayed(Uuid),
    SwitchNext,

    FocusCmdInput(Id),
    ChangeCmdInput(String),
    SubmitCmdInput,

    ChangePopup(Option<Popup>),

    None,
}

#[derive(Debug, Clone)]
pub enum Popup {
    CommandPrompt,
    Error,
}

#[derive(Debug, Clone)]
pub enum UserCommand {
    SearchForKeyword(String),
}

impl App {
    pub fn new() -> Self {
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
        }
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::KeyPressed(event) => match &event {
                keyboard::Event::KeyPressed {
                    key: key::Key::Character(key_smolstr),
                    modifiers,
                    ..
                } => match key_smolstr.as_str() {
                    "q" if modifiers.control() => return iced::exit(),
                    "i" if modifiers.control() => return Task::done(Message::OpenFileSelect),
                    "o" if modifiers.control() => return Task::done(Message::OpenFolderSelect),
                    "p" if modifiers.control() => match &self.popup {
                        Some(Popup::CommandPrompt) => {
                            return Task::done(Message::ChangePopup(None));
                        }
                        None => {
                            return Task::done(Message::ChangePopup(Some(Popup::CommandPrompt)));
                        }
                        _ => return Task::none(),
                    },
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
            Message::OpenFile(path) => {
                if let Some(path) = path {
                    let stig = Stig::from_xylok_txt(path.clone());

                    if let Some(stig) = stig {
                        let stig = Box::new(stig);
                        self.list.write().unwrap().add(stig.clone());
                        *self.displayed.write().unwrap() = Some(self.list.read().unwrap().first());
                    }

                    Task::done(Message::PushStigToContent)
                } else {
                    Task::none()
                }
            }
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
            Message::SetStigVec(stigs) => {
                if stigs.len() == 0 {
                    return Task::none();
                }

                self.list.write().unwrap().set_group(stigs);

                *self.displayed.write().unwrap() = Some(self.list.read().unwrap().first());

                Task::done(Message::PushStigToContent)
            }
            Message::PushStigToContent => {
                if let Some(stig) = &*self.displayed.read().unwrap() {
                    self.content[0] = text_editor::Content::with_text(&stig.1.version);
                    self.content[1] = text_editor::Content::with_text(&stig.1.intro);
                    self.content[2] = text_editor::Content::with_text(&stig.1.desc);
                    self.content[3] = text_editor::Content::with_text(&stig.1.check_text);
                    self.content[4] = text_editor::Content::with_text(&stig.1.fix_text);
                    self.content[5] = text_editor::Content::with_text(&stig.1.similar_checks);
                }

                Task::none()
            }
            Message::SelectContent(action, index) => {
                if let text_editor::Action::Edit(_) = action {
                    return Task::none();
                }

                self.content[index.to_owned()].perform(action.clone());

                Task::none()
            }
            Message::SwitchDisplayed(uuid) => {
                let new_displayed_stig = self.list.read().unwrap().get_by_uuid(uuid);

                if let Some(new_displayed_stig) = new_displayed_stig {
                    *self.displayed.write().unwrap() = Some(new_displayed_stig);
                }

                Task::done(Message::PushStigToContent)
            }
            Message::SwitchNext => {
                if let Some(displayed_stig) = self.displayed.read().unwrap().clone() {
                    for (mut index, stig) in self.list.read().unwrap().iter().enumerate() {
                        if stig.uuid != displayed_stig.uuid {
                            continue;
                        }

                        if index == (self.list.read().unwrap().len() - 1) {
                            index = 0;
                        } else {
                            index += 1;
                        }

                        return Task::done(Message::SwitchDisplayed(
                            self.list.read().unwrap()[index].uuid,
                        ));
                    }
                }

                Task::none()
            }
            Message::FocusCmdInput(id) => iced::widget::operation::focus(id),
            Message::ChangeCmdInput(input) => {
                self.cmd_input = input;
                Task::none()
            }
            Message::SubmitCmdInput => {
                let user_command = parse_command(&self.cmd_input);

                if let Some(user_command) = user_command {
                    let list_clone = self.list.clone();
                    Task::perform(
                        async move { run_search_cmd(user_command, list_clone) },
                        |_| Message::None,
                    )
                } else {
                    Task::none()
                }
            }
            Message::ChangePopup(new_popup) => {
                self.popup = new_popup;
                Task::none()
            }
            Message::None => {
                println!("Gamer nation!");
                Task::none()
            }
        }
    }

    pub fn get_view(&self) -> Element<'_, Message> {
        if let Some(_) = *self.displayed.read().unwrap() {
            self.get_view_displayed()
        } else {
            self.get_view_none_displayed()
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(|event| Some(Message::KeyPressed(event)))
    }

    pub fn theme(&self) -> Theme {
        let palette = theme::Palette {
            background: color!(0x2B2D31),
            text: Color::from_rgb(0.90, 0.90, 0.90),
            //primary: color!(0x6CA0DC),
            //primary: color!(0xC5D8E5),
            primary: color!(0xA2A2D0),
            success: color!(0x12664f),
            warning: color!(0xffc14e),
            danger: color!(0xc3423f),
        };

        Theme::Custom(Arc::new(Custom::new(String::from("Custom Dark"), palette)))
    }
}

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

        dirs = next_dirs;
        next_dirs = Vec::new();

        if dirs.len() == 0 {
            break;
        }
    }

    for txt in txts {
        if let Some(stig) = Stig::from_xylok_txt(&*txt) {
            stigs.push(Box::new(stig));
        }
    }

    stigs
}

fn parse_command(input: &str) -> Option<UserCommand> {
    let keyword_search_regex = Regex::new(r"search|find (.*)").unwrap();

    let captures = keyword_search_regex.captures(input)?;

    Some(UserCommand::SearchForKeyword(captures[1].to_string()))
}

async fn run_search_cmd(cmd: UserCommand, stigs: Arc<RwLock<SGroup>>) {
    if let UserCommand::SearchForKeyword(keyword) = &cmd {
        let mut stigs = stigs.write().unwrap().get_all();
        let re = Regex::new(keyword).unwrap();

        for stig in stigs.iter_mut() {
            let mut is_match = false;

            is_match |= re.is_match(&stig.1.version);
            is_match |= re.is_match(&stig.1.intro);
            is_match |= re.is_match(&stig.1.desc);
            is_match |= re.is_match(&stig.1.check_text);
            is_match |= re.is_match(&stig.1.fix_text);
            is_match |= re.is_match(&stig.1.similar_checks);

            /*if is_match {
                stig.pinned = true;
            }*/
        }
    }
}
