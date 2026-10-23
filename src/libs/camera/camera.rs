// SPDX-License-Identifier: EUPL-1.2

use crate::libs::camera::custom_presets::{CustomPreset, CustomPresetStore};
use crate::libs::camera::enums::{AIMode, CameraModel, ExposureMode, SleepMode, TrackingSpeed};
use crate::libs::camera::status::CameraStatus;
use crate::libs::camera::transport::CameraTransport;
use crate::libs::errors::T4lError;
use crate::libs::usbio::{V4L2_CID_PAN_ABSOLUTE, V4L2_CID_TILT_ABSOLUTE};
use crate::{
    AIModeCommand, ExposureModeCommand, ExposureModeTypeCommand, GotoPresetPositionCommand,
    HdrModeCommand, LegacyAiModeCommand, LegacyPresetPositionCommand, LegacySleepCommand,
    SleepCommand, TrackingSpeedCommand,
};
use errno::Errno;
use std::sync::atomic::{AtomicU16, Ordering};
use std::thread::sleep;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

/// Pause between two legacy frames sent in one sequence.
///
/// The Tiny 4K drops frames that arrive back to back. In captures of the vendor
/// app no two legacy frames are ever closer than about 50 ms, with a median of
/// roughly 95 ms, so the sequence is paced accordingly (#72).
const LEGACY_FRAME_GAP: Duration = Duration::from_millis(100);

/// How long to wait before looking for an answer, and how often to look.
const LEGACY_REPLY_GAP: Duration = Duration::from_millis(60);
const LEGACY_REPLY_ATTEMPTS: usize = 6;

/// Device hints tried in this order by [`Camera::detect`].
///
/// A hint has to be part of the V4L2 card name of the camera.
/// The Tiny 2 is the primarily supported model and therefore preferred,
/// the Tiny 4K is only found if no Tiny 2 is connected (#72).
const DEFAULT_CAMERA_HINTS: &[&str] = crate::libs::usbio::SUPPORTED_CAMERAS;

pub struct Camera {
    transport: CameraTransport,
    model: CameraModel,
    debugging: bool,
    /// Sequence number for the next legacy request that expects an answer.
    sequence_nr: AtomicU16,
}

impl Camera {
    pub fn new(hint: &str) -> Result<Self, T4lError> {
        Ok(Self {
            transport: CameraTransport::new(hint)?,
            model: CameraModel::from_hint(hint),
            debugging: false,
            sequence_nr: AtomicU16::new(Self::first_sequence_nr()),
        })
    }

    /// Opens the first camera matching one of the default hints, see [`DEFAULT_CAMERA_HINTS`].
    pub fn detect() -> Result<Self, T4lError> {
        DEFAULT_CAMERA_HINTS
            .iter()
            .find_map(|hint| Self::new(hint).ok())
            .ok_or(T4lError::NoCameraFound)
    }

    pub fn info(&self) -> Result<(), Errno> {
        self.transport.info()
    }

    pub fn send_cmd(&self, unit: u8, selector: u8, cmd: &[u8]) -> Result<(), T4lError> {
        self.transport.send_cmd(unit, selector, cmd, self.debugging)
    }

    pub fn get_status(&self) -> Result<CameraStatus, T4lError> {
        self.transport.get_status(self.debugging, self.model)
    }

    /// Starting point for the sequence numbers of requests that expect an answer.
    ///
    /// The camera keeps the previous answer on its command channel, and that answer
    /// outlives the process that asked for it. Starting every run at the same number
    /// would therefore accept the answer to a *previous* run's request, so the
    /// counter starts somewhere that differs between runs (#72).
    fn first_sequence_nr() -> u16 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|since_epoch| since_epoch.subsec_nanos() as u16)
            .unwrap_or(1)
    }

    /// The model this camera was detected as, see [`CameraModel`].
    pub fn model(&self) -> CameraModel {
        self.model
    }

    /// Returns the video device belonging to this camera for a separate capture handle.
    pub fn video_device_path(&self) -> std::io::Result<std::path::PathBuf> {
        self.transport.device_path()
    }

    /// Asks an OBSBOT Tiny 4K which tracking mode it is in.
    ///
    /// The answer is picked up from the command channel, retrying until the camera
    /// returns the answer to our own request rather than the previous one (#72).
    fn read_ai_mode_tiny_4k(&self) -> Result<AIMode, T4lError> {
        let sequence_nr = self.sequence_nr.fetch_add(1, Ordering::Relaxed);

        self.send_cmd(0x2, 0x2, &LegacyAiModeCommand::status_request(sequence_nr))?;

        for _ in 0..LEGACY_REPLY_ATTEMPTS {
            sleep(LEGACY_REPLY_GAP);

            if let Ok(reply) = self.transport.get_reply()
                && let Some(mode) = LegacyAiModeCommand::parse_status(&reply, sequence_nr)
            {
                return Ok(mode);
            }
        }

        Ok(AIMode::Unknown)
    }

    /// Whether this camera has the given AI tracking mode.
    ///
    /// The Tiny 4K only knows a part of the Tiny 2 modes (#72).
    pub fn supports_ai_mode(&self, mode: AIMode) -> bool {
        self.model.supports_ai_mode(mode)
    }

    /// Whether this camera has a tracking speed setting.
    ///
    /// The Tiny 4K has none: the setting does not exist in its protocol.
    pub fn supports_tracking_speed(&self) -> bool {
        self.model.supports_tracking_speed()
    }

    /// Whether this camera takes the manual exposure mode over this interface.
    ///
    /// The Tiny 4K drives manual exposure through the standard UVC camera
    /// terminal instead, which is reachable via V4L2.
    pub fn supports_manual_exposure(&self) -> bool {
        self.model.supports_manual_exposure()
    }

    pub fn dump(&self) -> Result<(), Errno> {
        self.transport.dump()
    }

    pub fn dump_02(&self) -> Result<(), Errno> {
        self.transport.dump_02()
    }

    pub fn set_debugging(&mut self, debugging: bool) {
        self.debugging = debugging
    }
}

