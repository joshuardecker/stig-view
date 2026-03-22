use iced::Subscription;
use iced::color;
use iced::theme::{Custom, Palette, Theme};
use iced::window::icon::from_file_data;
use iced::{keyboard, keyboard::key};
use image::ImageFormat;
use rfd::AsyncFileDialog;
use std::sync::Arc;
use stig_view_core::{DetectErr, Format, detect_stig_format};

use crate::app::command::*;
use crate::app::*;

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let settings = AppSettings::load().unwrap_or(AppSettings::default());

        (
            Self {
                benchmark: Benchmark::empty(),
                pins: BTreeMap::new(),
                displayed: None,
                contents: [
                    Content::new(),
                    Content::new(),
                    Content::new(),
                    Content::new(),
                    Content::new(),
                    Content::new(),
                ],
                filter_input: String::new(),
                popup: Popup::None,
                err_notif: ErrNotif::None,
                assets: Assets::new(),
                window_id: None,
                settings: settings,
                load_handle: None,
            },
            window::oldest().map(Message::InitWindow),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(|event| Some(Message::KeyPressed(event)))
    }

    pub fn theme(&self) -> Theme {
        let (palette, name) = match self.settings.theme {
            AppTheme::Dark => (
                Palette {
                    background: color!(0x1B1C1C),
                    text: color!(0xE6E6E6),
                    primary: color!(0xA2A2D0),
                    success: color!(0x22A67A),
                    warning: color!(0xffc14e),
                    danger: color!(0xc3423f),
                },
                String::from("Custom Dark"),
            ),
            AppTheme::Light => (
                Palette {
                    background: color!(0xF5F2F7),
                    text: color!(0x1E1A2E),
                    primary: color!(0x6B4FA0),
                    success: color!(0x22A67A),
                    warning: color!(0xD98C2A),
                    danger: color!(0xC0393A),
                },
                String::from("Custom Light"),
            ),
        };

        Theme::Custom(Arc::new(Custom::new(name, palette)))
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
            Message::WindowClose => {
                if let Some(handle) = self.load_handle.take() {
                    handle.abort();
                }

                iced::exit()
            }
            Message::WindowMin => {
                if let Some(id) = self.window_id {
                    window::minimize(id, true)
                } else {
                    Task::done(Message::SendErrNotif("Cant get window id to minimize."))
                }
            }
            Message::WindowFullscreenToggle => {
                if let Some(id) = self.window_id {
                    window::toggle_maximize(id)
                } else {
                    Task::done(Message::SendErrNotif(
                        "Cant get window id to toggle fullscreen.",
                    ))
                }
            }
            Message::WindowMove => {
                if let Some(id) = self.window_id {
                    window::drag(id)
                } else {
                    Task::done(Message::SendErrNotif("Cant get window id to move."))
                }
            }
            Message::WindowDragResize(dir) => {
                if let Some(id) = self.window_id {
                    window::drag_resize(id, dir)
                } else {
                    Task::done(Message::SendErrNotif("Cant get window id to resize."))
                }
            }

            Message::SwitchTheme(theme) => {
                self.settings.theme = theme;

                Task::done(Message::SaveSettings)
            }

            Message::OpenFile => Task::future(async move {
                let home_dir = dirs::home_dir();

                if let None = home_dir {
                    return Message::SendErrNotif("Home directory could not be found.");
                }

                let home_dir = home_dir.expect("Home dir is safe to unwrap here.");

                let file_handle = AsyncFileDialog::new()
                    .add_filter("STIG", &["toml", "xml", "zip"])
                    .set_directory(home_dir)
                    .set_title("Stig View - Select File")
                    .pick_file()
                    .await;

                // Do nothing if the user closed their file explorer before selecting a file.
                if let None = file_handle {
                    return Message::DoNothing;
                }

                let file_handle = file_handle.expect("File handle is safe to unwrap here.");

                let format = detect_stig_format(file_handle.path());

                match format {
                    Ok(Format::Xylok(xylok_toml)) => {
                        let benchmark = xylok_toml.convert();

                        if let Some(benchmark) = benchmark {
                            Message::SwitchBenchmark(benchmark)
                        } else {
                            Message::SendErrNotif("Could not parse selected toml.")
                        }
                    }
                    Err(err) => match err {
                        DetectErr::CantOpenFile(err_str) => Message::SendErrNotif(err_str),
                        DetectErr::InvalidFileFormat(err_str) => Message::SendErrNotif(err_str),
                        DetectErr::NotStig(err_str) => Message::SendErrNotif(err_str),
                    },
                    _ => unimplemented!(),
                }
            }),

            Message::SelectContent(action, slot) => {
                // Dont let the user delete or add letters to the displayed text.
                if let Action::Edit(_) = action {
                    return Task::none();
                }

                self.contents[slot as usize].perform(action);

                Task::none()
            }

            Message::Switch(id) => {
                let benchmark = self.benchmark.clone();

                Task::future(async move {
                    let rule = benchmark.rules.get(&id);

                    if let Some(rule) = rule {
                        Message::Display(rule.to_owned())
                    } else {
                        Message::DoNothing
                    }
                })
            }
            Message::SwitchBenchmark(benchmark) => {
                if let Some((name, _rule)) = benchmark.rules.first_key_value() {
                    let name = name.to_owned();

                    self.benchmark = benchmark;
                    // Reset pin values.
                    self.pins = BTreeMap::new();

                    Task::done(Message::Switch(name))
                } else {
                    self.benchmark = benchmark;
                    // Reset pin values.
                    self.pins = BTreeMap::new();

                    Task::none()
                }
            }
            Message::SetPins(pins) => {
                self.pins = pins;

                // Get the displayed STIG, if its already pinned, dont switch which STIG is viewed.
                if let Some(rule) = &self.displayed {
                    let pin_status = self.pins.get(&rule.group_id);

                    match pin_status.unwrap_or(&Pinned::Not) {
                        Pinned::ByFilter => return Task::done(Message::DoNothing),
                        Pinned::ByFilterAndUser => return Task::done(Message::DoNothing),
                        _ => (), // continue if not above options.
                    }
                }

                for (name, _rule) in self.benchmark.rules.iter() {
                    match self.pins.get(name).unwrap_or(&Pinned::Not) {
                        Pinned::ByFilter => return Task::done(Message::Switch(name.to_owned())),
                        Pinned::ByFilterAndUser => {
                            return Task::done(Message::Switch(name.to_owned()));
                        }
                        _ => (),
                    }
                }

                Task::none()
            }
            Message::SwitchWithError(id, err_str) => {
                let benchmark = self.benchmark.clone();

                Task::batch(vec![
                    Task::future(async move {
                        let rule = benchmark.rules.get(&id);

                        if let Some(rule) = rule {
                            Message::Display(rule.to_owned())
                        } else {
                            Message::DoNothing
                        }
                    }),
                    Task::done(Message::SendErrNotif(err_str)),
                ])
            }
            Message::SwitchNext => {
                let benchmark = self.benchmark.clone();
                let displayed_name = self.displayed.clone();

                if let Some(displayed_name) = displayed_name {
                    Task::future(async move {
                        let mut iter = benchmark.rules.iter();

                        let _ = iter.find(|entry| *entry.0 == displayed_name.group_id);

                        let entry: Option<(&String, &Rule)> = iter.next();

                        if let Some(entry) = entry {
                            return Message::Switch(entry.0.to_owned());
                        }

                        let first = benchmark.rules.first_key_value();

                        if let Some(first) = first {
                            return Message::Switch(first.0.clone());
                        }

                        Message::DoNothing
                    })
                } else {
                    Task::none()
                }
            }
            Message::Display(rule) => {
                // TODO: make display all info
                self.contents[ContentSlot::Version as usize] = Content::with_text(&rule.group_id);
                self.contents[ContentSlot::Intro as usize] = Content::with_text(&rule.title);
                self.contents[ContentSlot::Desc as usize] =
                    Content::with_text(&rule.vuln_discussion);
                self.contents[ContentSlot::CheckText as usize] =
                    Content::with_text(&rule.check_text);
                self.contents[ContentSlot::FixText as usize] = Content::with_text(&rule.fix_text);
                // TODO: fix
                self.contents[ContentSlot::SimilarChecks as usize] =
                    Content::with_text(&rule.group_id);

                self.displayed = Some(rule);

                Task::none()
            }

            Message::SwitchPopup(popup) => {
                match (&self.popup, &popup) {
                    (Popup::Filter, Popup::Filter) => self.popup = Popup::None,
                    (Popup::Settings, Popup::Settings) => self.popup = Popup::None,
                    _ => self.popup = popup,
                }

                Task::none()
            }

            Message::SendErrNotif(err_str) => {
                if let ErrNotif::None = self.err_notif {
                    self.err_notif = ErrNotif::Err(err_str);
                }

                Task::none()
            }
            Message::ClearErrNotif => {
                self.err_notif = ErrNotif::None;

                Task::none()
            }

            Message::Pin(id) => {
                let pin_status = self.pins.get(&id);

                match pin_status.unwrap_or(&Pinned::Not) {
                    Pinned::Not => {
                        let _ = self.pins.insert(id, Pinned::ByUser);
                    }
                    Pinned::ByUser => {
                        let _ = self.pins.insert(id, Pinned::Not);
                    }

                    Pinned::ByFilter => {
                        let _ = self.pins.insert(id, Pinned::ByFilterAndUser);
                    }
                    Pinned::ByFilterAndUser => {
                        let _ = self.pins.insert(id, Pinned::ByFilter);
                    }
                }

                Task::none()
            }

            Message::FocusWidget(widget_id) => iced::widget::operation::focus(widget_id),

            Message::TypeCmd(filter_input) => {
                self.filter_input = filter_input;

                Task::none()
            }
            Message::ProcessCmd(command_str) => {
                let benchmark = self.benchmark.clone();
                let pins = self.pins.clone();

                Task::future(async move {
                    let command = parse_command(&command_str);

                    match command {
                        Ok(command) => {
                            let new_pins = run_search_cmd(command.clone(), benchmark, pins);

                            match new_pins {
                                Ok(new_pins) => Message::SetPins(new_pins),
                                Err(_) => Message::SendErrNotif("Error when running the command."),
                            }
                        }
                        Err(e) => match e {
                            CommandErr::RegexErr => {
                                Message::SendErrNotif("Error when parsing the command.")
                            }
                            _ => Message::DoNothing,
                        },
                    }
                })
            }

            Message::KeyPressed(event) => match &event {
                keyboard::Event::KeyPressed {
                    key: key::Key::Character(key_smolstr),
                    modifiers,
                    ..
                } => match key_smolstr.as_str() {
                    "q" if modifiers.control() => return Task::done(Message::WindowClose),
                    "i" if modifiers.control() => return Task::done(Message::OpenFile),
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

            Message::SaveSettings => {
                let err = AppSettings::save(self.settings.clone());

                match err {
                    Ok(_) => Task::none(),
                    Err(AppSettingsErr::CantSave(err_str)) => {
                        Task::done(Message::SendErrNotif(err_str))
                    }
                }
            }

            Message::DoNothing => Task::none(),
        }
    }
}
