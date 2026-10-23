// SPDX-License-Identifier: EUPL-1.2

use rust_i18n::t;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SleepMode {
    Awake,
    Sleep,
    Unknown,
}

impl Display for SleepMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SleepMode::Awake => write!(f, "{}", t!("display.sleep_mode.awake")),
            SleepMode::Sleep => write!(f, "{}", t!("display.sleep_mode.sleep")),
            SleepMode::Unknown => write!(f, "{}", t!("display.sleep_mode.unknown")),
        }
    }
}

/// The camera model a [`crate::Camera`] is talking to.
///
/// The Tiny 4K speaks an older protocol than the Tiny 2 and needs different
/// commands for some functions (#72). Unknown hints are treated as a Tiny 2,
/// which keeps the behaviour of every previously supported camera unchanged.
///
/// The Tiny 2 Lite takes the same commands as the Tiny 2, but reports some values
/// at different places in the status block, see [`crate::CameraStatus::decode_for`]
/// (#172). The Tiny 3 currently uses the Tiny 2 path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraModel {
    Tiny2,
    Tiny2Lite,
    Tiny4K,
}

impl CameraModel {
    /// Whether this model has the given AI tracking mode.
    ///
    /// The Tiny 4K only knows a part of the Tiny 2 modes (#72).
    pub fn supports_ai_mode(self, mode: AIMode) -> bool {
        match self {
            CameraModel::Tiny2 | CameraModel::Tiny2Lite => true,
            CameraModel::Tiny4K => crate::LegacyAiModeCommand::supports(mode),
        }
    }

    /// Whether this model has a tracking speed setting.
    ///
    /// The Tiny 4K has none: the setting does not exist in its protocol.
    pub fn supports_tracking_speed(self) -> bool {
        self != CameraModel::Tiny4K
    }

    /// Whether this model takes the manual exposure mode over this interface.
    ///
    /// The Tiny 4K drives manual exposure through the standard UVC camera terminal
    /// instead, which is reachable via V4L2.
    pub fn supports_manual_exposure(self) -> bool {
        self != CameraModel::Tiny4K
    }

    /// Whether this model takes the settings added in #174: auto-sleep, sleep
    /// background, status LED, field of view, mirror, invert, audio, gestures and
    /// factory reset.
    ///
    /// They were verified on a Tiny 2 Lite only. The Tiny 2 and Tiny 3 use the same
    /// command family and are allowed, but unverified. The Tiny 4K speaks the older
    /// legacy protocol, which has none of these commands.
    pub fn supports_extended_settings(self) -> bool {
        self != CameraModel::Tiny4K
    }

    /// Whether the status block of this model is known to report the #174 settings.
    ///
    /// Their offsets were measured on a Tiny 2 Lite only and differ at least partly
    /// on the Tiny 2, see [`crate::CameraStatus`].
    pub fn reports_extended_status(self) -> bool {
        self == CameraModel::Tiny2Lite
    }

    /// Derives the model from the V4L2 card name hint the camera was opened with.
    pub fn from_hint(hint: &str) -> Self {
        if hint.contains("Tiny 4K") {
            CameraModel::Tiny4K
        } else {
            CameraModel::Tiny2
        }
    }

