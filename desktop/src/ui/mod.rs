mod styles;

use iced::Alignment::End;
use iced::Element;
use iced::Length::FillPortion;
use iced::widget::{
    Button, Column, Container, button, column, container, mouse_area, opaque, row, rule,
    scrollable, sensor, space, stack, svg, text, text_editor, text_input, tooltip,
};
use iced::widget::{Id, pick_list};
use iced::window::Direction::{
    East, North, NorthEast, NorthWest, South, SouthEast, SouthWest, West,
};
use iced::{Center, Fill, Shrink};
use stig_view_core::CKLStatus;

use crate::app::*;
use crate::ui::styles::*;

impl App {
    /// Get the application gui.
    pub fn get_view(&self) -> Element<'_, Message> {
        if let Some(stig_rule) = &self.displayed {
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

            // Overlays are applied here, outside main_gui, so Iced can diff the scrollable
            // subtree independently from the overlay container that changes every tick.
            let main_col: Element<'_, Message> = stack![
                scrollable(main_col).spacing(15),
                container(space())
                    .width(Fill)
                    .height(Fill)
                    .style(fade_overlay(1.0 - self.main_col_opacity)),
            ]
            .into();

            let list_col: Element<'_, Message> = self.get_stig_buttons().into();

            let main_gui = self.main_gui(self.window_decorations(), list_col, main_col);

