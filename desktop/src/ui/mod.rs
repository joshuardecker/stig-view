pub mod assets;
mod styles;

use iced::{
    Alignment::End,
    Center, Element, Fill, FillPortion, Shrink,
    widget::{
        Id, button, column, container, mouse_area, opaque, pick_list, row, rule, scrollable,
        sensor, space, stack, svg, text, text_editor, text_input, toggler, tooltip,
    },
};
use stig_view_core::{CKLStatus, Rule};

use crate::app::*;
use crate::ui::{assets::*, styles::*};

impl App {
    /// Get the view of the application.
    pub fn view(&self) -> Element<'_, Message> {
        let window_decorations = self.window_decorations();
        let content = row![self.stig_list(), space().width(15), self.displayed_stig()].into();

        let padded_content = self.padding(window_decorations, content);

        let popup = match self.popup {
            Popup::Filter => self.filter_menu(),
            Popup::Settings => self.settings_menu(),
            Popup::Save => self.save_menu(),
            _ => space().into(),
        };

        let err_notification = match self.err_notif {
            ErrNotif::Err(err_str) => self.display_error(err_str),
            _ => space().into(),
        };

        stack![padded_content, popup, err_notification].into()
    }

    /// A generic function that pads the content with window decorations
    /// and resize regions the user can click and drag to resize the window.
    /// A generic function that pads the content with window decorations
    /// and resize regions the user can click and drag to resize the window.
    fn padding<'a>(
        &self,
        window_decorations: Element<'a, Message>,
        content: Element<'a, Message>,
    ) -> Element<'a, Message>
    where
        Message: 'a,
    {
        use iced::window::Direction::{
            East, North, NorthEast, NorthWest, South, SouthEast, SouthWest, West,
        };

        // There are a few mouse areas here.
        // Without window decorations, we need to handle windoe drag and click resizing ourselves.
        // So we surround the gui on every edge with a mouse area to detect window resizing.

        container(column![
            // Top section above window decorations.
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
            // The main area of the application.
            // Surrounded on left and right by drag click resize areas.
            row![
                container(
                    mouse_area(container(space::horizontal()).width(15).height(Fill))
                        .on_press(Message::WindowDragResize(West))
                ),
                content,
                container(
                    mouse_area(container(space::horizontal()).width(15).height(Fill))
                        .on_press(Message::WindowDragResize(East))
                ),
            ],
            // Bottom section below the main content.
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
        ])
        .into()
    }

    /// Gets a column of all loaded STIGs, allowing the user to choose which one
    /// to display. Acts like a file tree.
    fn stig_list(&self) -> Element<'_, Message> {
        if self.benchmark.rules.is_empty() {
            return container(space::vertical())
                .style(background_container)
                .width(300)
                .into();
        }

        // A few buttons that allow the user to switch what value is displayed on the buttons.
        // Separate to the scrollable, should always be present.
        let header = column![
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

        let mut not_pin_col = column![];
        let mut user_pin_col = column![];
        let mut filter_pin_col = column![];
        let mut filter_user_pin_col = column![];

        // The amount of filtered STIGs.
        // Columns do not have a len() function, so I keep track here.
        // If this is greater than 0, a seperating rule will be placed between
        // filtered and non filtered STIGs.
        let mut total_filtered = 0;

        for (name, rule) in self.benchmark.rules.iter() {
            let pin_type = self.pins.get(name).unwrap_or(&Pinned::Not);

            let button = self.stig_button(pin_type.to_owned(), name, rule);

            match pin_type {
                Pinned::Not => not_pin_col = not_pin_col.push(button).push(space().height(8)),
                Pinned::ByUser => user_pin_col = user_pin_col.push(button).push(space().height(8)),
                Pinned::ByFilter => {
                    // Puts a nice strip of color on the left side of the button.
                    let button_with_accent: Element<'_, Message> = row![
                        container(space::horizontal())
                            .width(4)
                            .height(Fill)
                            .style(filter_accent),
                        button
                    ]
                    .into();

                    filter_pin_col = filter_pin_col
                        .push(button_with_accent)
                        .push(space().height(8));

                    total_filtered += 1;
                }
                Pinned::ByFilterAndUser => {
                    // Puts a nice strip of color on the left side of the button.
                    let button_with_accent: Element<'_, Message> = row![
                        container(space::horizontal())
                            .width(4)
                            .height(Fill)
                            .style(filter_accent),
                        button
                    ]
                    .into();

                    filter_user_pin_col = filter_user_pin_col
                        .push(button_with_accent)
                        .push(space().height(8));

                    total_filtered += 1
                }
            }
        }

        // Place a horizontal rule if there are any STIGs that have been filtered.
        let horizontal_rule: Element<'_, Message> = if total_filtered != 0 {
            column![rule::horizontal(2), space().height(8)].into()
        } else {
            space().into()
        };

        container(column![
            header,
            scrollable(column![
                filter_user_pin_col,
                filter_pin_col,
                horizontal_rule,
                user_pin_col,
                not_pin_col,
                space::vertical(), // Ensures this container is proper size.
            ])
            .spacing(8),
        ])
        .width(300)
        .style(background_container)
        .padding(8)
        .into()
    }

    /// Get a button the user can click to swich displayed STIGs.
    fn stig_button<'a>(&self, pin_type: Pinned, name: &str, rule: &Rule) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let bookmark_svg_handle = svg::Handle::from_memory(BOOKMARK);
        let filled_bookmark_svg_handle = svg::Handle::from_memory(FILLED_BOOKMARK);

        let check_handle = svg::Handle::from_memory(CHECKED_CIRCLE);
        let cross_handle = svg::Handle::from_memory(CROSS_CIRCLE);
        let minus_handle = svg::Handle::from_memory(MINUS_CIRCLE);

        // A visual indicator of the cki status of a STIG.
        let cki_status: Element<'_, Message> = match &rule.ckl_status {
            Some(CKLStatus::NotAFinding) => row![
                tooltip(
                    svg(check_handle.clone())
                        .width(24)
                        .height(24)
                        .style(good_svg),
                    container("Compliant.")
                        .style(background_container)
                        .padding(4),
                    tooltip::Position::Right
                ),
                space().width(5)
            ]
            .into(),
            Some(CKLStatus::Open) => row![
                tooltip(
                    svg(cross_handle.clone())
                        .width(24)
                        .height(24)
                        .style(bad_svg),
                    container("Non-Compliant.")
                        .style(background_container)
                        .padding(4),
                    tooltip::Position::Right
                ),
                space().width(5)
            ]
            .into(),
            Some(CKLStatus::NotApplicable) => row![
                tooltip(
                    svg(minus_handle.clone())
                        .width(24)
                        .height(24)
                        .style(warning_svg),
                    container("Not Applicable.")
                        .style(background_container)
                        .padding(4),
                    tooltip::Position::Right
                ),
                space().width(5)
            ]
            .into(),
            Some(CKLStatus::NotReviewed) => row![
                tooltip(
                    svg(minus_handle.clone())
                        .width(24)
                        .height(24)
                        .style(warning_svg),
                    container("Not Reviewed.")
                        .style(background_container)
                        .padding(4),
                    tooltip::Position::Right
                ),
                space().width(5)
            ]
            .into(),

            // If no status, dont add any visual element.
            None => space().into(),
        };

        // Button theme depends on whether a filter has pinned it.
        // Make the button more obvious when its contents matches a filter.
        let theme = match pin_type {
            Pinned::Not => rounded_boring_button,
            Pinned::ByUser => rounded_boring_button,
            Pinned::ByFilter => rounded_boring_button_right,
            Pinned::ByFilterAndUser => rounded_boring_button_right,
        };

        // Get the button text depending on what information the user has chosen to display
        // for button text.
        let button_text = match self.display_type {
            DisplayType::GroupId => name.to_owned(),
            DisplayType::RuleId => rule.rule_id.clone(),
            // If there is no STIG Id, fall back to Group Id since its always known.
            DisplayType::STIGId => rule.stig_id.clone().unwrap_or(name.to_owned()),
        };

        let bookmark_symbol = match pin_type {
            Pinned::Not => bookmark_svg_handle,
            Pinned::ByUser => filled_bookmark_svg_handle,
            Pinned::ByFilter => bookmark_svg_handle,
            Pinned::ByFilterAndUser => filled_bookmark_svg_handle,
        };

        button(
            column![
                row![
                    cki_status,
                    text(button_text).center(),
                    space::horizontal(),
                    button(svg(bookmark_symbol).width(32).height(32).style(colored_svg))
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
        .style(theme)
        .on_press(Message::Switch(name.to_owned()))
        .into()
    }

    /// Content of the currently selected STIG.
    fn displayed_stig(&self) -> Element<'_, Message> {
        // Get the displayed STIG.
        // If there is none, display a special screen.
        let stig_rule = match &self.displayed {
            Some(rule) => rule,
            None => return self.display_empty(),
        };

        // Content of the STIG.
        let content = column![
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
                    .on_action(|action| Message::SelectContent(action, ContentIndex::Discussion))
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
            text("CCIs").size(32),
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

        // Wrap it in a scrollable.
        let content = scrollable(content).spacing(8);

        let content = container(content)
            .center(Fill)
            .padding(8)
            .style(background_container);

        // Stack the content with a container that fades in and out.
        // This acts as animation, showing the user the STIG has changed when
        // a new STIG is selected.
        stack![
            content,
            container(space())
                .width(Fill)
                .height(Fill)
                .style(fade_overlay(1.0 - self.main_col_opacity))
        ]
        .into()
    }

    /// This gets displayed when no STIG is selected:
    /// A button prompting the user to choose a benchmark to load into the viewer.
    fn display_empty(&self) -> Element<'_, Message> {
        let file_svg_handle = svg::Handle::from_memory(FILE_ICON);

        // Load any benchmarks the user opted to save in the past.
        let cache = App::load_cache();

        // Change the displayed string based on if the cache loaded any items.
        let displayed_string = if cache.is_empty() {
            "Open a File to Get Started".to_string()
        } else {
            "Recently Saved Files".to_string()
        };

        let mut main_col = column![text(displayed_string).size(24).center(), space().height(20),]
            .align_x(Center)
            .width(400);

        // If the cache is empty, add an obvious button for the user to click that opens a new benchmark.
        if cache.is_empty() {
            main_col = main_col.push(
                button(text("Open").center())
                    .width(80)
                    .height(40)
                    .style(rounded_boring_button)
                    .on_press(Message::OpenFile),
            )
        }

        for path in cache {
            let name = match path.file_name().and_then(|os_str| os_str.to_str()) {
                Some(str) => {
                    // If this file for whatever reason isnt the type we are looking for.
                    if !str.ends_with(".msgpack.zstd") {
                        continue;
                    }

                    // Trim the file extension off, the user doesnt need to see it.
                    str.trim_end_matches(".msgpack.zstd").to_string()
                }

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
                        text(name).center(),
                    ]
                    .align_y(Center),
                )
                .width(Fill)
                .style(rounded_boring_button)
                .on_press(Message::LoadCachedBenchmark(path)),
            );

            // Space out each file entry nicely.
            main_col = main_col.push(space().height(8));
        }

        container(main_col)
            .center(Fill)
            .style(background_container)
            .into()
    }

    /// Display of the filter menu, gets stacked on top of the main application view.
    fn filter_menu(&self) -> Element<'_, Message> {
        let right_tick_svg_handle = svg::Handle::from_memory(RIGHT_TICK);
        let cross_svg_handle = svg::Handle::from_memory(CROSS);

        let id = Id::new("filter_text_input");

        container(
            sensor(opaque(stack![
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
                container(space())
                    .width(Fill)
                    .height(Fill)
                    .style(fade_overlay(1.0 - self.popup_opacity)),
            ]))
            .on_show(move |_| Message::FocusWidget(id.clone())),
        )
        .center(Fill)
        .into()
    }

    /// Display of the settings menu, gets stacked on top of the main application view.
    fn settings_menu(&self) -> Element<'_, Message> {
        let cross_svg_handle = svg::Handle::from_memory(CROSS);

        let themes = [
            AppTheme::Dark,
            AppTheme::Light,
            AppTheme::HighContrast,
            AppTheme::Coffee,
        ];
        let display_types = [
            DisplayType::GroupId,
            DisplayType::RuleId,
            DisplayType::STIGId,
        ];

        container(opaque(stack![
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
                    space().height(5),
                    row![
                        text("Animations"),
                        space::horizontal(),
                        toggler(self.settings.animate)
                            .on_toggle(Message::SaveAnimate)
                            .style(toggler_theme),
                    ]
                    .align_y(Center),
                ]
                .align_x(Center),
            )
            .width(375)
            .height(200)
            .padding(15)
            .style(cmd_container),
            container(space())
                .width(Fill)
                .height(Fill)
                .style(fade_overlay(1.0 - self.popup_opacity)),
        ]))
        .center(Fill)
        .into()
    }

    /// Display of an error that occured, gets stacked on top of the main application view.
    fn display_error<'a>(&self, err_str: &'a str) -> Element<'a, Message> {
        let cross_svg_handle = svg::Handle::from_memory(CROSS);

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
        .into()
    }

    /// A menu prompting the user to save the benchmark to the cache.
    fn save_menu(&self) -> Element<'_, Message> {
        let cross_svg_handle = svg::Handle::from_memory(CROSS);

        container(opaque(stack![
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
            container(space())
                .width(Fill)
                .height(Fill)
                .style(fade_overlay(1.0 - self.popup_opacity)),
        ]))
        .center(Fill)
        .into()
    }

    /// Return the window decorations container.
    fn window_decorations(&self) -> Element<'_, Message> {
        let settings_svg_handle = svg::Handle::from_memory(SETTINGS);
        let cross_svg_handle = svg::Handle::from_memory(CROSS);
        let square_svg_handle = svg::Handle::from_memory(SQUARE);
        let down_tick_svg_handle = svg::Handle::from_memory(DOWN_TICK);
        let switch_svg_handle = svg::Handle::from_memory(SWITCH);

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
                                .style(background_container)
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
                            container("Sort Content Based on Keywords (Ctrl + F)")
                                .style(background_container),
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
                            container("Returns to the Home Menu").style(background_container),
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
                                        container("Switch Benchmark").style(background_container),
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
        .into()
    }
}
