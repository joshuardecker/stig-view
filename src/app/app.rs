use std::{
    fs::{File, create_dir_all},
    io::Write,
    path::PathBuf,
};

use disa_stig::{Benchmark, Format, RuleID};
use iced::{
    Subscription, Theme,
    futures::channel::mpsc::Sender,
    keyboard::{self, key},
    stream,
    window::icon::from_file_data,
};
use image::ImageFormat;
use rfd::AsyncFileDialog;

use crate::app::search;
use crate::app::*;
use crate::ui::{APP_ICON, THEME_COFFEE, THEME_DARK, THEME_HIGH_CONTRAST, THEME_LIGHT};

/// The overarching state of the application.
#[derive(Debug, Clone)]
pub struct App {
    /// The internal id of the window.
    pub window_id: Option<window::Id>,

    /// Currently displayed benchmark.
    pub benchmark: Option<Benchmark>,
    /// Benchmarks that live in the background, but are not currently displayed.
    pub background_benchmarks: Vec<Benchmark>,
    /// What rules are pinned, and why the are pinned.
    pub pins: HashMap<RuleID, Pinned>,
    /// What data should be displayed in the rules list.
    pub display_type: DisplayType,
    /// The currently displayed rule.
    pub displayed: Option<Rule>,

    /// The text input for the user to type filters into.
    pub filter_text_field: String,

    /// The current popup being displayed.
    pub popup: Option<Popup>,

    /// Error notification text to be displayed.
    pub error_msgs: Vec<String>,
    /// Which of the error messages is the user looking at.
    pub error_index: usize,
    /// Should the error messages be displayed to the user.
    pub display_errors: bool,

    /// If true, display to the user there is an update available.
    pub update_available: bool,

    /// Settings applied to the app.
    pub settings: AppSettings,
    /// When benchmarks were last opened by the user.
    pub last_opened: LastOpened,
    /// An animation manager.
    pub animations: Animations,
}

/// Every way to change the state.
#[derive(Debug, Clone)]
pub enum Message {
    NewWindow(Option<window::Id>),
    FocusWidget(Id),
    KeyPressed(keyboard::Event),

    WindowClose,
    WindowMinimize,
    WindowFullscreenToggle,
    WindowMove,
    WindowDragResize(Direction),

    OpenFile,
    SwitchBenchmark(Benchmark),
    PushBenchmarkToBackground(Benchmark),
    SwitchToNextBackground,
    SaveAllBenchmarks,
    LoadCachedBenchmark(PathBuf),
    DeleteCachedBenchmark(PathBuf),
    SwitchDisplayType(DisplayType),

    SwitchRule(RuleID),
    SwitchNextRule,
    Display(Rule),

    TypeFilter(String),
    ResetFilter,
    Pin(String),

    ShowErrors(bool),
    SendErrorNotification(String),
    ClearErrorNotification,
    ShowPreviousError,
    ShowNextError,

    SwitchPopup(Option<Popup>),
    FetchLatestVersion,
    ShowUpdateAvailable,
    OpenURL(&'static str),

    SwitchTheme(AppTheme),
    SaveAnimate(bool),
    SaveUpdateNotify(bool),
    SaveSettings,
    Tick(Instant),

    ReturnHome,

    Log(String),
}

/// Popups that can appear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Popup {
    Filter,
    Settings,
    Save,
}

/// The color theme of the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    Dark,
    Light,
    HighContrast,
    Coffee,
}

/// Whether the stig has been pinned in the list for any reason.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pinned {
    Not,
    ByUser,
    ByFilter,
    ByFilterAndUser,
}

