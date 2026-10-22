// SPDX-License-Identifier: EUPL-1.2

use crate::styles::colors::*;
use iced::Theme;
use iced::theme::Palette;

pub fn obsbot_theme() -> Theme {
    Theme::custom(
        "Tiny4Linux Slate".to_string(),
        Palette {
            primary: COLOR_ACCENT,
            background: COLOR_BACKGROUND_DARK,
            text: COLOR_TEXT,
            success: COLOR_SUCCESS,
            danger: COLOR_DANGER,
        },
    )
}
