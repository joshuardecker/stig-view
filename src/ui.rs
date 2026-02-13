use iced::Element;
use iced::Length::{Fill, FillPortion};
use iced::widget::{Button, button, column, container, row, scrollable, space, text, text_editor};

use crate::app::{App, Message};

impl App {
    pub fn get_view_none_displayed(&self) -> Element<'_, Message> {
        column![
            row![
                space().width(5),
                column![
                    row![
                        button(text("File")).on_press(Message::OpenFileSelect),
                        space::horizontal(),
                        button(text("Folder")).on_press(Message::OpenFolderSelect)
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
        ]
        .into()
    }

    pub fn get_view_displayed(&self) -> Element<'_, Message> {
        let buttons_vec: Vec<Box<Button<Message>>> = self
            .list
            .iter()
            .map(|stig| {
                Box::new(
                    button(text(stig.version.clone()).height(Fill).width(Fill).center())
                        .height(50)
                        .width(Fill)
                        .style(button::primary),
                )
            })
            .collect();

        let mut button_col = column![];

        for button in buttons_vec {
            button_col = button_col.push(*button);
        }

        if let Some(stig) = &self.displayed {
            let stig_col = column![
                text("Version"),
                text_editor(&self.content[0]).on_action(|action| Message::SelectContent(action, 0)),
                text("Introduction"),
                text_editor(&self.content[1]).on_action(|action| Message::SelectContent(action, 1)),
                text("Description"),
                text_editor(&self.content[2]).on_action(|action| Message::SelectContent(action, 2)),
                text("Check"),
                text_editor(&self.content[3]).on_action(|action| Message::SelectContent(action, 3)),
                text("Fix"),
                text_editor(&self.content[4]).on_action(|action| Message::SelectContent(action, 4)),
                text("Similar Checks"),
                text_editor(&self.content[5]).on_action(|action| Message::SelectContent(action, 5)),
            ];

            column![
                row![
                    space().width(5),
                    column![
                        row![
                            button(text("File")).on_press(Message::OpenFileSelect),
                            space::horizontal(),
                            button(text("Folder")).on_press(Message::OpenFolderSelect)
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
            ]
            .into()
        } else {
            unreachable!();
        }
    }
}