pub trait Tiny2Camera {
    fn set_sleep_mode(&self, mode: SleepMode) -> Result<(), T4lError>;
    fn get_sleep_mode(&self) -> Result<SleepMode, T4lError>;
    fn set_ai_mode(&self, mode: AIMode) -> Result<(), T4lError>;
    fn get_ai_mode(&self) -> Result<AIMode, T4lError>;
    fn goto_preset_position(&self, preset_nr: i8) -> Result<(), T4lError>;
    fn get_tracking_speed(&self) -> Result<TrackingSpeed, T4lError>;
    fn set_tracking_speed(&self, speed: TrackingSpeed) -> Result<(), T4lError>;
    fn set_hdr_mode(&self, mode: bool) -> Result<(), T4lError>;
    fn set_exposure_mode(&self, mode: ExposureMode) -> Result<(), T4lError>;
    fn get_zoom_absolute(&self) -> Result<i32, T4lError>;
    fn set_zoom_absolute(&self, value: i32) -> Result<(), T4lError>;
    fn get_zoom_range(&self) -> Result<(i32, i32), T4lError>;
    fn get_pan_absolute(&self) -> Result<i32, T4lError>;
    fn set_pan_absolute(&self, value: i32) -> Result<(), T4lError>;
    fn get_pan_range(&self) -> Result<(i32, i32), T4lError>;
    fn get_tilt_absolute(&self) -> Result<i32, T4lError>;
    fn set_tilt_absolute(&self, value: i32) -> Result<(), T4lError>;
    fn get_tilt_range(&self) -> Result<(i32, i32), T4lError>;
    fn save_custom_preset(&self, name: &str) -> Result<(), T4lError>;
    fn recall_custom_preset(&self, name: &str) -> Result<(), T4lError>;
    fn delete_custom_preset(&self, name: &str) -> Result<(), T4lError>;
    fn get_custom_presets(&self) -> Vec<CustomPreset>;
    fn set_debugging(&mut self, debugging: bool);
}

impl Tiny2Camera for Camera {
    /// Sends the sleep command matching the detected model.
    ///
    /// The Tiny 4K accepts the Tiny 2 frame on USB but ignores it, so it gets the
    /// legacy frame instead (#72). Both models take the command on unit 2,
    /// selector 2.
    fn set_sleep_mode(&self, mode: SleepMode) -> Result<(), T4lError> {
        match self.model {
            CameraModel::Tiny2 => self.send_cmd(0x2, 0x2, &SleepCommand::build(mode)?),
            CameraModel::Tiny4K => self.send_cmd(0x2, 0x2, &LegacySleepCommand::build(mode)?),
        }
    }

    fn get_sleep_mode(&self) -> Result<SleepMode, T4lError> {
        Ok(self.get_status()?.awake)
    }

    /// Sends the AI tracking command matching the detected model.
    ///
    /// The Tiny 2 takes a single setting on selector 6. The Tiny 4K needs a
    /// sequence of legacy frames on selector 2 instead (#72).
    fn set_ai_mode(&self, mode: AIMode) -> Result<(), T4lError> {
        match self.model {
            CameraModel::Tiny2 => self.send_cmd(0x2, 0x6, &AIModeCommand::build(mode)?),
            CameraModel::Tiny4K => {
                for (index, frame) in LegacyAiModeCommand::build(mode)?.iter().enumerate() {
                    if index > 0 {
                        sleep(LEGACY_FRAME_GAP);
                    }

                    self.send_cmd(0x2, 0x2, frame)?;
                }

                Ok(())
            }
        }
    }

    /// Reads the tracking mode the camera is in.
    ///
    /// The Tiny 2 keeps it in the status buffer. The Tiny 4K leaves those bytes at
    /// zero and answers a separate request instead (#72).
    fn get_ai_mode(&self) -> Result<AIMode, T4lError> {
        match self.model {
            CameraModel::Tiny2 => Ok(self.get_status()?.ai_mode),
            CameraModel::Tiny4K => self.read_ai_mode_tiny_4k(),
        }
    }

