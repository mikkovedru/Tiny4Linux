// SPDX-License-Identifier: EUPL-1.2

use crate::Message;
use crate::styles::colors::{COLOR_BORDER, COLOR_CONTROL, COLOR_TEXT};
use iced::Border;
use iced::widget::{Container, container};

pub fn tooltip_content(content: Container<Message>) -> Container<Message> {
    content.padding(10).style(|_| tooltip_style())
}

fn tooltip_style() -> container::Style {
    container::Style {
        text_color: Some(COLOR_TEXT),
        background: Some(COLOR_CONTROL.into()),
        border: Border {
            color: COLOR_BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}
