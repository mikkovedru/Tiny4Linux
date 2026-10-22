// SPDX-License-Identifier: EUPL-1.2

use crate::styles::colors::{COLOR_BACKGROUND_SECONDARY_DARK, COLOR_TEXT};
use iced::widget::container;

pub fn general_area_style() -> container::Style {
    container::Style {
        text_color: Some(COLOR_TEXT),
        background: Some(COLOR_BACKGROUND_SECONDARY_DARK.into()),
        ..Default::default()
    }
}
