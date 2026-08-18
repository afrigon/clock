use std::path::PathBuf;

use ratatui::style::Color;
use serde::Deserialize;

const DEFAULT_TEMPLATE: &str = r##"# Colors default to the terminal's own colors; uncomment to override.
# background = "#2b3a55"
# foreground = "#f4ede4"
# master_volume = 0.5

[timer]
overtime_background = "#7f1d1d"
overtime_foreground = "#f4ede4"
chime_enabled = true
# chime_path = "/path/to/sound.wav"
# chime_volume = 1.0
# music_enabled = true
# music_path = "/path/to/a-file-or-directory"   default: the music directory next to this file
# music_order = "random"               "random" or "sequential", for directories
# music_volume = 0.7

[clock]
# format = "12h"                      default "24h"
# timezone = "America/New_York"       default: the system timezone
# date = "%A, %B %-d, %Y"             strftime pattern; "" hides the date
# message = "shown under the date"
"##;

const DEFAULT_OVERTIME_BACKGROUND: Color = Color::Rgb(0x7f, 0x1d, 0x1d);
const DEFAULT_OVERTIME_FOREGROUND: Color = Color::Rgb(0xf4, 0xed, 0xe4);

pub struct Configuration {
    pub background: Color,
    pub foreground: Color,
    pub master_volume: f32,
    pub timer: TimerConfiguration,
    pub clock: ClockConfiguration,
}

pub struct TimerConfiguration {
    pub overtime_background: Color,
    pub overtime_foreground: Color,
    pub chime_enabled: bool,
    pub chime_path: Option<PathBuf>,
    pub chime_volume: f32,
    pub music_enabled: bool,
    pub music_path: Option<PathBuf>,
    pub music_order: MusicOrder,
    pub music_volume: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MusicOrder {
    Random,
    Sequential,
}

pub struct ClockConfiguration {
    pub twelve_hour: bool,
    pub timezone: Option<String>,
    pub date_format: Option<String>,
    pub message: Option<String>,
}

const DEFAULT_DATE_FORMAT: &str = "%A, %B %-d, %Y";

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawConfiguration {
    background: Option<String>,
    foreground: Option<String>,
    master_volume: Option<f32>,
    timer: RawTimerConfiguration,
    clock: RawClockConfiguration,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawClockConfiguration {
    format: Option<String>,
    timezone: Option<String>,
    date: Option<String>,
    message: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawTimerConfiguration {
    overtime_background: Option<String>,
    overtime_foreground: Option<String>,
    chime_enabled: Option<bool>,
    chime_path: Option<PathBuf>,
    chime_volume: Option<f32>,
    music_enabled: Option<bool>,
    music_path: Option<PathBuf>,
    music_order: Option<String>,
    music_volume: Option<f32>,
}

pub fn load() -> Configuration {
    let Some(path) = configuration_path() else {
        return RawConfiguration::default().into();
    };
    match std::fs::read_to_string(&path) {
        Ok(contents) => match toml::from_str::<RawConfiguration>(&contents) {
            Ok(raw) => raw.into(),
            Err(error) => {
                eprintln!("invalid configuration at {}: {error}", path.display());
                RawConfiguration::default().into()
            }
        },
        Err(_) => {
            write_default(&path);
            RawConfiguration::default().into()
        }
    }
}

fn configuration_directory() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("clock"))
}

fn configuration_path() -> Option<PathBuf> {
    Some(configuration_directory()?.join("config.toml"))
}

fn write_default(path: &std::path::Path) {
    let written = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|_| std::fs::write(path, DEFAULT_TEMPLATE));
    if let Err(error) = written {
        eprintln!(
            "could not write default configuration to {}: {error}",
            path.display()
        );
    }
}

