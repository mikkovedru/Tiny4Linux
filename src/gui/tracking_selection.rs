// SPDX-License-Identifier: EUPL-1.2

use std::time::{Duration, Instant};
use tiny4linux::AIMode;

const CONFIRMATION_TIMEOUT: Duration = Duration::from_secs(3);

/// Keeps temporary firmware status from replacing a selection while it is applied.
pub struct TrackingSelection {
    mode: AIMode,
    deadline: Option<Instant>,
}

impl TrackingSelection {
    pub fn new(mode: AIMode) -> Self {
        Self {
            mode,
            deadline: None,
        }
    }

    pub fn mode(&self) -> AIMode {
        self.mode
    }

    pub fn is_pending(&self) -> bool {
        self.deadline.is_some()
    }

    /// Call only after the camera accepts the command on USB.
    pub fn request(&mut self, mode: AIMode, now: Instant) {
        self.mode = mode;
        self.deadline = Some(now + CONFIRMATION_TIMEOUT);
    }

    /// Returns true if the camera failed to confirm the selection before timeout.
    pub fn observe(&mut self, mode: AIMode, now: Instant) -> bool {
        if let Some(deadline) = self.deadline {
            if mode == self.mode {
                self.deadline = None;
            } else if now < deadline {
                return false;
            } else {
                self.deadline = None;
                self.mode = mode;
                return true;
            }
        }
        self.mode = mode;
        false
    }

    pub fn disconnect(&mut self) {
        self.mode = AIMode::Unknown;
        self.deadline = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_click_survives_old_and_temporary_camera_status() {
        let now = Instant::now();
        let mut selection = TrackingSelection::new(AIMode::CloseUp);
        selection.request(AIMode::UpperBody, now);
        for (ms, observed) in [
            (1, AIMode::CloseUp),
            (20, AIMode::Unknown),
            (200, AIMode::Hand),
        ] {
            assert!(!selection.observe(observed, now + Duration::from_millis(ms)));
            assert_eq!(selection.mode(), AIMode::UpperBody);
            assert!(selection.is_pending());
        }
        assert!(!selection.observe(AIMode::UpperBody, now + Duration::from_millis(800)));
        assert_eq!(selection.mode(), AIMode::UpperBody);
        assert!(!selection.is_pending());
    }

    #[test]
    fn latest_click_wins_while_previous_command_is_still_applying() {
        let now = Instant::now();
        let mut selection = TrackingSelection::new(AIMode::CloseUp);
        selection.request(AIMode::UpperBody, now);
        selection.request(AIMode::NormalTracking, now + Duration::from_millis(100));
        selection.observe(AIMode::UpperBody, now + Duration::from_millis(200));
        assert_eq!(selection.mode(), AIMode::NormalTracking);
        assert!(selection.is_pending());
        selection.observe(AIMode::NormalTracking, now + Duration::from_millis(900));
        assert!(!selection.is_pending());
    }

    #[test]
    fn unconfirmed_selection_expires_and_reports_actual_mode() {
        let now = Instant::now();
        let mut selection = TrackingSelection::new(AIMode::CloseUp);
        selection.request(AIMode::UpperBody, now);
        assert!(selection.observe(AIMode::CloseUp, now + CONFIRMATION_TIMEOUT));
        assert_eq!(selection.mode(), AIMode::CloseUp);
        assert!(!selection.is_pending());
    }

    #[test]
    fn confirmed_selection_follows_later_external_changes() {
        let now = Instant::now();
        let mut selection = TrackingSelection::new(AIMode::CloseUp);
        selection.request(AIMode::UpperBody, now);
        selection.observe(AIMode::UpperBody, now);
        selection.observe(AIMode::NoTracking, now + Duration::from_secs(20));
        assert_eq!(selection.mode(), AIMode::NoTracking);
    }

    #[test]
    fn disconnect_cancels_pending_confirmation() {
        let mut selection = TrackingSelection::new(AIMode::CloseUp);
        selection.request(AIMode::UpperBody, Instant::now());
        selection.disconnect();
        assert_eq!(selection.mode(), AIMode::Unknown);
        assert!(!selection.is_pending());
    }
}
