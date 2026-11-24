//! Hand-written argument parsing (ADR 0022 item 5); the program has one subcommand.
use std::path::PathBuf;
use std::time::Duration;

use sweepscope_export::ExportFormat;

use crate::exit::{CliError, ExitStatus};

const DEFAULT_COUNT: u32 = 1;
const DEFAULT_TIMEOUT_S: f64 = 10.0;
const DEFAULT_DEVICE_WAIT_S: f64 = 8.0;

pub const USAGE: &str = "\
usage: sweepscope-cli capture --setup FILE [options]

options:
  --setup FILE        setup file to load (required)
  --count N           frames to write, default 1
  --out DIR           output folder, default the current folder
  --format csv|raw    file format, default csv
  --timeout S         seconds to wait for each frame, default 10
  --device-wait S     seconds to wait for the scope, default 8
  --simulator         use the built-in simulator instead of hardware
  --config-dir DIR    app config folder holding the stored zero offsets
  --help              show this text

exit codes: 0 done, 1 failure, 2 usage, 10 no device, 11 frame timeout, 12 permission denied";

#[derive(Debug, Clone, PartialEq)]
pub struct CaptureOptions {
    pub setup: PathBuf,
    pub count: u32,
    pub out: PathBuf,
    pub format: ExportFormat,
    pub timeout: Duration,
    pub device_wait: Duration,
    pub simulator: bool,
    pub config_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Capture(CaptureOptions),
    Help,
}

fn usage(message: impl Into<String>) -> CliError {
    CliError::new(ExitStatus::Usage, message)
}

pub fn parse(args: &[String]) -> Result<Command, CliError> {
    let mut args = args.iter();
    match args.next().map(String::as_str) {
        None => Err(usage("missing subcommand, expected `capture`")),
        Some("--help" | "-h") => Ok(Command::Help),
        Some("capture") => parse_capture(args.map(String::as_str)),
        Some(other) => Err(usage(format!("unknown subcommand {other}"))),
    }
}

fn parse_capture<'a>(mut args: impl Iterator<Item = &'a str>) -> Result<Command, CliError> {
    let mut options = CaptureOptions {
        setup: PathBuf::new(),
        count: DEFAULT_COUNT,
        out: PathBuf::from("."),
        format: ExportFormat::Csv,
        timeout: Duration::from_secs_f64(DEFAULT_TIMEOUT_S),
        device_wait: Duration::from_secs_f64(DEFAULT_DEVICE_WAIT_S),
        simulator: false,
        config_dir: None,
    };
    let mut setup = None;
    while let Some(name) = args.next() {
        match name {
            "--help" | "-h" => return Ok(Command::Help),
            "--simulator" => options.simulator = true,
            "--setup" => setup = Some(PathBuf::from(value(name, &mut args)?)),
            "--out" => options.out = PathBuf::from(value(name, &mut args)?),
            "--config-dir" => options.config_dir = Some(PathBuf::from(value(name, &mut args)?)),
            "--count" => options.count = count(value(name, &mut args)?)?,
            "--format" => options.format = format(value(name, &mut args)?)?,
            "--timeout" => options.timeout = seconds(name, value(name, &mut args)?)?,
            "--device-wait" => options.device_wait = seconds(name, value(name, &mut args)?)?,
            other => return Err(usage(format!("unknown option {other}"))),
        }
    }
    options.setup = setup.ok_or_else(|| usage("--setup is required"))?;
    Ok(Command::Capture(options))
}

fn value<'a>(name: &str, args: &mut impl Iterator<Item = &'a str>) -> Result<&'a str, CliError> {
    args.next().ok_or_else(|| usage(format!("{name} needs a value")))
}

fn count(text: &str) -> Result<u32, CliError> {
    match text.parse::<u32>() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(usage(format!(
            "--count must be a whole number of at least 1, got {text}"
        ))),
    }
}

fn format(text: &str) -> Result<ExportFormat, CliError> {
    match text {
        "csv" => Ok(ExportFormat::Csv),
        "raw" => Ok(ExportFormat::Raw),
        _ => Err(usage(format!("--format must be csv or raw, got {text}"))),
    }
}

