// SPDX-License-Identifier: EUPL-1.2

use crate::Message;
use crate::styles::colors::*;
use iced::widget::button::{Status, Style};
use iced::widget::{Button, TextInput, button, text_input};
use iced::{Border, Element, Theme};

pub fn control_button<'a>(content: impl Into<Element<'a, Message>>) -> Button<'a, Message> {
    button(content).padding([8, 12]).style(control)
}

pub fn control(_: &Theme, status: Status) -> Style {
    let (background, text_color, border_color) = match status {
        Status::Active => (COLOR_CONTROL, COLOR_TEXT, COLOR_BORDER),
        Status::Hovered => (COLOR_CONTROL_HOVER, COLOR_TEXT, COLOR_ACCENT),
        Status::Pressed => (COLOR_BACKGROUND_SECONDARY_DARK, COLOR_TEXT, COLOR_ACCENT),
        Status::Disabled => (
            COLOR_BACKGROUND_SECONDARY_DARK,
            COLOR_TEXT_DISABLED,
            COLOR_CONTROL_HOVER,
        ),
    };
    Style {
        background: Some(background.into()),
        text_color,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Style::default()
    }
}

pub fn selected(theme: &Theme, status: Status) -> Style {
    if status == Status::Disabled {
        return control(theme, status);
    }
    Style {
        background: Some(match status {
            Status::Hovered => COLOR_SELECTED_HOVER.into(),
            _ => COLOR_SELECTED.into(),
        }),
        text_color: COLOR_TEXT,
        border: Border {
            color: COLOR_ACCENT,
            width: 2.0,
            radius: 6.0.into(),
        },
        ..Style::default()
    }
}

pub fn input<'a>(placeholder: &str, value: &str) -> TextInput<'a, Message> {
    text_input(placeholder, value)
        .padding([8, 10])
        .style(input_style)
}

fn input_style(_: &Theme, status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: COLOR_BACKGROUND_DARK.into(),
        border: Border {
            color: match status {
                text_input::Status::Focused | text_input::Status::Hovered => COLOR_ACCENT,
                _ => COLOR_BORDER,
            },
            width: if status == text_input::Status::Focused {
                2.0
            } else {
                1.0
            },
            radius: 6.0.into(),
        },
        icon: COLOR_TEXT_MUTED,
        placeholder: COLOR_TEXT_MUTED,
        value: if status == text_input::Status::Disabled {
            COLOR_TEXT_DISABLED
        } else {
            COLOR_TEXT
        },
        selection: COLOR_SELECTED,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::styles::button_non_styled::button_non_styled;
    use crate::styles::button_unsupported::button_unsupported;
    use crate::styles::theme::obsbot_theme;
    use iced::{Background, Color};

    fn luminance(color: Color) -> f32 {
        let linear = |channel: f32| {
            if channel <= 0.04045 {
                channel / 12.92
            } else {
                ((channel + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
    }

    fn readable(text: Color, background: Color) {
        assert_eq!(
            text.a, 1.0,
            "fading text must not bypass the contrast check"
        );
        let (a, b) = (luminance(text), luminance(background));
        let ratio = (a.max(b) + 0.05) / (a.min(b) + 0.05);
        assert!(ratio >= 4.5, "text contrast is only {ratio:.2}:1");
    }

    #[test]
    fn body_text_is_readable_on_both_surfaces() {
        let palette = obsbot_theme().palette();
        readable(palette.text, palette.background);
        readable(palette.text, COLOR_BACKGROUND_SECONDARY_DARK);
    }

    #[test]
    fn button_labels_remain_readable_in_every_state() {
        let theme = obsbot_theme();
        for status in [
            Status::Active,
            Status::Hovered,
            Status::Pressed,
            Status::Disabled,
        ] {
            for style in [
                control(&theme, status),
                selected(&theme, status),
                button_unsupported(&theme, status),
            ] {
                let Some(Background::Color(background)) = style.background else {
                    panic!("a control needs an opaque background");
                };
                readable(style.text_color, background);
            }
            let icon_button = button_non_styled(&theme, status);
            readable(icon_button.text_color, COLOR_BACKGROUND_SECONDARY_DARK);
        }
    }

    #[test]
    fn input_values_and_placeholders_are_readable() {
        let theme = obsbot_theme();
        for status in [
            text_input::Status::Active,
            text_input::Status::Hovered,
            text_input::Status::Focused,
            text_input::Status::Disabled,
        ] {
            let style = input_style(&theme, status);
            let Background::Color(background) = style.background else {
                panic!("an input needs an opaque background");
            };
            readable(style.value, background);
            readable(style.placeholder, background);
        }
    }
}