impl From<RawConfiguration> for Configuration {
    fn from(raw: RawConfiguration) -> Self {
        Self {
            background: color_or_default(raw.background, "background", Color::Reset),
            foreground: color_or_default(raw.foreground, "foreground", Color::Reset),
            master_volume: volume_or_default(raw.master_volume, "master_volume", 0.5),
            timer: TimerConfiguration {
                overtime_background: color_or_default(
                    raw.timer.overtime_background,
                    "timer.overtime_background",
                    DEFAULT_OVERTIME_BACKGROUND,
                ),
                overtime_foreground: color_or_default(
                    raw.timer.overtime_foreground,
                    "timer.overtime_foreground",
                    DEFAULT_OVERTIME_FOREGROUND,
                ),
                chime_enabled: raw.timer.chime_enabled.unwrap_or(true),
                chime_path: raw.timer.chime_path,
                chime_volume: volume_or_default(raw.timer.chime_volume, "timer.chime_volume", 1.0),
                music_enabled: raw.timer.music_enabled.unwrap_or(false),
                music_path: validated_music_path(raw.timer.music_path),
                music_order: music_order(raw.timer.music_order),
                music_volume: volume_or_default(raw.timer.music_volume, "timer.music_volume", 0.7),
            },
            clock: ClockConfiguration {
                twelve_hour: twelve_hour(raw.clock.format),
                timezone: validated_timezone(raw.clock.timezone),
                date_format: validated_date_format(raw.clock.date),
                message: raw.clock.message,
            },
        }
    }
}

fn twelve_hour(format: Option<String>) -> bool {
    match format.as_deref() {
        None | Some("24h") => false,
        Some("12h") => true,
        Some(other) => {
            eprintln!("invalid clock.format '{other}', expected \"24h\" or \"12h\"");
            false
        }
    }
}

fn volume_or_default(value: Option<f32>, field: &str, default: f32) -> f32 {
    match value {
        None => default,
        Some(volume) if volume >= 0.0 && volume.is_finite() => volume,
        Some(volume) => {
            eprintln!("invalid {field} {volume}, expected a value like 0.5, using {default}");
            default
        }
    }
}

fn validated_music_path(path: Option<PathBuf>) -> Option<PathBuf> {
    match path {
        Some(path) if path.exists() => Some(path),
        Some(path) => {
            eprintln!("timer.music_path {} does not exist", path.display());
            None
        }
        None => {
            let default = configuration_directory()?.join("music");
            default.exists().then_some(default)
        }
    }
}

fn music_order(order: Option<String>) -> MusicOrder {
    match order.as_deref() {
        None | Some("random") => MusicOrder::Random,
        Some("sequential") => MusicOrder::Sequential,
        Some(other) => {
            eprintln!("invalid timer.music_order '{other}', expected \"random\" or \"sequential\"");
            MusicOrder::Random
        }
    }
}

fn validated_date_format(date: Option<String>) -> Option<String> {
    let Some(pattern) = date else {
        return Some(DEFAULT_DATE_FORMAT.to_string());
    };
    if pattern.is_empty() {
        return None;
    }
    if jiff::fmt::strtime::format(&pattern, &jiff::Zoned::now()).is_ok() {
        Some(pattern)
    } else {
        eprintln!("invalid clock.date pattern '{pattern}', using the default");
        Some(DEFAULT_DATE_FORMAT.to_string())
    }
}

fn validated_timezone(timezone: Option<String>) -> Option<String> {
    let timezone = timezone?;
    if jiff::tz::TimeZone::get(&timezone).is_ok() {
        Some(timezone)
    } else {
        eprintln!("unknown clock.timezone '{timezone}', using the system timezone");
        None
    }
}

fn color_or_default(value: Option<String>, field: &str, default: Color) -> Color {
    let Some(value) = value else {
        return default;
    };
    match parse_hex_color(&value) {
        Ok(color) => color,
        Err(error) => {
            eprintln!("invalid {field} color '{value}': {error}");
            default
        }
    }
}

