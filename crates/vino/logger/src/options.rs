use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

// fn parse_write_style(spec: &str) -> Result<WriteStyle, LoggerError> {
//   match spec {
//     "auto" => Ok(WriteStyle::Auto),
//     "always" => Ok(WriteStyle::Always),
//     "never" => Ok(WriteStyle::Never),
//     _ => Err(LoggerError::StyleParse),
//   }
// }

#[derive(StructOpt, Debug, Default, Clone, Serialize, Deserialize)]
/// Logging options that can be used directly or via [StructOpt].
pub struct LoggingOptions {
  /// Disables logging.
  #[structopt(long = "quiet", short = "q", env = "VINO_LOG_QUIET")]
  pub quiet: bool,

  /// Turns on verbose logging.
  #[structopt(long = "verbose", short = "V", env = "VINO_LOG_VERBOSE")]
  pub verbose: bool,

  /// Turns on debug logging.
  #[structopt(long = "debug", env = "VINO_LOG_DEBUG")]
  pub debug: bool,

  /// Turns on trace logging.
  #[structopt(long = "trace", env = "VINO_LOG_TRACE")]
  pub trace: bool,

  /// Log as JSON.
  #[structopt(long = "log-json", env = "VINO_LOG_JSON")]
  pub log_json: bool,

  /// The directory to store log files,
  #[structopt(long = "log-dir", env = "VINO_LOG_DIR")]
  pub log_dir: Option<PathBuf>,

  /// The application doing the logging.
  #[structopt(skip)]
  pub app_name: String,
}

impl LoggingOptions {
  /// Set the name of the application doing the logging.
  pub fn name(&self, name: impl AsRef<str>) -> Self {
    Self {
      app_name: name.as_ref().to_owned(),
      ..self.clone()
    }
  }
}
