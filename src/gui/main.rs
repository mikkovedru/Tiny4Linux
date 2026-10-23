// SPDX-License-Identifier: EUPL-1.2

mod preview;
mod styles;
mod tracking_selection;
mod ui_modules;

use crate::styles::theme::obsbot_theme;
use crate::ui_modules::video_preview;
use crate::ui_modules::window_layout::window_layout;
use iced::widget::{Container, column, text};
use iced::window::Position;
use iced::{Element, Point};
use iced::{Length, Size, Subscription, Task, time, window};
use preview::{Capture, CaptureError, Placement, Preview};
use rust_i18n::{i18n, set_locale, t};
use std::time::{Duration, Instant};
use tiny4linux::{
    AIMode, Camera, CameraModel, CustomPreset, ExposureMode, SleepMode, Tiny2Camera, TrackingSpeed,
    get_language,
};
use tracking_selection::TrackingSelection;

i18n!("src/locales", fallback = "en");

#[derive(Debug, Clone, PartialEq)]
enum Message {
    RequestWindowModeChange(WindowMode),
    WindowCloseRequested(window::Id),
    SetPreviewEnabled(bool),
    DetachPreview,
    DockPreview,
    RetryPreview,
    PreviewTick,
    ChangeSleeping(bool),
    ChangeTracking(AIMode),
    ChangeTrackingSpeed(TrackingSpeed),
    ChangePresetPosition(i8),
    ChangeHDR(bool),
    ChangeExposure(ExposureMode),
    ChangeZoom(i32),
    ChangeZoomText(String),
    ChangePan(i32),
    ChangePanText(String),
    ChangeTilt(i32),
    ChangeTiltText(String),
    CustomPresetName(String),
    SaveCustomPreset,
    RecallCustomPreset(String),
    DeleteCustomPreset(String),
    ChangeDebugging(bool),
    TextInput(String),
    TextInput02(String),
    CheckCamera,
    CheckTracking,
    SendCommand,
    SendCommand02,
    HexDump,
    HexDump02,
}

struct MainPanel {
    camera: Option<Camera>,
    preview: Preview,
    main_window_id: Option<window::Id>,
    window_mode: WindowMode,
    awake: SleepMode,
    tracking: TrackingSelection,
    tracking_speed: TrackingSpeed,
    hdr_on: bool,
    operation_error: Option<String>,
    zoom: Option<i32>,
    zoom_min: i32,
    zoom_max: i32,
    zoom_text: String,
    pan: Option<i32>,
    pan_min: i32,
    pan_max: i32,
    pan_text: String,
    tilt: Option<i32>,
    tilt_min: i32,
    tilt_max: i32,
    tilt_text: String,
    custom_presets: Vec<CustomPreset>,
    custom_preset_name: String,
    debugging_on: bool,
    text_input: String,
    text_input_02: String,
}

impl MainPanel {
    fn init_state(window_mode: WindowMode) -> (Self, Task<Message>) {
        Self::init_with_camera(Camera::detect().ok(), window_mode)
    }

