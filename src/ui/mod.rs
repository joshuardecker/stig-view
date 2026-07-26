#[allow(unused)]
mod assets;
mod styles;
mod themes;

// Re-exports.
pub use assets::*;
pub use themes::*;

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::Duration,
};

use disa_stig::CKLStatus;
use iced::{
    Alignment::End,
    Center, Element, Fill, FillPortion, Shrink,
    widget::{
        Id, button, column, container, lazy, mouse_area, opaque, pick_list, row, rule, scrollable,
        sensor, space, stack, svg, text, text_input, toggler, tooltip,
    },
};

use crate::{
    app::{App, AppTheme, DisplayType, Message, Pinned, Popup},
    widgets::{markdown, selectable_text},
};

/// The default seperation between elements.
/// I use magic values around because they look better.
const SEPERATION: f32 = 8.0;

impl App {
    /// Get the view of the application.
    pub fn view(&self) -> Element<'_, Message> {
        let window_decorations = self.window_decorations();
        let content = row![
            self.stig_list(),
            space().width(SEPERATION * 2.0),
            self.displayed_stig()
        ]
        .into();

        let padded_content = self.padding(window_decorations, content);

        let popup = match self.popup {
            Some(Popup::Filter) => self.filter_menu(),
            Some(Popup::Settings) => self.settings_menu(),
            Some(Popup::Save) => self.save_menu(),
            None => space().into(),
        };

        /*
        let err_notification = if let Some(error) = self.error_msgs.last() {
            self.display_error(error)
        } else {
            space().into()
        };*/

        let error_details = if self.display_errors {
            self.error_menu()
        } else {
            space().into()
        };