    /// Derives the model from the hint and the V4L2 card name of the opened camera.
    ///
    /// The hint "OBSBOT Tiny 2" also matches a Tiny 2 Lite, so the Lite can only be
    /// told apart by the card name of the camera that was actually opened, e.g.
    /// "OBSBOT Tiny 2 Lite: OBSBOT Tiny" (#172). Every other model keeps the model
    /// derived from the hint.
    pub fn detect(hint: &str, card_name: Option<&str>) -> Self {
        match (Self::from_hint(hint), card_name) {
            (CameraModel::Tiny2, Some(card_name)) if card_name.contains("Tiny 2 Lite") => {
                CameraModel::Tiny2Lite
            }
            (model, _) => model,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AIMode {
    NoTracking,
    NormalTracking,
    UpperBody,
    CloseUp,
    Headless,
    LowerBody,
    DeskMode,
    Whiteboard,
    Hand,
    Group,
    Unknown,
}

impl Display for AIMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIMode::NoTracking => write!(f, "{}", t!("display.ai_mode.static")),
            AIMode::NormalTracking => write!(f, "{}", t!("display.ai_mode.normal")),
            AIMode::UpperBody => write!(f, "{}", t!("display.ai_mode.upper_body")),
            AIMode::CloseUp => write!(f, "{}", t!("display.ai_mode.close_up")),
            AIMode::Headless => write!(f, "{}", t!("display.ai_mode.headless")),
            AIMode::LowerBody => write!(f, "{}", t!("display.ai_mode.lower_body")),
            AIMode::DeskMode => write!(f, "{}", t!("display.ai_mode.desk")),
            AIMode::Whiteboard => write!(f, "{}", t!("display.ai_mode.whiteboard")),
            AIMode::Hand => write!(f, "{}", t!("display.ai_mode.hand")),
            AIMode::Group => write!(f, "{}", t!("display.ai_mode.group")),
            AIMode::Unknown => write!(f, "{}", t!("display.ai_mode.unknown")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrackingSpeed {
    Standard,
    Sport,
}

impl Display for TrackingSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackingSpeed::Standard => write!(f, "{}", t!("display.tracking_speed.standard")),
            TrackingSpeed::Sport => write!(f, "{}", t!("display.tracking_speed.sport")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExposureMode {
    Manual,
    Global,
    Face,
}

impl Display for ExposureMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExposureMode::Manual => write!(f, "{}", t!("display.exposure_mode.manual")),
            ExposureMode::Global => write!(f, "{}", t!("display.exposure_mode.global")),
            ExposureMode::Face => write!(f, "{}", t!("display.exposure_mode.face")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExposureModeType {
    Auto,
    Manual,
}

/// Auto-sleep time as reported in the status block and sent with the auto-sleep command.
///
/// The camera stores seconds as `i16`; a negative value means auto-sleep is off and keeps
/// the last time, so turning it on again restores it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoSleepTime(pub i16);

impl AutoSleepTime {
    /// Factory default, used when there is no previous time to keep.
    pub const DEFAULT_SECONDS: i16 = 120;

    pub fn is_enabled(&self) -> bool {
        self.0 > 0
    }

    pub fn seconds(&self) -> u16 {
        self.0.unsigned_abs()
    }

    /// The value that turns auto-sleep off: OBSBOT Center sends the current time negated
    /// (e.g. -120 when 120 s were set). An already disabled time is kept as it is.
    pub fn disabled(&self) -> Self {
        match self.0 {
            t if t > 0 => AutoSleepTime(-t),
            0 => AutoSleepTime(-Self::DEFAULT_SECONDS),
            t => AutoSleepTime(t),
        }
    }
}

impl Display for AutoSleepTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_enabled() {
            write!(
                f,
                "{}",
                t!("display.auto_sleep.on", seconds = self.seconds())
            )
        } else if self.0 == 0 {
            write!(f, "{}", t!("display.states.off"))
        } else {
            write!(
                f,
                "{}",
                t!("display.auto_sleep.off", seconds = self.seconds())
            )
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SleepBackground {
    Black,
    Image,
    Unknown,
}

impl Display for SleepBackground {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SleepBackground::Black => write!(f, "{}", t!("display.sleep_background.black")),
            SleepBackground::Image => write!(f, "{}", t!("display.sleep_background.image")),
            SleepBackground::Unknown => write!(f, "{}", t!("display.sleep_background.unknown")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldOfView {
    Wide,
    Medium,
    Narrow,
    Unknown,
}

impl Display for FieldOfView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldOfView::Wide => write!(f, "{}", t!("display.field_of_view.wide")),
            FieldOfView::Medium => write!(f, "{}", t!("display.field_of_view.medium")),
            FieldOfView::Narrow => write!(f, "{}", t!("display.field_of_view.narrow")),
            FieldOfView::Unknown => write!(f, "{}", t!("display.field_of_view.unknown")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoiseReduction {
    Off,
    Weak,
    Medium,
    Strong,
    Unknown,
}

impl Display for NoiseReduction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoiseReduction::Off => write!(f, "{}", t!("display.states.off")),
            NoiseReduction::Weak => write!(f, "{}", t!("display.noise_reduction.weak")),
            NoiseReduction::Medium => write!(f, "{}", t!("display.noise_reduction.medium")),
            NoiseReduction::Strong => write!(f, "{}", t!("display.noise_reduction.strong")),
            NoiseReduction::Unknown => write!(f, "{}", t!("display.noise_reduction.unknown")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PickupDistance {
    Near,
    Standard,
    Far,
    Unknown,
}

impl Display for PickupDistance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PickupDistance::Near => write!(f, "{}", t!("display.pickup_distance.near")),
            PickupDistance::Standard => write!(f, "{}", t!("display.pickup_distance.standard")),
            PickupDistance::Far => write!(f, "{}", t!("display.pickup_distance.far")),
            PickupDistance::Unknown => write!(f, "{}", t!("display.pickup_distance.unknown")),
        }
    }
}

/// Gestures that can be turned on or off.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gesture {
    TargetLock,
    Zoom,
    DynamicZoom,
    DirectionChange,
}

#[cfg(test)]
mod tests {
    mod unit {
        mod camera_model {
            use crate::CameraModel;
            use test_case::test_case;

            #[test_case("OBSBOT Tiny 4K", CameraModel::Tiny4K; "tiny 4k")]
            #[test_case("OBSBOT Tiny 2", CameraModel::Tiny2; "tiny 2")]
            #[test_case("OBSBOT Tiny 2 Lite", CameraModel::Tiny2; "tiny 2 lite")]
            #[test_case("Something else", CameraModel::Tiny2; "unknown hints default to tiny 2")]
            fn model_from_hint(hint: &str, expected: CameraModel) {
                assert_eq!(CameraModel::from_hint(hint), expected);
            }

            #[test_case("OBSBOT Tiny 2", Some("OBSBOT Tiny 2 Lite: OBSBOT Tiny"), CameraModel::Tiny2Lite; "lite behind the tiny 2 hint")]
            #[test_case("/dev/video0", Some("OBSBOT Tiny 2 Lite: OBSBOT Tiny"), CameraModel::Tiny2Lite; "lite opened by path")]
            #[test_case("OBSBOT Tiny 2", Some("OBSBOT Tiny 2"), CameraModel::Tiny2; "tiny 2")]
            #[test_case("OBSBOT Tiny 2", None, CameraModel::Tiny2; "unreadable card name keeps the hint")]
            #[test_case("OBSBOT Tiny 3", Some("OBSBOT Tiny 3"), CameraModel::Tiny2; "tiny 3 keeps the tiny 2 path")]
            #[test_case("OBSBOT Tiny 4K", Some("OBSBOT Tiny 4K"), CameraModel::Tiny4K; "tiny 4k")]
            fn model_from_hint_and_card_name(
                hint: &str,
                card_name: Option<&str>,
                expected: CameraModel,
            ) {
                assert_eq!(CameraModel::detect(hint, card_name), expected);
            }

            #[test]
            fn lite_has_the_tiny_2_capabilities() {
                // The Lite takes the Tiny 2 commands; only its status layout differs (#172)
                assert!(CameraModel::Tiny2Lite.supports_tracking_speed());
                assert!(CameraModel::Tiny2Lite.supports_manual_exposure());
                assert!(CameraModel::Tiny2Lite.supports_ai_mode(crate::AIMode::Hand));
            }

            // The #174 settings were verified on the Lite; the Tiny 2 and Tiny 3 share
            // its command family, the Tiny 4K's legacy protocol has none of them.
            #[test_case(CameraModel::Tiny2Lite, true, true; "lite")]
            #[test_case(CameraModel::Tiny2, true, false; "tiny 2 takes them but its status layout differs")]
            #[test_case(CameraModel::Tiny4K, false, false; "tiny 4k")]
            fn extended_settings(model: CameraModel, settings: bool, status: bool) {
                assert_eq!(model.supports_extended_settings(), settings);
                assert_eq!(model.reports_extended_status(), status);
            }
        }

        mod display {
            mod sleep_mode {
                use crate::SleepMode;
                use test_case::test_case;

                #[test_case(SleepMode::Awake, "Awake"; "Awake")]
                #[test_case(SleepMode::Sleep, "Sleeping"; "Sleep")]
                #[test_case(SleepMode::Unknown, "Unknown"; "Unknown")]
                fn sleep_mode(mode: SleepMode, expected: &str) {
                    assert_eq!(&mode.to_string(), expected);
                }
            }

            mod ai_mode {
                use crate::AIMode;
                use test_case::test_case;

                #[test_case(AIMode::NoTracking, "Static"; "no tracking")]
                #[test_case(AIMode::NormalTracking, "Normal Tracking"; "normal tracking")]
                #[test_case(AIMode::UpperBody, "Upper Body"; "upper body")]
                #[test_case(AIMode::CloseUp, "Close-up"; "close up")]
                #[test_case(AIMode::Headless, "Headless"; "headless")]
                #[test_case(AIMode::LowerBody, "Lower Body"; "lower body")]
                #[test_case(AIMode::DeskMode, "Desk Mode"; "desk mode")]
                #[test_case(AIMode::Whiteboard, "Whiteboard"; "whiteboard")]
                #[test_case(AIMode::Hand, "Hand"; "hand")]
                #[test_case(AIMode::Group, "Group"; "group")]
                #[test_case(AIMode::Unknown, "Unknown"; "unknown")]
                fn ai_mode(mode: AIMode, expected: &str) {
                    assert_eq!(&mode.to_string(), expected);
                }
            }

            mod tracking_speed {
                use crate::TrackingSpeed;
                use test_case::test_case;

                #[test_case(TrackingSpeed::Standard, "Standard"; "standard")]
                #[test_case(TrackingSpeed::Sport, "Sport"; "sport")]
                fn tracking_speed(mode: TrackingSpeed, expected: &str) {
                    assert_eq!(&mode.to_string(), expected);
                }
            }

            mod exposure_mode {
                use crate::ExposureMode;
                use test_case::test_case;

                #[test_case(ExposureMode::Manual, "Manual"; "manual")]
                #[test_case(ExposureMode::Global, "Global"; "global")]
                #[test_case(ExposureMode::Face, "Face"; "face")]
                fn exposure_mode(mode: ExposureMode, expected: &str) {
                    assert_eq!(&mode.to_string(), expected);
                }
            }

            mod auto_sleep_time {
                use crate::AutoSleepTime;
                use test_case::test_case;

                #[test_case(120, "120 s"; "enabled")]
                #[test_case(-120, "Off (120 s)"; "disabled keeps the time")]
                #[test_case(0, "Off"; "zero")]
                fn auto_sleep_time(seconds: i16, expected: &str) {
                    assert_eq!(&AutoSleepTime(seconds).to_string(), expected);
                }
            }

            mod field_of_view {
                use crate::FieldOfView;
                use test_case::test_case;

                #[test_case(FieldOfView::Wide, "Wide"; "wide")]
                #[test_case(FieldOfView::Medium, "Medium"; "medium")]
                #[test_case(FieldOfView::Narrow, "Narrow"; "narrow")]
                #[test_case(FieldOfView::Unknown, "Unknown"; "unknown")]
                fn field_of_view(fov: FieldOfView, expected: &str) {
                    assert_eq!(&fov.to_string(), expected);
                }
            }

            mod noise_reduction {
                use crate::NoiseReduction;
                use test_case::test_case;

                #[test_case(NoiseReduction::Off, "Off"; "off")]
                #[test_case(NoiseReduction::Weak, "Weak"; "weak")]
                #[test_case(NoiseReduction::Medium, "Medium"; "medium")]
                #[test_case(NoiseReduction::Strong, "Strong"; "strong")]
                #[test_case(NoiseReduction::Unknown, "Unknown"; "unknown")]
                fn noise_reduction(level: NoiseReduction, expected: &str) {
                    assert_eq!(&level.to_string(), expected);
                }
            }

            mod pickup_distance {
                use crate::PickupDistance;
                use test_case::test_case;

                #[test_case(PickupDistance::Near, "Near"; "near")]
                #[test_case(PickupDistance::Standard, "Standard"; "standard")]
                #[test_case(PickupDistance::Far, "Far"; "far")]
                #[test_case(PickupDistance::Unknown, "Unknown"; "unknown")]
                fn pickup_distance(distance: PickupDistance, expected: &str) {
                    assert_eq!(&distance.to_string(), expected);
                }
            }

            mod sleep_background {
                use crate::SleepBackground;
                use test_case::test_case;

                #[test_case(SleepBackground::Black, "Black"; "black")]
                #[test_case(SleepBackground::Image, "Image"; "image")]
                #[test_case(SleepBackground::Unknown, "Unknown"; "unknown")]
                fn sleep_background(background: SleepBackground, expected: &str) {
                    assert_eq!(&background.to_string(), expected);
                }
            }
        }

        mod auto_sleep_time {
            use crate::AutoSleepTime;
            use test_case::test_case;

            // Turning auto-sleep off must keep the time (negated) so the camera, like
            // OBSBOT Center, can restore it when auto-sleep is turned on again.
            #[test_case(120, -120; "enabled time is negated")]
            #[test_case(600, -600; "other enabled time is negated")]
            #[test_case(-30, -30; "already disabled stays as it is")]
            #[test_case(0, -120; "no time falls back to the factory default")]
            fn disabled(current: i16, expected: i16) {
                assert_eq!(AutoSleepTime(current).disabled(), AutoSleepTime(expected));
            }

            #[test_case(120, true; "positive is enabled")]
            #[test_case(-120, false; "negative is disabled")]
            #[test_case(0, false; "zero is disabled")]
            fn is_enabled(seconds: i16, expected: bool) {
                assert_eq!(AutoSleepTime(seconds).is_enabled(), expected);
            }
        }
    }
}
