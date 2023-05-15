use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
/// Logging options that can be used directly or via [Args].
pub struct LoggingOptions {
  /// Turns on verbose logging.
  pub verbose: bool,

  /// Greatly increases logging.
  pub silly: bool,

  /// Turns on debug logging.
  pub level: LogLevel,

  /// Log as JSON.
  pub log_json: bool,

  /// The directory to store log files.
  pub log_dir: Option<PathBuf>,

  /// The endpoint to send jaeger-format traces.
  pub jaeger_endpoint: Option<String>,

  /// The application doing the logging.
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

#[derive(Debug, Clone, Copy, PartialEq)]
/// The log levels.
pub enum LogLevel {
  /// No logging.
  Quiet,
  /// Only log errors.
  Error,
  /// Only log warnings and errors.
  Warn,
  /// The default log level.
  Info,
  /// Log debug messages.
  Debug,
  /// Log trace messages.
  Trace,
}

impl Default for LogLevel {
  fn default() -> Self {
    Self::Info
  }
}
