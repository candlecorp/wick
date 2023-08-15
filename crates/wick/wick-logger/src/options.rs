use std::cmp;
use std::path::PathBuf;

use tracing::{Level, Metadata};
use tracing_subscriber::layer::Context;

#[derive(Debug, Default, Clone)]
/// Logging options.
pub struct LoggingOptions {
  /// Turns on verbose logging.
  pub verbose: bool,

  /// Log as JSON.
  pub log_json: bool,

  /// The directory to store log files.
  pub log_dir: Option<PathBuf>,

  /// The endpoint to send jaeger-format traces.
  pub otlp_endpoint: Option<String>,

  /// The application doing the logging.
  pub app_name: String,

  /// Whether to install the global logger.
  pub global: bool,

  /// Log filtering options
  pub levels: LogFilters,
}

#[derive(Debug, Default, Clone)]
/// The filter configuration per log event destination.
pub struct LogFilters {
  /// The log level for the open telemetry events.
  pub telemetry: FilterOptions,
  /// The log level for the events printed to STDERR.
  pub stderr: FilterOptions,
}

impl LogFilters {
  /// Create a new filter configuration with the given log level.
  #[must_use]
  pub fn with_level(level: LogLevel) -> Self {
    Self {
      telemetry: FilterOptions {
        level,
        ..Default::default()
      },
      stderr: FilterOptions {
        level,
        ..Default::default()
      },
    }
  }
}

/// Options for filtering logs.
#[derive(Debug, Clone)]
pub struct FilterOptions {
  /// The default log level for anything that does not match an include or exclude filter.
  pub level: LogLevel,
  /// The targets and their log levels to include.
  pub include: Vec<TargetLevel>,
  /// The targets and their log levels to exclude.
  pub exclude: Vec<TargetLevel>,
}

impl Default for FilterOptions {
  fn default() -> Self {
    Self {
      level: LogLevel::Info,
      include: vec![],
      exclude: vec![],
    }
  }
}

impl<S> tracing_subscriber::layer::Filter<S> for FilterOptions
where
  S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
  fn enabled(&self, metadata: &Metadata<'_>, _cx: &Context<'_, S>) -> bool {
    metadata.target().starts_with("wick")
      || metadata.target().starts_with("flow")
      || metadata.target().starts_with("wasmrs")
  }

  fn event_enabled(&self, event: &tracing::Event<'_>, _cx: &Context<'_, S>) -> bool {
    let module = event.metadata().target().split("::").next().unwrap_or_default();
    let level = event.metadata().level();

    let excluded = self
      .exclude
      .iter()
      .find(|config| module.starts_with(&config.target) || config.target == "*")
      .map(|excluded| excluded.level < *level);

    let included = self
      .include
      .iter()
      .find(|config| module.starts_with(&config.target) || config.target == "*")
      .map(|included| included.level >= (*level));

    if included == Some(true) {
      true
    } else if excluded == Some(true) {
      return false;
    } else {
      self.level >= *level
    }
  }
}

/// The log level for specific targets.
#[derive(Debug, Default, PartialEq, Clone)]
pub struct TargetLevel {
  /// The target (module name).
  pub target: String,
  /// The level to log at.
  pub level: LogLevel,
}

impl TargetLevel {
  /// Create a new instance for the given target and log level.
  #[must_use]
  pub fn new(target: impl AsRef<str>, level: LogLevel) -> Self {
    Self {
      target: target.as_ref().to_owned(),
      level,
    }
  }
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

/// The log levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(usize)]
pub enum LogLevel {
  /// No logging.
  Quiet = 0,
  /// Only log errors.
  Error = 1,
  /// Only log warnings and errors.
  Warn = 2,
  /// The default log level.
  Info = 3,
  /// Log debug messages.
  Debug = 4,
  /// Log trace messages.
  Trace = 5,
}

impl Default for LogLevel {
  fn default() -> Self {
    Self::Info
  }
}

impl std::str::FromStr for LogLevel {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "quiet" => Ok(LogLevel::Quiet),
      "error" => Ok(LogLevel::Error),
      "warn" => Ok(LogLevel::Warn),
      "info" => Ok(LogLevel::Info),
      "debug" => Ok(LogLevel::Debug),
      "trace" => Ok(LogLevel::Trace),
      _ => Err(format!("Unknown log level: {}", s)),
    }
  }
}

impl std::fmt::Display for LogLevel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      LogLevel::Quiet => write!(f, "QUIET"),
      LogLevel::Error => write!(f, "ERROR"),
      LogLevel::Warn => write!(f, "WARN"),
      LogLevel::Info => write!(f, "INFO"),
      LogLevel::Debug => write!(f, "DEBUG"),
      LogLevel::Trace => write!(f, "TRACE"),
    }
  }
}

impl PartialEq<Level> for LogLevel {
  #[inline(always)]
  fn eq(&self, other: &Level) -> bool {
    match self {
      LogLevel::Quiet => false,
      LogLevel::Error => other.eq(&Level::ERROR),
      LogLevel::Warn => other.eq(&Level::WARN),
      LogLevel::Info => other.eq(&Level::INFO),
      LogLevel::Debug => other.eq(&Level::DEBUG),
      LogLevel::Trace => other.eq(&Level::TRACE),
    }
  }
}

impl PartialOrd<Level> for LogLevel {
  #[inline(always)]
  fn partial_cmp(&self, other: &Level) -> Option<cmp::Ordering> {
    Some((*self as usize).cmp(&filter_as_usize(other)))
  }

  #[inline(always)]
  fn lt(&self, other: &Level) -> bool {
    (*self as usize) < filter_as_usize(other)
  }

  #[inline(always)]
  fn le(&self, other: &Level) -> bool {
    (*self as usize) <= filter_as_usize(other)
  }

  #[inline(always)]
  fn gt(&self, other: &Level) -> bool {
    (*self as usize) > filter_as_usize(other)
  }

  #[inline(always)]
  fn ge(&self, other: &Level) -> bool {
    (*self as usize) >= filter_as_usize(other)
  }
}

#[inline(always)]
fn filter_as_usize(x: &Level) -> usize {
  (match *x {
    Level::ERROR => 0,
    Level::WARN => 1,
    Level::INFO => 2,
    Level::DEBUG => 3,
    Level::TRACE => 4,
  } + 1)
}