    fn init_with_camera(camera: Option<Camera>, window_mode: WindowMode) -> (Self, Task<Message>) {
        let status = camera
            .as_ref()
            .and_then(|c| c.get_status().ok())
            .unwrap_or_else(|| tiny4linux::CameraStatus::default());

        // the Tiny 4K keeps its tracking mode outside the status buffer, so it has to
        // be asked for separately to start with the right button selected (#72)
        let tracking = camera
            .as_ref()
            .and_then(|c| match c.model() {
                CameraModel::Tiny2 => Some(status.ai_mode),
                CameraModel::Tiny4K => c.get_ai_mode().ok(),
            })
            .unwrap_or(status.ai_mode);
        let zoom_range = camera.as_ref().and_then(|c| c.get_zoom_range().ok());
        let zoom = zoom_range.and_then(|_| camera.as_ref()?.get_zoom_absolute().ok());

        let pan_range = camera.as_ref().and_then(|c| c.get_pan_range().ok());
        let pan = pan_range.and_then(|_| camera.as_ref()?.get_pan_absolute().ok());

        let tilt_range = camera.as_ref().and_then(|c| c.get_tilt_range().ok());
        let tilt = tilt_range.and_then(|_| camera.as_ref()?.get_tilt_absolute().ok());

        let custom_presets = camera
            .as_ref()
            .map(|c| c.get_custom_presets())
            .unwrap_or_default();

        let (main_id, open_task) = window::open(get_window_settings_for_window_mode(window_mode));
        (
            MainPanel {
                camera,
                preview: Preview::default(),
                main_window_id: Some(main_id),
                window_mode,
                awake: status.awake,
                tracking: TrackingSelection::new(tracking),
                tracking_speed: status.speed,
                hdr_on: status.hdr_on,
                operation_error: None,
                zoom,
                zoom_min: zoom_range.map(|r| r.0).unwrap_or(0),
                zoom_max: zoom_range.map(|r| r.1).unwrap_or(0),
                zoom_text: zoom.map(|z| z.to_string()).unwrap_or_default(),
                pan,
                pan_min: pan_range.map(|r| r.0).unwrap_or(0),
                pan_max: pan_range.map(|r| r.1).unwrap_or(0),
                pan_text: pan.map(|p| p.to_string()).unwrap_or_default(),
                tilt,
                tilt_min: tilt_range.map(|r| r.0).unwrap_or(0),
                tilt_max: tilt_range.map(|r| r.1).unwrap_or(0),
                tilt_text: tilt.map(|t| t.to_string()).unwrap_or_default(),
                custom_presets,
                custom_preset_name: String::new(),
                debugging_on: false,
                text_input: String::new(),
                text_input_02: String::new(),
            },
            open_task.discard(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        // Window and preview controls must remain usable without a camera.
        match message {
            Message::RequestWindowModeChange(mode) => return self.change_window_mode(mode),
            Message::WindowCloseRequested(id) => {
                if Some(id) == self.main_window_id {
                    self.preview.stop();
                    return iced::exit();
                }
                if self.preview.placement == Placement::Detached(id) {
                    return self.dock_preview();
                }
                return Task::none();
            }
            Message::SetPreviewEnabled(enabled) => {
                if enabled && !self.preview.enabled() {
                    self.start_preview();
                    if self.window_mode == WindowMode::Widget {
                        return self.detach_preview();
                    }
                } else if !enabled {
                    return self
                        .preview
                        .stop()
                        .map(window::close)
                        .unwrap_or_else(Task::none);
                }
                return Task::none();
            }
            Message::DetachPreview => return self.detach_preview(),
            Message::DockPreview => return self.dock_preview(),
            Message::RetryPreview => {
                if self.preview.enabled() && !self.preview.capturing() {
                    self.start_preview();
                }
                return Task::none();
            }
            Message::PreviewTick => {
                self.preview.poll();
                return Task::none();
            }
            _ => {}
        }

        if message == Message::CheckTracking {
            self.refresh_tracking();
            return Task::none();
        }
        let refresh = matches!(
            message,
            Message::ChangeSleeping(_)
                | Message::ChangeTracking(_)
                | Message::ChangeTrackingSpeed(_)
                | Message::ChangePresetPosition(_)
                | Message::ChangeHDR(_)
                | Message::ChangeExposure(_)
                | Message::ChangeZoom(_)
                | Message::ChangePan(_)
                | Message::ChangeTilt(_)
                | Message::RecallCustomPreset(_)
                | Message::SaveCustomPreset
                | Message::DeleteCustomPreset(_)
        );
        if !matches!(message, Message::CheckCamera) {
            self.operation_error = None;
        }
        let task = match self.try_update(message) {
            Ok(task) => task,
            Err(error) => {
                eprintln!("{error}");
                self.operation_error = Some(error.to_string());
                Task::none()
            }
        };
        if refresh {
            self.refresh_camera_state();
        }
        task
    }

    fn refresh_camera_state(&mut self) {
        let Some(camera) = self.camera.as_ref() else {
            return;
        };
        let Ok(status) = camera.get_status() else {
            self.camera = None;
            self.awake = SleepMode::Unknown;
            self.tracking.disconnect();
            return;
        };
        self.awake = status.awake;
        let tracking = match camera.model() {
            CameraModel::Tiny2 => status.ai_mode,
            CameraModel::Tiny4K => camera.get_ai_mode().unwrap_or(AIMode::Unknown),
        };
        if self.tracking.observe(tracking, Instant::now()) {
            self.operation_error = Some(t!("shared.errors.tracking_not_confirmed").to_string());
        }
        self.tracking_speed = status.speed;
        self.hdr_on = status.hdr_on;
        if let Ok((min, max)) = camera.get_zoom_range() {
            self.zoom_min = min;
            self.zoom_max = max;
            self.zoom = camera.get_zoom_absolute().ok();
            self.zoom_text = self.zoom.map(|value| value.to_string()).unwrap_or_default();
        } else {
            self.zoom = None;
        }
        if let Ok((min, max)) = camera.get_pan_range() {
            self.pan_min = min;
            self.pan_max = max;
            self.pan = camera.get_pan_absolute().ok();
            self.pan_text = self.pan.map(|value| value.to_string()).unwrap_or_default();
        } else {
            self.pan = None;
        }
        if let Ok((min, max)) = camera.get_tilt_range() {
            self.tilt_min = min;
            self.tilt_max = max;
            self.tilt = camera.get_tilt_absolute().ok();
            self.tilt_text = self.tilt.map(|value| value.to_string()).unwrap_or_default();
        } else {
            self.tilt = None;
        }
    }

    fn refresh_tracking(&mut self) {
        let Some(camera) = self.camera.as_ref() else {
            self.tracking.disconnect();
            return;
        };
        match camera.get_ai_mode() {
            Ok(mode) => {
                if self.tracking.observe(mode, Instant::now()) {
                    self.operation_error =
                        Some(t!("shared.errors.tracking_not_confirmed").to_string());
                }
            }
            Err(error) => {
                self.operation_error = Some(error.to_string());
                self.tracking.disconnect();
                self.camera = None;
            }
        }
    }

    fn try_update(&mut self, message: Message) -> Result<Task<Message>, tiny4linux::T4lError> {
        if self.camera.is_none() {
            self.camera = Camera::detect().ok();

            if self.camera.is_none() {
                return Ok(Task::none());
            }
        } else if !self.camera.as_ref().unwrap().get_status().is_ok() {
            self.camera = None;
            self.tracking.disconnect();
            return Ok(Task::none());
        }

        let camera = self.camera.as_ref().unwrap();

        match message {
            Message::RequestWindowModeChange(_)
            | Message::WindowCloseRequested(_)
            | Message::SetPreviewEnabled(_)
            | Message::DetachPreview
            | Message::DockPreview
            | Message::RetryPreview
            | Message::PreviewTick => Ok(Task::none()),
            Message::ChangeSleeping(should_sleep) => {
                if should_sleep {
                    self.awake = SleepMode::Sleep;
                    camera.set_sleep_mode(SleepMode::Sleep)?;
                } else {
                    self.awake = SleepMode::Awake;
                    camera.set_sleep_mode(SleepMode::Awake)?;
                }

                Ok(Task::none())
            }
            Message::ChangeTracking(tracking_type) => {
                if !camera.supports_ai_mode(tracking_type) {
                    eprintln!("{}", t!("shared.errors.unsupported_tracking_mode"));
                    return Ok(Task::none());
                }

                camera.set_ai_mode(tracking_type)?;
                self.tracking.request(tracking_type, Instant::now());
                Ok(Task::none())
            }
            Message::ChangeTrackingSpeed(new_speed) => {
                if !camera.supports_tracking_speed() {
                    eprintln!("{}", t!("shared.errors.unsupported_tracking_speed"));
                    return Ok(Task::none());
                }

                self.tracking_speed = new_speed;
                camera.set_tracking_speed(new_speed)?;
                Ok(Task::none())
            }
            Message::ChangePresetPosition(new_position) => {
                self.awake = SleepMode::Awake;
                camera.set_ai_mode(AIMode::NoTracking)?;
                self.tracking.request(AIMode::NoTracking, Instant::now());
                if camera.goto_preset_position(new_position).is_err() {
                    eprintln!("{}", t!("shared.errors.unknown_preset_position"));
                }
                Ok(Task::none())
            }
            Message::ChangeHDR(new_mode) => {
                self.hdr_on = new_mode;
                camera.set_hdr_mode(new_mode)?;
                Ok(Task::none())
            }
            Message::ChangeExposure(mode) => {
                if mode == ExposureMode::Manual && !camera.supports_manual_exposure() {
                    eprintln!("{}", t!("shared.errors.unsupported_manual_exposure"));
                    return Ok(Task::none());
                }

                camera.set_exposure_mode(mode)?;
                Ok(Task::none())
            }
            Message::ChangeZoom(value) => {
                let value = value.clamp(self.zoom_min, self.zoom_max);
                self.zoom = Some(value);
                self.zoom_text = value.to_string();
                camera.set_zoom_absolute(value)?;
                Ok(Task::none())
            }
            Message::ChangeZoomText(s) => {
                self.zoom_text = s;
                Ok(Task::none())
            }
            Message::ChangePan(value) => {
                let value = value.clamp(self.pan_min, self.pan_max);
                self.pan = Some(value);
                self.pan_text = value.to_string();
                camera.set_pan_absolute(value)?;
                Ok(Task::none())
            }
            Message::ChangePanText(s) => {
                self.pan_text = s;
                Ok(Task::none())
            }
            Message::ChangeTilt(value) => {
                let value = value.clamp(self.tilt_min, self.tilt_max);
                self.tilt = Some(value);
                self.tilt_text = value.to_string();
                camera.set_tilt_absolute(value)?;
                Ok(Task::none())
            }
            Message::ChangeTiltText(s) => {
                self.tilt_text = s;
                Ok(Task::none())
            }
            Message::CustomPresetName(s) => {
                self.custom_preset_name = s;
                Ok(Task::none())
            }
            Message::SaveCustomPreset => {
                let name = self.custom_preset_name.trim();
                if !name.is_empty() {
                    camera.save_custom_preset(name)?;
                    self.custom_presets = camera.get_custom_presets();
                    self.custom_preset_name.clear();
                }
                Ok(Task::none())
            }
            Message::RecallCustomPreset(name) => {
                camera.recall_custom_preset(&name)?;
                Ok(Task::none())
            }
            Message::DeleteCustomPreset(name) => {
                camera.delete_custom_preset(&name)?;
                self.custom_presets = camera.get_custom_presets();
                Ok(Task::none())
            }
            Message::ChangeDebugging(new_mode) => {
                self.debugging_on = new_mode;
                let mutable_camera = self.camera.as_mut().unwrap();
                mutable_camera.set_debugging(new_mode);
                Ok(Task::none())
            }
            Message::TextInput(s) => {
                self.text_input = s;
                Ok(Task::none())
            }
            Message::TextInput02(s) => {
                self.text_input_02 = s;
                Ok(Task::none())
            }
            Message::SendCommand => {
                report_debug_result(
                    hex::decode(&self.text_input)
                        .ok()
                        .map(|c| camera.send_cmd(0x2, 0x6, &c).is_ok()),
                );
                Ok(Task::none())
            }
            Message::SendCommand02 => {
                report_debug_result(
                    hex::decode(&self.text_input_02)
                        .ok()
                        .map(|c| camera.send_cmd(0x2, 0x2, &c).is_ok()),
                );
                Ok(Task::none())
            }
            Message::HexDump => {
                report_debug_result(Some(camera.dump().is_ok()));
                Ok(Task::none())
            }
            Message::HexDump02 => {
                // the Tiny 4K stalls a cold GET_CUR on selector 2, which used to take
                // the whole window down through unwrap (#72)
                report_debug_result(Some(camera.dump_02().is_ok()));
                Ok(Task::none())
            }
            Message::CheckTracking => Ok(Task::none()),
            Message::CheckCamera => {
                self.refresh_camera_state();
                Ok(Task::none())
            }
        }
    }

    fn view(&'_ self, id: window::Id) -> Element<'_, Message> {
        if self.preview.placement == Placement::Detached(id) {
            return video_preview::detached_window(self);
        }
        if self.camera.is_some() {
            let content = column![get_current_ui_elements(self)].height(Length::Fill);
            if let Some(error) = &self.operation_error {
                content.push(text(error.clone())).into()
            } else {
                content.into()
            }
        } else {
            let mut content = column![video_preview::toolbar(self)].spacing(16);
            if self.preview.placement == Placement::Docked {
                content = content.push(video_preview::panel(self, false));
            }
            content
                .push(text(t!("shared.errors.no_camera")).size(20))
                .padding(16)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let camera = if self.camera.is_none() {
            time::every(Duration::from_secs(4)).map(|_| Message::CheckCamera)
        } else if self.tracking.is_pending() {
            time::every(Duration::from_millis(100)).map(|_| Message::CheckTracking)
        } else {
            time::every(Duration::from_secs(20)).map(|_| Message::CheckCamera)
        };
        let preview = if self.preview.capturing() {
            time::every(Duration::from_millis(66)).map(|_| Message::PreviewTick)
        } else {
            Subscription::none()
        };
        Subscription::batch([
            camera,
            preview,
            window::close_requests().map(Message::WindowCloseRequested),
        ])
    }

    fn title(&self, id: window::Id) -> String {
        if self.preview.placement == Placement::Detached(id) {
            format!("Tiny4Linux — {}", t!("gui.preview.title"))
        } else {
            "Tiny4Linux".into()
        }
    }

    fn start_preview(&mut self) {
        let result = self
            .camera
            .as_ref()
            .ok_or_else(|| CaptureError::Failed(t!("shared.errors.no_camera").to_string()))
            .and_then(|camera| {
                let path = camera
                    .video_device_path()
                    .map_err(|error| CaptureError::Failed(error.to_string()))?;
                Capture::start(&path, camera.model())
            });
        self.preview.start(result);
    }

    fn detach_preview(&mut self) -> Task<Message> {
        if self.preview.placement != Placement::Docked {
            return Task::none();
        }
        let (id, task) = window::open(preview_window_settings());
        self.preview.detach(id);
        task.discard()
    }

    fn dock_preview(&mut self) -> Task<Message> {
        let Some(id) = self.preview.dock() else {
            return Task::none();
        };
        Task::batch([
            window::close(id),
            self.change_window_mode(WindowMode::Dashboard),
        ])
    }

    fn change_window_mode(&mut self, mode: WindowMode) -> Task<Message> {
        if mode == self.window_mode || mode == WindowMode::Invalid {
            return Task::none();
        }
        let detach = if mode == WindowMode::Widget {
            self.detach_preview()
        } else {
            Task::none()
        };
        let close = self
            .main_window_id
            .map(window::close)
            .unwrap_or_else(Task::none);
        let (id, open) = window::open(get_window_settings_for_window_mode(mode));
        self.main_window_id = Some(id);
        self.window_mode = mode;
        Task::batch([detach, close, open.discard()])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowMode {
    Dashboard,
    Widget,
    Invalid,
}

fn get_size_for_window_mode(window_mode: WindowMode) -> Size {
    match window_mode {
        WindowMode::Dashboard => Size::new(1120.0, 800.0),
        WindowMode::Widget => Size::new(300.0, 550.0), // 6:11
        WindowMode::Invalid => Size::ZERO,
    }
}

fn get_position_for_window_mode(window_mode: WindowMode) -> Position {
    match window_mode {
        WindowMode::Dashboard => Position::Centered,
        WindowMode::Widget => Position::SpecificWith(|window_size, screen_size| Point {
            x: (screen_size.width - window_size.width),
            y: (screen_size.height - window_size.height),
        }),
        WindowMode::Invalid => Position::Centered,
    }
}

fn get_window_settings_for_window_mode(window_mode: WindowMode) -> window::Settings {
    let window_size = get_size_for_window_mode(window_mode);
    window::Settings {
        size: window_size,
        resizable: window_mode == WindowMode::Dashboard,
        min_size: Some(if window_mode == WindowMode::Dashboard {
            Size::new(1000.0, 600.0)
        } else {
            window_size
        }),
        max_size: (window_mode != WindowMode::Dashboard).then_some(window_size),
        position: get_position_for_window_mode(window_mode),
        decorations: true,
        exit_on_close_request: false,
        ..Default::default()
    }
}

fn preview_window_settings() -> window::Settings {
    window::Settings {
        size: Size::new(800.0, 520.0),
        min_size: Some(Size::new(420.0, 300.0)),
        resizable: true,
        position: Position::Centered,
        exit_on_close_request: false,
        ..Default::default()
    }
}

fn get_start_mode() -> WindowMode {
    let args: Vec<String> = std::env::args().collect();

    if let Some(start_mode_flag_pos) = args.iter().position(|a| a == ("--start-as")) {
        if let Some(start_mode_arg) = args.get(start_mode_flag_pos + 1) {
            return if start_mode_arg.eq_ignore_ascii_case("dashboard") {
                WindowMode::Dashboard
            } else if start_mode_arg.eq_ignore_ascii_case("widget") {
                WindowMode::Widget
            } else {
                WindowMode::Invalid
            };
        }
    }

    WindowMode::Dashboard
}

fn get_current_ui_elements(app: &MainPanel) -> Container<'static, Message> {
    window_layout(app).width(Length::Fill).height(Length::Fill)
}

fn main() -> iced::Result {
    let language = get_language(false);
    set_locale(language.as_str());

    let start_mode = get_start_mode();

    if start_mode == WindowMode::Invalid {
        println!(
            "Invalid start mode. Please use --start-as dashboard or --start-as widget or remove the flag."
        );
        panic!();
    }

    println!("Starting Tiny4Linux in {:?} mode", start_mode);

    iced::daemon(MainPanel::title, MainPanel::update, MainPanel::view)
        .theme(|_, _| obsbot_theme())
        .subscription(MainPanel::subscription)
        .run_with(move || MainPanel::init_state(start_mode))
}

/// Reports the outcome of a debug-area action instead of taking the window down.
///
/// `None` means the entered hex could not be parsed, `Some(false)` that the camera
/// rejected the request. Neither is a reason to panic in a GUI.
fn report_debug_result(outcome: Option<bool>) {
    match outcome {
        Some(true) => {}
        Some(false) => eprintln!("{}", t!("shared.errors.debug_request_failed")),
        None => eprintln!("{}", t!("shared.errors.debug_input_invalid")),
    }
}

#[cfg(test)]
mod window_tests {
    use super::*;

    #[test]
    fn dashboard_can_expand_without_a_maximum_size() {
        let settings = get_window_settings_for_window_mode(WindowMode::Dashboard);
        assert!(settings.resizable);
        assert!(settings.max_size.is_none());
        let minimum = settings
            .min_size
            .expect("the dashboard must remain usable when shrunk");
        assert!(minimum.width < settings.size.width);
        assert!(minimum.height < settings.size.height);
    }

    #[test]
    fn preview_actions_work_without_a_camera_and_closing_its_window_docks_it() {
        let (mut app, _) = MainPanel::init_with_camera(None, WindowMode::Dashboard);
        let main_id = app.main_window_id;
        let _ = app.update(Message::SetPreviewEnabled(true));
        assert_eq!(app.preview.placement, Placement::Docked);
        assert!(app.preview.error.is_some());
        let _ = app.update(Message::DetachPreview);
        let Placement::Detached(id) = app.preview.placement else {
            panic!("preview should detach");
        };
        let _ = app.update(Message::WindowCloseRequested(id));
        assert_eq!(app.preview.placement, Placement::Docked);
        assert_eq!(app.main_window_id, main_id);
        let _ = app.update(Message::SetPreviewEnabled(false));
        let _ = app.update(Message::WindowCloseRequested(id));
        assert!(
            !app.preview.enabled(),
            "an old window must not reopen the preview"
        );
    }

    #[test]
    fn switching_to_widget_detaches_preview_and_docking_restores_dashboard() {
        let (mut app, _) = MainPanel::init_with_camera(None, WindowMode::Dashboard);
        let _ = app.update(Message::SetPreviewEnabled(true));
        let _ = app.update(Message::RequestWindowModeChange(WindowMode::Widget));
        assert!(matches!(app.preview.placement, Placement::Detached(_)));
        assert_eq!(app.window_mode, WindowMode::Widget);
        let _ = app.update(Message::DockPreview);
        assert_eq!(app.preview.placement, Placement::Docked);
        assert_eq!(app.window_mode, WindowMode::Dashboard);
    }

    #[test]
    fn closing_the_main_window_turns_preview_off() {
        let (mut app, _) = MainPanel::init_with_camera(None, WindowMode::Dashboard);
        let _ = app.update(Message::SetPreviewEnabled(true));
        let _ = app.update(Message::DetachPreview);
        let _ = app.update(Message::WindowCloseRequested(app.main_window_id.unwrap()));
        assert!(!app.preview.enabled());
    }

    #[test]
    fn turning_preview_on_in_widget_opens_a_separate_window() {
        let (mut app, _) = MainPanel::init_with_camera(None, WindowMode::Widget);
        let _ = app.update(Message::SetPreviewEnabled(true));
        assert!(matches!(app.preview.placement, Placement::Detached(_)));
    }

    #[test]
    fn preview_window_is_resizable_and_uses_managed_close_requests() {
        let settings = preview_window_settings();
        assert!(settings.resizable);
        assert!(settings.max_size.is_none());
        assert!(!settings.exit_on_close_request);
        assert!(!get_window_settings_for_window_mode(WindowMode::Dashboard).exit_on_close_request);
    }

    #[test]
    fn widget_keeps_its_compact_window_bounds() {
        let settings = get_window_settings_for_window_mode(WindowMode::Widget);
        assert!(!settings.resizable);
        assert_eq!(settings.min_size, Some(settings.size));
        assert_eq!(settings.max_size, Some(settings.size));
    }
}
