use iced::Element;
use iced::Subscription;
use iced::Task;
use iced::color;
use iced::keyboard;
use iced::theme::{Custom, Palette, Theme};
use iced::window::icon::from_file_data;
use image::ImageFormat;
use std::sync::Arc;

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
            },
            window::oldest().map(Message::InitWindow),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(|event| Some(Message::KeyPressed(event)))
    }

    pub fn theme(&self) -> Theme {
        // Dark theme:
        let palette = Palette {
            background: color!(0x1B1C1C),
            text: color!(0xE6E6E6),
            primary: color!(0xA2A2D0),
            success: color!(0x188B6C),
            warning: color!(0xffc14e),
            danger: color!(0xc3423f),
        };

        // Light theme
        /*let palette = Palette {
            background: color!(0xDFD7D5),
            text: color!(0x1B1C1C),
            primary: color!(0x444488),
            success: color!(0x188B6C),
            warning: color!(0xffc14e),
            danger: color!(0xc3423f),
        };*/

        Theme::Custom(Arc::new(Custom::new(String::from("Custom Dark"), palette)))
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
                        //window::toggle_decorations(self.window_id.unwrap()),
                        //window::set_resizable(self.window_id.unwrap(), false),
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

            _ => Task::none(),
        }
    }
}
