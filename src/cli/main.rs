// SPDX-License-Identifier: EUPL-1.2

use clap::{ArgGroup, Parser, Subcommand, ValueEnum};
use clap_complete::generate;
use dialoguer::{Confirm, FuzzySelect, Select};
use rust_i18n::{i18n, set_locale, t};
use std::borrow::Cow;
use tiny4linux::{
    AIMode, AutoSleepTime, Camera, FieldOfView, Gesture, GestureZoomRatioCommand, NoiseReduction,
    PickupDistance, SleepBackground, SleepMode, Tiny2Camera, get_language,
};

i18n!("src/locales", fallback = "en");

#[derive(Parser)]
#[command(name = "t4l", bin_name = "t4l", version, about, long_about = None, disable_version_flag = true)]
struct Args {
    #[command(subcommand)]
    subcommand: Command,
    #[arg(short, long, help = t!("cli.help.args.debug_logging"), global = true)]
    verbose: bool,
    // lang is being evaluated manually (without clap)
    #[arg(long, help = t!("cli.help.args.lang"), global = true)]
    lang: Option<String>,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = t!("cli.help.sleep"))]
    Sleep,
    #[command(about = t!("cli.help.wake"))]
    Wake,
    #[command(hide = true, subcommand_required = false, about = t!("cli.help.turn.command"))]
    Turn {
        #[command(subcommand)]
        action: Option<OnOffArg>,
    },
    #[command(alias = "track", subcommand_required = false, about = t!("cli.help.tracking"))]
    Tracking {
        #[command(subcommand)]
        tracking_mode: Option<TrackingArg>,
    },
    #[command(alias = "tracking-speed", subcommand_required = false, about = t!("cli.help.speed"))]
    Speed {
        #[command(subcommand)]
        speed: Option<TrackingSpeedArg>,
    },
    #[command(alias = "position", subcommand_required = false, about = t!("cli.help.preset"))]
    Preset { position_id: Option<i8> },
    #[command(about = t!("cli.help.preset_save"))]
    PresetSave { name: String },
    #[command(about = t!("cli.help.preset_recall"))]
    PresetRecall { name: String },
    #[command(about = t!("cli.help.preset_list"))]
    PresetList,
    #[command(about = t!("cli.help.preset_delete"))]
    PresetDelete { name: String },
    #[command(about = t!("cli.help.hdr"))]
    Hdr {
        #[command(subcommand)]
        hdr_mode: Option<OnOffArg>,
    },
    #[command(about = t!("cli.help.exposure"))]
    Exposure {
        #[command(subcommand)]
        exposure_mode: Option<ExposureArg>,
    },
    #[command(about = t!("cli.help.zoom"))]
    Zoom { value: Option<i32> },
    #[command(about = t!("cli.help.pan"))]
    Pan { value: Option<i32> },
    #[command(about = t!("cli.help.tilt"))]
    Tilt { value: Option<i32> },
    #[command(about = t!("cli.help.auto_sleep"))]
    AutoSleep {
        #[arg(value_parser = parse_auto_sleep, help = t!("cli.help.args.auto_sleep_time"))]
        time: AutoSleepArg,
    },
    #[command(about = t!("cli.help.sleep_background"))]
    #[command(group(ArgGroup::new("setting").required(true).multiple(true).args(["background", "mirror"])))]
    SleepBackground {
        #[arg(value_enum)]
        background: Option<SleepBackgroundArg>,
        #[arg(long, value_enum, help = t!("cli.help.args.sleep_background_mirror"))]
        mirror: Option<OnOffArg>,
    },
    #[command(about = t!("cli.help.led"))]
    Led {
        #[arg(value_enum)]
        brightness: LedArg,
    },
    #[command(alias = "field-of-view", about = t!("cli.help.fov"))]
    Fov {
        #[arg(value_enum)]
        fov: FovArg,
    },
    #[command(about = t!("cli.help.mirror"))]
    Mirror {
        #[arg(value_enum)]
        state: OnOffArg,
    },
    #[command(about = t!("cli.help.invert"))]
    Invert {
        #[arg(value_enum)]
        state: OnOffArg,
    },
    #[command(about = t!("cli.help.audio.command"))]
    Audio {
        #[command(subcommand)]
        setting: AudioArg,
    },
    #[command(alias = "gestures", about = t!("cli.help.gesture.command"))]
    Gesture {
        #[command(subcommand)]
        setting: GestureArg,
    },
    #[command(about = t!("cli.help.factory_reset"))]
    FactoryReset {
        #[arg(long, short, help = t!("cli.help.args.yes"))]
        yes: bool,
    },
    #[command(about = t!("cli.help.info"))]
    Info,
    #[command(about = t!("cli.help.version"))]
    Version,
    #[command(about = t!("cli.help.completions"))]
    Completions { shell: clap_complete::Shell },
}

#[derive(Subcommand, ValueEnum, Clone)]
enum OnOffArg {
    On,
    Off,
}