    /// Moves the camera to a stored preset position.
    ///
    /// The Tiny 2 has a recall command. The Tiny 4K has not: it stores the positions
    /// and hands them out on request, so the recall is a read followed by an absolute
    /// move (#72). Presets are stored by the vendor app; this only recalls them.
    fn goto_preset_position(&self, preset_nr: i8) -> Result<(), T4lError> {
        match self.model {
            CameraModel::Tiny2 => {
                self.send_cmd(0x2, 0x2, &GotoPresetPositionCommand::build(preset_nr)?)
            }
            CameraModel::Tiny4K => {
                let slot = u8::try_from(preset_nr).map_err(|_| {
                    T4lError::UnsupportedIntValue("preset position".to_string(), preset_nr as i32)
                })?;
                let table = self.transport.get_preset_positions(self.debugging)?;
                let preset = LegacyPresetPositionCommand::find(&table, slot)?;

                for (index, frame) in LegacyPresetPositionCommand::build(&preset)
                    .iter()
                    .enumerate()
                {
                    if index > 0 {
                        sleep(LEGACY_FRAME_GAP);
                    }

                    self.send_cmd(0x2, 0x2, frame)?;
                }

                Ok(())
            }
        }
    }

    fn get_tracking_speed(&self) -> Result<TrackingSpeed, T4lError> {
        Ok(self.get_status()?.speed)
    }

    fn set_tracking_speed(&self, speed: TrackingSpeed) -> Result<(), T4lError> {
        let cmd = TrackingSpeedCommand::build(speed)?;

        self.send_cmd(0x2, 0x2, &cmd)
    }

    fn set_hdr_mode(&self, mode: bool) -> Result<(), T4lError> {
        let cmd = HdrModeCommand::build(mode);

        self.send_cmd(0x2, 0x6, &cmd)
    }

    fn set_exposure_mode(&self, mode: ExposureMode) -> Result<(), T4lError> {
        let exposure_mode_type_command = ExposureModeTypeCommand::build(mode);

        self.send_cmd(0x2, 0x2, &exposure_mode_type_command)?;

        if let Some(command) = ExposureModeCommand::build(mode) {
            self.send_cmd(0x2, 0x6, &command)?;
        }

        Ok(())
    }

    fn set_debugging(&mut self, debugging: bool) {
        self.set_debugging(debugging);
    }

    fn get_zoom_absolute(&self) -> Result<i32, T4lError> {
        self.transport.get_zoom_absolute()
    }

    fn set_zoom_absolute(&self, value: i32) -> Result<(), T4lError> {
        self.transport.set_zoom_absolute(value)
    }

    fn get_zoom_range(&self) -> Result<(i32, i32), T4lError> {
        self.transport.get_zoom_range()
    }

    fn get_pan_absolute(&self) -> Result<i32, T4lError> {
        self.transport.get_control(V4L2_CID_PAN_ABSOLUTE)
    }

    fn set_pan_absolute(&self, value: i32) -> Result<(), T4lError> {
        self.set_ai_mode(AIMode::NoTracking)?;
        self.transport.set_control(V4L2_CID_PAN_ABSOLUTE, value)
    }

    fn get_pan_range(&self) -> Result<(i32, i32), T4lError> {
        self.transport.get_control_range(V4L2_CID_PAN_ABSOLUTE)
    }

    fn get_tilt_absolute(&self) -> Result<i32, T4lError> {
        self.transport.get_control(V4L2_CID_TILT_ABSOLUTE)
    }

    fn set_tilt_absolute(&self, value: i32) -> Result<(), T4lError> {
        self.set_ai_mode(AIMode::NoTracking)?;
        self.transport.set_control(V4L2_CID_TILT_ABSOLUTE, value)
    }

    fn get_tilt_range(&self) -> Result<(i32, i32), T4lError> {
        self.transport.get_control_range(V4L2_CID_TILT_ABSOLUTE)
    }

    fn save_custom_preset(&self, name: &str) -> Result<(), T4lError> {
        let store = CustomPresetStore::new();

        store.save(&CustomPreset {
            name: name.to_string(),
            pan: self.get_pan_absolute()?,
            tilt: self.get_tilt_absolute()?,
            zoom: self.get_zoom_absolute()?,
        })
    }

    fn recall_custom_preset(&self, name: &str) -> Result<(), T4lError> {
        let store = CustomPresetStore::new();

        let preset = store
            .load()
            .into_iter()
            .find(|p| p.name == name)
            .ok_or_else(|| T4lError::PresetNotFound(name.to_string()))?;

        self.set_ai_mode(AIMode::NoTracking)?;

        self.set_pan_absolute(preset.pan)?;
        self.set_tilt_absolute(preset.tilt)?;
        self.set_zoom_absolute(preset.zoom)?;

        Ok(())
    }

    fn delete_custom_preset(&self, name: &str) -> Result<(), T4lError> {
        CustomPresetStore::new().delete(name)
    }

    fn get_custom_presets(&self) -> Vec<CustomPreset> {
        CustomPresetStore::new().load()
    }
}
