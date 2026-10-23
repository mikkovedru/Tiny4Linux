// SPDX-License-Identifier: EUPL-1.2

use crate::preview::{CaptureError, Placement};
use crate::styles::colors::{COLOR_BACKGROUND_SECONDARY_DARK, COLOR_DANGER, COLOR_TEXT};
use crate::styles::controls::control_button as button;
use crate::{MainPanel, Message, WindowMode};
use iced::widget::{Container, Row, checkbox, column, container, image, row, scrollable, text};
use iced::{ContentFit, Element, Length};
use rust_i18n::t;

pub fn toolbar(app: &MainPanel) -> Row<'static, Message> {
    let mut controls = row![
        checkbox(t!("gui.preview.title"), app.preview.enabled())
            .on_toggle(Message::SetPreviewEnabled)
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center);
    controls = match app.preview.placement {
        Placement::Off => controls,
        Placement::Docked => {
            controls.push(button(text(t!("gui.preview.detach"))).on_press(Message::DetachPreview))
        }
        Placement::Detached(_) => controls.push(
            button(text(if app.window_mode == WindowMode::Widget {
                t!("gui.preview.dock_short")
            } else {
                t!("gui.preview.dock")
            }))
            .on_press(Message::DockPreview),
        ),
    };
    controls
}

pub fn panel(app: &MainPanel, detached: bool) -> Container<'static, Message> {
    let content: Element<'static, Message> = if let Some(error) = &app.preview.error {
        let message = match error {
            CaptureError::MissingFfmpeg => t!("gui.preview.missing_ffmpeg"),
            CaptureError::Failed(details) => t!("gui.preview.failed", details = details),
            CaptureError::TimedOut => t!("gui.preview.timed_out"),
        };
        column![
            scrollable(text(message).color(COLOR_DANGER)).height(Length::Fill),
            button(text(t!("gui.preview.retry"))).on_press(Message::RetryPreview)
        ]
        .spacing(12)
        .height(Length::Fill)
        .into()
    } else if let Some(frame) = &app.preview.frame {
        image(frame.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(ContentFit::Contain)
            .into()
    } else {
        text(t!("gui.preview.starting")).into()
    };
    let content = container(content)
        .center_x(Length::Fill)
        .center_y(if detached {
            Length::Fill
        } else {
            Length::Fixed(240.0)
        });
    container(content)
        .padding(12)
        .style(|_| container::Style {
            background: Some(COLOR_BACKGROUND_SECONDARY_DARK.into()),
            text_color: Some(COLOR_TEXT),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .width(Length::Fill)
        .height(if detached {
            Length::Fill
        } else {
            Length::Shrink
        })
}

pub fn detached_window(app: &MainPanel) -> Element<'static, Message> {
    container(column![toolbar(app), panel(app, true)].spacing(12))
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
