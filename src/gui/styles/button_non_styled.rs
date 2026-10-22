// SPDX-License-Identifier: EUPL-1.2

use crate::styles::controls::control;
use iced::Theme;
use iced::widget::button::{Status, Style};

pub fn button_non_styled(theme: &Theme, status: Status) -> Style {
    let base = control(theme, status);
    Style {
        background: if matches!(status, Status::Hovered | Status::Pressed) {
            base.background
        } else {
            None
        },
        border: iced::border::rounded(8),
        ..base
    }
}
