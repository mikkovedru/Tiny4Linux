// SPDX-License-Identifier: EUPL-1.2

use crate::Message;
use crate::styles::button_unsupported::button_unsupported;
use crate::styles::controls::control as secondary;
use crate::styles::controls::control_button as button;
use crate::styles::tooltip_style::tooltip_content;
use iced::widget::tooltip::Position;
use iced::widget::{Container, container, text, tooltip};
use rust_i18n::t;
use tiny4linux::ExposureMode;

/// An exposure mode disabled for modes the connected camera does not take.
pub fn button_exposure_mode(mode: ExposureMode, supported: bool) -> Container<'static, Message> {
    container(tooltip(
        button(text(format!("{}", mode)))
            .on_press_maybe(supported.then_some(Message::ChangeExposure(mode)))
            .style(if supported {
                secondary
            } else {
                button_unsupported
            }),
        tooltip_content(container(text(t!(
            "gui.tooltips.changes_exposure",
            mode = mode
        )))),
        Position::Bottom,
    ))
}
