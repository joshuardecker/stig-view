use iced::Length::{Fill, FillPortion};
use iced::widget::{
    Button, Container, Id, button, column, container, row, scrollable, sensor, space, stack, text,
    text_editor, text_input,
};
use iced::{Background, Border};
use iced::{Element, Theme, color};

use crate::app::{App, Message, Popup};

impl App {
    pub fn get_view_none_displayed(&self) -> Element<'_, Message> {
        let final_gui = column![
            row![
                space().width(5),
                column![
                    row![
                        button(text("File").center())
                            .width(65)
                            .on_press(Message::OpenFileSelect),
                        space::horizontal(),
                        button(text("Folder").center())
                            .width(65)
                            .on_press(Message::OpenFolderSelect)
                    ],
                    space().height(5),
                    container(space::vertical())
                        .style(container::rounded_box)
                        .width(FillPortion(1))
                ],
                space().width(10),
                container(space::vertical())
                    .style(container::rounded_box)
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
        let buttons_vec: Vec<Box<Button<Message>>> = self
            .list
            .read()
            .unwrap()
            .iter()
            .map(|stig| {
                Box::new(
                    button(text(stig.version.clone()).height(Fill).width(Fill).center())
                        .height(50)
                        .width(Fill)
                        .style(button::primary)
                        .on_press(Message::SwitchDisplayed(stig.uuid.clone())),
                )
            })
            .collect();

        let mut button_col = column![];

        for button in buttons_vec {
            button_col = button_col.push(*button);
            button_col = button_col.push(space().height(1)) // Add a tiny seperation between each button.
        }

        if let Some(_stig) = &*self.displayed.read().unwrap() {
            let stig_col = column![
                text("Version"),
                text_editor(&self.content[0])
                    .style(text_editor_no_style)
                    .on_action(|action| Message::SelectContent(action, 0)),
                text("Introduction"),
                text_editor(&self.content[1])
                    .style(text_editor_no_style)
                    .on_action(|action| Message::SelectContent(action, 1)),
                text("Description"),
                text_editor(&self.content[2])
                    .style(text_editor_no_style)
                    .on_action(|action| Message::SelectContent(action, 2)),
                text("Check"),
                text_editor(&self.content[3])
                    .style(text_editor_no_style)
                    .on_action(|action| Message::SelectContent(action, 3)),
                text("Fix"),
                text_editor(&self.content[4])
                    .style(text_editor_no_style)
                    .on_action(|action| Message::SelectContent(action, 4)),
                text("Similar Checks"),
                text_editor(&self.content[5])
                    .style(text_editor_no_style)
                    .on_action(|action| Message::SelectContent(action, 5)),
            ];

            let final_gui = column![
                row![
                    space().width(5),
                    column![
                        row![
                            button(text("File").center())
                                .width(65)
                                .on_press(Message::OpenFileSelect),
                            space::horizontal(),
                            button(text("Folder").center())
                                .width(65)
                                .on_press(Message::OpenFolderSelect)
                        ],
                        space().height(5),
                        container(column![
                            scrollable(button_col).spacing(5),
                            space::vertical()
                        ])
                        .style(container::rounded_box)
                        .width(FillPortion(1))
                    ],
                    space().width(10),
                    container(column![scrollable(stig_col).spacing(5), space::vertical()])
                        .style(container::rounded_box)
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

        container(
            sensor(
                container(column![
                    text("Command Prompt:").width(Fill).center(),
                    space().height(30),
                    text_input("Type commands here...", &self.cmd_input)
                        .on_input(Message::ChangeCmdInput)
                        .on_submit(Message::SubmitCmdInput)
                        .id(id.clone())
                ])
                .width(500)
                .height(150)
                .padding(5)
                .style(container::dark),
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