#[derive(Subcommand, Clone)]
enum TrackingArg {
    #[command(aliases = ["none", "off"])]
    Static,
    #[command(aliases = ["standard", "on"])]
    Normal,
    #[command(alias = "close")]
    CloseUp,
    UpperBody,
    Headless,
    LowerBody,
    Desk,
    Whiteboard,
    #[command(alias = "point")]
    Hand,
    Group,
}

#[derive(Subcommand, Clone)]
enum TrackingSpeedArg {
    #[command(aliases = ["normal", "default", "slow", "low"])]
    Standard,
    #[command(aliases = ["sport", "high"])]
    Fast,
}

#[derive(Subcommand, Clone)]
enum ExposureArg {
    Manual,
    Global,
    Face,
}

#[derive(Clone, Debug, PartialEq)]
enum AutoSleepArg {
    Off,
    Seconds(i16),
}

#[derive(ValueEnum, Clone)]
enum LedArg {
    Off,
    #[value(name = "1")]
    Dim,
    #[value(name = "2")]
    Medium,
    #[value(name = "3")]
    Bright,
}

#[derive(ValueEnum, Clone)]
enum FovArg {
    Wide,
    Medium,
    Narrow,
}

#[derive(Subcommand, Clone)]
enum AudioArg {
    #[command(about = t!("cli.help.audio.noise_reduction"))]
    NoiseReduction {
        #[arg(value_enum)]
        level: NoiseReductionArg,
    },
    #[command(about = t!("cli.help.audio.auto_gain"))]
    AutoGain {
        #[arg(value_enum)]
        state: OnOffArg,
    },
    #[command(about = t!("cli.help.audio.pickup_distance"))]
    PickupDistance {
        #[arg(value_enum)]
        distance: PickupDistanceArg,
    },
    #[command(about = t!("cli.help.audio.mic_in_sleep"))]
    MicInSleep {
        #[arg(value_enum)]
        state: OnOffArg,
    },
    #[command(alias = "microphone", about = t!("cli.help.audio.mic"))]
    Mic {
        #[arg(value_enum)]
        state: OnOffArg,
    },
}

#[derive(Subcommand, Clone)]
enum GestureArg {
    #[command(about = t!("cli.help.gesture.target_lock"))]
    TargetLock {
        #[arg(value_enum)]
        state: OnOffArg,
    },
    #[command(about = t!("cli.help.gesture.zoom"))]
    Zoom {
        #[arg(value_enum)]
        state: OnOffArg,
    },
    #[command(about = t!("cli.help.gesture.dynamic_zoom"))]
    DynamicZoom {
        #[arg(value_enum)]
        state: OnOffArg,
    },
    #[command(alias = "direction-change", about = t!("cli.help.gesture.direction"))]
    Direction {
        #[arg(value_enum)]
        state: OnOffArg,
    },
    #[command(about = t!("cli.help.gesture.zoom_ratio"))]
    ZoomRatio {
        #[arg(value_parser = parse_zoom_ratio)]
        ratio: f32,
    },
}

/// Parses the zoom ratio of the zoom gesture, 1.0 to 4.0.
fn parse_zoom_ratio(value: &str) -> Result<f32, String> {
    value
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|ratio| {
            (GestureZoomRatioCommand::MIN_RATIO..=GestureZoomRatioCommand::MAX_RATIO)
                .contains(ratio)
        })
        .ok_or_else(|| t!("cli.errors.invalid_zoom_ratio").to_string())
}

#[derive(ValueEnum, Clone)]
enum NoiseReductionArg {
    Off,
    Weak,
    Medium,
    Strong,
}

#[derive(ValueEnum, Clone)]
enum PickupDistanceArg {
    Near,
    Standard,
    Far,
}

#[derive(ValueEnum, Clone)]
enum SleepBackgroundArg {
    Black,
    Image,
}

/// Parses `off`, plain seconds (`90`), seconds with unit (`30s`) or minutes (`2m`, `10m`).
fn parse_auto_sleep(value: &str) -> Result<AutoSleepArg, String> {
    let value = value.trim().to_lowercase();
    if value == "off" {
        return Ok(AutoSleepArg::Off);
    }

    let (number, factor) = if let Some(minutes) = value.strip_suffix('m') {
        (minutes, 60)
    } else {
        (value.strip_suffix('s').unwrap_or(&value), 1)
    };

    number
        .parse::<i16>()
        .ok()
        .and_then(|n| n.checked_mul(factor))
        .filter(|seconds| *seconds > 0)
        .map(AutoSleepArg::Seconds)
        .ok_or_else(|| t!("cli.errors.invalid_auto_sleep").to_string())
}

fn on_off(state: bool) -> Cow<'static, str> {
    if state {
        t!("display.states.on")
    } else {
        t!("display.states.off")
    }
}

