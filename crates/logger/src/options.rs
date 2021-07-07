use anyhow::anyhow;
use env_logger::WriteStyle;
use structopt::StructOpt;

fn parse_write_style(spec: &str) -> anyhow::Result<WriteStyle> {
  match spec {
    "auto" => Ok(WriteStyle::Auto),
    "always" => Ok(WriteStyle::Always),
    "never" => Ok(WriteStyle::Never),
    _ => Err(anyhow!("Configuration error")),
  }
}

#[derive(StructOpt, Debug, Clone, Copy)]
/// Logging options that can be used directly or via [StructOpt]
pub struct LoggingOptions {
  /// Disables logging
  #[structopt(long = "quiet", short = "q")]
  pub quiet: bool,

  /// Outputs the version
  #[structopt(long = "version", short = "v")]
  pub version: bool,

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

  /// Log style
  #[structopt(
        long = "log-style",
        env = "VINO_LOG_STYLE",
        parse(try_from_str = parse_write_style),
        default_value="auto"
    )]
  pub log_style: WriteStyle,
}
