use iced::Length::{Fill, FillPortion, Shrink};
use iced::alignment::Alignment::{Center, End};
use iced::alignment::Horizontal::Left;
use iced::time;
use iced::widget::{
    Button, Container, Id, button, column, container, row, scrollable, sensor, space, stack, svg,
    text, text_editor, text_input, tooltip,
};
use iced::{Element, widget};

use crate::app::{App, Message, Popup};
use crate::sgroup::Pinned;
use crate::styles::*;

impl App {
    /// What should be displayed when nothing has been loaded.
    pub fn get_view_none_displayed(&self) -> Element<'_, Message> {
        let file_svg_handle = svg::Handle::from_memory(self.assets.file_svg.clone());
        let folder_svg_handle = svg::Handle::from_memory(self.assets.folder_svg.clone());
        let terminal_svg_handle = svg::Handle::from_memory(self.assets.terminal_svg.clone());

        let final_gui = column![
            space().height(5),
            self.window_decorations(),
            space().height(5),
            row![
                space().width(15),
                column![
                    row![
                        space().width(5),
                        tooltip(
                            button(svg(file_svg_handle).style(colored_svg))
                                .padding(2)
                                .width(40)
                                .on_press(Message::OpenFileSelect)
                                .style(no_button),
                            container(text("Open a File"))
                                .padding(4)
                                .style(tooltip_container),
                            tooltip::Position::FollowCursor,
                        )
                        .delay(time::milliseconds(600)),
                        space::horizontal(),
                        tooltip(
                            button(svg(folder_svg_handle).style(colored_svg))
                                .padding(1)
                                .width(40)
                                .on_press(Message::OpenFolderSelect)
                                .style(no_button),
                            container(text("Open a Folder"))
                                .padding(4)
                                .style(tooltip_container),
                            tooltip::Position::FollowCursor
                        )
                        .delay(time::milliseconds(600)),
                        space::horizontal(),
                        tooltip(
                            button(svg(terminal_svg_handle).style(colored_svg))
                                .padding(1)
                                .width(40)
                                .on_press(Message::ToggleCmdInput)
                                .style(no_button),
                            container(text("Open a Command Prompt"))
                                .padding(4)
                                .style(tooltip_container),
                            tooltip::Position::FollowCursor
                        )
                        .delay(time::milliseconds(600)),
                        space().width(5),
                    ],
                    space().height(15),
                    container(space::vertical())
                        .style(background_container)
                        .width(FillPortion(1)),
                ],
                space().width(15),
                container(column![
                    space::vertical(),
                    text("Tap the file or folder icon to get started.")
                        .width(Fill)
                        .align_x(Center)
                        .size(24),
                    space::vertical()
                ])
                .style(background_container)
                .width(FillPortion(5))
                .height(Fill),
                space().width(15),
            ],
            space().height(15),
        ];

