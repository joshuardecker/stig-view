use iced::Subscription;
use iced::color;
use iced::theme::{Custom, Palette, Theme};
use iced::window::icon::from_file_data;
use iced::{keyboard, keyboard::key};
use image::ImageFormat;
use rfd::AsyncFileDialog;
use std::sync::Arc;
use std::time::Instant;
use stig_view_core::{Benchmark, Format, detect_stig_format, load_ckl, load_v1_1};

use crate::app::command::*;
use crate::app::*;

const MAIN_FADE_START: f32 = 0.20;
const MAIN_FADE_DURATION_SECS: f32 = 0.2;

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let settings = AppSettings::load().unwrap_or(AppSettings::default());

        (
            Self {
                benchmark: Benchmark::empty(),
                benchmarks: Vec::new(),
                pins: HashMap::new(),
                displayed: None,
                contents: [
                    Content::new(),
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
                display_type: settings.default_display_type,
                main_col_opacity: 1.0,
                main_col_last_tick: None,
            },
            window::oldest().map(Message::InitWindow),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard = keyboard::listen().filter_map(|event| Some(Message::KeyPressed(event)));

        if self.main_col_last_tick.is_some() {
            let tick = window::frames().map(Message::Tick);
            Subscription::batch([keyboard, tick])
        } else {
            keyboard
        }
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
                    success: color!(0x1A8A63),
                    warning: color!(0xB45309),
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

                let home_dir = match home_dir {
                    Some(dir) => dir,
                    None => return Message::SendErrNotif("Home directory could not be found."),
                };

                let file_handle = AsyncFileDialog::new()
                    .add_filter("STIG", &["toml", "xml", "zip", "ckl", "cklb"])
                    .set_directory(home_dir)
                    .set_title("Stig View - Select File")
                    .pick_file()
                    .await;

                // Do nothing if the user closed their file explorer before selecting a file.
                let file_handle = match file_handle {
                    Some(handle) => handle,
                    None => return Message::DoNothing,
                };

                let format = detect_stig_format(file_handle.path());

                match format {
                    Some(Format::Xylok(xylok_toml)) => {
                        let benchmark = xylok_toml.convert();

                        if let Some(benchmark) = benchmark {
                            Message::SwitchBenchmark(benchmark)
                        } else {
                            Message::SendErrNotif(
                                "Xylok toml could not be converted into a Benchmark.",
                            )
                        }
                    }

                    Some(Format::XccdfV1_1(file_str)) => {
                        let benchmark = load_v1_1(&file_str);

                        if let Some(benchmark) = benchmark {
                            Message::SwitchBenchmark(benchmark)
                        } else {
                            Message::SendErrNotif("Xml could not be converted into a Benchmark.")
                        }
                    }

                    Some(Format::XccdfV1_2) => {
                        Message::SendErrNotif("SCAP's are not a supported file type.")
                    }

                    Some(Format::CKL(file_str)) => {
                        let benchmarks = load_ckl(&file_str);

                        if benchmarks.is_empty() {
                            Message::SendErrNotif("CKL could not be converted into a Benchmark.")
                        } else {
                            Message::SwitchBenchmarks(benchmarks)
                        }
                    }

                    Some(Format::CKLB(cklb)) => {
                        let benchmarks = cklb.convert();

                        if benchmarks.is_empty() {
                            Message::SendErrNotif("CKLB could not be converted into a Benchmark.")
                        } else {
                            Message::SwitchBenchmarks(benchmarks)
                        }
                    }

                    None => Message::SendErrNotif("Selected file is an unsupported type."),
                }
            }),

            Message::SelectContent(action, index) => {
                // Dont let the user delete or add letters to the displayed text.
                if let Action::Edit(_) = action {
                    return Task::none();
                }

                self.contents[index as usize].perform(action);

                Task::none()
            }

            Message::Switch(id) => {
                // If the rule already displayed is being switched to, do nothing.
                if let Some(rule) = &self.displayed {
                    if rule.group_id == id {
                        return Task::none();
                    }
                }

                let benchmark = self.benchmark.clone();

                let rule = benchmark.rules.get(&id);

                if let Some(rule) = rule {
                    Task::done(Message::Display(rule.to_owned()))
                } else {
                    Task::done(Message::DoNothing)
                }
            }
            Message::SwitchBenchmark(benchmark) => {
                if let Some((name, _rule)) = benchmark.rules.first_key_value() {
                    let name = name.to_owned();

                    self.benchmark = benchmark;

                    // Reset pin values.
                    self.pins = HashMap::new();
                    // Reset background Benchmarks.
                    self.benchmarks = Vec::new();

                    let tasks = vec![
                        Task::done(Message::Switch(name)),
                        Task::done(Message::SwitchPopup(Popup::Save)),
                    ];

                    Task::batch(tasks)
                } else {
                    // Do nothing when an attempting to switch an empty benchmark.
                    Task::none()
                }
            }
            Message::SwitchBenchmarks(mut benchmarks) => {
                if benchmarks.len() == 0 {
                    return Task::none();
                }

                let first = benchmarks.remove(0);
                let mut tasks = vec![Task::done(Message::SwitchBenchmark(first))];

                for benchmark in benchmarks {
                    tasks.push(Task::done(Message::PushBackgroundBenchmark(benchmark)));
                }

                Task::batch(tasks)
            }
            Message::PushBackgroundBenchmark(benchmark) => {
                self.benchmarks.push(benchmark);

                Task::none()
            }
            Message::SwitchToBackground => {
                match (!self.benchmarks.is_empty()).then(|| self.benchmarks.remove(0)) {
                    Some(benchmark) => {
                        let old = std::mem::replace(&mut self.benchmark, benchmark);
                        self.benchmarks.push(old);

                        // Reset pin values when switching to this new benchmark.
                        self.pins = HashMap::new();

                        Task::done(Message::DoNothing)
                    }
                    None => Task::none(),
                }
            }

            Message::SetPins(pins) => {
                self.pins = pins;

                // When the pins are set, check if the displayed rule has a filter applied.
                // If not, switch to the first one that does.

                // Get the displayed STIG, if its already pinned, dont switch which STIG is viewed.
                if let Some(rule) = &self.displayed {
                    let pin_status = self.pins.get(&rule.group_id);

                    match pin_status.unwrap_or(&Pinned::Not) {
                        Pinned::ByFilter => return Task::done(Message::DoNothing),
                        Pinned::ByFilterAndUser => return Task::done(Message::DoNothing),
                        _ => (), // Continue if not above options.
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
                self.contents[ContentIndex::Title as usize] = Content::with_text(&rule.title);
                self.contents[ContentIndex::Discussion as usize] =
                    Content::with_text(&rule.vuln_discussion);
                self.contents[ContentIndex::Check as usize] = Content::with_text(&rule.check_text);
                self.contents[ContentIndex::Fix as usize] = Content::with_text(&rule.fix_text);
                self.contents[ContentIndex::CCIRefs as usize] = Content::with_text(
                    &rule
                        .cci_refs
                        .clone()
                        .unwrap_or(vec!["".to_string()])
                        .join("\n"),
                );
                self.contents[ContentIndex::FalsePositives as usize] =
                    Content::with_text(&rule.false_positives.clone().unwrap_or("".to_string()));
                self.contents[ContentIndex::FalseNegatives as usize] =
                    Content::with_text(&rule.false_negatives.clone().unwrap_or("".to_string()));

                self.displayed = Some(rule);

                // Only animate if configured to.
                if self.settings.animate {
                    self.main_col_opacity = MAIN_FADE_START;
                    self.main_col_last_tick = Some(Instant::now());
                }

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
                let err = AppSettings::save(self.settings);

                match err {
                    Ok(_) => Task::none(),
                    Err(AppSettingsErr::CantSave(err_str)) => {
                        Task::done(Message::SendErrNotif(err_str))
                    }
                }
            }
            Message::SaveBenchmark => {
                let all = std::iter::once(&self.benchmark).chain(self.benchmarks.iter());

                for benchmark in all {
                    if let Err(_) = benchmark.save() {
                        return Task::done(Message::SendErrNotif("Couldn't save benchmark."));
                    }
                }

                // After saving, turn off the save menu.
                Task::done(Message::SwitchPopup(Popup::None))
            }

            Message::LoadCachedBenchmark(path) => match Benchmark::load(&path) {
                Ok(benchmark) => {
                    if let Some((name, _rule)) = benchmark.rules.first_key_value() {
                        let name = name.to_owned();

                        self.benchmark = benchmark;

                        // Reset pin values.
                        self.pins = HashMap::new();
                        // Reset background Benchmarks.
                        self.benchmarks = Vec::new();

                        Task::done(Message::Switch(name))
                    } else {
                        // Do nothing when an attempting to switch an empty benchmark.
                        Task::none()
                    }
                }
                Err(_) => Task::done(Message::SendErrNotif("Couldn't load cached benchmark.")),
            },

            Message::SwitchDisplayType(display_type) => {
                self.display_type = display_type;

                Task::none()
            }
            // Instead of just switching display types, save it as the default for next time.
            Message::SaveDisplayType(display_type) => {
                self.display_type = display_type;
                self.settings.default_display_type = display_type;

                Task::done(Message::SaveSettings)
            }

            Message::SaveAnimate(animate) => {
                self.settings.animate = animate;

                Task::done(Message::SaveSettings)
            }

            Message::ReturnHome => {
                self.benchmark = Benchmark::empty();
                self.benchmarks = Vec::new();
                self.displayed = None;

                Task::none()
            }

            Message::Tick(now) => {
                if let Some(last) = self.main_col_last_tick {
                    let dt = now.duration_since(last).as_secs_f32();
                    self.main_col_opacity =
                        (self.main_col_opacity + dt / MAIN_FADE_DURATION_SECS).min(1.0);
                    if self.main_col_opacity >= 1.0 {
                        self.main_col_last_tick = None;
                    } else {
                        self.main_col_last_tick = Some(now);
                    }
                }

                Task::none()
            }

            Message::DoNothing => Task::none(),
        }
    }

    pub fn load_cache() -> Vec<std::path::PathBuf> {
        let Some(mut cache_dir) = dirs::cache_dir() else {
            return Vec::new();
        };

        cache_dir.push("stig-view/");

        let entries = match std::fs::read_dir(&cache_dir) {
            Ok(entries) => entries,
            Err(_) => return Vec::new(),
        };

        entries
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                let name = path.file_name()?.to_str()?;
                if name.ends_with(".msgpack.zstd") && name != ".msgpack.zstd" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect()
    }
}
