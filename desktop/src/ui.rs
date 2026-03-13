use iced::Length::{Fill, FillPortion, Shrink};
use iced::alignment::Alignment::{Center, End};
use iced::alignment::Horizontal::Left;
use iced::widget::{
    Button, Column, Container, Id, button, column, container, row, scrollable, sensor, space,
    stack, svg, text, text_editor, text_input, tooltip,
};
use iced::{Element, widget};

use crate::app;
use crate::app::{App, Message, Popup};
use crate::styles::*;
use stig_view_core::db::{DB, Pinned};

impl App {
    /// Get the application gui.
    pub fn get_view(&self) -> Element<'_, Message> {
        if let Some(_name) = &self.displayed {
            let db = self.db.clone();

            let mut list_col = self.get_stig_buttons(db.clone());
            list_col = list_col.push(space::vertical());

            let main_col = column![
                text("Version").size(32),
                row![
                    space().width(10),
                    text_editor(&self.content[0])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(action, 0))
                ],
                space().height(15),
                text("Introduction").size(32),
                row![
                    space().width(10),
                    text_editor(&self.content[1])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(action, 1))
                ],
                space().height(15),
                text("Description").size(32),
                row![
                    space().width(10),
                    text_editor(&self.content[2])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(action, 2))
                ],
                space().height(15),
                text("Check").size(32),
                row![
                    space().width(10),
                    text_editor(&self.content[3])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(action, 3))
                ],
                space().height(15),
                text("Fix").size(32),
                row![
                    space().width(10),
                    text_editor(&self.content[4])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(action, 4))
                ],
                space().height(15),
                text("Similar Checks").size(32),
                row![
                    space().width(10),
                    text_editor(&self.content[5])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(action, 5))
                ],
            ];

            let main_gui = self.main_gui(self.window_decorations(), list_col, main_col);

            match self.popup {
                Some(Popup::CommandPrompt) => self.get_stack(self.command_prompt_popup(), main_gui),
                None => main_gui,
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
                Some(Popup::CommandPrompt) => self.get_stack(self.command_prompt_popup(), main_gui),
                None => main_gui,
            }
        }
    }

    /// The main gui of the application with some inputs.
    fn main_gui<'a, Message>(
        &self,
        window_decorations: Container<'a, Message>,
        list_col: Column<'a, Message>,
        main_col: Column<'a, Message>,
    ) -> Element<'a, Message>
    where
        Message: 'a,
    {
        column![
            space().height(5),
            window_decorations,
            space().height(5),
            row![
                space().width(15),
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
                    .width(FillPortion(1))
                    .height(Fill)
                    .padding(15),
                space().width(15),
            ],
            space().height(15),
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

        let id = Id::new("cmd_text_input");

        let prompt_svg: svg::Svg;

        if let Some(_command) = app::parse_command(&self.cmd_input) {
            prompt_svg =
                svg(svg::Handle::from_memory(self.assets.check_svg.clone())).style(good_svg);
        } else {
            prompt_svg =
                svg(svg::Handle::from_memory(self.assets.cross_svg.clone())).style(bad_svg);
        }

        container(
            sensor(
                container(
                    column![
                        space().height(5),
                        row![
                            //prompt_svg.width(Shrink),
                            space().width(1),
                            text_input("Type commands here, then press enter...", &self.cmd_input)
                                .on_input(Message::ChangeCmdInput)
                                .on_submit(Message::SubmitCmdInput)
                                .id(id.clone()),
                            space().width(15),
                            prompt_svg.width(Shrink),
                        ]
                        .height(32),
                        space().height(20),
                        row![
                            svg(right_tick_svg_handle.clone())
                                .style(boring_svg)
                                .width(Shrink),
                            space().width(5),
                            text("(title|name) (...) to filter by title."),
                        ]
                        .height(24),
                        row![
                            svg(right_tick_svg_handle.clone())
                                .style(boring_svg)
                                .width(Shrink),
                            space().width(5),
                            text("(search|find) (...) to filter by keywords."),
                        ]
                        .height(24),
                        row![
                            svg(right_tick_svg_handle.clone())
                                .style(boring_svg)
                                .width(Shrink),
                            space().width(5),
                            text("(reset) to undo applied filters."),
                        ]
                        .height(24),
                        space().height(15),
                        row![
                            svg(right_tick_svg_handle.clone())
                                .style(boring_svg)
                                .width(Shrink),
                            space().width(5),
                            text("All commands are case sensitive."),
                        ]
                        .height(24),
                        space().height(5),
                    ]
                    .align_x(Left),
                )
                .width(550)
                .height(Shrink)
                .padding(15)
                .style(cmd_container),
            )
            .on_show(move |_| Message::FocusCmdInput(id.clone())),
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
            widget::mouse_area(
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
                        .on_press(Message::OpenFileSelect),
                        space().width(8),
                        tooltip(
                            button(text("File").center().size(14))
                                .padding(6)
                                .width(Shrink)
                                .height(Shrink)
                                .style(rounded_dark_button)
                                .on_press(Message::OpenFileSelect),
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
                                .on_press(Message::OpenFolderSelect),
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
                                .on_press(Message::ToggleCmdInput),
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
                        .on_press(Message::MinimizeApp),
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
                        .on_press(Message::ToggleFullscreenApp),
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
                        .on_press(Message::CloseApp),
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
            .on_press(Message::MoveWindow),
        )
    }

    /// Return a column of the sorted buttons to display.
    fn get_stig_buttons(&self, db: DB) -> Column<'_, Message> {
        let mut user_pin_col: Vec<Box<Button<'_, Message>>> = vec![];
        let mut filter_pin_col: Vec<Box<Button<'_, Message>>> = vec![];
        let mut not_pin_col: Vec<Box<Button<'_, Message>>> = vec![];

        let filter_svg_handle = svg::Handle::from_memory(self.assets.filter_svg.clone());
        let bookmark_svg_handle = svg::Handle::from_memory(self.assets.bookmark_svg.clone());
        let filled_bookmark_svg_handle =
            svg::Handle::from_memory(self.assets.bookmark_filled_svg.clone());

        let snapshot = db.snapshot().expect("DB Snapshot error.");

        for (name, data) in snapshot.iter() {
            match data.get_pin() {
                Pinned::ByUser => {
                    user_pin_col.push(Box::new(
                        button(
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
                                .on_press(Message::UserPin(name.to_owned()))
                            ]
                            .align_y(Center),
                        )
                        .height(50)
                        .padding(8)
                        .width(Fill)
                        .style(rounded_boring_button)
                        .on_press(Message::SetDisplayed(name.to_owned())),
                    ));
                }
                Pinned::ByFilter => {
                    filter_pin_col.push(Box::new(
                        button(
                            row![
                                text(name.to_owned()).center(),
                                space::horizontal(),
                                button(
                                    svg(filter_svg_handle.clone())
                                        .width(32)
                                        .height(32)
                                        .style(good_svg)
                                )
                                .padding(2)
                                .style(no_button)
                                .on_press(Message::UserPin(name.to_owned()))
                            ]
                            .align_y(Center),
                        )
                        .height(50)
                        .padding(8)
                        .width(Fill)
                        .style(rounded_boring_button)
                        .on_press(Message::SetDisplayed(name.to_owned())),
                    ));
                }
                Pinned::Not => {
                    not_pin_col.push(Box::new(
                        button(
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
                                .on_press(Message::UserPin(name.to_owned()))
                            ]
                            .align_y(Center),
                        )
                        .height(50)
                        .padding(8)
                        .width(Fill)
                        .style(rounded_boring_button)
                        .on_press(Message::SetDisplayed(name.to_owned())),
                    ));
                }
            }
        }

        user_pin_col.append(&mut filter_pin_col);
        user_pin_col.append(&mut not_pin_col);

        let mut col = column![].padding(1).spacing(8);

        for button in user_pin_col {
            col = col.push(*button);
        }

        col
    }
}