fn led_brightness(brightness: u8) -> Cow<'static, str> {
    match brightness {
        0 => t!("display.states.off"),
        level => Cow::Owned(level.to_string()),
    }
}

/// Whether the command changes one of the settings added in #174, see
/// [`Camera::supports_extended_settings`].
fn is_extended_setting(command: &Command) -> bool {
    matches!(
        command,
        Command::AutoSleep { .. }
            | Command::SleepBackground { .. }
            | Command::Led { .. }
            | Command::Fov { .. }
            | Command::Mirror { .. }
            | Command::Invert { .. }
            | Command::Audio { .. }
            | Command::Gesture { .. }
            | Command::FactoryReset { .. }
    )
}

fn print_setting(setting: Cow<str>, value: impl std::fmt::Display) {
    println!(
        "{}",
        t!(
            "cli.settings.response",
            setting = setting,
            value = value.to_string()
        )
    );
}

fn main() {
    let language = get_language(false);
    set_locale(language.as_str());

    let args = Args::parse();

    match &args.subcommand {
        Command::Version => {
            println!("t4l version: {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        Command::Completions { shell } => {
            use clap::CommandFactory;
            generate(*shell, &mut Args::command(), "t4l", &mut std::io::stdout());
            return;
        }
        _ => {}
    }

    let mut camera = Camera::detect().ok();

    if camera.is_none() {
        eprintln!("{}", t!("shared.errors.no_camera"));
        std::process::exit(1);
    }

    if args.verbose {
        camera.as_mut().unwrap().set_debugging(true);
    }

    let camera = camera.unwrap();

    // The settings from #174 were verified on a Tiny 2 Lite. The Tiny 4K's legacy
    // protocol has none of them, so it is refused instead of being sent frames it
    // would ignore.
    if is_extended_setting(&args.subcommand) && !camera.supports_extended_settings() {
        unsupported(&t!("shared.errors.unsupported_extended_setting"));
    }

    match &args.subcommand {
        Command::Turn { action } => evaluate_sleep_arg(action.clone(), camera),
        Command::Sleep => evaluate_sleep_arg(Option::from(OnOffArg::Off), camera),
        Command::Wake => evaluate_sleep_arg(Option::from(OnOffArg::On), camera),
        Command::Tracking { tracking_mode } => evaluate_tracking_arg(tracking_mode.clone(), camera),
        Command::Speed { speed } => evaluate_speed_arg(speed.clone(), camera),
        Command::Preset { position_id } => evaluate_preset_arg(*position_id, camera),
        Command::PresetSave { name } => evaluate_preset_save_arg(name.clone(), camera),
        Command::PresetRecall { name } => evaluate_preset_recall_arg(name.clone(), camera),
        Command::PresetList => evaluate_preset_list_arg(camera),
        Command::PresetDelete { name } => evaluate_preset_delete_arg(name.clone(), camera),
        Command::Hdr { hdr_mode } => evaluate_hdr_arg(hdr_mode.clone(), camera),
        Command::Exposure { exposure_mode } => evaluate_exposure_arg(exposure_mode.clone(), camera),
        Command::Zoom { value } => evaluate_zoom_arg(*value, camera),
        Command::Pan { value } => evaluate_pan_arg(*value, camera),
        Command::Tilt { value } => evaluate_tilt_arg(*value, camera),
        Command::Led { brightness } => {
            let brightness = match brightness {
                LedArg::Off => 0,
                LedArg::Dim => 1,
                LedArg::Medium => 2,
                LedArg::Bright => 3,
            };
            print_setting(t!("shared.info.led"), led_brightness(brightness));
            camera.set_led_brightness(brightness).unwrap();
        }
        Command::Fov { fov } => {
            let fov = match fov {
                FovArg::Wide => FieldOfView::Wide,
                FovArg::Medium => FieldOfView::Medium,
                FovArg::Narrow => FieldOfView::Narrow,
            };
            print_setting(t!("shared.info.field_of_view"), fov);
            camera.set_field_of_view(fov).unwrap();
        }
        Command::Mirror { state } => {
            let mirrored = matches!(state, OnOffArg::On);
            print_setting(t!("shared.info.mirror"), on_off(mirrored));
            camera.set_mirror(mirrored).unwrap();
        }
        Command::Invert { state } => {
            let inverted = matches!(state, OnOffArg::On);
            print_setting(t!("shared.info.invert"), on_off(inverted));
            camera.set_invert(inverted).unwrap();
        }
        Command::Audio { setting } => evaluate_audio_arg(setting.clone(), camera),
        Command::Gesture { setting } => evaluate_gesture_arg(setting.clone(), camera),
        Command::FactoryReset { yes } => evaluate_factory_reset(*yes, camera),
        Command::AutoSleep { time } => evaluate_auto_sleep_arg(time.clone(), camera),
        Command::SleepBackground { background, mirror } => {
            evaluate_sleep_background_arg(background.clone(), mirror.clone(), camera)
        }
        Command::Info => {
            let info = camera.get_status();

            if info.is_err() {
                println!("{}", t!("cli.errors.info_error"),);
                return;
            } else {
                let info = info.unwrap();

                println!("{}:", t!("shared.info.camera_status"));
                println!("  💤  {}: {}", t!("shared.info.sleep_mode"), info.awake);
                println!(
                    "  🤖  {}: {}",
                    t!("shared.info.ai_mode"),
                    camera.get_ai_mode().unwrap_or(AIMode::Unknown)
                );
                if camera.supports_tracking_speed() {
                    println!("  🏃  {}: {}", t!("shared.info.tracking_speed"), info.speed);
                } else {
                    println!(
                        "  🏃  {}: {}",
                        t!("shared.info.tracking_speed"),
                        t!("shared.info.not_available")
                    );
                }
                println!("  💐  {}: {}", t!("shared.info.hdr"), info.hdr_on);
                // The offsets of these fields were measured on a Tiny 2 Lite only (#174)
                if camera.reports_extended_status() {
                    println!(
                        "  🔭  {}: {}",
                        t!("shared.info.field_of_view"),
                        info.field_of_view
                    );
                    println!(
                        "  ↔️  {}: {}",
                        t!("shared.info.mirror"),
                        on_off(info.mirror)
                    );
                    println!(
                        "  💡  {}: {}",
                        t!("shared.info.led"),
                        led_brightness(info.led_brightness)
                    );
                    println!(
                        "  ✋  {}: {}",
                        t!("shared.info.gesture_target_lock"),
                        on_off(info.gesture_target_lock)
                    );
                    println!(
                        "  🎙️  {}: {}",
                        t!("shared.info.noise_reduction"),
                        info.noise_reduction
                    );
                    println!(
                        "  🎚️  {}: {}",
                        t!("shared.info.auto_gain"),
                        on_off(info.auto_gain)
                    );
                    println!(
                        "  📏  {}: {}",
                        t!("shared.info.pickup_distance"),
                        info.pickup_distance
                    );
                    println!(
                        "  😴  {}: {}",
                        t!("shared.info.mic_in_sleep"),
                        on_off(info.mic_in_sleep)
                    );
                    println!(
                        "  ⏲️  {}: {}",
                        t!("shared.info.auto_sleep"),
                        info.auto_sleep
                    );
                    println!(
                        "  🌙  {}: {}",
                        t!("shared.info.sleep_background"),
                        info.sleep_background
                    );
                    println!(
                        "  🪞  {}: {}",
                        t!("shared.info.sleep_background_mirror"),
                        on_off(info.sleep_background_mirror)
                    );
                } else {
                    println!(
                        "  ⚙️  {}: {}",
                        t!("shared.info.extended_status"),
                        t!("shared.info.not_available")
                    );
                }
            }
        }
        Command::Version => {
            println!("t4l version: {}", env!("CARGO_PKG_VERSION"));
        }
        Command::Completions { shell } => {
            use clap::CommandFactory;
            let mut cmd = Args::command();
            generate(*shell, &mut cmd, "t4l", &mut std::io::stdout());
        }
    }
}

struct SelectionOption<'a, T> {
    result: T,
    option: &'a str,
}

