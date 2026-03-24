use iced::Shadow;
use iced::border;
use iced::border::Border;
use iced::border::Radius;
use iced::color;
use iced::theme::Theme;
use iced::widget::{button, container, svg, text_editor};

const BORDER_RAD: f32 = 8.0;

/// A rounded button in the primary color.
#[allow(dead_code)]
pub fn rounded_primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette.primary.weak.color.into()),
            text_color: palette.background.base.text,
            border: Border {
                ..border::rounded(BORDER_RAD)
            },
            shadow: Shadow {
                ..Shadow::default()
            },
            snap: false,
        },
        _ => button::Style {
            background: Some(palette.primary.base.color.into()),
            text_color: palette.background.base.text,
            border: Border {
                ..border::rounded(BORDER_RAD)
            },
            shadow: Shadow {
                ..Shadow::default()
            },
            snap: false,
        },
    }
}

/// A rounded button in a less obvious background color.
pub fn rounded_boring_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette.background.weak.color.into()),
            text_color: palette.background.base.text,
            border: Border {
                ..border::rounded(BORDER_RAD)
            },
            shadow: Shadow {
                ..Shadow::default()
            },
            snap: false,
        },
        _ => button::Style {
            background: Some(palette.background.strong.color.into()),
            text_color: palette.background.base.text,
            border: Border {
                ..border::rounded(BORDER_RAD)
            },
            shadow: Shadow {
                ..Shadow::default()
            },
            snap: false,
        },
    }
}

pub fn rounded_dark_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette.background.weakest.color.into()),
            text_color: palette.background.base.text,
            border: Border {
                ..border::rounded(BORDER_RAD)
            },
            shadow: Shadow {
                ..Shadow::default()
            },
            snap: false,
        },
        _ => button::Style {
            background: Some(color!(0, 0, 0, 0.0).into()),
            text_color: palette.primary.base.color,
            border: Border {
                ..border::rounded(BORDER_RAD)
            },
            shadow: Shadow {
                ..Shadow::default()
            },
            snap: false,
        },
    }
}

/// A button that is not visible.
pub fn no_button(theme: &Theme, _status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    button::Style {
        background: Some(color!(0, 0, 0, 0.0).into()),
        text_color: palette.background.base.text,
        border: Border {
            ..border::rounded(BORDER_RAD)
        },
        shadow: Shadow {
            ..Shadow::default()
        },
        snap: false,
    }
}

/// A button with only the right corners rounded, for use next to the filter accent strip.
pub fn rounded_boring_button_right(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette.background.weak.color.into()),
            text_color: palette.background.base.text,
            border: Border {
                radius: Radius {
                    top_left: 0.0,
                    top_right: BORDER_RAD,
                    bottom_right: BORDER_RAD,
                    bottom_left: 0.0,
                },

                ..Border::default()
            },
            shadow: Shadow::default(),
            snap: false,
        },
        _ => button::Style {
            background: Some(palette.background.strong.color.into()),
            text_color: palette.background.base.text,
            border: Border {
                radius: Radius {
                    top_left: 0.0,
                    top_right: BORDER_RAD,
                    bottom_right: BORDER_RAD,
                    bottom_left: 0.0,
                },
                ..Border::default()
            },
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

/// A thin accent strip indicating a filter match.
pub fn filter_accent(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.primary.base.color.into()),
        border: Border {
            radius: Radius {
                top_left: 0.0,
                top_right: BORDER_RAD,
                bottom_right: BORDER_RAD,
                bottom_left: 0.0,
            },
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// A svg with the primary color.
pub fn colored_svg(theme: &Theme, status: svg::Status) -> svg::Style {
    let palette = theme.extended_palette();

    match status {
        svg::Status::Hovered => svg::Style {
            color: Some(palette.background.base.text),
        },
        _ => svg::Style {
            color: Some(palette.primary.base.color),
        },
    }
}

/// A svg with the background color.
pub fn boring_svg(theme: &Theme, _status: svg::Status) -> svg::Style {
    let palette = theme.extended_palette();

    svg::Style {
        color: Some(palette.background.base.text),
    }
}

/// A svg with the success color.
pub fn good_svg(theme: &Theme, _status: svg::Status) -> svg::Style {
    let palette = theme.extended_palette();

    svg::Style {
        color: Some(palette.success.base.color),
    }
}

/// A rounded container to place elements into, lives in the backgound.
pub fn background_container(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.base.text),
        background: Some(palette.background.weakest.color.into()),
        border: Border {
            color: palette.background.base.color,
            width: 2.0,
            radius: BORDER_RAD.into(),
        },
        shadow: Shadow {
            ..Shadow::default()
        },
        snap: false,
    }
}

/// The container style tooltips will display.
pub fn tooltip_container(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.base.text),
        background: Some(palette.background.weakest.color.into()),
        border: Border {
            color: palette.background.base.color,
            width: 2.0,
            radius: BORDER_RAD.into(),
        },
        shadow: Shadow {
            ..Shadow::default()
        },
        snap: false,
    }
}

/// The container style the cmd prompt has.
pub fn cmd_container(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.base.text),
        background: Some(palette.background.strong.color.into()),
        border: Border {
            color: palette.background.weak.color,
            width: 0.0,
            radius: BORDER_RAD.into(),
        },
        shadow: Shadow {
            color: palette.background.base.color,
            offset: iced::Vector::ZERO,
            blur_radius: 8.0,
        },
        snap: false,
    }
}

/// The container style the err notification has.
pub fn err_container(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.base.text),
        background: Some(palette.danger.base.color.into()),
        border: Border {
            color: palette.danger.base.color,
            width: 0.0,
            radius: BORDER_RAD.into(),
        },
        shadow: Shadow {
            color: palette.background.base.color,
            offset: iced::Vector::ZERO,
            blur_radius: 8.0,
        },
        snap: false,
    }
}

/// Removes visible styling from a text editor.
pub fn no_text_editor(theme: &Theme, _status: text_editor::Status) -> text_editor::Style {
    let palette = theme.extended_palette();

    text_editor::Style {
        background: color!(0, 0, 0, 0.0).into(),
        border: Border {
            ..Border::default()
        },
        placeholder: color!(0, 0, 0, 0.0),
        value: palette.background.base.text,
        selection: palette.primary.weak.color,
    }
}