fn seconds(name: &str, text: &str) -> Result<Duration, CliError> {
    match text.parse::<f64>() {
        Ok(s) if s.is_finite() && s > 0.0 && s <= 86_400.0 => Ok(Duration::from_secs_f64(s)),
        _ => Err(usage(format!(
            "{name} must be a positive number of seconds, got {text}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(line: &str) -> Result<Command, CliError> {
        let args: Vec<String> = line.split_whitespace().map(str::to_owned).collect();
        parse(&args)
    }

    fn options(line: &str) -> CaptureOptions {
        match parsed(line) {
            Ok(Command::Capture(options)) => options,
            other => panic!("expected capture options, got {other:?}"),
        }
    }

    fn usage_message(line: &str) -> String {
        let error = parsed(line).expect_err("a usage error");
        assert_eq!(error.status, ExitStatus::Usage);
        error.message
    }

    #[test]
    fn given_only_the_setup_when_parsed_then_the_defaults_apply() {
        let options = options("capture --setup s.json");

        assert_eq!(options.setup, PathBuf::from("s.json"));
        assert_eq!(options.count, 1);
        assert_eq!(options.out, PathBuf::from("."));
        assert_eq!(options.format, ExportFormat::Csv);
        assert_eq!(options.timeout, Duration::from_secs(10));
        assert_eq!(options.device_wait, Duration::from_secs(8));
        assert!(!options.simulator);
        assert_eq!(options.config_dir, None);
    }

    #[test]
    fn given_every_option_when_parsed_then_each_value_is_kept() {
        let options = options(
            "capture --setup s.json --count 3 --out o --format raw --timeout 0.5 --device-wait 2 --simulator --config-dir c",
        );

        assert_eq!(options.count, 3);
        assert_eq!(options.out, PathBuf::from("o"));
        assert_eq!(options.format, ExportFormat::Raw);
        assert_eq!(options.timeout, Duration::from_millis(500));
        assert_eq!(options.device_wait, Duration::from_secs(2));
        assert!(options.simulator);
        assert_eq!(options.config_dir, Some(PathBuf::from("c")));
    }

    #[test]
    fn given_no_setup_when_parsed_then_it_is_a_usage_error() {
        assert_eq!(usage_message("capture --count 2"), "--setup is required");
    }

    #[test]
    fn given_no_arguments_when_parsed_then_it_is_a_usage_error() {
        assert!(usage_message("").contains("missing subcommand"));
    }

    #[test]
    fn given_an_unknown_subcommand_when_parsed_then_it_is_a_usage_error() {
        assert_eq!(usage_message("record"), "unknown subcommand record");
    }

    #[test]
    fn given_an_unknown_option_when_parsed_then_it_is_a_usage_error() {
        assert_eq!(usage_message("capture --setup s.json --fast"), "unknown option --fast");
    }

    #[test]
    fn given_an_option_without_its_value_when_parsed_then_it_is_a_usage_error() {
        assert_eq!(usage_message("capture --setup"), "--setup needs a value");
    }

    #[test]
    fn given_a_count_that_is_zero_or_not_a_number_when_parsed_then_it_is_a_usage_error() {
        assert!(usage_message("capture --setup s --count 0").contains("--count"));
        assert!(usage_message("capture --setup s --count many").contains("--count"));
    }

    #[test]
    fn given_an_unknown_format_when_parsed_then_it_is_a_usage_error() {
        assert!(usage_message("capture --setup s --format png").contains("csv or raw"));
    }

    #[test]
    fn given_a_timeout_that_is_not_positive_when_parsed_then_it_is_a_usage_error() {
        assert!(usage_message("capture --setup s --timeout 0").contains("--timeout"));
        assert!(usage_message("capture --setup s --device-wait -1").contains("--device-wait"));
        assert!(usage_message("capture --setup s --timeout NaN").contains("--timeout"));
    }

    #[test]
    fn given_help_when_parsed_then_the_help_command_is_returned() {
        assert_eq!(parsed("--help"), Ok(Command::Help));
        assert_eq!(parsed("capture --help"), Ok(Command::Help));
    }
}