fn evaluate_sleep_arg(state: Option<OnOffArg>, camera: Camera) {
    match state {
        Some(OnOffArg::Off) => {
            println!("{}", t!("cli.sleep.response_to_sleep"));
            camera.set_sleep_mode(SleepMode::Sleep).unwrap();
        }
        Some(OnOffArg::On) => {
            println!("{}", t!("cli.sleep.response_to_awake"));
            camera.set_sleep_mode(SleepMode::Awake).unwrap();
        }
        None => {
            let option_on1 = t!("cli.sleep.option_on1");
            let option_off1 = t!("cli.sleep.option_off1");
            let option_on2 = t!("cli.sleep.option_on2");
            let option_off2 = t!("cli.sleep.option_off2");
            let option_off3 = t!("cli.sleep.option_off3");

            let options = [
                SelectionOption {
                    result: OnOffArg::On,
                    option: &option_on1,
                },
                SelectionOption {
                    result: OnOffArg::Off,
                    option: &option_off1,
                },
                SelectionOption {
                    result: OnOffArg::On,
                    option: &option_on2,
                },
                SelectionOption {
                    result: OnOffArg::Off,
                    option: &option_off2,
                },
                SelectionOption {
                    result: OnOffArg::Off,
                    option: &option_off3,
                },
            ];
            let selection = FuzzySelect::new()
                .with_prompt(format!("{}", t!("cli.sleep.request_should_be")))
                .default(0)
                .items(
                    &options
                        .iter()
                        .map(|option| option.option)
                        .collect::<Vec<&str>>(),
                )
                .interact()
                .unwrap();

            evaluate_sleep_arg(Option::from(options[selection].result.clone()), camera);
        }
    }
}

