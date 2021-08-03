use env_logger::WriteStyle;
use structopt::StructOpt;

use crate::error::LoggerError;

fn parse_write_style(spec: &str) -> Result<WriteStyle, LoggerError> {
  match spec {
    "auto" => Ok(WriteStyle::Auto),
    "always" => Ok(WriteStyle::Always),
    "never" => Ok(WriteStyle::Never),
    _ => Err(LoggerError::StyleParse),
  }
}

#[derive(StructOpt, Debug, Default, Clone, Copy)]
/// Logging options that can be used directly or via [StructOpt]
pub struct LoggingOptions {
  /// Disables logging
  #[structopt(long = "quiet", short = "q")]
  pub quiet: bool,

  /// Turns on verbose logging
  #[structopt(long = "verbose", short = "V")]
  pub verbose: bool,

  /// Turns on debug logging
  #[structopt(long = "debug")]
  pub debug: bool,

  /// Turns on trace logging
  #[structopt(long = "trace")]
  pub trace: bool,

  /// Log as JSON
  #[structopt(long = "json")]
  pub json: bool,

  /// Print log output as color. Options are auto | always | never.
  #[structopt(
        long = "color",
        env = "LOG_COLOR",
        parse(try_from_str = parse_write_style),
        default_value="auto"
    )]
  pub color: WriteStyle,
}
