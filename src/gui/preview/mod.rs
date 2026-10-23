// SPDX-License-Identifier: EUPL-1.2

mod capture;

pub use capture::{Capture, CaptureError};
use iced::widget::image;
use iced::window;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Placement {
    #[default]
    Off,
    Docked,
    Detached(window::Id),
}

#[derive(Default)]
pub struct Preview {
    pub placement: Placement,
    pub frame: Option<image::Handle>,
    pub error: Option<CaptureError>,
    capture: Option<Capture>,
}

impl Preview {
    pub fn enabled(&self) -> bool {
        self.placement != Placement::Off
    }

    pub fn capturing(&self) -> bool {
        self.capture.is_some()
    }

    pub fn start(&mut self, result: Result<Capture, CaptureError>) {
        if !self.enabled() {
            self.placement = Placement::Docked;
        }
        self.frame = None;
        self.error = None;
        self.capture = match result {
            Ok(capture) => Some(capture),
            Err(error) => {
                self.error = Some(error);
                None
            }
        };
    }

    pub fn detach(&mut self, id: window::Id) {
        if self.placement == Placement::Docked {
            self.placement = Placement::Detached(id);
        }
    }

    pub fn dock(&mut self) -> Option<window::Id> {
        match self.placement {
            Placement::Detached(id) => {
                self.placement = Placement::Docked;
                Some(id)
            }
            _ => None,
        }
    }

    pub fn stop(&mut self) -> Option<window::Id> {
        let window = self.dock();
        self.placement = Placement::Off;
        self.capture = None;
        self.frame = None;
        self.error = None;
        window
    }

    pub fn poll(&mut self) {
        if let Some(capture) = self.capture.as_mut() {
            let (frame, error) = capture.poll();
            if let Some(error) = error {
                self.capture = None;
                self.frame = None;
                self.error = Some(error);
            } else if let Some(frame) = frame {
                self.frame = Some(image::Handle::from_rgba(
                    capture::WIDTH,
                    capture::HEIGHT,
                    frame,
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_starts_off_without_capture() {
        let preview = Preview::default();
        assert!(!preview.enabled());
        assert!(!preview.capturing());
    }

    #[test]
    fn detaching_and_docking_preserve_the_current_frame() {
        let frame = image::Handle::from_rgba(1, 1, vec![10, 20, 30, 255]);
        let mut preview = Preview {
            placement: Placement::Docked,
            frame: Some(frame.clone()),
            ..Preview::default()
        };
        let id = window::Id::unique();
        preview.detach(id);
        assert_eq!(preview.placement, Placement::Detached(id));
        assert_eq!(preview.frame.as_ref().unwrap().id(), frame.id());
        assert_eq!(preview.dock(), Some(id));
        assert_eq!(preview.placement, Placement::Docked);
        assert_eq!(preview.frame.as_ref().unwrap().id(), frame.id());
    }

    #[test]
    fn turning_off_clears_the_image_and_identifies_the_window_to_close() {
        let id = window::Id::unique();
        let mut preview = Preview {
            placement: Placement::Detached(id),
            error: Some(CaptureError::TimedOut),
            ..Preview::default()
        };
        assert_eq!(preview.stop(), Some(id));
        assert!(!preview.enabled());
        assert!(preview.frame.is_none());
        assert!(preview.error.is_none());
    }

    #[test]
    fn docking_or_detaching_an_off_preview_does_not_start_capture() {
        let mut preview = Preview::default();
        assert_eq!(preview.dock(), None);
        preview.detach(window::Id::unique());
        assert!(!preview.enabled());
        assert!(!preview.capturing());
    }

    #[test]
    fn failed_start_keeps_the_panel_available_for_retry() {
        let mut preview = Preview::default();
        preview.start(Err(CaptureError::MissingFfmpeg));
        assert_eq!(preview.placement, Placement::Docked);
        assert_eq!(preview.error, Some(CaptureError::MissingFfmpeg));
        let id = window::Id::unique();
        preview.detach(id);
        preview.start(Err(CaptureError::TimedOut));
        assert_eq!(preview.placement, Placement::Detached(id));
    }
}