/// Applies an AI tracking mode, or reports that the connected camera does not have it.
///
/// The Tiny 4K only knows a part of the Tiny 2 modes, and silently ignoring the
/// rest is what this whole change is trying to get rid of (#72).
fn apply_ai_mode(camera: &Camera, mode: AIMode, message: &str) {
    if !camera.supports_ai_mode(mode) {
        eprintln!("{}", t!("shared.errors.unsupported_tracking_mode"));
        std::process::exit(1);
    }

    println!("{message}");
    camera.set_ai_mode(mode).unwrap();
}

/// Reports that the connected camera does not have a feature, and exits.
fn unsupported(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(1);
}

fn evaluate_tracking_arg(tracking_mode: Option<TrackingArg>, camera: Camera) {
    let response_setting_to = t!("cli.tracking_mode.response_setting_to");
    let static_mode = t!("cli.tracking_mode.static");
    let normal = t!("cli.tracking_mode.normal");
    let close_up = t!("cli.tracking_mode.close_up");
    let upper_body = t!("cli.tracking_mode.upper_body");
    let headless = t!("cli.tracking_mode.headless");
    let lower_body = t!("cli.tracking_mode.lower_body");
    let desk = t!("cli.tracking_mode.desk");
    let whiteboard = t!("cli.tracking_mode.whiteboard");
    let hand = t!("cli.tracking_mode.hand");
    let group = t!("cli.tracking_mode.group");

    match tracking_mode {
        Some(TrackingArg::Static) => {
            apply_ai_mode(
                &camera,
                AIMode::NoTracking,
                &format!("{} {}", response_setting_to, static_mode),
            );
        }
        Some(TrackingArg::Normal) => {
            apply_ai_mode(
                &camera,
                AIMode::NormalTracking,
                &format!("{} {}", response_setting_to, normal),
            );
        }
        Some(TrackingArg::CloseUp) => {
            apply_ai_mode(
                &camera,
                AIMode::CloseUp,
                &format!("{} {}", response_setting_to, close_up),
            );
        }
        Some(TrackingArg::UpperBody) => {
            apply_ai_mode(
                &camera,
                AIMode::UpperBody,
                &format!("{} {}", response_setting_to, upper_body),
            );
        }
        Some(TrackingArg::Headless) => {
            apply_ai_mode(
                &camera,
                AIMode::Headless,
                &format!("{} {}", response_setting_to, headless),
            );
        }
        Some(TrackingArg::LowerBody) => {
            apply_ai_mode(
                &camera,
                AIMode::LowerBody,
                &format!("{} {}", response_setting_to, lower_body),
            );
        }
        Some(TrackingArg::Desk) => {
            apply_ai_mode(
                &camera,
                AIMode::DeskMode,
                &format!("{} {}", response_setting_to, desk),
            );
        }
        Some(TrackingArg::Whiteboard) => {
            apply_ai_mode(
                &camera,
                AIMode::Whiteboard,
                &format!("{} {}", response_setting_to, whiteboard),
            );
        }
        Some(TrackingArg::Hand) => {
            apply_ai_mode(
                &camera,
                AIMode::Hand,
                &format!("{} {}", response_setting_to, hand),
            );
        }
        Some(TrackingArg::Group) => {
            apply_ai_mode(
                &camera,
                AIMode::Group,
                &format!("{} {}", response_setting_to, group),
            );
        }
        None => {
            let options = [
                SelectionOption {
                    result: TrackingArg::Static,
                    option: &static_mode,
                },
                SelectionOption {
                    result: TrackingArg::Normal,
                    option: &normal,
                },
                SelectionOption {
                    result: TrackingArg::CloseUp,
                    option: &close_up,
                },
                SelectionOption {
                    result: TrackingArg::UpperBody,
                    option: &upper_body,
                },
                SelectionOption {
                    result: TrackingArg::Headless,
                    option: &headless,
                },
                SelectionOption {
                    result: TrackingArg::LowerBody,
                    option: &lower_body,
                },
                SelectionOption {
                    result: TrackingArg::Desk,
                    option: &desk,
                },
                SelectionOption {
                    result: TrackingArg::Whiteboard,
                    option: &whiteboard,
                },
                SelectionOption {
                    result: TrackingArg::Hand,
                    option: &hand,
                },
                SelectionOption {
                    result: TrackingArg::Group,
                    option: &group,
                },
            ];
            let selection = FuzzySelect::new()
                .with_prompt(t!("cli.tracking_mode.request_should_be"))
                .default(0)
                .items(
                    &options
                        .iter()
                        .map(|option| option.option)
                        .collect::<Vec<&str>>(),
                )
                .interact()
                .unwrap();

            evaluate_tracking_arg(Option::from(options[selection].result.clone()), camera);
        }
    }
}

