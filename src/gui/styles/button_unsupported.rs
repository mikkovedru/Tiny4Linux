// SPDX-License-Identifier: EUPL-1.2

use crate::styles::controls::control;
use iced::Theme;
use iced::widget::button::{Status, Style};

/// Unavailable controls use muted, opaque text so their labels remain readable.
pub fn button_unsupported(theme: &Theme, _: Status) -> Style {
    control(theme, Status::Disabled)
}
