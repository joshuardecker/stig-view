use iced::Element;
use iced::Subscription;
use iced::Task;
use iced::color;
use iced::theme::{Custom, Palette, Theme};
use iced::window::icon::from_file_data;
use iced::{keyboard, keyboard::key};
use image::ImageFormat;
use stig_view_core::db::{Data, Pinned};

use crate::app::async_fns::{FileError, open_file, open_folder};
use crate::app::*;

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                db: DB::new(),
                displayed: None,
                contents: [
                    Content::new(),
                    Content::new(),
                    Content::new(),
                    Content::new(),
                    Content::new(),
                    Content::new(),
                ],
                popup: Popup::None,
                assets: Assets::new(),
                window_id: None,
                current_theme: AppTheme::Dark,
            },
            window::oldest().map(Message::InitWindow),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(|event| Some(Message::KeyPressed(event)))
    }

    pub fn theme(&self) -> Theme {
        let (palette, name) = match self.current_theme {
            AppTheme::Dark => (
                Palette {
                    background: color!(0x1B1C1C),
                    text: color!(0xE6E6E6),
                    primary: color!(0xA2A2D0),
                    success: color!(0x188B6C),
                    warning: color!(0xffc14e),
                    danger: color!(0xc3423f),
                },
                String::from("Custom Dark"),
            ),
            AppTheme::Light => (
                Palette {
                    background: color!(0xDFD7D5),
                    text: color!(0x1B1C1C),
                    primary: color!(0x444488),
                    success: color!(0x188B6C),
                    warning: color!(0xffc14e),
                    danger: color!(0xc3423f),
                },
                String::from("Custom Light"),
            ),
        };

        Theme::Custom(Arc::new(Custom::new(name, palette)))
    }

    pub fn get_view(&self) -> Element<'_, Message> {
        iced::widget::container(iced::widget::text("hello"))
            .style(iced::widget::container::primary)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InitWindow(id) => {
                let id = id.expect("Not able to retrieve window id.");
                self.window_id = Some(id);

                // Toggle window decorations and set the app icon.
                Task::batch(vec![
                    window::toggle_decorations(id),
                    window::set_resizable(id, true),
                    window::set_icon(
                        id,
                        from_file_data(&self.assets.app_icon, Some(ImageFormat::Png))
                            .expect("Could not load app icon!"),
                    ),
                ])
            }
            Message::WindowClose => iced::exit(),
            Message::WindowMin => window::minimize(self.window_id.unwrap(), true),
            Message::WindowFullscreenToggle => window::toggle_maximize(self.window_id.unwrap()),
            Message::WindowMove => window::drag(self.window_id.unwrap()),
            Message::WindowDragResize(dir) => window::drag_resize(self.window_id.unwrap(), dir),

            Message::OpenFile => {
                let db = self.db.clone();

                Task::future(async move {
                    let id = open_file(db).await;

                    match id {
                        Ok(id) => Message::Switch(id),
                        Err(e) => match e {
                            FileError::HomeDir(err_msg) => Message::SendErrNotif(err_msg),
                            FileError::FilePick(err_msg) => Message::SendErrNotif(err_msg),
                            FileError::NotAStig(err_msg) => Message::SendErrNotif(err_msg),
                            _ => unreachable!(),
                        },
                    }
                })
            }
            Message::OpenFolder => {
                let db = self.db.clone();

                Task::future(async move {
                    let (id, error) = open_folder(db).await;

                    match (id, error) {
                        (Some(id), None) => Message::Switch(id),
                        (Some(id), Some(err)) => match err {
                            FileError::HomeDir(err_msg) => Message::SwitchWithError(id, err_msg),
                            FileError::FilePick(err_msg) => Message::SwitchWithError(id, err_msg),
                            FileError::ReadDir(err_msg) => Message::SwitchWithError(id, err_msg),
                            _ => unreachable!(),
                        },
                        (None, Some(err)) => match err {
                            FileError::HomeDir(err_msg) => Message::SendErrNotif(err_msg),
                            FileError::FilePick(err_msg) => Message::SendErrNotif(err_msg),
                            FileError::ReadDir(err_msg) => Message::SendErrNotif(err_msg),
                            _ => unreachable!(),
                        },
                        (None, None) => Message::DoNothing,
                    }
                })
            }

            Message::SelectContent(action, slot) => {
                // Dont let the user delete or add letters to the displayed text.
                if let Action::Edit(_) = action {
                    return Task::none();
                }

                self.contents[slot as usize].perform(action);

                Task::none()
            }

            Message::Switch(id) => {
                let db = self.db.clone();

                Task::future(async move {
                    let stig = db.get(&id).await;

                    if let Some(stig) = stig {
                        Message::Display(stig.get_stig().clone())
                    } else {
                        Message::DoNothing
                    }
                })
            }
            Message::SwitchWithError(id, err_str) => {
                let db = self.db.clone();

                Task::batch(vec![
                    Task::future(async move {
                        let stig = db.get(&id).await;

                        if let Some(stig) = stig {
                            Message::Display(stig.get_stig().clone())
                        } else {
                            Message::DoNothing
                        }
                    }),
                    Task::done(Message::SendErrNotif(err_str)),
                ])
            }
            Message::SwitchNext => {
                let db = self.db.clone();
                let displayed_name = self.displayed.clone();

                if let Some(displayed_name) = displayed_name {
                    Task::future(async move {
                        let snapshot = db.snapshot().await;

                        let mut iter = snapshot.iter();

                        let _ = iter.find(|entry| *entry.0 == displayed_name.version);

                        let entry: Option<(&String, &Data)> = iter.next();

                        if let Some(entry) = entry {
                            return Message::Switch(entry.0.to_owned());
                        }

                        let first = snapshot.first_key_value();

                        if let Some(first) = first {
                            return Message::Switch(first.0.clone());
                        }

                        Message::DoNothing
                    })
                } else {
                    Task::none()
                }
            }
            Message::Display(stig) => {
                self.contents[ContentSlot::Version as usize] = Content::with_text(&stig.version);
                self.contents[ContentSlot::Intro as usize] = Content::with_text(&stig.intro);
                self.contents[ContentSlot::Desc as usize] = Content::with_text(&stig.desc);
                self.contents[ContentSlot::CheckText as usize] = Content::with_text(&stig.check_text);
                self.contents[ContentSlot::FixText as usize] = Content::with_text(&stig.fix_text);
                self.contents[ContentSlot::SimilarChecks as usize] = Content::with_text(&stig.similar_checks);

                Task::none()
            }

            Message::SwitchPopup(popup) => {
                self.popup = popup;

                Task::none()
            }

            Message::SendErrNotif(err_str) => todo!(),

            Message::Pin(id) => {
                let db = self.db.clone();

                Task::future(async move {
                    let stig = db.get(&id).await;

                    if let Some(mut stig) = stig {
                        match stig.get_pin() {
                            Pinned::Not => stig.set_pin(Pinned::ByUser),
                            Pinned::ByUser => stig.set_pin(Pinned::Not),
                            Pinned::ByFilter => return Message::DoNothing,
                        }

                        db.insert(id, stig).await;
                    }

                    Message::DoNothing
                })
            }

            Message::FocusWidget(widget_id) => iced::widget::operation::focus(widget_id),

            Message::ProcessCmd => todo!(),

            Message::KeyPressed(event) => match &event {
                keyboard::Event::KeyPressed {
                    key: key::Key::Character(key_smolstr),
                    modifiers,
                    ..
                } => match key_smolstr.as_str() {
                    "q" if modifiers.control() => return iced::exit(),
                    "i" if modifiers.control() => return Task::done(Message::OpenFile),
                    "o" if modifiers.control() => return Task::done(Message::OpenFolder),
                    "p" if modifiers.control() => {
                        return Task::done(Message::SwitchPopup(Popup::Filter));
                    }
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

            Message::DoNothing => Task::none(),
        }
    }
}