fn parse_hex_color(text: &str) -> Result<Color, String> {
    let digits = text
        .strip_prefix('#')
        .ok_or_else(|| "expected leading '#'".to_string())?;
    if digits.len() != 6 {
        return Err("expected six hex digits".to_string());
    }
    let component = |range: std::ops::Range<usize>| {
        u8::from_str_radix(&digits[range], 16).map_err(|_| "invalid hex digit".to_string())
    };
    Ok(Color::Rgb(
        component(0..2)?,
        component(2..4)?,
        component(4..6)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_colors() {
        assert_eq!(parse_hex_color("#2b3a55"), Ok(Color::Rgb(0x2b, 0x3a, 0x55)));
        assert!(parse_hex_color("2b3a55").is_err());
        assert!(parse_hex_color("#2b3a5").is_err());
        assert!(parse_hex_color("#2b3a5g").is_err());
    }

    #[test]
    fn default_template_matches_default_configuration() {
        let raw: RawConfiguration = toml::from_str(DEFAULT_TEMPLATE).unwrap();
        let configuration: Configuration = raw.into();
        assert_eq!(configuration.background, Color::Reset);
        assert_eq!(configuration.foreground, Color::Reset);
        assert_eq!(
            configuration.timer.overtime_background,
            DEFAULT_OVERTIME_BACKGROUND
        );
        assert_eq!(
            configuration.timer.overtime_foreground,
            DEFAULT_OVERTIME_FOREGROUND
        );
        assert!(configuration.timer.chime_enabled);
        assert!(configuration.timer.chime_path.is_none());
        assert!(!configuration.clock.twelve_hour);
        assert!(configuration.clock.timezone.is_none());
        assert_eq!(
            configuration.clock.date_format.as_deref(),
            Some(DEFAULT_DATE_FORMAT)
        );
        assert!(configuration.clock.message.is_none());
    }

    #[test]
    fn empty_date_pattern_hides_the_date() {
        let raw: RawConfiguration = toml::from_str("[clock]\ndate = \"\"\n").unwrap();
        let configuration: Configuration = raw.into();
        assert!(configuration.clock.date_format.is_none());
    }

    #[test]
    fn default_date_pattern_formats() {
        assert!(jiff::fmt::strtime::format(DEFAULT_DATE_FORMAT, &jiff::Zoned::now()).is_ok());
    }

    #[test]
    fn volume_defaults_and_validation() {
        let raw: RawConfiguration = toml::from_str("").unwrap();
        let configuration: Configuration = raw.into();
        assert_eq!(configuration.master_volume, 0.5);
        assert_eq!(configuration.timer.chime_volume, 1.0);
        assert_eq!(configuration.timer.music_volume, 0.7);
        assert!(!configuration.timer.music_enabled);
        assert_eq!(volume_or_default(Some(0.8), "field", 1.0), 0.8);
        assert_eq!(volume_or_default(Some(-0.2), "field", 1.0), 1.0);
    }

    #[test]
    fn music_order_parses() {
        assert_eq!(music_order(None), MusicOrder::Random);
        assert_eq!(music_order(Some("random".to_string())), MusicOrder::Random);
        assert_eq!(
            music_order(Some("sequential".to_string())),
            MusicOrder::Sequential
        );
        assert_eq!(music_order(Some("shuffled".to_string())), MusicOrder::Random);
    }

    #[test]
    fn clock_section_parses() {
        let raw: RawConfiguration =
            toml::from_str("[clock]\nformat = \"12h\"\ntimezone = \"America/New_York\"\n").unwrap();
        let configuration: Configuration = raw.into();
        assert!(configuration.clock.twelve_hour);
        assert_eq!(
            configuration.clock.timezone.as_deref(),
            Some("America/New_York")
        );
    }

    #[test]
    fn partial_file_fills_missing_fields_with_defaults() {
        let raw: RawConfiguration = toml::from_str("[timer]\nchime_enabled = false\n").unwrap();
        let configuration: Configuration = raw.into();
        assert_eq!(configuration.background, Color::Reset);
        assert!(!configuration.timer.chime_enabled);
    }
}