/// What name should be displayed on the buttons that switch the displayed STIG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisplayType {
    GroupId,
    RuleId,
    STIGId,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let mut tasks = vec![window::oldest().map(Message::NewWindow)];

        let settings = AppSettings::load().unwrap_or(AppSettings::default());

        let last_opened = match LastOpened::load() {
            Ok(time_opened) => time_opened,
            Err(error) => {
                tasks.push(Task::done(Message::Log(error.to_string())));

                LastOpened::new()
            }
        };

        if settings.notify_if_update {
            tasks.push(Task::done(Message::FetchLatestVersion));
        }

        (
            Self {
                window_id: None,
                benchmark: None,
                background_benchmarks: Vec::new(),
                pins: HashMap::new(),
                display_type: settings.default_display_type,
                displayed: None,
                filter_text_field: String::new(),
                popup: None,
                error_msgs: Vec::new(),
                error_index: 0,
                display_errors: false,
                update_available: false,
                settings,
                last_opened,
                animations: Animations::new(),
            },
            Task::batch(tasks),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard = keyboard::listen().filter_map(|event| Some(Message::KeyPressed(event)));
        let tick = window::frames().map(Message::Tick);

        Subscription::batch([keyboard, tick])
    }

    pub fn theme(&self) -> Theme {
        match self.settings.theme {
            AppTheme::Dark => THEME_DARK.clone(),
            AppTheme::Light => THEME_LIGHT.clone(),
            AppTheme::HighContrast => THEME_HIGH_CONTRAST.clone(),
            AppTheme::Coffee => THEME_COFFEE.clone(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NewWindow(id) => {
                let id = id.expect("Not able to retrieve window id.");
                self.window_id = Some(id);

                // Toggle window decorations and set the app icon.
                Task::batch(vec![
                    window::toggle_decorations(id),
                    window::set_resizable(id, true),
                    window::set_icon(
                        id,
                        from_file_data(APP_ICON, Some(ImageFormat::Png))
                            .expect("Could not load app icon!"),
                    ),
                ])
            }
            Message::KeyPressed(event) => {
                if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                    match key {
                        key::Key::Character(char) => match char.as_str() {
                            "q" if modifiers.control() => Task::done(Message::WindowClose),
                            "o" if modifiers.control() => Task::done(Message::OpenFile),
                            "f" if modifiers.control() => {
                                Task::done(Message::SwitchPopup(Some(Popup::Filter)))
                            }
                            _ => Task::none(),
                        },
                        key::Key::Named(name) => match name {
                            key::Named::Tab if modifiers.control() => {
                                Task::done(Message::SwitchNextRule)
                            }
                            _ => Task::none(),
                        },
                        _ => Task::none(),
                    }
                } else {
                    Task::none()
                }
            }
            Message::FocusWidget(widget_id) => iced::widget::operation::focus(widget_id),

            Message::WindowClose => iced::exit(),
            Message::WindowMinimize => {
                if let Some(id) = self.window_id {
                    window::minimize(id, true)
                } else {
                    Task::none()
                }
            }
            Message::WindowFullscreenToggle => {
                if let Some(id) = self.window_id {
                    window::toggle_maximize(id)
                } else {
                    Task::none()
                }
            }
            Message::WindowMove => {
                if let Some(id) = self.window_id {
                    window::drag(id)
                } else {
                    Task::none()
                }
            }
            Message::WindowDragResize(dir) => {
                if let Some(id) = self.window_id {
                    window::drag_resize(id, dir)
                } else {
                    Task::none()
                }
            }

            Message::OpenFile => {
                Task::stream(stream::channel(
                    100,
                    |mut output: Sender<Message>| async move {
                        let home_dir = dirs::home_dir();

                        let home_dir = match home_dir {
                            Some(dir) => dir,
                            None => {
                                let _ = output.try_send(Message::SendErrorNotification(
                                    "Error requesting the home file directory.".to_string(),
                                ));
                                return;
                            }
                        };

                        let file_handle = AsyncFileDialog::new()
                            .add_filter("STIG", &["toml", "xml", "zip", "ckl", "cklb"])
                            .set_directory(home_dir)
                            .set_title("Xylok View - Select File")
                            .pick_file()
                            .await;

                        // Do nothing if the user closed their file explorer before selecting a file.
                        let file_handle = match file_handle {
                            Some(handle) => handle,
                            None => return,
                        };

                        let Ok(mut benchmarks) = Benchmark::load_from_file(file_handle.path())
                        else {
                            let _ = output.try_send(Message::SendErrorNotification(
                                "Could not load requested file.".to_string(),
                            ));
                            return;
                        };

                        if let Some(benchmark) = benchmarks.pop() {
                            let _ = output.try_send(Message::SwitchBenchmark(benchmark));
                        }

                        for benchmark in benchmarks {
                            let _ = output.try_send(Message::PushBenchmarkToBackground(benchmark));
                        }
                    },
                ))
            }
            Message::SwitchBenchmark(benchmark) => {
                if let Some((name, _rule)) = benchmark.rules.first_key_value() {
                    let name = name.to_owned();
                    let mut tasks = vec![];

                    // Store the current benchmark in the background.
                    if let Some(current_benchmark) = self.benchmark.take() {
                        tasks.push(Task::done(Message::PushBenchmarkToBackground(
                            current_benchmark,
                        )));
                    }

                    self.benchmark = Some(benchmark);

                    // Reset pin values.
                    self.pins = HashMap::new();
                    // Reset background Benchmarks.
                    self.background_benchmarks = Vec::new();

                    // Remember when this was opened.
                    if let Some(benchmark) = &self.benchmark {
                        if let Err(error) = self.last_opened.insert(benchmark.id.clone()) {
                            tasks.push(Task::done(Message::Log(error.to_string())));
                        };
                    }

                    tasks.append(&mut vec![
                        Task::done(Message::SwitchRule(name)),
                        Task::done(Message::SwitchPopup(Some(Popup::Save))),
                    ]);

                    Task::batch(tasks)
                } else {
                    // Do nothing when an attempting to switch an empty benchmark.
                    Task::none()
                }
            }
            Message::PushBenchmarkToBackground(benchmark) => {
                let id = benchmark.id.clone();

                self.background_benchmarks.push(benchmark);

                // When a benchmark is pushed into the background, update when it was last opened to now.
                // This improves the experience by making benchmarks opened together appear next to each other
                // in the recently opened list.
                if let Err(error) = self.last_opened.insert(id) {
                    return Task::done(Message::Log(error.to_string()));
                };

                Task::none()
            }
            Message::SwitchToNextBackground => {
                if self.background_benchmarks.is_empty() {
                    return Task::none();
                }

                // Get the benchmark that has been setting in the background for
                // the longest.
                let new_benchmark = self.background_benchmarks.remove(0);

                if let Some(old_benchmark) =
                    std::mem::replace(&mut self.benchmark, Some(new_benchmark))
                {
                    self.background_benchmarks.push(old_benchmark);
                }

                // Reset pin values when switching to this new benchmark.
                self.pins = HashMap::new();

                // Remember when this was opened.
                if let Some(benchmark) = &self.benchmark {
                    if let Err(error) = self.last_opened.insert(benchmark.id.clone()) {
                        return Task::done(Message::Log(error.to_string()));
                    };
                }

                Task::none()
            }
            Message::SaveAllBenchmarks => {
                let benchmarks: Vec<&Benchmark> = if let Some(benchmark) = &self.benchmark {
                    std::iter::once(benchmark)
                        .chain(self.background_benchmarks.iter())
                        .collect()
                } else {
                    self.background_benchmarks.iter().collect()
                };

                let mut tasks = Vec::new();

                for benchmark in benchmarks {
                    let Some(mut cache_dir) = dirs::cache_dir() else {
                        tasks.push(Task::done(Message::SendErrorNotification(
                            "Error fetching cache file directory.".to_string(),
                        )));

                        continue;
                    };

                    // Create the save directory if it does not exist.
                    cache_dir.push("xylok-view/");
                    if let Err(_) = create_dir_all(&cache_dir) {
                        tasks.push(Task::done(Message::SendErrorNotification(
                            "Error creating the cache directory.".to_string(),
                        )));

                        continue;
                    };

                    // Add proper file extensions.
                    cache_dir.push(format!("{}.json.zstd", benchmark.id.clone()));

                    let Ok(mut file) = File::create(cache_dir) else {
                        tasks.push(Task::done(Message::SendErrorNotification(
                            "Error creating benchmark file.".to_string(),
                        )));

                        continue;
                    };

                    let Ok(benchmark_bytes) = benchmark.serialize(Format::InHouse) else {
                        tasks.push(Task::done(Message::SendErrorNotification(
                            "Failed to serialize benchmark to a file.".to_string(),
                        )));

                        continue;
                    };

                    if let Err(_) = file.write_all(&benchmark_bytes) {
                        tasks.push(Task::done(Message::SendErrorNotification(
                            "Failed writing benchmark to a file.".to_string(),
                        )));

                        continue;
                    };
                }

                // After saving, turn off the save menu.
                Task::done(Message::SwitchPopup(None))
            }
            Message::LoadCachedBenchmark(path) => match Benchmark::load_from_file(&path) {
                Ok(mut benchmarks) => {
                    // Should only return one benchmark, but pop it from the vector to make it a single benchmark.
                    let Some(benchmark) = benchmarks.pop() else {
                        return Task::done(Message::SendErrorNotification(
                            "Error loading cached benchmark.".to_string(),
                        ));
                    };

                    // If this benchmark is already in the background, drop the old version, add the new version.
                    self.background_benchmarks
                        .retain(|background_benchmark| background_benchmark != &benchmark);

                    let Some((_name, rule)) = benchmark.rules.first_key_value() else {
                        return Task::done(Message::SendErrorNotification(
                            "Error loading cached benchmark.".to_string(),
                        ));
                    };

                    let rule = rule.clone();

                    let mut tasks = Vec::new();

                    // Store the current benchmark in the background.
                    if let Some(current_benchmark) = self.benchmark.take() {
                        tasks.push(Task::done(Message::PushBenchmarkToBackground(
                            current_benchmark,
                        )));
                    }

                    self.benchmark = Some(benchmark);

                    // Reset pin values.
                    self.pins = HashMap::new();

                    // Remember when this was opened.
                    if let Some(benchmark) = &self.benchmark {
                        if let Err(error) = self.last_opened.insert(benchmark.id.clone()) {
                            tasks.push(Task::done(Message::Log(error.to_string())));
                        };
                    }

                    tasks.push(Task::done(Message::Display(rule.clone())));

                    Task::batch(tasks)
                }
                Err(error) => Task::batch(vec![
                    Task::done(Message::SendErrorNotification(
                        "Error loading cached benchmark.".to_string(),
                    )),
                    Task::done(Message::Log(error.to_string())),
                ]),
            },
            Message::DeleteCachedBenchmark(path) => {
                if let Err(_error) = std::fs::remove_file(path) {
                    Task::batch(vec![Task::done(Message::SendErrorNotification(
                        "Couldn't delete cached benchmark.".to_string(),
                    ))])
                } else {
                    Task::none()
                }
            }
            Message::SwitchDisplayType(display_type) => {
                self.display_type = display_type;

                Task::none()
            }

            Message::SwitchRule(id) => {
                // If the rule already displayed is being switched to, do nothing.
                if let Some(rule) = &self.displayed {
                    if rule.group_id == id {
                        return Task::none();
                    }
                }

                let Some(benchmark) = &self.benchmark else {
                    return Task::none();
                };

                if let Some(rule) = benchmark.rules.get(&id) {
                    Task::done(Message::Display(rule.to_owned()))
                } else {
                    Task::none()
                }
            }
            Message::SwitchNextRule => {
                let Some(benchmark) = &self.benchmark else {
                    return Task::none();
                };

                let Some(displayed) = &self.displayed else {
                    return Task::none();
                };

                use std::ops::Bound::{Excluded, Unbounded};

                let next = benchmark
                    .rules
                    .range::<String, _>((Excluded(displayed.group_id.clone()), Unbounded))
                    .next()
                    .or_else(|| benchmark.rules.first_key_value());

                if let Some((key, _)) = next {
                    return Task::done(Message::SwitchRule(key.clone()));
                }

                Task::none()
            }
            Message::Display(rule) => {
                self.displayed = Some(rule);

                // Only animate if configured to.
                if self.settings.animate {
                    self.animations.start("main_col");
                }

                Task::none()
            }

            Message::TypeFilter(filter_input) => {
                let Some(benchmark) = &self.benchmark else {
                    return Task::none();
                };

                self.filter_text_field = filter_input;

                let filter = self.filter_text_field.trim();

                if filter.is_empty() {
                    return Task::none();
                }

                search::run_search_cmd(filter, benchmark, &mut self.pins);

                Task::none()
            }
            Message::ResetFilter => {
                self.filter_text_field = "".to_string();

                search::reset_search_cmd(&mut self.pins);

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

            Message::ShowErrors(should_show) => {
                // Display a fade in animation for the error menu if the
                // menu was not showing and has been requested.
                if !self.display_errors && should_show {
                    self.animations.start("error_menu");
                }

                self.display_errors = should_show;

                Task::none()
            }
            Message::SendErrorNotification(error) => {
                self.error_msgs.push(error);

                Task::none()
            }
            Message::ClearErrorNotification => {
                if self.error_index >= self.error_msgs.len() {
                    return Task::none();
                }

                self.error_msgs.remove(self.error_index);

                if self.error_index == 0 {
                    return Task::none();
                }

                self.error_index -= 1;

                Task::none()
            }
            Message::ShowPreviousError => {
                if self.error_index <= 0 {
                    return Task::none();
                }

                self.error_index -= 1;

                Task::none()
            }
            Message::ShowNextError => {
                // Dont let the error index point at an error that does not exist.
                if (self.error_index + 1) >= self.error_msgs.len() {
                    return Task::none();
                }

                self.error_index += 1;

                Task::none()
            }

            Message::SwitchPopup(popup) => {
                let Some(popup) = popup else {
                    self.popup = None;

                    return Task::none();
                };

                match (&self.popup, &popup) {
                    (Some(Popup::Filter), Popup::Filter) => self.popup = None,
                    (Some(Popup::Settings), Popup::Settings) => self.popup = None,
                    _ => {
                        if self.settings.animate {
                            self.animations.start("popup");
                        }

                        self.popup = Some(popup);
                    }
                }

                Task::none()
            }
            Message::FetchLatestVersion => {
                Task::stream(stream::channel(
                    1,
                    |mut output: Sender<Message>| async move {
                        use crate::app::latest_release::is_latest_version;

                        match is_latest_version() {
                            Some(true) => return,
                            Some(false) => {
                                let _ = output.try_send(Message::ShowUpdateAvailable);
                                return;
                            }
                            // Silently fail.
                            None => return,
                        }
                    },
                ))
            }
            Message::ShowUpdateAvailable => {
                self.update_available = true;

                Task::none()
            }
            Message::OpenURL(url) => {
                let _ = open::that(url);

                Task::none()
            }

            Message::SwitchTheme(theme) => {
                self.settings.theme = theme;

                Task::done(Message::SaveSettings)
            }
            Message::SaveAnimate(animate) => {
                self.settings.animate = animate;

                Task::done(Message::SaveSettings)
            }

            Message::SaveUpdateNotify(notify) => {
                self.settings.notify_if_update = notify;

                Task::done(Message::SaveSettings)
            }
            Message::SaveSettings => {
                if let Err(error) = self.settings.save() {
                    Task::done(Message::Log(error.to_string()))
                } else {
                    Task::none()
                }
            }
            Message::Tick(now) => {
                self.animations.tick_all(now);

                Task::none()
            }

            Message::ReturnHome => {
                self.displayed = None;

                // Store the current benchmark in the background.
                if let Some(current_benchmark) = self.benchmark.take() {
                    Task::done(Message::PushBenchmarkToBackground(current_benchmark))
                } else {
                    Task::none()
                }
            }

            Message::Log(_message) => {
                // TODO.
                Task::none()
            }
        }
    }

    pub fn load_cache() -> Vec<std::path::PathBuf> {
        let Some(mut cache_dir) = dirs::cache_dir() else {
            return Vec::new();
        };

        cache_dir.push("xylok-view/");

        let entries = match std::fs::read_dir(&cache_dir) {
            Ok(entries) => entries,
            Err(_) => return Vec::new(),
        };

        entries
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                let name = path.file_name()?.to_str()?;
                if name.ends_with(".json.zstd") && name != ".json.zstd" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AppTheme::Dark => "Dark",
            AppTheme::Light => "Light",
            AppTheme::HighContrast => "High Contrast",
            AppTheme::Coffee => "Coffee",
        })
    }
}

impl std::fmt::Display for DisplayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DisplayType::GroupId => "Group ID",
            DisplayType::RuleId => "Rule ID",
            DisplayType::STIGId => "STIG ID",
        })
    }
}
