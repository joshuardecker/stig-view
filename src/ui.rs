use iced::Alignment::Center;
use iced::Length::{Fill, FillPortion, Shrink};
use iced::alignment::Horizontal::Left;
use iced::widget::{
    Button, Container, Id, button, column, container, row, scrollable, sensor, space, stack, svg,
    text, text_editor, text_input,
};
use iced::{Background, Border, Shadow, border};
use iced::{Element, Theme, color};

use crate::app::{App, Message, Popup};
use crate::sgroup::{Pinned, SGroup, StigWrapper};

impl App {
    pub fn get_view_none_displayed(&self) -> Element<'_, Message> {
        let assets_dir = std::env::current_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            + "/assets/images/";

        let final_gui = column![
            row![
                space().width(5),
                column![
                    row![
                        button(svg(assets_dir.clone() + "file.svg"))
                            .padding(1)
                            .width(40)
                            .on_press(Message::OpenFileSelect),
                        space::horizontal(),
                        button(svg(assets_dir.clone() + "folder.svg"))
                            .padding(1)
                            .width(40)
                            .on_press(Message::OpenFolderSelect),
                        space::horizontal(),
                        button(svg(assets_dir + "terminal.svg"))
                            .padding(1)
                            .width(40)
                            .on_press(Message::OpenCmdInput),
                    ],
                    space().height(5),
                    container(space::vertical())
                        .style(stig_list_container_style)
                        .width(FillPortion(1))
                ],
                space().width(10),
                container(space::vertical())
                    .style(stig_container_initial_style)
                    .width(FillPortion(5))
                    .height(Fill),
            ],
            space().height(5),
        ];

        if let Some(popup) = &self.popup {
            match popup {
                Popup::CommandPrompt => stack!(final_gui, self.command_prompt_popup()).into(),
                Popup::Error => stack!(final_gui, self.error_popup()).into(),
            }
        } else {
            final_gui.into()
        }
    }

    pub fn get_view_displayed(&self) -> Element<'_, Message> {
        let assets_dir = std::env::current_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            + "/assets/images/";

        let buttons_vec: Vec<Box<Button<Message>>> = self
            .list
            .read()
            .unwrap()
            .get_all()
            .iter()
            .map(|stig_wrapper| {
                let icon_path: String;

                match stig_wrapper.pinned {
                    Pinned::Not => icon_path = assets_dir.clone() + "lightbulb.svg",
                    Pinned::ByUser => icon_path = assets_dir.clone() + "lightbulb-filled.svg",
                    Pinned::ByCmd => icon_path = assets_dir.clone() + "terminal.svg",
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
                            button(svg(icon_path).height(32))
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
                row![
                    space().width(5),
                    column![
                        row![
                            button(svg(assets_dir.clone() + "file.svg"))
                                .padding(1)
                                .width(40)
                                .on_press(Message::OpenFileSelect),
                            space::horizontal(),
                            button(svg(assets_dir.clone() + "folder.svg"))
                                .padding(1)
                                .width(40)
                                .on_press(Message::OpenFolderSelect),
                            space::horizontal(),
                            button(svg(assets_dir + "terminal.svg"))
                                .padding(1)
                                .width(40)
                                .on_press(Message::OpenCmdInput),
                        ],
                        space().height(5),
                        container(column![
                            scrollable(button_col).spacing(5),
                            space::vertical()
                        ])
                        .style(stig_list_container_style)
                        .padding(5)
                        .width(FillPortion(1))
                    ],
                    space().width(10),
                    container(column![scrollable(stig_col).spacing(5), space::vertical()])
                        .style(stig_container_style)
                        .padding(10)
                        .width(FillPortion(5))
                        .height(Fill),
                ],
                space().height(5),
            ];

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

    fn command_prompt_popup(&self) -> Container<'_, Message> {
        let id = Id::new("cmd_text_input");

        let assets_dir = std::env::current_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            + "/assets/images/";

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
                            svg(assets_dir.clone() + "right-tick.svg")
                                .style(right_tick_svg_style)
                                .width(Shrink),
                            space().width(10),
                            text("(name|title) to filter by title keywords."),
                        ]
                        .height(24),
                        row![
                            svg(assets_dir.clone() + "right-tick.svg")
                                .style(right_tick_svg_style)
                                .width(Shrink),
                            space().width(10),
                            text("(search|find) to filter by any keywords."),
                        ]
                        .height(24),
                        row![
                            svg(assets_dir + "right-tick.svg")
                                .style(right_tick_svg_style)
                                .width(Shrink),
                            space().width(10),
                            text("(reset) to undo applied filters."),
                        ]
                        .height(24),
                        space::vertical()
                    ]
                    .align_x(Left),
                )
                .width(500)
                .height(Shrink)
                .padding(15)
                .style(cmd_prompt_container_style),
            )
            .on_show(move |_| Message::FocusCmdInput(id.clone())),
        )
        .center(Fill)
    }

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
            blur_radius: 4.0,
            ..Shadow::default()
        },
        ..container::Style::default()
    }
}

fn right_tick_svg_style(theme: &Theme, _status: svg::Status) -> svg::Style {
    let palette = theme.extended_palette();

    svg::Style {
        color: Some(palette.background.weak.text),
    }
}

fn button_no_style(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(color!(0, 0, 0, 0.0).into()),
        ..button::Style::default()
    }
}