            match (self.popup.clone(), self.err_notif.clone()) {
                (Popup::None, ErrNotif::None) => main_gui,
                (Popup::Filter, ErrNotif::None) => {
                    stack![main_gui, self.command_prompt_popup()].into()
                }
                (Popup::Settings, ErrNotif::None) => stack![main_gui, self.settings_menu()].into(),
                (Popup::Save, ErrNotif::None) => stack![main_gui, self.save_menu()].into(),

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
                (Popup::Save, ErrNotif::Err(err_str)) => stack![
                    main_gui,
                    self.save_menu(),
                    self.error_notification(err_str.to_owned())
                ]
                .into(),
            }
        } else {
            let file_svg_handle = svg::Handle::from_memory(self.assets.file_svg.clone());

            let list_col = column![space::vertical()];

            let cache = App::load_cache();

            // Change the displayed string based on if the cache loaded any items.
            let displayed_string = if cache.len() == 0 {
                String::from("Open a File to Get Started")
            } else {
                String::from("Recently Saved Files")
            };

            let mut main_col =
                column![text(displayed_string).size(24).center(), space().height(20),]
                    .padding(15)
                    .align_x(Center)
                    .width(400);

            // If the cache is empty, add an obvious button for the user to click that opens a new Benchmark.
            if cache.len() == 0 {
                main_col = main_col.push(
                    button(text("Open").center())
                        .width(80)
                        .height(40)
                        .style(rounded_boring_button)
                        .on_press(Message::OpenFile),
                )
            }

            for path in cache {
                let label = match path.file_name().and_then(|os_str| os_str.to_str()) {
                    Some(str) => str.trim_end_matches(".msgpack.zstd").to_string(),
                    None => continue,
                };

                main_col = main_col.push(
                    button(
                        row![
                            svg(file_svg_handle.clone())
                                .style(boring_svg)
                                .width(20)
                                .height(20),
                            space().width(15),
                            text(label).center(),
                        ]
                        .align_y(Center),
                    )
                    .width(Fill)
                    .style(rounded_boring_button)
                    .on_press(Message::LoadCachedBenchmark(path)),
                );

                main_col = main_col.push(space().height(8));
            }

            let main_gui =
                self.main_gui(self.window_decorations(), list_col.into(), main_col.into());

            match (self.popup.clone(), self.err_notif.clone()) {
                (Popup::None, ErrNotif::None) => main_gui,
                (Popup::Filter, ErrNotif::None) => {
                    stack![main_gui, self.command_prompt_popup()].into()
                }
                (Popup::Settings, ErrNotif::None) => stack![main_gui, self.settings_menu()].into(),
                (Popup::Save, ErrNotif::None) => stack![main_gui, self.save_menu()].into(),

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
                (Popup::Save, ErrNotif::Err(err_str)) => stack![
                    main_gui,
                    self.save_menu(),
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
        list_col: Element<'a, Message>,
        main_col: Element<'a, Message>,
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
                container(main_col)
                    .style(background_container)
                    .center(Fill)
                    .padding(8),
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
            sensor(opaque(
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
            ))
            .on_show(move |_| Message::FocusWidget(id.clone())),
        )
        .center(Fill)
    }

    fn settings_menu(&self) -> Container<'_, Message> {
        let cross_svg_handle = svg::Handle::from_memory(self.assets.cross_svg.clone());

        let themes = [AppTheme::Dark, AppTheme::Light];
        let display_types = [
            DisplayType::GroupId,
            DisplayType::RuleId,
            DisplayType::STIGId,
        ];

        container(opaque(
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
                    .align_y(Center),
                    space().height(5),
                    row![
                        text("Default Display Type"),
                        space::horizontal(),
                        pick_list(
                            display_types,
                            Some(self.settings.default_display_type),
                            Message::SaveDisplayType
                        ),
                    ]
                    .align_y(Center),
                ]
                .align_x(Center),
            )
            .width(375)
            .height(150)
            .padding(15)
            .style(cmd_container),
        ))
        .center(Fill)
    }

    fn save_menu(&self) -> Container<'_, Message> {
        let cross_svg_handle = svg::Handle::from_memory(self.assets.cross_svg.clone());

        container(opaque(
            container(
                column![
                    row![
                        space::horizontal(),
                        space().width(13),
                        text("Save Benchmark for Later?"),
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
                    space::vertical(),
                    row![
                        space::horizontal(),
                        button(text("Cancel").size(14).center())
                            .style(rounded_danger_button)
                            .width(65)
                            .height(30)
                            .on_press(Message::SwitchPopup(Popup::None)),
                        space().width(60),
                        button(text("Confirm").size(14).center())
                            .style(rounded_success_button)
                            .width(70)
                            .height(30)
                            .on_press(Message::SaveBenchmark),
                        space::horizontal()
                    ]
                    .align_y(Center),
                    space().height(15),
                ]
                .align_x(Center),
            )
            .width(375)
            .height(150)
            .padding(15)
            .style(cmd_container),
        ))
        .center(Fill)
    }

    fn error_notification(&self, err_str: String) -> Container<'_, Message> {
        let cross_svg_handle = svg::Handle::from_memory(self.assets.cross_svg.clone());

        container(opaque(
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
        ))
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
        let switch_svg_handle = svg::Handle::from_memory(self.assets.switch_svg.clone());

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
                            container("Open a New File (Ctrl + I)")
                                .style(tooltip_container)
                                .padding(4),
                            tooltip::Position::Right
                        )
                        .delay(iced::time::Duration::from_millis(600)),
                        space().width(4),
                        tooltip(
                            button(text("Filter").center().size(14))
                                .padding(6)
                                .width(Shrink)
                                .height(Shrink)
                                .style(rounded_dark_button)
                                .on_press(Message::SwitchPopup(Popup::Filter)),
                            container("Sort Content Based on a Filter (Ctrl + P)")
                                .style(tooltip_container),
                            tooltip::Position::Right
                        )
                        .delay(iced::time::Duration::from_millis(600)),
                        space().width(4),
                        tooltip(
                            button(text("Home").center().size(14))
                                .padding(6)
                                .width(Shrink)
                                .height(Shrink)
                                .style(rounded_dark_button)
                                .on_press(Message::ReturnHome),
                            container("Returns to the Home Menu").style(tooltip_container),
                            tooltip::Position::Right
                        )
                        .delay(iced::time::Duration::from_millis(600)),
                        space::horizontal(),
                        text(&self.benchmark.id).size(14),
                        {
                            let switch_element: Element<Message> = if self.benchmarks.len() != 0 {
                                row![
                                    space().width(15),
                                    tooltip(
                                        button(
                                            svg(switch_svg_handle)
                                                .style(colored_svg)
                                                .width(20)
                                                .height(20)
                                        )
                                        .padding(1)
                                        .width(Shrink)
                                        .height(Shrink)
                                        .style(no_button)
                                        .on_press(Message::SwitchToBackground),
                                        container("Switch Benchmark").style(tooltip_container),
                                        tooltip::Position::Right
                                    )
                                    .delay(iced::time::Duration::from_millis(600)),
                                ]
                                .into()
                            } else {
                                space().width(0).into()
                            };
                            switch_element
                        },
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

        let check_handle = svg::Handle::from_memory(self.assets.check_circle_svg.clone());
        let cross_handle = svg::Handle::from_memory(self.assets.cross_circle_svg.clone());
        let minus_handle = svg::Handle::from_memory(self.assets.minus_circle_svg.clone());

        for (name, rule) in self.benchmark.rules.iter() {
            let pin_type = self.pins.get(name).unwrap_or(&Pinned::Not);

            match pin_type {
                Pinned::Not => {
                    not_pin_col.push(Box::new(
                        button(
                            column![
                                row![
                                    {
                                        let cki_status: Element<'_, Message> =
                                            match &rule.ckl_status {
                                                Some(CKLStatus::NotAFinding) => row![
                                                    tooltip(
                                                        svg(check_handle.clone())
                                                            .width(20)
                                                            .height(20)
                                                            .style(good_svg),
                                                        container("Compliant.")
                                                            .style(tooltip_container)
                                                            .padding(4),
                                                        tooltip::Position::Right
                                                    ),
                                                    space().width(5)
                                                ]
                                                .into(),
                                                Some(CKLStatus::Open) => row![
                                                    tooltip(
                                                        svg(cross_handle.clone())
                                                            .width(20)
                                                            .height(20)
                                                            .style(bad_svg),
                                                        container("Non-Compliant.")
                                                            .style(tooltip_container)
                                                            .padding(4),
                                                        tooltip::Position::Right
                                                    ),
                                                    space().width(5)
                                                ]
                                                .into(),
                                                Some(CKLStatus::NotApplicable) => row![
                                                    tooltip(
                                                        svg(minus_handle.clone())
                                                            .width(20)
                                                            .height(20)
                                                            .style(warning_svg),
                                                        container("Not Applicable.")
                                                            .style(tooltip_container)
                                                            .padding(4),
                                                        tooltip::Position::Right
                                                    ),
                                                    space().width(5)
                                                ]
                                                .into(),
                                                Some(CKLStatus::NotReviewed) => row![
                                                    tooltip(
                                                        svg(minus_handle.clone())
                                                            .width(20)
                                                            .height(20)
                                                            .style(warning_svg),
                                                        container("Not Reviewed.")
                                                            .style(tooltip_container)
                                                            .padding(4),
                                                        tooltip::Position::Right
                                                    ),
                                                    space().width(5)
                                                ]
                                                .into(),
                                                None => space().width(0).into(),
                                            };
                                        cki_status
                                    },
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
                                    {
                                        let cki_status: Element<'_, Message> =
                                            match &rule.ckl_status {
                                                Some(CKLStatus::NotAFinding) => row![
                                                    tooltip(
                                                        svg(check_handle.clone())
                                                            .width(20)
                                                            .height(20)
                                                            .style(good_svg),
                                                        container("Compliant.")
                                                            .style(tooltip_container)
                                                            .padding(4),
                                                        tooltip::Position::Right
                                                    ),
                                                    space().width(5)
                                                ]
                                                .into(),
                                                Some(CKLStatus::Open) => row![
                                                    tooltip(
                                                        svg(cross_handle.clone())
                                                            .width(20)
                                                            .height(20)
                                                            .style(bad_svg),
                                                        container("Non-Compliant.")
                                                            .style(tooltip_container)
                                                            .padding(4),
                                                        tooltip::Position::Right
                                                    ),
                                                    space().width(5)
                                                ]
                                                .into(),
                                                Some(CKLStatus::NotApplicable) => row![
                                                    tooltip(
                                                        svg(minus_handle.clone())
                                                            .width(20)
                                                            .height(20)
                                                            .style(warning_svg),
                                                        container("Not Applicable.")
                                                            .style(tooltip_container)
                                                            .padding(4),
                                                        tooltip::Position::Right
                                                    ),
                                                    space().width(5)
                                                ]
                                                .into(),
                                                Some(CKLStatus::NotReviewed) => row![
                                                    tooltip(
                                                        svg(minus_handle.clone())
                                                            .width(20)
                                                            .height(20)
                                                            .style(warning_svg),
                                                        container("Not Reviewed.")
                                                            .style(tooltip_container)
                                                            .padding(4),
                                                        tooltip::Position::Right
                                                    ),
                                                    space().width(5)
                                                ]
                                                .into(),
                                                None => space().width(0).into(),
                                            };
                                        cki_status
                                    },
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
                                    {
                                        let cki_status: Element<'_, Message> =
                                            match &rule.ckl_status {
                                                Some(CKLStatus::NotAFinding) => row![
                                                    tooltip(
                                                        svg(check_handle.clone())
                                                            .width(20)
                                                            .height(20)
                                                            .style(good_svg),
                                                        container("Compliant.")
                                                            .style(tooltip_container)
                                                            .padding(4),
                                                        tooltip::Position::Right
                                                    ),
                                                    space().width(5)
                                                ]
                                                .into(),
                                                Some(CKLStatus::Open) => row![
                                                    tooltip(
                                                        svg(cross_handle.clone())
                                                            .width(20)
                                                            .height(20)
                                                            .style(bad_svg),
                                                        container("Non-Compliant.")
                                                            .style(tooltip_container)
                                                            .padding(4),
                                                        tooltip::Position::Right
                                                    ),
                                                    space().width(5)
                                                ]
                                                .into(),
                                                Some(CKLStatus::NotApplicable) => row![
                                                    tooltip(
                                                        svg(minus_handle.clone())
                                                            .width(20)
                                                            .height(20)
                                                            .style(warning_svg),
                                                        container("Not Applicable.")
                                                            .style(tooltip_container)
                                                            .padding(4),
                                                        tooltip::Position::Right
                                                    ),
                                                    space().width(5)
                                                ]
                                                .into(),
                                                Some(CKLStatus::NotReviewed) => row![
                                                    tooltip(
                                                        svg(minus_handle.clone())
                                                            .width(20)
                                                            .height(20)
                                                            .style(warning_svg),
                                                        container("Not Reviewed.")
                                                            .style(tooltip_container)
                                                            .padding(4),
                                                        tooltip::Position::Right
                                                    ),
                                                    space().width(5)
                                                ]
                                                .into(),
                                                None => space().width(0).into(),
                                            };
                                        cki_status
                                    },
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
                                    {
                                        let cki_status: Element<'_, Message> =
                                            match &rule.ckl_status {
                                                Some(CKLStatus::NotAFinding) => row![
                                                    tooltip(
                                                        svg(check_handle.clone())
                                                            .width(20)
                                                            .height(20)
                                                            .style(good_svg),
                                                        container("Compliant.")
                                                            .style(tooltip_container)
                                                            .padding(4),
                                                        tooltip::Position::Right
                                                    ),
                                                    space().width(5)
                                                ]
                                                .into(),
                                                Some(CKLStatus::Open) => row![
                                                    tooltip(
                                                        svg(cross_handle.clone())
                                                            .width(20)
                                                            .height(20)
                                                            .style(bad_svg),
                                                        container("Non-Compliant.")
                                                            .style(tooltip_container)
                                                            .padding(4),
                                                        tooltip::Position::Right
                                                    ),
                                                    space().width(5)
                                                ]
                                                .into(),
                                                Some(CKLStatus::NotApplicable) => row![
                                                    tooltip(
                                                        svg(minus_handle.clone())
                                                            .width(20)
                                                            .height(20)
                                                            .style(warning_svg),
                                                        container("Not Applicable.")
                                                            .style(tooltip_container)
                                                            .padding(4),
                                                        tooltip::Position::Right
                                                    ),
                                                    space().width(5)
                                                ]
                                                .into(),
                                                Some(CKLStatus::NotReviewed) => row![
                                                    tooltip(
                                                        svg(minus_handle.clone())
                                                            .width(20)
                                                            .height(20)
                                                            .style(warning_svg),
                                                        container("Not Reviewed.")
                                                            .style(tooltip_container)
                                                            .padding(4),
                                                        tooltip::Position::Right
                                                    ),
                                                    space().width(5)
                                                ]
                                                .into(),
                                                None => space().width(0).into(),
                                            };
                                        cki_status
                                    },
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
                space().width(8),
                button(text("Rule ID").size(12).center())
                    .on_press(Message::SwitchDisplayType(DisplayType::RuleId))
                    .style(rounded_boring_button)
                    .width(FillPortion(1)),
                space().width(8),
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

        let mut scrollable_col = column![].spacing(8).align_x(Center);

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

        col = col.push(scrollable(scrollable_col).spacing(8));
        col = col.push(space::vertical());

        col
    }
}
