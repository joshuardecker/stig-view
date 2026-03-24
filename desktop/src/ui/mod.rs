mod styles;

use iced::Alignment::End;
use iced::Element;
use iced::Length::FillPortion;
use iced::widget::{
    Button, Column, Container, button, column, container, mouse_area, row, rule, scrollable,
    sensor, space, stack, svg, text, text_editor, text_input, tooltip,
};
use iced::widget::{Id, pick_list};
use iced::window::Direction::{
    East, North, NorthEast, NorthWest, South, SouthEast, SouthWest, West,
};
use iced::{Center, Fill, Shrink};

use crate::app::*;
use crate::ui::styles::*;

impl App {
    /// Get the application gui.
    pub fn get_view(&self) -> Element<'_, Message> {
        if let Some(stig_rule) = &self.displayed {
            let mut list_col = self.get_stig_buttons();
            list_col = list_col.push(space::vertical());

            let main_col = column![
                row![
                    column![
                        text("Group ID").size(24),
                        space().height(5),
                        text(&stig_rule.group_id),
                        space().height(7),
                        rule::horizontal(2),
                        space().height(7),
                        text("Severity").size(24),
                        space().height(5),
                        text(stig_rule.severity.clone().to_string()),
                    ]
                    .align_x(Center)
                    .width(FillPortion(1)),
                    space().width(5),
                    rule::vertical(2),
                    space().width(5),
                    column![
                        text("Rule ID").size(24),
                        space().height(5),
                        text(&stig_rule.rule_id),
                        space().height(7),
                        rule::horizontal(2),
                        space().height(7),
                    ]
                    .align_x(Center)
                    .width(FillPortion(1)),
                    space().width(5),
                    rule::vertical(2),
                    space().width(5),
                    column![
                        text("STIG ID").size(24),
                        space().height(5),
                        text(stig_rule.stig_id.clone().unwrap_or("None".to_string())),
                        space().height(7),
                        rule::horizontal(2),
                        space().height(7),
                        text("Documentable").size(24),
                        space().height(5),
                        text(stig_rule.documentable_str()),
                    ]
                    .align_x(Center)
                    .width(FillPortion(1)),
                ],
                space().height(15),
                text("Introduction").size(32),
                rule::horizontal(2),
                space().height(15),
                row![
                    space().width(10),
                    text_editor(&self.contents[ContentIndex::Title as usize])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(action, ContentIndex::Title))
                ],
                space().height(15),
                text("Description").size(32),
                rule::horizontal(2),
                space().height(15),
                row![
                    space().width(10),
                    text_editor(&self.contents[ContentIndex::Discussion as usize])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(
                            action,
                            ContentIndex::Discussion
                        ))
                ],
                space().height(15),
                text("Check").size(32),
                rule::horizontal(2),
                space().height(15),
                row![
                    space().width(10),
                    text_editor(&self.contents[ContentIndex::Check as usize])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(action, ContentIndex::Check))
                ],
                space().height(15),
                text("Fix").size(32),
                rule::horizontal(2),
                space().height(15),
                row![
                    space().width(10),
                    text_editor(&self.contents[ContentIndex::Fix as usize])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(action, ContentIndex::Fix))
                ],
                space().height(15),
                text("Similar Checks").size(32),
                rule::horizontal(2),
                space().height(15),
                row![
                    space().width(10),
                    text_editor(&self.contents[ContentIndex::CCIRefs as usize])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(action, ContentIndex::CCIRefs))
                ],
                space().height(15),
                text("False Positives").size(32),
                rule::horizontal(2),
                space().height(15),
                row![
                    space().width(10),
                    text_editor(&self.contents[ContentIndex::FalsePositives as usize])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(
                            action,
                            ContentIndex::FalsePositives
                        ))
                ],
                space().height(15),
                text("False Negatives").size(32),
                rule::horizontal(2),
                space().height(15),
                row![
                    space().width(10),
                    text_editor(&self.contents[ContentIndex::FalseNegatives as usize])
                        .style(no_text_editor)
                        .on_action(|action| Message::SelectContent(
                            action,
                            ContentIndex::FalseNegatives
                        ))
                ],
            ];

            let main_gui = self.main_gui(self.window_decorations(), list_col, main_col);

            match (self.popup.clone(), self.err_notif.clone()) {
                (Popup::None, ErrNotif::None) => main_gui,
                (Popup::Filter, ErrNotif::None) => {
                    stack![main_gui, self.command_prompt_popup()].into()
                }
                (Popup::Settings, ErrNotif::None) => stack![main_gui, self.settings_menu()].into(),

                (Popup::None, ErrNotif::Err(err_str)) => {
                    stack![main_gui, self.error_notification(err_str.to_owned())].into()
                }
                (Popup::Filter, ErrNotif::Err(err_str)) => stack![
                    main_gui,
                    self.command_prompt_popup(),
                    self.error_notification(err_str.to_owned())
                ]
                .into(),
                (Popup::Settings, ErrNotif::Err(err_str)) => stack![
                    main_gui,
                    self.settings_menu(),
                    self.error_notification(err_str.to_owned())
                ]
                .into(),
            }
        } else {
            let list_col = column![space::vertical()];
            let main_col = column![
                row![text("Open a file or folder to get started.").size(24)]
                    .align_y(Center)
                    .height(Fill),
            ]
            .align_x(Center)
            .width(Fill);

            let main_gui = self.main_gui(self.window_decorations(), list_col, main_col);

            match (self.popup.clone(), self.err_notif.clone()) {
                (Popup::None, ErrNotif::None) => main_gui,
                (Popup::Filter, ErrNotif::None) => {
                    stack![main_gui, self.command_prompt_popup()].into()
                }
                (Popup::Settings, ErrNotif::None) => stack![main_gui, self.settings_menu()].into(),

                (Popup::None, ErrNotif::Err(err_str)) => {
                    stack![main_gui, self.error_notification(err_str.to_owned())].into()
                }
                (Popup::Filter, ErrNotif::Err(err_str)) => stack![
                    main_gui,
                    self.command_prompt_popup(),
                    self.error_notification(err_str.to_owned())
                ]
                .into(),
                (Popup::Settings, ErrNotif::Err(err_str)) => stack![
                    main_gui,
                    self.settings_menu(),
                    self.error_notification(err_str.to_owned())
                ]
                .into(),
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
                    container(list_col)
                        .style(background_container)
                        .width(300)
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
                        pick_list(themes, Some(self.settings.theme), Message::SwitchTheme),
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

    fn error_notification(&self, err_str: String) -> Container<'_, Message> {
        let cross_svg_handle = svg::Handle::from_memory(self.assets.cross_svg.clone());

        container(
            container(
                column![
                    row![
                        space::horizontal(),
                        space().width(13),
                        text("Error Occurred"),
                        space::horizontal(),
                        button(svg(cross_svg_handle).style(boring_svg).width(25).height(25))
                            .padding(1)
                            .width(Shrink)
                            .height(Shrink)
                            .style(no_button)
                            .on_press(Message::ClearErrNotif),
                    ]
                    .align_y(Center),
                    space().height(10),
                    row![text(err_str).size(12).height(Fill)].align_y(Center)
                ]
                .align_x(Center),
            )
            .width(250)
            .height(100)
            .padding(15)
            .style(err_container),
        )
        .align_right(Fill)
        .align_bottom(Fill)
        .padding(30)
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
    fn get_stig_buttons(&self) -> Column<'_, Message> {
        let mut not_pin_col: Vec<Box<Button<'_, Message>>> = vec![];
        let mut user_pin_col: Vec<Box<Button<'_, Message>>> = vec![];
        let mut filter_pin_col: Vec<Box<Button<'_, Message>>> = vec![];
        let mut filter_user_pin_col: Vec<Box<Button<'_, Message>>> = vec![];

        let bookmark_svg_handle = svg::Handle::from_memory(self.assets.bookmark_svg.clone());
        let filled_bookmark_svg_handle =
            svg::Handle::from_memory(self.assets.bookmark_filled_svg.clone());

        for (name, rule) in self.benchmark.rules.iter() {
            let pin_type = self.pins.get(name).unwrap_or(&Pinned::Not);

            match pin_type {
                Pinned::Not => {
                    not_pin_col.push(Box::new(
                        button(
                            column![
                                row![
                                    text(match self.display_type {
                                        DisplayType::GroupId => name.to_owned(),
                                        DisplayType::RuleId => rule.rule_id.clone(),
                                        // If there is no STIG Id, fall back to Group Id since its always known.
                                        DisplayType::STIGId =>
                                            rule.stig_id.clone().unwrap_or(name.to_owned()),
                                    })
                                    .center(),
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
                        .on_press(Message::Switch(name.to_owned())),
                    ));
                }
                Pinned::ByUser => {
                    user_pin_col.push(Box::new(
                        button(
                            column![
                                row![
                                    text(match self.display_type {
                                        DisplayType::GroupId => name.to_owned(),
                                        DisplayType::RuleId => rule.rule_id.clone(),
                                        // If there is no STIG Id, fall back to Group Id since its always known.
                                        DisplayType::STIGId =>
                                            rule.stig_id.clone().unwrap_or(name.to_owned()),
                                    })
                                    .center(),
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
                        .on_press(Message::Switch(name.to_owned())),
                    ));
                }
                Pinned::ByFilter => {
                    filter_pin_col.push(Box::new(
                        button(
                            column![
                                row![
                                    text(match self.display_type {
                                        DisplayType::GroupId => name.to_owned(),
                                        DisplayType::RuleId => rule.rule_id.clone(),
                                        // If there is no STIG Id, fall back to Group Id since its always known.
                                        DisplayType::STIGId =>
                                            rule.stig_id.clone().unwrap_or(name.to_owned()),
                                    })
                                    .center(),
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
                        .style(rounded_boring_button_right)
                        .on_press(Message::Switch(name.to_owned())),
                    ));
                }
                Pinned::ByFilterAndUser => {
                    filter_user_pin_col.push(Box::new(
                        button(
                            column![
                                row![
                                    text(match self.display_type {
                                        DisplayType::GroupId => name.to_owned(),
                                        DisplayType::RuleId => rule.rule_id.clone(),
                                        // If there is no STIG Id, fall back to Group Id since its always known.
                                        DisplayType::STIGId =>
                                            rule.stig_id.clone().unwrap_or(name.to_owned()),
                                    })
                                    .center(),
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
                        .style(rounded_boring_button_right)
                        .on_press(Message::Switch(name.to_owned())),
                    ));
                }
            }
        }

        filter_user_pin_col.append(&mut filter_pin_col);
        user_pin_col.append(&mut not_pin_col);

        let mut col = column![
            row![
                button(text("Group ID").size(12).center())
                    .on_press(Message::SwitchDisplayType(DisplayType::GroupId))
                    .style(rounded_boring_button)
                    .width(FillPortion(1)),
                space().width(5),
                button(text("Rule ID").size(12).center())
                    .on_press(Message::SwitchDisplayType(DisplayType::RuleId))
                    .style(rounded_boring_button)
                    .width(FillPortion(1)),
                space().width(5),
                button(text("STIG ID").size(12).center())
                    .on_press(Message::SwitchDisplayType(DisplayType::STIGId))
                    .style(rounded_boring_button)
                    .width(FillPortion(1)),
            ],
            space().height(2)
        ]
        .padding(1)
        .spacing(8)
        .align_x(Center);

        let mut scrollable_col = column![].padding(1).spacing(8).align_x(Center);

        // This bool is calculated before items in the vector are consumed.
        // It is needed later.
        let has_filter = filter_user_pin_col.len() != 0;

        for button in filter_user_pin_col {
            scrollable_col = scrollable_col.push(
                row![
                    container(space::horizontal())
                        .width(4)
                        .height(Fill)
                        .style(filter_accent),
                    (*button).width(Fill),
                ]
                .height(64),
            );
        }

        // Add only if there are filtered items.
        // This rule is a visual seperator.
        if has_filter {
            scrollable_col = scrollable_col.push(rule::horizontal(2));
        }

        for button in user_pin_col {
            scrollable_col = scrollable_col.push(*button);
        }

        col = col.push(scrollable(scrollable_col).spacing(4));

        col
    }
}
