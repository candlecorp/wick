use std::path::PathBuf;

use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug, Default, Clone, Serialize, Deserialize)]
/// Logging options that can be used directly or via [Args].
pub struct LoggingOptions {
  /// Disables logging.
  #[clap(long = "quiet", short = 'q')]
  pub quiet: bool,

  /// Turns on verbose logging.
  #[clap(long = "verbose", short = 'V')]
  pub verbose: bool,

  /// Greatly increases logging.
  #[clap(long = "silly", short = 'S')]
  pub silly: bool,

  /// Turns on debug logging.
  #[clap(long = "debug")]
  pub debug: bool,

  /// Turns on trace logging.
  #[clap(long = "trace")]
  pub trace: bool,

  /// Log as JSON.
  #[clap(long = "log-json")]
  pub log_json: bool,

  /// The directory to store log files,
  #[clap(long = "log-dir", env = "WAFL_LOG_DIR")]
  pub log_dir: Option<PathBuf>,

  /// The application doing the logging.
  #[clap(skip)]
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