        stack![padded_content, popup, error_details].into()
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
        use iced::mouse::Interaction;
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
                    mouse_area(
                        container(space::horizontal())
                            .width(SEPERATION)
                            .height(SEPERATION)
                    )
                    .on_press(Message::WindowDragResize(NorthWest))
                    .interaction(Interaction::ResizingDiagonallyDown)
                ),
                container(
                    mouse_area(
                        container(space::horizontal())
                            .width(Fill)
                            .height(SEPERATION)
                    )
                    .on_press(Message::WindowDragResize(North))
                    .interaction(Interaction::ResizingVertically)
                ),
                container(
                    mouse_area(
                        container(space::horizontal())
                            .width(SEPERATION)
                            .height(SEPERATION)
                    )
                    .on_press(Message::WindowDragResize(NorthEast))
                    .interaction(Interaction::ResizingDiagonallyUp)
                ),
            ],
            window_decorations,
            space().height(SEPERATION),
            // The main area of the application.
            // Surrounded on left and right by drag click resize areas.
            row![
                container(
                    mouse_area(
                        container(space::horizontal())
                            .width(SEPERATION * 2.0)
                            .height(Fill)
                    )
                    .on_press(Message::WindowDragResize(West))
                    .interaction(Interaction::ResizingHorizontally)
                ),
                content,
                container(
                    mouse_area(
                        container(space::horizontal())
                            .width(SEPERATION * 2.0)
                            .height(Fill)
                    )
                    .on_press(Message::WindowDragResize(East))
                    .interaction(Interaction::ResizingHorizontally)
                ),
            ],
            // Bottom section below the main content.
            row![
                container(
                    mouse_area(
                        container(space::horizontal())
                            .width(SEPERATION * 2.0)
                            .height(SEPERATION * 2.0)
                    )
                    .on_press(Message::WindowDragResize(SouthWest))
                    .interaction(Interaction::ResizingDiagonallyUp)
                ),
                container(
                    mouse_area(
                        container(space::horizontal())
                            .width(Fill)
                            .height(SEPERATION * 2.0)
                    )
                    .on_press(Message::WindowDragResize(South))
                    .interaction(Interaction::ResizingVertically)
                ),
                container(
                    mouse_area(
                        container(space::horizontal())
                            .width(SEPERATION * 2.0)
                            .height(SEPERATION * 2.0)
                    )
                    .on_press(Message::WindowDragResize(SouthEast))
                    .interaction(Interaction::ResizingDiagonallyDown)
                ),
            ],
        ])
        .into()
    }

    /// Gets a column of all loaded STIGs, allowing the user to choose which one
    /// to display. Acts like a file tree.
    fn stig_list(&self) -> Element<'_, Message> {
        let Some(benchmark) = &self.benchmark else {
            return container(space::vertical())
                .style(styles::background_container)
                .width(300)
                .into();
        };

        let mut hasher = DefaultHasher::new();

        for (_id, rule) in benchmark.rules.iter() {
            rule.rule_id.hash(&mut hasher);
        }

        for (_id, pinned) in self.pins.iter() {
            pinned.hash(&mut hasher);
        }

        // Make sure to change the hash if the user wants to display based on
        // a different display type.
        self.display_type.hash(&mut hasher);

        // The hash is determined by which rules are currently displayed,
        // and whether their order has been changed because something has been pinned.
        // Only rebuild the widget tree when this changes.
        let hash = hasher.finish();

        lazy(hash, |_| {
            // A few buttons that allow the user to switch what value is displayed on the buttons.
            // Separate to the scrollable, should always be present.
            let header = column![
                row![
                    button(text("Group ID").size(12).center())
                        .on_press(Message::SwitchDisplayType(DisplayType::GroupId))
                        .style(styles::rounded_primary_button)
                        .width(FillPortion(1)),
                    space().width(SEPERATION),
                    button(text("Rule ID").size(12).center())
                        .on_press(Message::SwitchDisplayType(DisplayType::RuleId))
                        .style(styles::rounded_primary_button)
                        .width(FillPortion(1)),
                    space().width(SEPERATION),
                    button(text("STIG ID").size(12).center())
                        .on_press(Message::SwitchDisplayType(DisplayType::STIGId))
                        .style(styles::rounded_primary_button)
                        .width(FillPortion(1)),
                ],
                space().height(SEPERATION)
            ]
            .align_x(Center);

            let mut not_pin_col = column![];
            let mut user_pin_col = column![];
            let mut filter_pin_col = column![];
            let mut filter_user_pin_col = column![];

            // Counters used to keep track of the total number of compliant,
            // noncompliant, and manual review recommendations.
            let mut compliant_counter = 0;
            let mut manual_counter = 0;
            let mut noncompliant_counter = 0;

            // A counter to remember how many rules are only pinned by the user.
            let mut pinned_only_by_user = 0;

            // The amount of filtered STIGs.
            // Columns do not have a len() function, so I keep track here.
            // If this is greater than 0, a seperating rule will be placed between
            // filtered and non filtered STIGs.
            let mut total_filtered = 0;

            for (name, rule) in benchmark.rules.iter() {
                match &rule.ckl_status {
                    Some(CKLStatus::NotAFinding) => compliant_counter += 1,
                    Some(CKLStatus::Open) => noncompliant_counter += 1,
                    Some(CKLStatus::NotApplicable) => compliant_counter += 1,
                    Some(CKLStatus::NotReviewed) => manual_counter += 1,
                    None => (),
                }

                let pin_type = self.pins.get(name).unwrap_or(&Pinned::Not);

                let button = self.stig_button(
                    pin_type.to_owned(),
                    name.to_owned(),
                    rule.ckl_status.clone(),
                    rule.rule_id.clone(),
                    rule.stig_id.clone(),
                );

                match pin_type {
                    Pinned::Not => not_pin_col = not_pin_col.push(button).push(space().height(8)),
                    Pinned::ByUser => {
                        user_pin_col = user_pin_col.push(button).push(space().height(8));

                        pinned_only_by_user += 1;
                    }
                    Pinned::ByFilter => {
                        // Puts a nice strip of color on the left side of the button.
                        let button_with_accent: Element<'_, Message> = row![
                            container(space::horizontal())
                                .width(SEPERATION * 0.5)
                                .height(Fill)
                                .style(styles::filter_accent),
                            button
                        ]
                        .into();

                        filter_pin_col = filter_pin_col
                            .push(button_with_accent)
                            .push(space().height(SEPERATION));

                        total_filtered += 1;
                    }
                    Pinned::ByFilterAndUser => {
                        // Puts a nice strip of color on the left side of the button.
                        let button_with_accent: Element<'_, Message> = row![
                            container(space::horizontal())
                                .width(SEPERATION * 0.5)
                                .height(Fill)
                                .style(styles::filter_accent),
                            button
                        ]
                        .into();

                        filter_user_pin_col = filter_user_pin_col
                            .push(button_with_accent)
                            .push(space().height(SEPERATION));

                        total_filtered += 1
                    }
                }
            }

            // Counters visually displays how many of each ckl status is present.
            // If a non ckl was loaded, this will not be displayed.
            let counters: Element<'_, Message> =
                if (compliant_counter + manual_counter + noncompliant_counter) != 0 {
                    column![
                        rule::horizontal(2),
                        space().height(SEPERATION),
                        row![
                            tooltip(
                                svg(SQUARE_FILLED.clone())
                                    .width(12)
                                    .height(12)
                                    .style(styles::good_svg),
                                container("Total Compliant")
                                    .style(styles::background_container)
                                    .padding(4),
                                tooltip::Position::Bottom
                            ),
                            space().width(SEPERATION * 0.5),
                            text(compliant_counter.to_string()),
                            space().width(SEPERATION * 2.0),
                            tooltip(
                                svg(SQUARE_FILLED.clone())
                                    .width(12)
                                    .height(12)
                                    .style(styles::bad_svg),
                                container("Total Non-Compliant")
                                    .style(styles::background_container)
                                    .padding(4),
                                tooltip::Position::Bottom
                            ),
                            space().width(SEPERATION * 0.5),
                            text(noncompliant_counter.to_string()),
                            space().width(SEPERATION * 2.0),
                            tooltip(
                                svg(SQUARE_FILLED.clone())
                                    .width(12)
                                    .height(12)
                                    .style(styles::warning_svg),
                                container("Total Manual Review")
                                    .style(styles::background_container)
                                    .padding(4),
                                tooltip::Position::Bottom
                            ),
                            space().width(SEPERATION * 0.5),
                            text(manual_counter.to_string()),
                        ]
                        .align_y(Center),
                        space().height(SEPERATION),
                    ]
                    .align_x(Center)
                    .into()
                } else {
                    space().into()
                };

            // Place a horizontal rule if there are any STIGs that have been pinned by the user.
            let user_pinned_horizontal_rule: Element<'_, Message> = if pinned_only_by_user != 0 {
                column![
                    space().height(SEPERATION),
                    rule::horizontal(2),
                    space().height(SEPERATION * 2.0)
                ]
                .into()
            } else {
                space().into()
            };

            // Place a horizontal rule if there are any STIGs that have been filtered.
            let filter_horizontal_rule: Element<'_, Message> = if total_filtered != 0 {
                column![
                    space().height(SEPERATION),
                    rule::horizontal(2),
                    space().height(SEPERATION * 2.0)
                ]
                .into()
            } else {
                space().into()
            };

            container(column![
                header,
                counters,
                scrollable(column![
                    filter_user_pin_col,
                    filter_pin_col,
                    filter_horizontal_rule,
                    user_pin_col,
                    user_pinned_horizontal_rule,
                    not_pin_col,
                    space::vertical(), // Ensures this container is proper size.
                ])
                .spacing(SEPERATION),
            ])
            .width(300)
            .style(styles::background_container)
            .padding(8)
        })
        .into()
    }

    /// Get a button the user can click to swich displayed STIGs.
    fn stig_button(
        &self,
        pin_type: Pinned,
        name: String,
        ckl_status: Option<CKLStatus>,
        rule_id: String,
        stig_id: Option<String>,
    ) -> Element<'static, Message> {
        // A visual indicator of the cki status of a STIG.
        let cki_status: Element<'_, Message> = match &ckl_status {
            Some(CKLStatus::NotAFinding) => row![
                tooltip(
                    svg(CHECKED_CIRCLE.clone())
                        .width(18)
                        .height(18)
                        .style(styles::good_svg),
                    container("Compliant.")
                        .style(styles::background_container)
                        .padding(4),
                    tooltip::Position::Right
                ),
                space().width(SEPERATION)
            ]
            .into(),
            Some(CKLStatus::Open) => row![
                tooltip(
                    svg(CROSS_CIRCLE.clone())
                        .width(18)
                        .height(18)
                        .style(styles::bad_svg),
                    container("Non-Compliant.")
                        .style(styles::background_container)
                        .padding(4),
                    tooltip::Position::Right
                ),
                space().width(SEPERATION)
            ]
            .into(),
            Some(CKLStatus::NotApplicable) => row![
                tooltip(
                    svg(CHECKED_CIRCLE.clone())
                        .width(18)
                        .height(18)
                        .style(styles::good_svg),
                    container("Not Applicable.")
                        .style(styles::background_container)
                        .padding(4),
                    tooltip::Position::Right
                ),
                space().width(SEPERATION)
            ]
            .into(),
            Some(CKLStatus::NotReviewed) => row![
                tooltip(
                    svg(MINUS_CIRCLE.clone())
                        .width(18)
                        .height(18)
                        .style(styles::warning_svg),
                    container("Not Reviewed.")
                        .style(styles::background_container)
                        .padding(4),
                    tooltip::Position::Right
                ),
                space().width(SEPERATION)
            ]
            .into(),

            // If no status, dont add any visual element.
            None => space().into(),
        };

        // Button theme depends on whether a filter has pinned it.
        // Make the button more obvious when its contents matches a filter.
        let theme = match pin_type {
            Pinned::Not => styles::rounded_boring_button,
            Pinned::ByUser => styles::rounded_boring_button,
            Pinned::ByFilter => styles::rounded_boring_button_right,
            Pinned::ByFilterAndUser => styles::rounded_boring_button_right,
        };

        // Get the button text depending on what information the user has chosen to display
        // for button text.
        let button_text = match self.display_type {
            DisplayType::GroupId => name.clone(),
            DisplayType::RuleId => rule_id,
            // If there is no STIG Id, fall back to Group Id since its always known.
            DisplayType::STIGId => stig_id.unwrap_or(name.clone()),
        };

        let bookmark_symbol = match pin_type {
            Pinned::Not => BOOKMARK.clone(),
            Pinned::ByUser => BOOKMARK_FILLED.clone(),
            Pinned::ByFilter => BOOKMARK.clone(),
            Pinned::ByFilterAndUser => BOOKMARK_FILLED.clone(),
        };

        button(
            column![
                row![
                    cki_status,
                    text(button_text).center(),
                    space::horizontal(),
                    button(
                        svg(bookmark_symbol)
                            .width(22)
                            .height(22)
                            .style(styles::colored_svg)
                    )
                    .padding(1)
                    .style(styles::no_button)
                    .on_press(Message::Pin(name.clone()))
                ]
                .align_y(Center)
                .height(Fill),
            ]
            .align_x(Center)
            .width(Fill),
        )
        .height(SEPERATION * 8.0)
        .padding(8)
        .width(Fill)
        .style(theme)
        .on_press(Message::SwitchRule(name))
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

        let content = column![
            row![
                column![
                    text("Group ID").size(18),
                    space().height(SEPERATION),
                    selectable_text(stig_rule.group_id.clone()).highlight_str(
                        self.filter_text_field.trim(),
                        |theme| theme.extended_palette().primary.weak.color
                    ),
                    space().height(SEPERATION),
                    rule::horizontal(2),
                    space().height(SEPERATION),
                    text("Severity").size(18),
                    space().height(SEPERATION),
                    selectable_text(format!("{}", stig_rule.severity)).highlight_str(
                        self.filter_text_field.trim(),
                        |theme| theme.extended_palette().primary.weak.color
                    ),
                ]
                .align_x(Center)
                .width(FillPortion(1)),
                space().width(SEPERATION),
                rule::vertical(2),
                space().width(SEPERATION),
                column![
                    text("Rule ID").size(18),
                    space().height(SEPERATION),
                    selectable_text(stig_rule.rule_id.clone()).highlight_str(
                        self.filter_text_field.trim(),
                        |theme| theme.extended_palette().primary.weak.color
                    ),
                    space().height(SEPERATION),
                    rule::horizontal(2),
                    space().height(SEPERATION),
                ]
                .align_x(Center)
                .width(FillPortion(1)),
                space().width(SEPERATION),
                rule::vertical(2),
                space().width(SEPERATION),
                column![
                    text("STIG ID").size(18),
                    space().height(SEPERATION),
                    selectable_text(stig_rule.stig_id.clone().unwrap_or("None".into()))
                        .highlight_str(self.filter_text_field.trim(), |theme| theme
                            .extended_palette()
                            .primary
                            .weak
                            .color),
                    space().height(SEPERATION),
                    rule::horizontal(2),
                    space().height(SEPERATION),
                    text("Documentable").size(18),
                    space().height(SEPERATION),
                    selectable_text(match stig_rule.documentable {
                        Some(true) => "True",
                        _ => "False,",
                    })
                    .highlight_str(
                        self.filter_text_field.trim(),
                        |theme| theme.extended_palette().primary.weak.color
                    ),
                ]
                .align_x(Center)
                .width(FillPortion(1)),
            ],
            space().height(SEPERATION),
            row![
                space().width(SEPERATION),
                markdown::view_selectable(
                    markdown::parse(&format!(
                        "# Introduction\n{}\n# Description\n{}\n# Check\n{}\n# Fix\n{}\n# CCIs\n{}\n# False Positives\n{}\n# False Negatives\n{}",
                        stig_rule.title.clone(),
                        stig_rule.vuln_discussion.clone(),
                        stig_rule.check_text.clone(),
                        stig_rule.fix_text.clone(),
                        stig_rule
                            .cci_refs
                            .clone()
                            .map(|strings| strings.join("\n"))
                            .unwrap_or_default(),
                        stig_rule.false_positives.clone().unwrap_or("".into()),
                        stig_rule.false_negatives.clone().unwrap_or("".into()),
                    )),
                    markdown::Settings::from(self.theme()),
                )
                .highlight_str(self.filter_text_field.trim(), |theme| {
                    theme.extended_palette().primary.weak.color
                })
            ],
        ];

        // Wrap it in a scrollable.
        let content = scrollable(content).spacing(SEPERATION);

        let content = container(content)
            .center(Fill)
            .padding(8)
            .style(styles::background_container);

        // Stack the content with a container that fades in and out.
        // This acts as animation, showing the user the STIG has changed when
        // a new STIG is selected.
        container(stack![
            content,
            container(space())
                .width(Fill)
                .height(Fill)
                .style(|theme| styles::fade_overlay(
                    theme,
                    self.animations.get_opacity("main_col")
                ))
        ])
        .width(Fill)
        .height(Fill)
        .into()
    }

    /// This gets displayed when no STIG is selected:
    /// A button prompting the user to choose a benchmark to load into the viewer.
    fn display_empty(&self) -> Element<'_, Message> {
        use std::path::PathBuf;

        // Load any benchmarks the user opted to save in the past.
        let cache = App::load_cache();

        // Change the displayed string based on if the cache loaded any items.
        let displayed_string = if cache.is_empty() {
            "Open a File to Get Started"
        } else {
            "Recently Saved Files"
        };

        let mut main_col = column![];

        // If the cache is empty, add an obvious button for the user to click that opens a new benchmark.
        if cache.is_empty() {
            main_col = main_col.push(
                button(text("Open").center())
                    .width(SEPERATION * 10.0)
                    .height(SEPERATION * 5.0)
                    .style(styles::rounded_boring_button)
                    .on_press(Message::OpenFile),
            )
        }

        // A vector that contains the unix time when this cache entry was last opened,
        // its path, and its nicely formatted name.
        let mut times_last_loaded: Vec<(u64, PathBuf, String)> = Vec::new();

        for path in cache {
            match path.file_name().and_then(|os_str| os_str.to_str()) {
                Some(str) => {
                    // If this file for whatever reason isnt the type we are looking for.
                    if !str.ends_with(".json.zstd") {
                        continue;
                    }

                    let str = str.trim_end_matches(".json.zstd");

                    // Get the last time this benchmark was accessed.
                    let time_last = self.last_opened.get_time_used(str);

                    // Trim the file extension off, and make the name a little prettier.
                    let name: String = str
                        .chars()
                        .flat_map(|c| match c {
                            '_' | '-' => ' '.to_lowercase(),
                            c => c.to_lowercase(),
                        })
                        .collect();

                    // Save the time last accessed, path, and formatted name.
                    times_last_loaded.push((time_last, path, name));
                }

                None => continue,
            };
        }

        // Sort most recent to oldest.
        times_last_loaded.sort_by(|a, b| b.0.cmp(&a.0));

        for time_loaded in times_last_loaded {
            main_col = main_col.push(
                button(
                    row![
                        svg(FILE.clone())
                            .style(styles::boring_svg)
                            .width(20)
                            .height(20),
                        space().width(SEPERATION),
                        text(time_loaded.2).center(),
                        space::horizontal(),
                        button(
                            svg(TRASH.clone())
                                .style(styles::colored_svg)
                                .width(20)
                                .height(20)
                        )
                        .style(styles::no_button)
                        .on_press(Message::DeleteCachedBenchmark(time_loaded.1.clone())),
                    ]
                    .align_y(Center),
                )
                .width(Fill)
                .style(styles::rounded_boring_button)
                .on_press(Message::LoadCachedBenchmark(time_loaded.1)),
            );

            // Space out each file entry nicely.
            main_col = main_col.push(space().height(SEPERATION));
        }

        container(
            column![
                text(displayed_string).size(24).center(),
                space().height(SEPERATION * 3.0),
                scrollable(main_col).spacing(SEPERATION)
            ]
            .align_x(Center)
            .width(400),
        )
        .padding(30)
        .center(Fill)
        .style(styles::background_container)
        .into()
    }

    /// Display of the filter menu, gets stacked on top of the main application view.
    fn filter_menu(&self) -> Element<'_, Message> {
        let id = Id::new("filter_text_input");

        // The filter popup itself.
        let popup: Element<'_, Message> =
            container(
                sensor(opaque(stack![
                    container(
                        row![
                            space().width(SEPERATION / 2.0),
                            button(
                                svg(REFRESH.clone())
                                    .style(styles::colored_svg)
                                    .width(18)
                                    .height(18)
                            )
                            .padding(1)
                            .width(Shrink)
                            .height(Shrink)
                            .style(styles::no_button)
                            .on_press(Message::ResetFilter),
                            space().width(SEPERATION),
                            text_input(
                                "Type keywords here, then press enter...",
                                &self.filter_text_field
                            )
                            .on_input(Message::TypeFilter)
                            .id(id.clone())
                            .width(320)
                            .style(styles::transparent_text_input),
                            space().width(SEPERATION),
                            button(
                                svg(CROSS.clone())
                                    .style(styles::colored_svg)
                                    .width(16)
                                    .height(16)
                            )
                            .padding(1)
                            .width(Shrink)
                            .height(Shrink)
                            .style(styles::no_button)
                            .on_press(Message::SwitchPopup(None)),
                            space().width(SEPERATION / 2.0),
                        ]
                        .align_y(Center),
                    )
                    //.width(400)
                    //.height(Shrink)
                    .padding(SEPERATION)
                    .style(styles::cmd_container),
                    container(space()).width(Fill).height(Fill).style(
                        |theme| styles::fade_overlay(theme, self.animations.get_opacity("popup"))
                    ),
                ]))
                .on_show(move |_| Message::FocusWidget(id.clone())),
            )
            .width(Fill)
            .height(Fill)
            .align_x(Center)
            .align_y(End)
            .into();

        // Add some space below it, that way it is not hugging the bottom of the window.
        // Looks nicer this way.
        column![popup, space().height(SEPERATION * 4.0)].into()
    }

    /// Display of the settings menu, gets stacked on top of the main application view.
    fn settings_menu(&self) -> Element<'_, Message> {
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
                        text("Settings Menu"),
                        space::horizontal(),
                        button(
                            svg(CROSS.clone())
                                .style(styles::colored_svg)
                                .width(16)
                                .height(16)
                        )
                        .padding(1)
                        .width(Shrink)
                        .height(Shrink)
                        .style(styles::no_button)
                        .on_press(Message::SwitchPopup(None)),
                    ]
                    .align_y(Center),
                    space().height(SEPERATION * 4.0),
                    row![
                        text("Theme"),
                        space::horizontal(),
                        pick_list(themes, Some(self.settings.theme), Message::SwitchTheme),
                    ]
                    .align_y(Center),
                    space().height(SEPERATION),
                    row![
                        text("Default Display Type"),
                        space::horizontal(),
                        pick_list(
                            display_types,
                            Some(self.settings.default_display_type),
                            Message::SwitchDisplayType
                        ),
                    ]
                    .align_y(Center),
                    space().height(SEPERATION),
                    row![
                        text("Animations"),
                        space::horizontal(),
                        toggler(self.settings.animate)
                            .on_toggle(Message::SaveAnimate)
                            .style(styles::toggler_theme),
                    ]
                    .align_y(Center),
                    space().height(SEPERATION),
                    row![
                        text("Notify About Updates"),
                        space::horizontal(),
                        toggler(self.settings.notify_if_update)
                            .on_toggle(Message::SaveUpdateNotify)
                            .style(styles::toggler_theme),
                    ]
                    .align_y(Center),
                ]
                .align_x(Center),
            )
            .width(375)
            .height(200)
            .padding(8)
            .style(styles::cmd_container),
            container(space())
                .width(Fill)
                .height(Fill)
                .style(|theme| styles::fade_overlay(theme, self.animations.get_opacity("popup"))),
        ]))
        .center(Fill)
        .into()
    }

    /// If its even shown, the error notification button.
    fn error_button(&self) -> Element<'_, Message> {
        if self.error_msgs.is_empty() {
            return space().into();
        }

        tooltip(
            button(
                row![
                    text(self.error_msgs.len())
                        .size(14)
                        .style(text::warning)
                        .center(),
                    space().width(SEPERATION / 4.0),
                    svg(EXCLAMATION_CIRCLE.clone())
                        .width(16)
                        .height(16)
                        .style(styles::warning_svg)
                ]
                .align_y(Center),
            )
            .style(styles::rounded_dark_button)
            .padding(4)
            .on_press(Message::ShowErrors(true)),
            container("Errors have occured")
                .style(styles::background_container)
                .padding(4),
            tooltip::Position::Bottom,
        )
        .delay(Duration::from_millis(600))
        .into()
    }

    /// A fancier error drop down menu.
    fn error_menu(&self) -> Element<'_, Message> {
        let title = match self.error_msgs.len() {
            0 => "No Errors Here".to_string(),
            1 => "Error Occured".to_string(),
            _ => {
                format!(
                    "Error Occured {}/{}",
                    self.error_index + 1,
                    self.error_msgs.len()
                )
            }
        };

        let error_string = match self.error_msgs.get(self.error_index) {
            Some(error_string) => error_string.clone(),
            None => "No more errors to view.".to_string(),
        };

        let previous_error_button: Element<'_, Message> = if self.error_index != 0 {
            button(
                svg(ARROW_LEFT.clone())
                    .style(styles::colored_svg)
                    .width(18)
                    .height(18),
            )
            .padding(1)
            .style(styles::no_button)
            .on_press(Message::ShowPreviousError)
            .into()
        } else {
            space().into()
        };

        let next_error_button: Element<'_, Message> =
            if (self.error_index + 1) < self.error_msgs.len() {
                button(
                    svg(ARROW_RIGHT.clone())
                        .style(styles::colored_svg)
                        .width(18)
                        .height(18),
                )
                .padding(1)
                .style(styles::no_button)
                .on_press(Message::ShowNextError)
                .into()
            } else {
                space().into()
            };

        let clear_error_button: Element<'_, Message> = if !self.error_msgs.is_empty() {
            tooltip(
                button(
                    svg(CHECK.clone())
                        .style(styles::colored_svg)
                        .width(16)
                        .height(16),
                )
                .padding(1)
                .style(styles::no_button)
                .on_press(Message::ClearErrorNotification),
                container("Clear this error from error history")
                    .style(styles::background_container)
                    .padding(4),
                tooltip::Position::Bottom,
            )
            .delay(Duration::from_millis(600))
            .into()
        } else {
            space().into()
        };

        container(opaque(stack![
            container(
                column![
                    row![
                        space::horizontal(),
                        text(title).center(),
                        space::horizontal(),
                        button(
                            svg(CROSS.clone())
                                .style(styles::colored_svg)
                                .width(14)
                                .height(14)
                        )
                        .padding(1)
                        .style(styles::no_button)
                        .on_press(Message::ShowErrors(false))
                    ]
                    .align_y(Center),
                    space().height(SEPERATION),
                    row![text(error_string)],
                    space().height(SEPERATION),
                    row![
                        previous_error_button,
                        space::horizontal(),
                        next_error_button,
                        space().width(SEPERATION * 2.0),
                        clear_error_button,
                    ]
                    .align_y(Center),
                ]
                .width(Fill)
                .align_x(Center)
            )
            .width(250)
            .style(styles::cmd_container)
            .padding(SEPERATION),
            container(space())
                .width(Fill)
                .height(Fill)
                .style(|theme| styles::fade_overlay(
                    theme,
                    self.animations.get_opacity("error_menu")
                ))
        ]))
        .align_top(Fill)
        .align_right(Fill)
        .padding(SEPERATION * 7.0)
        .into()
    }

    /// Display to the user that an update is available.
    fn display_update_available(&self) -> Element<'_, Message> {
        if !self.update_available {
            return space().into();
        }

        tooltip(
            button(
                svg(DOWNLOAD_ARROW.clone())
                    .style(styles::colored_svg)
                    .width(18)
                    .height(18),
            )
            .padding(1)
            .width(Shrink)
            .height(Shrink)
            .style(styles::no_button)
            .on_press(Message::OpenURL(
                "https://github.com/joshuardecker/xylok-view/releases",
            )),
            container("Update Available")
                .style(styles::background_container)
                .padding(4),
            tooltip::Position::Bottom,
        )
        .delay(Duration::from_millis(600))
        .into()
    }

    /// A menu prompting the user to save the benchmark to the cache.
    fn save_menu(&self) -> Element<'_, Message> {
        container(opaque(stack![
            container(
                column![
                    row![
                        space::horizontal(),
                        text("Save Benchmark for Later?"),
                        space::horizontal(),
                        button(
                            svg(CROSS.clone())
                                .style(styles::colored_svg)
                                .width(16)
                                .height(16)
                        )
                        .padding(1)
                        .width(Shrink)
                        .height(Shrink)
                        .style(styles::no_button)
                        .on_press(Message::SwitchPopup(None)),
                    ]
                    .align_y(Center),
                    space::vertical(),
                    row![
                        space::horizontal(),
                        button(text("Cancel").size(14).center())
                            .style(styles::rounded_danger_button)
                            .width(65)
                            .height(30)
                            .on_press(Message::SwitchPopup(None)),
                        space().width(SEPERATION * 8.0),
                        button(text("Confirm").size(14).center())
                            .style(styles::rounded_success_button)
                            .width(70)
                            .height(30)
                            .on_press(Message::SaveAllBenchmarks),
                        space::horizontal()
                    ]
                    .align_y(Center),
                ]
                .align_x(Center),
            )
            .width(270)
            .height(120)
            .padding(8)
            .style(styles::cmd_container),
            container(space())
                .width(Fill)
                .height(Fill)
                .style(|theme| styles::fade_overlay(theme, self.animations.get_opacity("popup"))),
        ]))
        .center(Fill)
        .into()
    }

    /// Get the benchmark name nicely formatted.
    fn benchmark_name(&self) -> Element<'_, Message> {
        let Some(benchmark) = &self.benchmark else {
            return space().into();
        };

        let fancy_name = benchmark.id.trim().replace(['-', '_'], " ");

        text(fancy_name).size(15).center().into()
    }

    fn switch_benchmark_button(&self) -> Element<'_, Message> {
        if self.benchmark.is_none() {
            return space().into();
        }

        if self.background_benchmarks.is_empty() {
            return space().into();
        }

        tooltip(
            button(
                svg(SWITCH.clone())
                    .style(styles::colored_svg)
                    .width(18)
                    .height(18),
            )
            .padding(1)
            .style(styles::no_button)
            .on_press(Message::SwitchToNextBackground),
            container("Switch Benchmark")
                .style(styles::background_container)
                .padding(4),
            tooltip::Position::Bottom,
        )
        .delay(Duration::from_millis(600))
        .into()
    }

    /// Return the window decorations container.
    fn window_decorations(&self) -> Element<'_, Message> {
        // A complicated way of getting mouse_area to work.
        // Captures mouse input in the window decorations so the window can be dragged.
        container(
            mouse_area(
                container(
                    row![
                        space().width(SEPERATION * 2.0),
                        tooltip(
                            button(
                                svg(SETTINGS.clone())
                                    .style(styles::colored_svg)
                                    .width(18)
                                    .height(18)
                            )
                            .padding(1)
                            .style(styles::no_button)
                            .on_press(Message::SwitchPopup(Some(Popup::Settings))),
                            container("Customize Settings")
                                .style(styles::background_container)
                                .padding(4),
                            tooltip::Position::Right
                        )
                        .delay(Duration::from_millis(600)),
                        space().width(SEPERATION),
                        tooltip(
                            button(
                                svg(HOME.clone())
                                    .style(styles::colored_svg)
                                    .width(18)
                                    .height(18)
                            )
                            .padding(1)
                            .style(styles::no_button)
                            .on_press(Message::ReturnHome),
                            container("Return to the Start Screen")
                                .style(styles::background_container)
                                .padding(4),
                            tooltip::Position::Right
                        )
                        .delay(Duration::from_millis(600)),
                        space().width(SEPERATION),
                        tooltip(
                            button(text("Open").center().size(15))
                                .padding(4)
                                .style(styles::rounded_dark_button)
                                .on_press(Message::OpenFile),
                            container("Open a New File (Ctrl + O)")
                                .style(styles::background_container)
                                .padding(4),
                            tooltip::Position::Right
                        )
                        .delay(Duration::from_millis(600)),
                        space().width(SEPERATION / 2.0),
                        tooltip(
                            button(text("Find").center().size(15))
                                .padding(4)
                                .style(styles::rounded_dark_button)
                                .on_press(Message::SwitchPopup(Some(Popup::Filter))),
                            container("Find Content Based on Keywords (Ctrl + F)")
                                .style(styles::background_container)
                                .padding(4),
                            tooltip::Position::Right
                        )
                        .delay(Duration::from_millis(600)),
                        space::horizontal(),
                        self.switch_benchmark_button(),
                        space().width(SEPERATION * 3.0),
                        self.benchmark_name(),
                        space::horizontal(),
                        self.error_button(),
                        space().width(SEPERATION * 3.0),
                        self.display_update_available(),
                        space().width(SEPERATION * 3.0),
                        button(
                            svg(DOWN_TICK.clone())
                                .style(styles::colored_svg)
                                .width(24)
                                .height(24)
                        )
                        .padding(1)
                        .style(styles::no_button)
                        .on_press(Message::WindowMinimize),
                        space().width(SEPERATION),
                        button(
                            svg(SQUARE.clone())
                                .style(styles::colored_svg)
                                .width(16)
                                .height(16)
                        )
                        .padding(1)
                        .style(styles::no_button)
                        .on_press(Message::WindowFullscreenToggle),
                        space().width(SEPERATION + 4.0),
                        button(
                            svg(CROSS.clone())
                                .style(styles::colored_svg)
                                .width(16)
                                .height(16)
                        )
                        .padding(1)
                        .style(styles::no_button)
                        .on_press(Message::WindowClose),
                        space().width(SEPERATION * 2.0),
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
