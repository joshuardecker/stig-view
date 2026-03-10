use iced::Element;
use iced::Subscription;
use iced::Task;
use iced::color;
use iced::keyboard;
use iced::theme::{Custom, Palette, Theme};
use iced::window::icon::from_file_data;
use image::ImageFormat;
use std::sync::Arc;

use crate::app::async_fns::open_file;
use crate::app::*;

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                db: DB::new(),
                displayed: None,
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
                if let Some(id) = id {
                    self.window_id = Some(id);

                    // Toggle window decorations and set the app icon.
                    return Task::batch(vec![
                        window::toggle_decorations(self.window_id.unwrap()),
                        window::set_resizable(self.window_id.unwrap(), true),
                        window::set_icon(
                            self.window_id.unwrap(),
                            from_file_data(&self.assets.app_icon, Some(ImageFormat::Png))
                                .expect("Could not load app icon!"),
                        ),
                    ]);
                }

                panic!("Not able to retrieve window id.")
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

                    Message::Switch(id)
                })
            }
            Message::OpenFolder => todo!(),

            Message::SelectContent(action, idx) => todo!(),

            Message::Switch(id) => todo!(),
            Message::SwitchNext => todo!(),

            Message::SwitchPopup(popup) => todo!(),

            Message::Pin(id) => todo!(),

            Message::KeyPressed(key_event) => todo!(),
        }
    }
}