fn evaluate_speed_arg(speed: Option<TrackingSpeedArg>, camera: Camera) {
    if !camera.supports_tracking_speed() {
        unsupported(&t!("shared.errors.unsupported_tracking_speed"));
    }

    match speed {
        Some(TrackingSpeedArg::Standard) => {
            println!("{}", t!("cli.tracking_speed.response_to_standard"));
            camera
                .set_tracking_speed(tiny4linux::TrackingSpeed::Standard)
                .unwrap();
        }
        Some(TrackingSpeedArg::Fast) => {
            println!("{}", t!("cli.tracking_speed.response_to_fast"));
            camera
                .set_tracking_speed(tiny4linux::TrackingSpeed::Sport)
                .unwrap();
        }
        None => {
            let option_standard = t!("cli.tracking_speed.option_standard");
            let option_sport = t!("cli.tracking_speed.option_sport");

            let options = [
                SelectionOption {
                    result: TrackingSpeedArg::Standard,
                    option: &option_standard,
                },
                SelectionOption {
                    result: TrackingSpeedArg::Fast,
                    option: &option_sport,
                },
            ];
            let selection = FuzzySelect::new()
                .with_prompt(t!("cli.tracking_speed.request_should_be"))
                .default(0)
                .items(
                    &options
                        .iter()
                        .map(|option| option.option)
                        .collect::<Vec<&str>>(),
                )
                .interact()
                .unwrap();

            evaluate_speed_arg(Option::from(options[selection].result.clone()), camera);
        }
    }
}

fn evaluate_zoom_arg(value: Option<i32>, camera: Camera) {
    match value {
        Some(value) => {
            println!("{}", t!("cli.zoom.response_to_set", value = value));
            camera.set_zoom_absolute(value).unwrap();
        }
        None => {
            let (min, max) = camera.get_zoom_range().unwrap();
            let current = camera.get_zoom_absolute().unwrap();

            println!(
                "{}",
                t!("cli.zoom.current", current = current, min = min, max = max)
            );
        }
    }
}

fn evaluate_pan_arg(value: Option<i32>, camera: Camera) {
    match value {
        Some(value) => {
            println!("{}", t!("cli.pan.response_to_set", value = value));
            camera.set_pan_absolute(value).unwrap();
        }
        None => {
            let (min, max) = camera.get_pan_range().unwrap();
            let current = camera.get_pan_absolute().unwrap();

            println!(
                "{}",
                t!("cli.pan.current", current = current, min = min, max = max)
            );
        }
    }
}

fn evaluate_tilt_arg(value: Option<i32>, camera: Camera) {
    match value {
        Some(value) => {
            println!("{}", t!("cli.tilt.response_to_set", value = value));
            camera.set_tilt_absolute(value).unwrap();
        }
        None => {
            let (min, max) = camera.get_tilt_range().unwrap();
            let current = camera.get_tilt_absolute().unwrap();

            println!(
                "{}",
                t!("cli.tilt.current", current = current, min = min, max = max)
            );
        }
    }
}

fn evaluate_preset_arg(position_id: Option<i8>, camera: Camera) {
    if position_id.is_none() {
        let options = [1, 2, 3];
        let selection = Select::new()
            .with_prompt(t!("cli.preset_position.request_position_id"))
            .default(0)
            .items(options)
            .interact()
            .unwrap();

        return evaluate_preset_arg(Option::from(options[selection]), camera);
    }

    if !matches!(position_id, Some(1..=3)) {
        unsupported(&t!("shared.errors.unknown_preset_position"));
    }

    println!("{}", t!("cli.preset_position.stopping_tracking"));
    camera.set_ai_mode(AIMode::NoTracking).unwrap();

    println!(
        "{}",
        t!(
            "cli.preset_position.response_to_position",
            position_id = position_id.unwrap()
        ),
    );
    if camera
        .goto_preset_position(position_id.unwrap() - 1)
        .is_err()
    {
        unsupported(&t!("shared.errors.unknown_preset_position"));
    }
}

fn evaluate_preset_save_arg(name: String, camera: Camera) {
    match camera.save_custom_preset(&name) {
        Ok(_) => println!("{}", t!("cli.preset_save.response_to_save", name = name)),
        Err(e) => unsupported(&e.to_string()),
    }
}

fn evaluate_preset_recall_arg(name: String, camera: Camera) {
    match camera.recall_custom_preset(&name) {
        Ok(_) => println!(
            "{}",
            t!("cli.preset_recall.response_to_recall", name = name)
        ),
        Err(e) => unsupported(&e.to_string()),
    }
}

fn evaluate_preset_list_arg(camera: Camera) {
    let presets = camera.get_custom_presets();

    if presets.is_empty() {
        println!("{}", t!("cli.preset_list.empty"));
        return;
    }

    for preset in presets {
        println!(
            "{}",
            t!(
                "cli.preset_list.entry",
                name = preset.name,
                pan = preset.pan,
                tilt = preset.tilt,
                zoom = preset.zoom
            )
        );
    }
}