        // If there is a popup, stack that on top of the main gui.
        if let Some(popup) = &self.popup {
            match popup {
                Popup::CommandPrompt => stack!(final_gui, self.command_prompt_popup()).into(),
                Popup::Error => stack!(final_gui, self.error_popup()).into(),
            }
        } else {
            final_gui.into()
        }
    }

    /// Get what should be drawn to the screen when content has been loaded.
    pub fn get_view_displayed(&self) -> Element<'_, Message> {
        let file_svg_handle = svg::Handle::from_memory(self.assets.file_svg.clone());
        let folder_svg_handle = svg::Handle::from_memory(self.assets.folder_svg.clone());
        let terminal_svg_handle = svg::Handle::from_memory(self.assets.terminal_svg.clone());

        let filter_svg_handle = svg::Handle::from_memory(self.assets.filter_svg.clone());
        let bookmark_svg_handle = svg::Handle::from_memory(self.assets.bookmark_svg.clone());
        let filled_bookmark_svg_handle =
            svg::Handle::from_memory(self.assets.bookmark_filled_svg.clone());

        // Create the buttons on the side of the application.
        let buttons_vec: Vec<Box<Button<Message>>> = self
            .list
            .read()
            .unwrap()
            .get_all()
            .iter()
            .map(|stig_wrapper| {
                let icon: svg::Handle;

                match stig_wrapper.pinned {
                    Pinned::Not => icon = bookmark_svg_handle.clone(),
                    Pinned::ByUser => icon = filled_bookmark_svg_handle.clone(),
                    Pinned::ByCmd => icon = filter_svg_handle.clone(),
                }

                Box::new(
                    button(
                        row![
                            space().width(32),
                            text(stig_wrapper.stig.version.clone())
                                .height(Fill)
                                .width(Fill)
                                .center(),
                            space::horizontal(),
                            button(svg(icon).height(32).style(colored_svg))
                                .padding(1)
                                .style(no_button)
                                .on_press(Message::UserPin(stig_wrapper.uuid))
                        ]
                        .align_y(Center),
                    )
                    .height(50)
                    .padding(8)
                    .width(Fill)
                    .style(rounded_boring_button)
                    .on_press(Message::SwitchDisplayed(stig_wrapper.uuid.clone())),
                )
            })
            .collect();

        let mut button_col = column![].padding(1);

        for button in buttons_vec {
            button_col = button_col.push(*button);
            button_col = button_col.push(space().height(8)) // Add a tiny seperation between each button.
        }

        // Always a displayed stig when this function is called.
        if let Some(_stig) = &*self.displayed.read().unwrap() {
            let stig_col = column![
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

            let final_gui = column![
                space().height(5),
                self.window_decorations(),
                space().height(5),
                row![
                    space().width(15),
                    column![
                        row![
                            space().width(5),
                            tooltip(
                                button(svg(file_svg_handle).style(colored_svg))
                                    .padding(2)
                                    .width(40)
                                    .on_press(Message::OpenFileSelect)
                                    .style(no_button),
                                container(text("Open a File"))
                                    .padding(4)
                                    .style(tooltip_container),
                                tooltip::Position::FollowCursor,
                            )
                            .delay(time::milliseconds(600)),
                            space::horizontal(),
                            tooltip(
                                button(svg(folder_svg_handle).style(colored_svg))
                                    .padding(1)
                                    .width(40)
                                    .on_press(Message::OpenFolderSelect)
                                    .style(no_button),
                                container(text("Open a Folder"))
                                    .padding(4)
                                    .style(tooltip_container),
                                tooltip::Position::FollowCursor
                            )
                            .delay(time::milliseconds(600)),
                            space::horizontal(),
                            tooltip(
                                button(svg(terminal_svg_handle).style(colored_svg))
                                    .padding(1)
                                    .width(40)
                                    .on_press(Message::ToggleCmdInput)
                                    .style(no_button),
                                container(text("Open a Command Prompt"))
                                    .padding(4)
                                    .style(tooltip_container),
                                tooltip::Position::FollowCursor
                            )
                            .delay(time::milliseconds(600)),
                            space().width(5),
                        ],
                        space().height(15),
                        container(column![
                            scrollable(button_col).spacing(5),
                            space::vertical()
                        ])
                        .style(background_container)
                        .padding(5)
                        .width(FillPortion(1))
                    ],
                    space().width(15),
                    container(column![scrollable(stig_col).spacing(5), space::vertical()])
                        .style(background_container)
                        .padding(15)
                        .width(FillPortion(5))
                        .height(Fill),
                    space().width(15),
                ],
                space().height(15),
            ];

            // If there is a popup, stack that above the main gui.
            if let Some(popup) = &self.popup {
                match popup {
                    Popup::CommandPrompt => stack!(final_gui, self.command_prompt_popup()).into(),
                    Popup::Error => stack!(final_gui, self.error_popup()).into(),
                }
            } else {
                final_gui.into()
            }
        } else {
            unreachable!();
        }
    }

    /// A function that returns the cmd prompt popup ui.
    fn command_prompt_popup(&self) -> Container<'_, Message> {
        let filter_svg_handle = svg::Handle::from_memory(self.assets.filter_svg.clone());
        let right_tick_svg_handle = svg::Handle::from_memory(self.assets.right_tick_svg.clone());

        let id = Id::new("cmd_text_input");

        container(
            sensor(
                container(
                    column![
                        row![
                            svg(filter_svg_handle).width(Shrink).style(boring_svg),
                            space().width(5),
                            text_input("Type commands here, then press enter...", &self.cmd_input)
                                .on_input(Message::ChangeCmdInput)
                                .on_submit(Message::SubmitCmdInput)
                                .id(id.clone()),
                        ]
                        .height(30),
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

    /// A function that returns the error popup ui.
    /// Currently unused.
    fn error_popup(&self) -> Container<'_, Message> {
        container(
            container(column![
                text("An Error has Occured.")
                    .width(Fill)
                    .height(Fill)
                    .center(),
            ])
            .width(500)
            .height(150)
            .padding(5)
            .style(container::danger),
        )
        .center(Fill)
    }

    fn window_decorations(&self) -> Container<'_, Message> {
        let cross_svg_handle = svg::Handle::from_memory(self.assets.cross_svg.clone());
        let square_svg_handle = svg::Handle::from_memory(self.assets.square_svg.clone());
        let down_tick_svg_handle = svg::Handle::from_memory(self.assets.down_tick_svg.clone());

        // A complicated way of getting mouse_area to work.
        // Captures mouse input in the window decorations so the window can be dragged.
        container(
            widget::mouse_area(
                container(
                    row![
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
}
