mod styles;

use iced::Alignment::End;
use iced::Element;
use iced::widget::{
    Button, Column, Container, button, column, container, mouse_area, row, rule, scrollable,
    sensor, space, stack, svg, text, text_editor, text_input, tooltip,
};
use iced::widget::{Id, pick_list};
use iced::window::Direction::{
    East, North, NorthEast, NorthWest, South, SouthEast, SouthWest, West,
};
use iced::{Center, Fill, Shrink};
use stig_view_core::db::{DB, Pinned};

use crate::app::*;
use crate::ui::styles::*;

impl App {
    /// Get the application gui.
    pub fn get_view(&self) -> Element<'_, Message> {
        if let Some(_name) = &self.displayed {
            let db = self.db.clone();

            let mut list_col = self.get_stig_buttons(db.clone());
            list_col = list_col.push(space::vertical());

            let main_col = column![
                text("Version").size(32),
                rule::horizontal(2),
                space().height(15),
                row![
                    space().width(10),
                    text_editor(&self.contents[0])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(action, ContentSlot::Version))
                ],
                space().height(15),
                text("Introduction").size(32),
                rule::horizontal(2),
                space().height(15),
                row![
                    space().width(10),
                    text_editor(&self.contents[1])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(action, ContentSlot::Intro))
                ],
                space().height(15),
                text("Description").size(32),
                rule::horizontal(2),
                space().height(15),
                row![
                    space().width(10),
                    text_editor(&self.contents[2])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(action, ContentSlot::Desc))
                ],
                space().height(15),
                text("Check").size(32),
                rule::horizontal(2),
                space().height(15),
                row![
                    space().width(10),
                    text_editor(&self.contents[3])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(action, ContentSlot::CheckText))
                ],
                space().height(15),
                text("Fix").size(32),
                rule::horizontal(2),
                space().height(15),
                row![
                    space().width(10),
                    text_editor(&self.contents[4])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(action, ContentSlot::FixText))
                ],
                space().height(15),
                text("Similar Checks").size(32),
                rule::horizontal(2),
                space().height(15),
                row![
                    space().width(10),
                    text_editor(&self.contents[5])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(
                            action,
                            ContentSlot::SimilarChecks
                        ))
                ],
            ];

            let main_gui = self.main_gui(self.window_decorations(), list_col, main_col);

            match self.popup {
                Popup::Filter => self.get_stack(self.command_prompt_popup(), main_gui),
                Popup::Settings => self.get_stack(self.settings_menu(), main_gui),
                Popup::None => main_gui,
            }
        } else {
            let list_col = column![space::vertical()];
            let main_col = column![
                space().height(300),
                text("Open a file or folder to get started.")
                    .width(Fill)
                    .center()
                    .size(24),
            ];

            let main_gui = self.main_gui(self.window_decorations(), list_col, main_col);

            match self.popup {
                Popup::Filter => self.get_stack(self.command_prompt_popup(), main_gui),
                Popup::Settings => self.get_stack(self.settings_menu(), main_gui),
                Popup::None => main_gui,
            }
        }
    }

    /// The main gui of the application with some inputs.
    fn main_gui<'a>(
        &self,
        window_decorations: Container<'a, Message>,
        list_col: Column<'a, Message>,
        main_col: Column<'a, Message>,
    ) -> Element<'a, Message>
    where
        Message: 'a,
    {
        // There are a few mouse areas here.
        // Without window decorations, we need to handle windoe drag and click resizing ourselves.
        // So we surround the gui on every edge with a mouse area to detect window resizing.

        column![
            row![
                container(
                    mouse_area(container(space::horizontal()).width(10).height(10))
                        .on_press(Message::WindowDragResize(NorthWest))
                ),
                container(
                    mouse_area(container(space::horizontal()).width(Fill).height(10))
                        .on_press(Message::WindowDragResize(North))
                ),
                container(
                    mouse_area(container(space::horizontal()).width(10).height(10))
                        .on_press(Message::WindowDragResize(NorthEast))
                ),
            ],
            window_decorations,
            space().height(10),
            row![
                container(
                    mouse_area(container(space::horizontal()).width(15).height(Fill))
                        .on_press(Message::WindowDragResize(West))
                ),
                column![
                    container(scrollable(list_col).spacing(4))
                        .style(background_container)
                        .width(250)
                        .height(Fill)
                        .padding(8)
                ],
                space().width(15),
                container(scrollable(main_col).spacing(4))
                    .style(background_container)
                    .width(Fill)
                    .height(Fill)
                    .padding(15),
                container(
                    mouse_area(container(space::horizontal()).width(15).height(Fill))
                        .on_press(Message::WindowDragResize(East))
                ),
            ],
            row![
                container(
                    mouse_area(container(space::horizontal()).width(15).height(15))
                        .on_press(Message::WindowDragResize(SouthWest))
                ),
                container(
                    mouse_area(container(space::horizontal()).width(Fill).height(15))
                        .on_press(Message::WindowDragResize(South))
                ),
                container(
                    mouse_area(container(space::horizontal()).width(15).height(15))
                        .on_press(Message::WindowDragResize(SouthEast))
                ),
            ],
        ]
        .into()
    }

    /// Stack the main gui with a popup.
    fn get_stack<'a, Message>(
        &self,
        popup: Container<'a, Message>,
        main_gui: Element<'a, Message>,
    ) -> Element<'a, Message>
    where
        Message: 'a,
    {
        stack![main_gui, popup].into()
    }

    /// A function that returns the cmd prompt popup ui as a container.
    fn command_prompt_popup(&self) -> Container<'_, Message> {
        let right_tick_svg_handle = svg::Handle::from_memory(self.assets.right_tick_svg.clone());
        let cross_svg_handle = svg::Handle::from_memory(self.assets.cross_svg.clone());

        let id = Id::new("filter_text_input");

        container(
            sensor(
                container(
                    column![
                        row![
                            space::horizontal(),
                            space().width(13),
                            text("Filter Menu"),
                            space::horizontal(),
                            button(
                                svg(cross_svg_handle)
                                    .style(colored_svg)
                                    .width(25)
                                    .height(25)
                            )
                            .padding(1)
                            .width(Shrink)
                            .height(Shrink)
                            .style(no_button)
                            .on_press(Message::SwitchPopup(Popup::None)),
                        ]
                        .align_y(Center),
                        space().height(20),
                        text_input(
                            "Type commands here, then press enter...",
                            &self.filter_input
                        )
                        .on_input(Message::TypeCmd)
                        .on_submit(Message::ProcessCmd(self.filter_input.clone()))
                        .id(id.clone()),
                        space().height(20),
                        row![
                            svg(right_tick_svg_handle.clone())
                                .style(boring_svg)
                                .width(Shrink),
                            space().width(5),
                            text("(name|title)  (...) to filter by title."),
                            space::horizontal(),
                        ]
                        .height(24)
                        .align_y(Center),
                        row![
                            svg(right_tick_svg_handle.clone())
                                .style(boring_svg)
                                .width(Shrink),
                            space().width(5),
                            text("(find|search) (...) to filter by keywords."),
                            space::horizontal(),
                        ]
                        .height(24)
                        .align_y(Center),
                        row![
                            svg(right_tick_svg_handle.clone())
                                .style(boring_svg)
                                .width(Shrink),
                            space().width(5),
                            text("(reset) to undo applied filters."),
                            space::horizontal(),
                        ]
                        .height(24)
                        .align_y(Center),
                    ]
                    .align_x(Center),
                )
                .width(500)
                .height(200)
                .padding(15)
                .style(cmd_container),
            )
            .on_show(move |_| Message::FocusWidget(id.clone())),
        )
        .center(Fill)
    }

    fn settings_menu(&self) -> Container<'_, Message> {
        let cross_svg_handle = svg::Handle::from_memory(self.assets.cross_svg.clone());

        let themes = [AppTheme::Dark, AppTheme::Light];

        container(
            container(
                column![
                    row![
                        space::horizontal(),
                        space().width(13),
                        text("Settings Menu"),
                        space::horizontal(),
                        button(
                            svg(cross_svg_handle)
                                .style(colored_svg)
                                .width(25)
                                .height(25)
                        )
                        .padding(1)
                        .width(Shrink)
                        .height(Shrink)
                        .style(no_button)
                        .on_press(Message::SwitchPopup(Popup::None)),
                    ]
                    .align_y(Center),
                    space().height(20),
                    row![
                        text("Theme"),
                        space::horizontal(),
                        pick_list(themes, self.current_theme, Message::SwitchTheme),
                    ]
                    .align_y(Center)
                ]
                .align_x(Center),
            )
            .width(375)
            .height(150)
            .padding(15)
            .style(cmd_container),
        )
        .center(Fill)
    }

    /// Return the window decorations container.
    fn window_decorations(&self) -> Container<'_, Message> {
        let settings_svg_handle = svg::Handle::from_memory(self.assets.settings_svg.clone());
        let cross_svg_handle = svg::Handle::from_memory(self.assets.cross_svg.clone());
        let square_svg_handle = svg::Handle::from_memory(self.assets.square_svg.clone());
        let down_tick_svg_handle = svg::Handle::from_memory(self.assets.down_tick_svg.clone());

        // A complicated way of getting mouse_area to work.
        // Captures mouse input in the window decorations so the window can be dragged.
        container(
            mouse_area(
                container(
                    row![
                        space().width(11),
                        button(
                            svg(settings_svg_handle)
                                .style(colored_svg)
                                .width(20)
                                .height(20)
                        )
                        .padding(1)
                        .width(Shrink)
                        .height(Shrink)
                        .style(no_button)
                        .on_press(Message::SwitchPopup(Popup::Settings)),
                        space().width(8),
                        tooltip(
                            button(text("File").center().size(14))
                                .padding(6)
                                .width(Shrink)
                                .height(Shrink)
                                .style(rounded_dark_button)
                                .on_press(Message::OpenFile),
                            container("Ctrl + I").style(tooltip_container).padding(4),
                            tooltip::Position::Right
                        )
                        .delay(iced::time::Duration::from_secs(1)),
                        space().width(4),
                        tooltip(
                            button(text("Folder").center().size(14))
                                .padding(6)
                                .width(Shrink)
                                .height(Shrink)
                                .style(rounded_dark_button)
                                .on_press(Message::OpenFolder),
                            container("Ctrl + O").style(tooltip_container).padding(4),
                            tooltip::Position::Right
                        )
                        .delay(iced::time::Duration::from_secs(1)),
                        space().width(4),
                        tooltip(
                            button(text("Filter").center().size(14))
                                .padding(6)
                                .width(Shrink)
                                .height(Shrink)
                                .style(rounded_dark_button)
                                .on_press(Message::SwitchPopup(Popup::Filter)),
                            container("Ctrl + P").style(tooltip_container),
                            tooltip::Position::Right
                        )
                        .delay(iced::time::Duration::from_secs(1)),
                        space::horizontal(),
                        button(
                            svg(down_tick_svg_handle)
                                .style(colored_svg)
                                .width(20)
                                .height(20)
                        )
                        .padding(1)
                        .width(Shrink)
                        .height(Shrink)
                        .style(no_button)
                        .on_press(Message::WindowMin),
                        space().width(15),
                        button(
                            svg(square_svg_handle)
                                .style(colored_svg)
                                .width(16)
                                .height(16)
                        )
                        .padding(1)
                        .width(Shrink)
                        .height(Shrink)
                        .style(no_button)
                        .on_press(Message::WindowFullscreenToggle),
                        space().width(15),
                        button(
                            svg(cross_svg_handle)
                                .style(colored_svg)
                                .width(25)
                                .height(25)
                        )
                        .padding(1)
                        .width(Shrink)
                        .height(Shrink)
                        .style(no_button)
                        .on_press(Message::WindowClose),
                        space().width(8)
                    ]
                    .align_y(Center),
                )
                .height(26)
                .padding(1)
                .align_x(End)
                .align_y(Center)
                .width(Fill),
            )
            .on_press(Message::WindowMove),
        )
    }

    /// Return a column of the sorted buttons to display.
    fn get_stig_buttons(&self, db: DB) -> Column<'_, Message> {
        let mut not_pin_col: Vec<Box<Button<'_, Message>>> = vec![];
        let mut user_pin_col: Vec<Box<Button<'_, Message>>> = vec![];
        let mut filter_pin_col: Vec<Box<Button<'_, Message>>> = vec![];
        let mut filter_user_pin_col: Vec<Box<Button<'_, Message>>> = vec![];

        let filter_svg_handle = svg::Handle::from_memory(self.assets.filter_svg.clone());
        let bookmark_svg_handle = svg::Handle::from_memory(self.assets.bookmark_svg.clone());
        let filled_bookmark_svg_handle =
            svg::Handle::from_memory(self.assets.bookmark_filled_svg.clone());

        let snapshot = db.snapshot().unwrap_or_default();

        for (name, data) in snapshot.iter() {
            match data.get_pin() {
                Pinned::Not => {
                    not_pin_col.push(Box::new(
                        button(
                            column![
                                row![
                                    text(name.to_owned()).center(),
                                    space::horizontal(),
                                    button(
                                        svg(bookmark_svg_handle.clone())
                                            .width(32)
                                            .height(32)
                                            .style(colored_svg)
                                    )
                                    .padding(2)
                                    .style(no_button)
                                    .on_press(Message::Pin(name.to_owned()))
                                ]
                                .align_y(Center)
                                .height(Fill),
                            ]
                            .align_x(Center)
                            .width(Fill),
                        )
                        .height(64)
                        .padding(8)
                        .width(Fill)
                        .style(rounded_boring_button)
                        .on_press(Message::Display(data.get_stig())),
                    ));
                }
                Pinned::ByUser => {
                    user_pin_col.push(Box::new(
                        button(
                            column![
                                row![
                                    text(name.to_owned()).center(),
                                    space::horizontal(),
                                    button(
                                        svg(filled_bookmark_svg_handle.clone())
                                            .width(32)
                                            .height(32)
                                            .style(colored_svg)
                                    )
                                    .padding(2)
                                    .style(no_button)
                                    .on_press(Message::Pin(name.to_owned()))
                                ]
                                .align_y(Center)
                                .height(Fill),
                            ]
                            .align_x(Center)
                            .width(Fill),
                        )
                        .height(64)
                        .padding(8)
                        .width(Fill)
                        .style(rounded_boring_button)
                        .on_press(Message::Display(data.get_stig())),
                    ));
                }
                Pinned::ByFilter => {
                    filter_pin_col.push(Box::new(
                        button(
                            column![
                                row![
                                    text(name.to_owned()).center(),
                                    space::horizontal(),
                                    svg(filter_svg_handle.clone())
                                        .width(32)
                                        .height(32)
                                        .style(good_svg),
                                    button(
                                        svg(bookmark_svg_handle.clone())
                                            .width(32)
                                            .height(32)
                                            .style(colored_svg)
                                    )
                                    .padding(2)
                                    .style(no_button)
                                    .on_press(Message::Pin(name.to_owned()))
                                ]
                                .align_y(Center)
                                .height(Fill),
                            ]
                            .align_x(Center)
                            .width(Fill),
                        )
                        .height(64)
                        .padding(8)
                        .width(Fill)
                        .style(rounded_boring_button)
                        .on_press(Message::Display(data.get_stig())),
                    ));
                }
                Pinned::ByFilterAndUser => {
                    filter_user_pin_col.push(Box::new(
                        button(
                            column![
                                row![
                                    text(name.to_owned()).center(),
                                    space::horizontal(),
                                    svg(filter_svg_handle.clone())
                                        .width(32)
                                        .height(32)
                                        .style(good_svg),
                                    button(
                                        svg(filled_bookmark_svg_handle.clone())
                                            .width(32)
                                            .height(32)
                                            .style(colored_svg)
                                    )
                                    .padding(2)
                                    .style(no_button)
                                    .on_press(Message::Pin(name.to_owned()))
                                ]
                                .align_y(Center)
                                .height(Fill),
                            ]
                            .align_x(Center)
                            .width(Fill),
                        )
                        .height(64)
                        .padding(8)
                        .width(Fill)
                        .style(rounded_boring_button)
                        .on_press(Message::Display(data.get_stig())),
                    ));
                }
            }
        }

        filter_user_pin_col.append(&mut user_pin_col);
        filter_user_pin_col.append(&mut filter_pin_col);
        filter_user_pin_col.append(&mut not_pin_col);

        let mut col = column![].padding(1).spacing(8);

        for button in filter_user_pin_col {
            col = col.push(*button);
        }

        col
    }
}
