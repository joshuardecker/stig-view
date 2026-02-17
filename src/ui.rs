use iced::Element;
use iced::Length::{Fill, FillPortion, Shrink};
use iced::alignment::Alignment::Center;
use iced::alignment::Horizontal::Left;
use iced::widget::{
    Button, Container, Id, button, column, container, row, scrollable, sensor, space, stack, svg,
    text, text_editor, text_input,
};
use iced::{Background, Shadow};
use iced::{Border, Theme, border, color};

use crate::app::{App, Message, Popup};
use crate::sgroup::Pinned;

impl App {
    /// What should be displayed when nothing has been loaded.
    pub fn get_view_none_displayed(&self) -> Element<'_, Message> {
        let file_svg_handle = svg::Handle::from_memory(self.assets.file_svg.clone());
        let folder_svg_handle = svg::Handle::from_memory(self.assets.folder_svg.clone());
        let terminal_svg_handle = svg::Handle::from_memory(self.assets.terminal_svg.clone());

        let final_gui = column![
            space().height(15),
            row![
                space().width(15),
                column![
                    row![
                        button(svg(file_svg_handle))
                            .padding(1)
                            .width(40)
                            .on_press(Message::OpenFileSelect),
                        space::horizontal(),
                        button(svg(folder_svg_handle))
                            .padding(1)
                            .width(40)
                            .on_press(Message::OpenFolderSelect),
                        space::horizontal(),
                        button(svg(terminal_svg_handle))
                            .padding(1)
                            .width(40)
                            .on_press(Message::ToggleCmdInput),
                    ],
                    space().height(15),
                    container(space::vertical())
                        .style(stig_list_container_style)
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
                .style(stig_container_initial_style)
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
        let lightbulb_svg_handle = svg::Handle::from_memory(self.assets.lightbulb_svg.clone());
        let lightbulb_filled_svg_handle =
            svg::Handle::from_memory(self.assets.lightbulb_filled_svg.clone());

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
                    Pinned::Not => icon = lightbulb_svg_handle.clone(),
                    Pinned::ByUser => icon = lightbulb_filled_svg_handle.clone(),
                    Pinned::ByCmd => icon = terminal_svg_handle.clone(),
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
                            button(svg(icon).height(32))
                                .padding(1)
                                .style(button_no_style)
                                .on_press(Message::UserPin(stig_wrapper.uuid))
                        ]
                        .align_y(Center),
                    )
                    .height(50)
                    .padding(8)
                    .width(Fill)
                    .style(button::primary)
                    .on_press(Message::SwitchDisplayed(stig_wrapper.uuid.clone())),
                )
            })
            .collect();

        let mut button_col = column![];

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
                        .style(text_editor_no_style)
                        .on_action(|action| Message::SelectContent(action, 0))
                ],
                space().height(10),
                text("Introduction").size(32),
                row![
                    space().width(10),
                    text_editor(&self.content[1])
                        .style(text_editor_no_style)
                        .on_action(|action| Message::SelectContent(action, 1))
                ],
                space().height(10),
                text("Description").size(32),
                row![
                    space().width(10),
                    text_editor(&self.content[2])
                        .style(text_editor_no_style)
                        .on_action(|action| Message::SelectContent(action, 2))
                ],
                space().height(10),
                text("Check").size(32),
                row![
                    space().width(10),
                    text_editor(&self.content[3])
                        .style(text_editor_no_style)
                        .on_action(|action| Message::SelectContent(action, 3))
                ],
                space().height(10),
                text("Fix").size(32),
                row![
                    space().width(10),
                    text_editor(&self.content[4])
                        .style(text_editor_no_style)
                        .on_action(|action| Message::SelectContent(action, 4))
                ],
                space().height(10),
                text("Similar Checks").size(32),
                row![
                    space().width(10),
                    text_editor(&self.content[5])
                        .style(text_editor_no_style)
                        .on_action(|action| Message::SelectContent(action, 5))
                ],
            ];

            let final_gui = column![
                space().height(15),
                row![
                    space().width(15),
                    column![
                        row![
                            button(svg(file_svg_handle))
                                .padding(1)
                                .width(40)
                                .on_press(Message::OpenFileSelect),
                            space::horizontal(),
                            button(svg(folder_svg_handle))
                                .padding(1)
                                .width(40)
                                .on_press(Message::OpenFolderSelect),
                            space::horizontal(),
                            button(svg(terminal_svg_handle))
                                .padding(1)
                                .width(40)
                                .on_press(Message::ToggleCmdInput),
                        ],
                        space().height(15),
                        container(column![
                            scrollable(button_col).spacing(5),
                            space::vertical()
                        ])
                        .style(stig_list_container_style)
                        .padding(5)
                        .width(FillPortion(1))
                    ],
                    space().width(15),
                    container(column![scrollable(stig_col).spacing(5), space::vertical()])
                        .style(stig_container_style)
                        .padding(10)
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
        let right_tick_svg_handle = svg::Handle::from_memory(self.assets.right_tick_svg.clone());

        let id = Id::new("cmd_text_input");

        container(
            sensor(
                container(
                    column![
                        text_input("Type commands here, then press enter...", &self.cmd_input)
                            .on_input(Message::ChangeCmdInput)
                            .on_submit(Message::SubmitCmdInput)
                            .id(id.clone()),
                        space().height(20),
                        row![
                            svg(right_tick_svg_handle.clone())
                                .style(right_tick_svg_style)
                                .width(Shrink),
                            space().width(5),
                            text("(title|name) (...) to filter by title."),
                        ]
                        .height(24),
                        row![
                            svg(right_tick_svg_handle.clone())
                                .style(right_tick_svg_style)
                                .width(Shrink),
                            space().width(5),
                            text("(search|find) (...) to filter by keywords."),
                        ]
                        .height(24),
                        row![
                            svg(right_tick_svg_handle.clone())
                                .style(right_tick_svg_style)
                                .width(Shrink),
                            space().width(5),
                            text("(reset) to undo applied filters."),
                        ]
                        .height(24),
                        space().height(15),
                        row![
                            svg(right_tick_svg_handle.clone())
                                .style(right_tick_svg_style)
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
                .style(cmd_prompt_container_style),
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
}

/// Get a style for a text editor that is transparent when possible.
fn text_editor_no_style(theme: &Theme, _status: text_editor::Status) -> text_editor::Style {
    let palette = theme.extended_palette();

    text_editor::Style {
        background: Background::Color(color!(0, 0, 0, 0.0)),
        border: Border {
            color: color!(0, 0, 0, 0.0),
            ..Border::default()
        },
        placeholder: color!(0, 0, 0, 0.0),
        value: palette.background.base.text,
        selection: palette.primary.weak.color,
    }
}

/// Get the style of the container that stig contents will be displayed in.
fn stig_container_initial_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.weak.text),
        background: Some(palette.background.weakest.color.into()),
        border: Border {
            color: palette.primary.base.color,
            width: 2.0,
            ..border::rounded(4)
        },
        ..container::Style::default()
    }
}

/// Get the style of the container that stig contents are being displayed in.
fn stig_container_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.weak.text),
        background: Some(palette.background.weaker.color.into()),
        border: Border {
            color: palette.primary.base.color,
            width: 2.0,
            ..border::rounded(4)
        },
        ..container::Style::default()
    }
}

/// Get the style of the container where stigs are listed for the user to choose between.
fn stig_list_container_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.weak.text),
        background: Some(palette.background.weakest.color.into()),
        border: Border {
            ..border::rounded(4)
        },
        ..container::Style::default()
    }
}

/// Get the style of the cmd prompt.
fn cmd_prompt_container_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.weak.text),
        background: Some(color!(0x111111).into()),
        border: Border {
            ..border::rounded(4)
        },
        shadow: Shadow {
            color: color!(0x000000),
            blur_radius: 8.0,
            ..Shadow::default()
        },
        ..container::Style::default()
    }
}

/// Get the style for the rick arrows in the cmd prompt.
fn right_tick_svg_style(theme: &Theme, _status: svg::Status) -> svg::Style {
    let palette = theme.extended_palette();

    svg::Style {
        color: Some(palette.background.weak.text),
    }
}

/// Get the style of a button with no style.
fn button_no_style(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(color!(0, 0, 0, 0.0).into()),
        ..button::Style::default()
    }
}