fn evaluate_preset_delete_arg(name: String, camera: Camera) {
    match camera.delete_custom_preset(&name) {
        Ok(_) => println!(
            "{}",
            t!("cli.preset_delete.response_to_delete", name = name)
        ),
        Err(e) => unsupported(&e.to_string()),
    }
}

fn evaluate_hdr_arg(hdr_mode: Option<OnOffArg>, camera: Camera) {
    match hdr_mode {
        Some(OnOffArg::On) => {
            println!("{}", t!("cli.hdr.response_to_hdr_on"));
            camera.set_hdr_mode(true).unwrap();
        }
        Some(OnOffArg::Off) => {
            println!("{}", t!("cli.hdr.response_to_hdr_off"));
            camera.set_hdr_mode(false).unwrap();
        }
        None => {
            let option_on = t!("shared.options.hdr.on");
            let option_off = t!("shared.options.hdr.off");

            let options = [
                SelectionOption {
                    result: OnOffArg::On,
                    option: &option_on,
                },
                SelectionOption {
                    result: OnOffArg::Off,
                    option: &option_off,
                },
            ];
            let selection = FuzzySelect::new()
                .with_prompt(t!("cli.hdr.request_should_be"))
                .default(0)
                .items(
                    &options
                        .iter()
                        .map(|option| option.option)
                        .collect::<Vec<&str>>(),
                )
                .interact()
                .unwrap();

            evaluate_hdr_arg(Option::from(options[selection].result.clone()), camera);
        }
    }
}

fn evaluate_exposure_arg(exposure_mode: Option<ExposureArg>, camera: Camera) {
    match exposure_mode {
        Some(ExposureArg::Manual) => {
            if !camera.supports_manual_exposure() {
                unsupported(&t!("shared.errors.unsupported_manual_exposure"));
            }

            println!("{}", t!("cli.exposure.response_to_manual"));
            camera
                .set_exposure_mode(tiny4linux::ExposureMode::Manual)
                .unwrap();
        }
        Some(ExposureArg::Global) => {
            println!("{}", t!("cli.exposure.response_to_global"));
            camera
                .set_exposure_mode(tiny4linux::ExposureMode::Global)
                .unwrap();
        }
        Some(ExposureArg::Face) => {
            println!("{}", t!("cli.exposure.response_to_face"));
            camera
                .set_exposure_mode(tiny4linux::ExposureMode::Face)
                .unwrap();
        }
        None => {
            let option_manual = t!("cli.exposure.option_manual");
            let option_global = t!("cli.exposure.option_global");
            let option_face = t!("cli.exposure.option_face");

            let options = [
                SelectionOption {
                    result: ExposureArg::Manual,
                    option: &option_manual,
                },
                SelectionOption {
                    result: ExposureArg::Global,
                    option: &option_global,
                },
                SelectionOption {
                    result: ExposureArg::Face,
                    option: &option_face,
                },
            ];
            let selection = FuzzySelect::new()
                .with_prompt(t!("cli.exposure.request_should_be"))
                .default(0)
                .items(
                    &options
                        .iter()
                        .map(|option| option.option)
                        .collect::<Vec<&str>>(),
                )
                .interact()
                .unwrap();

            evaluate_exposure_arg(Option::from(options[selection].result.clone()), camera);
        }
    }
}

fn evaluate_auto_sleep_arg(time: AutoSleepArg, camera: Camera) {
    match time {
        AutoSleepArg::Off => {
            print_setting(t!("shared.info.auto_sleep"), t!("display.states.off"));
            camera.disable_auto_sleep().unwrap();
        }
        AutoSleepArg::Seconds(seconds) => {
            let time = AutoSleepTime(seconds);
            print_setting(t!("shared.info.auto_sleep"), time);
            camera.set_auto_sleep(time).unwrap();
        }
    }
}

fn evaluate_sleep_background_arg(
    background: Option<SleepBackgroundArg>,
    mirror: Option<OnOffArg>,
    camera: Camera,
) {
    if let Some(background) = background {
        let background = match background {
            SleepBackgroundArg::Black => SleepBackground::Black,
            SleepBackgroundArg::Image => SleepBackground::Image,
        };
        print_setting(t!("shared.info.sleep_background"), background);
        camera.set_sleep_background(background).unwrap();
    }

    if let Some(mirror) = mirror {
        let mirrored = matches!(mirror, OnOffArg::On);
        print_setting(t!("shared.info.sleep_background_mirror"), on_off(mirrored));
        camera.set_sleep_background_mirror(mirrored).unwrap();
    }
}

