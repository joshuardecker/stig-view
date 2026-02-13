use iced::{Element, Subscription, Task, keyboard, widget::text_editor};
use rfd::AsyncFileDialog;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::stig::Stig;

/// This applications state.
#[derive(Debug, Clone)]
pub struct App {
    pub list: Vec<Box<Stig>>,
    pub displayed: Option<Box<Stig>>,
    pub content: [text_editor::Content; 6],
}

#[derive(Debug, Clone)]
pub enum Message {
    KeyPressed(keyboard::Key),
    OpenFileSelect,
    OpenFolderSelect,
    OpenFile(Option<PathBuf>),
    OpenFolder(Option<PathBuf>),
    LoadStigVec(Vec<Box<Stig>>),
    PushStigToContent,
    SelectContent(text_editor::Action, usize),
    SwitchDisplayed(usize),
}

impl App {
    pub fn new() -> Self {
        App {
            list: Vec::new(),
            displayed: None,
            content: [
                text_editor::Content::new(),
                text_editor::Content::new(),
                text_editor::Content::new(),
                text_editor::Content::new(),
                text_editor::Content::new(),
                text_editor::Content::new(),
            ],
        }
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::OpenFileSelect => Task::perform(
                async {
                    let home_dir = std::env::home_dir().unwrap_or(PathBuf::from_str("/").unwrap());

                    let file_handle = AsyncFileDialog::new()
                        .add_filter("text", &["txt"])
                        .set_directory(home_dir)
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
                    let home_dir = std::env::home_dir().unwrap_or(PathBuf::from_str("/").unwrap());

                    let folder_handle = AsyncFileDialog::new()
                        .add_filter("text", &["txt"])
                        .set_directory(home_dir)
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
                    if !Stig::check_if_xylok_txt(&path) {
                        // todo: tell user couldnt load stig.
                        return Task::none();
                    }

                    let stig = Stig::from_xylok_txt(&path);

                    if let Some(stig) = stig {
                        let stig = Box::new(stig);

                        self.list = vec![stig.clone()];
                        self.displayed = Some(stig);

                        return Task::done(Message::PushStigToContent);
                    } else {
                        // todo: tell user couldnt load stig.
                        return Task::none();
                    }
                }

                Task::none()
            }
            Message::OpenFolder(path) => {
                if let Some(path) = path {
                    Task::perform(
                        async move { load_dir(path.clone()).await },
                        Message::LoadStigVec,
                    )
                } else {
                    Task::none()
                }
            }
            Message::LoadStigVec(stigs) => {
                self.list = stigs.clone();

                if self.list.len() != 0 {
                    self.displayed = Some(self.list[0].clone());
                }

                Task::done(Message::PushStigToContent)
            }
            Message::PushStigToContent => {
                if let Some(stig) = &self.displayed {
                    self.content[0] = text_editor::Content::with_text(&stig.version);
                    self.content[1] = text_editor::Content::with_text(&stig.intro);
                    self.content[2] = text_editor::Content::with_text(&stig.desc);
                    self.content[3] = text_editor::Content::with_text(&stig.check_text);
                    self.content[4] = text_editor::Content::with_text(&stig.fix_text);
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
            Message::SwitchDisplayed(index) => {
                self.displayed = Some(self.list[index].clone());

                Task::done(Message::PushStigToContent)
            }
            // todo: remove by production
            _ => Task::none(),
        }
    }

    pub fn get_view(&self) -> Element<'_, Message> {
        if let Some(_) = self.displayed {
            self.get_view_displayed()
        } else {
            self.get_view_none_displayed()
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(|event| {
            if let keyboard::Event::KeyPressed { key, .. } = event {
                Some(Message::KeyPressed(key))
            } else {
                None
            }
        })
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
        if Stig::check_if_xylok_txt(&*txt) {
            if let Some(stig) = Stig::from_xylok_txt(&*txt) {
                stigs.push(Box::new(stig));
            }
        }
    }

    stigs
}