fn evaluate_audio_arg(setting: AudioArg, camera: Camera) {
    match setting {
        AudioArg::NoiseReduction { level } => {
            let level = match level {
                NoiseReductionArg::Off => NoiseReduction::Off,
                NoiseReductionArg::Weak => NoiseReduction::Weak,
                NoiseReductionArg::Medium => NoiseReduction::Medium,
                NoiseReductionArg::Strong => NoiseReduction::Strong,
            };
            print_setting(t!("shared.info.noise_reduction"), level);
            camera.set_noise_reduction(level).unwrap();
        }
        AudioArg::AutoGain { state } => {
            let enabled = matches!(state, OnOffArg::On);
            print_setting(t!("shared.info.auto_gain"), on_off(enabled));
            camera.set_auto_gain(enabled).unwrap();
        }
        AudioArg::PickupDistance { distance } => {
            let distance = match distance {
                PickupDistanceArg::Near => PickupDistance::Near,
                PickupDistanceArg::Standard => PickupDistance::Standard,
                PickupDistanceArg::Far => PickupDistance::Far,
            };
            print_setting(t!("shared.info.pickup_distance"), distance);
            camera.set_pickup_distance(distance).unwrap();
        }
        AudioArg::MicInSleep { state } => {
            let enabled = matches!(state, OnOffArg::On);
            print_setting(t!("shared.info.mic_in_sleep"), on_off(enabled));
            camera.set_mic_in_sleep(enabled).unwrap();
        }
        AudioArg::Mic { state } => {
            let enabled = matches!(state, OnOffArg::On);
            print_setting(t!("shared.info.microphone"), on_off(enabled));
            println!("{}", t!("cli.audio.warning_restart"));
            camera.set_microphone(enabled).unwrap();
        }
    }
}

fn evaluate_gesture_arg(setting: GestureArg, camera: Camera) {
    let (gesture, state, label) = match setting {
        GestureArg::ZoomRatio { ratio } => {
            print_setting(t!("shared.info.gesture_zoom_ratio"), ratio);
            camera.set_gesture_zoom_ratio(ratio).unwrap();
            return;
        }
        GestureArg::TargetLock { state } => (
            Gesture::TargetLock,
            state,
            t!("shared.info.gesture_target_lock"),
        ),
        GestureArg::Zoom { state } => (Gesture::Zoom, state, t!("shared.info.gesture_zoom")),
        GestureArg::DynamicZoom { state } => (
            Gesture::DynamicZoom,
            state,
            t!("shared.info.gesture_dynamic_zoom"),
        ),
        GestureArg::Direction { state } => (
            Gesture::DirectionChange,
            state,
            t!("shared.info.gesture_direction"),
        ),
    };

    let enabled = matches!(state, OnOffArg::On);
    print_setting(label, on_off(enabled));
    camera.set_gesture(gesture, enabled).unwrap();
}

fn evaluate_factory_reset(confirmed: bool, camera: Camera) {
    println!("{}", t!("cli.factory_reset.warning"));

    let confirmed = confirmed
        || Confirm::new()
            .with_prompt(t!("cli.factory_reset.request_confirmation"))
            .default(false)
            .interact()
            .unwrap_or(false);

    if !confirmed {
        println!("{}", t!("cli.factory_reset.aborted"));
        return;
    }

    println!("{}", t!("cli.factory_reset.response"));
    camera.factory_reset().unwrap();
}

#[cfg(test)]
mod tests {
    use crate::{AutoSleepArg, parse_auto_sleep, parse_zoom_ratio};
    use test_case::test_case;

    #[test_case("off", AutoSleepArg::Off; "off")]
    #[test_case("30s", AutoSleepArg::Seconds(30); "seconds with unit")]
    #[test_case("2m", AutoSleepArg::Seconds(120); "minutes are converted to seconds")]
    #[test_case("10m", AutoSleepArg::Seconds(600); "ten minutes")]
    #[test_case("90", AutoSleepArg::Seconds(90); "plain seconds")]
    fn valid_auto_sleep(value: &str, expected: AutoSleepArg) {
        assert_eq!(parse_auto_sleep(value), Ok(expected));
    }

    // 0 and negative times would turn auto-sleep off on the camera, which `off` already does
    // while keeping the configured time; 32768 s and more do not fit the camera's i16.
    #[test_case("0"; "zero")]
    #[test_case("-5"; "negative")]
    #[test_case("600m"; "minutes overflowing i16")]
    #[test_case("32768"; "seconds overflowing i16")]
    #[test_case("soon"; "not a number")]
    fn invalid_auto_sleep(value: &str) {
        assert!(parse_auto_sleep(value).is_err());
    }

    #[test_case("1", 1.0; "minimum")]
    #[test_case("2.5", 2.5; "fraction")]
    #[test_case("4.0", 4.0; "maximum")]
    fn valid_zoom_ratio(value: &str, expected: f32) {
        assert_eq!(parse_zoom_ratio(value), Ok(expected));
    }

    // The camera accepts ratios from 1.0 to 4.0 only.
    #[test_case("0.5"; "below minimum")]
    #[test_case("4.5"; "above maximum")]
    #[test_case("NaN"; "not a number")]
    #[test_case("wide"; "text")]
    fn invalid_zoom_ratio(value: &str) {
        assert!(parse_zoom_ratio(value).is_err());
    }
}
