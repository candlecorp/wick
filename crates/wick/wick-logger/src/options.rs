use std::cmp;
use std::path::PathBuf;
use std::str::FromStr;

use tracing::{Level, Metadata};
use tracing_subscriber::layer::Context;

#[derive(Debug, Clone)]
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

impl LoggingOptions {
  /// Create a new instance with the given log level.
  #[must_use]
  pub fn with_level(level: LogLevel) -> Self {
    Self {
      levels: LogFilters::with_level(level),
      global: true,
      ..Default::default()
    }
  }
}

impl Default for LoggingOptions {
  fn default() -> Self {
    Self {
      verbose: Default::default(),
      log_json: Default::default(),
      log_dir: Default::default(),
      otlp_endpoint: Default::default(),
      app_name: "app".to_owned(),
      global: true,
      levels: Default::default(),
    }
  }
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
  /// The targets and their log levels.
  pub filter: Vec<TargetLevel>,
}

impl Default for FilterOptions {
  fn default() -> Self {
    Self {
      level: LogLevel::Info,
      filter: vec![],
    }
  }
}

impl FilterOptions {
  fn test_enabled(&self, module: &str, level: Level) -> bool {
    let matches = self.filter.iter().filter(|config| module.starts_with(&config.target));
    let match_hit = matches.fold(None, |acc, next| {
      let enabled = next.modifier.compare(filter_as_usize(level), next.level as usize);
      let next_len = next.target.len();
      acc.map_or(Some((next_len, enabled)), |(last_len, last_enabled)| {
        match next_len.cmp(&last_len) {
          cmp::Ordering::Greater => {
            // if we're more specific, use the most recent match result.
            Some((next_len, enabled))
          }
          cmp::Ordering::Equal => {
            // if we're the same specifity, keep testing
            Some((last_len, enabled && last_enabled))
          }
          cmp::Ordering::Less => {
            // otherwise, keep the last match result
            Some((last_len, last_enabled))
          }
        }
      })
    });
    match_hit.map_or(self.level >= level, |(_, enabled)| enabled)
  }
}

impl<S> tracing_subscriber::layer::Filter<S> for FilterOptions
where
  S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
  fn enabled(&self, metadata: &Metadata<'_>, _cx: &Context<'_, S>) -> bool {
    let enabled = metadata.target().starts_with("wick")
      || metadata.target().starts_with("flow")
      || metadata.target().starts_with("wasmrs");

    metadata.is_span() || enabled
  }

  fn event_enabled(&self, event: &tracing::Event<'_>, _cx: &Context<'_, S>) -> bool {
    let module = event.metadata().target().split("::").next().unwrap_or_default();
    let level = event.metadata().level();
    self.test_enabled(module, *level)
  }
}

/// The log level for specific targets.
#[derive(Debug, Default, PartialEq, Clone)]
pub struct TargetLevel {
  /// The target (module name).
  pub target: String,
  /// The level to log at.
  pub level: LogLevel,
  /// The modifier that controls how to use this log level.
  pub modifier: LogModifier,
}

impl TargetLevel {
  /// Create a new instance for the given target, log level, and modifier.
  pub fn new(target: impl AsRef<str>, level: LogLevel, modifier: LogModifier) -> Self {
    Self {
      target: target.as_ref().to_owned(),
      level,
      modifier,
    }
  }

  /// Create a new negated instance for the given target and log level.
  #[must_use]
  pub fn not(target: impl AsRef<str>, level: LogLevel) -> Self {
    Self::new(target, level, LogModifier::Not)
  }

  /// Create a new instance that matches the given target and any log level greater than the one specified.
  #[must_use]
  pub fn gt(target: impl AsRef<str>, level: LogLevel) -> Self {
    Self::new(target, level, LogModifier::GreaterThan)
  }

  /// Create a new instance that matches the given target and any log level greater than or equal to the one specified.
  #[must_use]
  pub fn gte(target: impl AsRef<str>, level: LogLevel) -> Self {
    Self::new(target, level, LogModifier::GreaterThanOrEqualTo)
  }

  /// Create a new instance that matches the given target and any log level less than or equal to the one specified.
  #[must_use]
  pub fn lt(target: impl AsRef<str>, level: LogLevel) -> Self {
    Self::new(target, level, LogModifier::LessThan)
  }

  /// Create a new instance that matches the given target and any log level less than or equal to the one specified.
  #[must_use]
  pub fn lte(target: impl AsRef<str>, level: LogLevel) -> Self {
    Self::new(target, level, LogModifier::LessThanOrEqualTo)
  }

  /// Create a new instance that matches the given target and any log level equal to the one specified.
  #[must_use]
  pub fn is(target: impl AsRef<str>, level: LogLevel) -> Self {
    Self::new(target, level, LogModifier::Equal)
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

#[derive(Debug, Clone, PartialEq, Copy)]
/// Whether to include logs higher, lower, equal, or to not include them at all.
pub enum LogModifier {
  /// Do not log the associated level.
  Not,
  /// Only log events greater than the associated level.
  GreaterThan,
  /// Only log events greater than or equal to the associated level.
  GreaterThanOrEqualTo,
  /// Only log events less than the associated level.
  LessThan,
  /// Only log events less than or equal to the associated level.
  LessThanOrEqualTo,
  /// Only log events equal to the associated level.
  Equal,
}

impl Default for LogModifier {
  fn default() -> Self {
    Self::LessThanOrEqualTo
  }
}

impl std::fmt::Display for LogModifier {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      LogModifier::Not => write!(f, "!="),
      LogModifier::GreaterThan => write!(f, ">"),
      LogModifier::GreaterThanOrEqualTo => write!(f, ">="),
      LogModifier::LessThan => write!(f, "<"),
      LogModifier::LessThanOrEqualTo => write!(f, "<="),
      LogModifier::Equal => write!(f, "="),
    }
  }
}

impl LogModifier {
  fn compare(self, a: usize, b: usize) -> bool {
    match self {
      LogModifier::Not => a != b,
      LogModifier::GreaterThan => a > b,
      LogModifier::GreaterThanOrEqualTo => a >= b,
      LogModifier::LessThan => a < b,
      LogModifier::LessThanOrEqualTo => a <= b,
      LogModifier::Equal => a == b,
    }
  }
}

impl FromStr for LogModifier {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "!=" => Ok(LogModifier::Not),
      ">" => Ok(LogModifier::GreaterThan),
      ">=" => Ok(LogModifier::GreaterThanOrEqualTo),
      "<" => Ok(LogModifier::LessThan),
      "<=" => Ok(LogModifier::LessThanOrEqualTo),
      "=" | "==" => Ok(LogModifier::Equal),

      _ => Err(()),
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

impl FromStr for LogLevel {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "quiet" => Ok(LogLevel::Quiet),
      "error" => Ok(LogLevel::Error),
      "warn" => Ok(LogLevel::Warn),
      "info" => Ok(LogLevel::Info),
      "debug" => Ok(LogLevel::Debug),
      "trace" => Ok(LogLevel::Trace),
      _ => Err(()),
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
    Some((*self as usize).cmp(&filter_as_usize(*other)))
  }

  #[inline(always)]
  fn lt(&self, other: &Level) -> bool {
    (*self as usize) < filter_as_usize(*other)
  }

  #[inline(always)]
  fn le(&self, other: &Level) -> bool {
    (*self as usize) <= filter_as_usize(*other)
  }

  #[inline(always)]
  fn gt(&self, other: &Level) -> bool {
    (*self as usize) > filter_as_usize(*other)
  }

  #[inline(always)]
  fn ge(&self, other: &Level) -> bool {
    (*self as usize) >= filter_as_usize(*other)
  }
}

#[inline(always)]
fn filter_as_usize(x: Level) -> usize {
  (match x {
    Level::ERROR => 0,
    Level::WARN => 1,
    Level::INFO => 2,
    Level::DEBUG => 3,
    Level::TRACE => 4,
  } + 1)
}

#[cfg(test)]
mod test {

  use super::*;

  #[test]
  fn test_modifier_compare() {
    assert!(LogModifier::Equal.compare(2, 2));
    assert!(LogModifier::GreaterThan.compare(4, 2));
    assert!(LogModifier::GreaterThanOrEqualTo.compare(4, 2));
    assert!(LogModifier::GreaterThanOrEqualTo.compare(2, 2));
    assert!(LogModifier::Not.compare(4, 3));
    assert!(LogModifier::LessThan.compare(1, 2));
    assert!(LogModifier::LessThanOrEqualTo.compare(1, 2));
    assert!(LogModifier::LessThanOrEqualTo.compare(2, 2));
  }

  #[test]
  fn test_modifier_compare_level() {
    assert!(LogModifier::Equal.compare(filter_as_usize(Level::TRACE), LogLevel::Trace as usize));
    assert!(LogModifier::GreaterThan.compare(filter_as_usize(Level::TRACE), LogLevel::Warn as usize));
    assert!(LogModifier::GreaterThanOrEqualTo.compare(filter_as_usize(Level::INFO), LogLevel::Info as usize));
    assert!(LogModifier::GreaterThanOrEqualTo.compare(filter_as_usize(Level::TRACE), LogLevel::Debug as usize));
    assert!(LogModifier::Not.compare(filter_as_usize(Level::ERROR), LogLevel::Trace as usize));
    assert!(LogModifier::LessThan.compare(filter_as_usize(Level::INFO), LogLevel::Debug as usize));
    assert!(LogModifier::LessThanOrEqualTo.compare(filter_as_usize(Level::TRACE), LogLevel::Trace as usize));
    assert!(LogModifier::LessThanOrEqualTo.compare(filter_as_usize(Level::INFO), LogLevel::Trace as usize));
  }

  #[allow(clippy::needless_pass_by_value)]
  fn opts<const K: usize>(default: LogLevel, targets: [TargetLevel; K]) -> FilterOptions {
    FilterOptions {
      level: default,
      filter: targets.to_vec(),
    }
  }

  #[test]
  fn test_default_level() {
    assert!(opts(LogLevel::Info, []).test_enabled("wick", Level::INFO));
    assert!(!opts(LogLevel::Info, []).test_enabled("wick", Level::TRACE));
  }

  #[rstest::rstest]
  #[case(LogLevel::Info, [TargetLevel::lte("wick",LogLevel::Trace)], "wick", Level::TRACE, true)]
  #[case(LogLevel::Info, [TargetLevel::lte("wick",LogLevel::Info),TargetLevel::lte("wick_packet",LogLevel::Trace)], "wick_packet", Level::TRACE, true)]
  #[case(LogLevel::Info, [
    TargetLevel::lte("a",LogLevel::Info),
    TargetLevel::not("ab",LogLevel::Trace),
    TargetLevel::lte("abc",LogLevel::Trace)
  ], "abcdef", Level::TRACE, true)]
  #[case(LogLevel::Info, [
    TargetLevel::lte("a",LogLevel::Info),
    TargetLevel::lte("abc",LogLevel::Trace),
    TargetLevel::not("ab",LogLevel::Trace),
  ], "abcdef", Level::TRACE, true)]
  fn test_specificity<const K: usize>(
    #[case] default: LogLevel,
    #[case] filter: [TargetLevel; K],
    #[case] span_target: &str,
    #[case] span_level: Level,
    #[case] expect_enabled: bool,
  ) {
    assert_eq!(
      opts(default, filter).test_enabled(span_target, span_level),
      expect_enabled
    );
  }

  #[rstest::rstest]
  #[case(LogLevel::Info, LogLevel::Trace, Level::TRACE, false)]
  #[case(LogLevel::Info, LogLevel::Trace, Level::DEBUG, true)]
  #[case(LogLevel::Info, LogLevel::Trace, Level::INFO, true)]
  #[case(LogLevel::Info, LogLevel::Trace, Level::WARN, true)]
  #[case(LogLevel::Info, LogLevel::Trace, Level::ERROR, true)]
  fn test_not(
    #[case] default: LogLevel,
    #[case] target_level: LogLevel,
    #[case] span_level: Level,
    #[case] expect_enabled: bool,
  ) {
    assert_eq!(
      opts(default, [TargetLevel::not("wick", target_level)]).test_enabled("wick", span_level),
      expect_enabled
    );
  }

  #[rstest::rstest]
  #[case(LogLevel::Info, LogLevel::Trace, Level::TRACE, false)]
  #[case(LogLevel::Info, LogLevel::Trace, Level::DEBUG, true)]
  #[case(LogLevel::Info, LogLevel::Trace, Level::INFO, true)]
  #[case(LogLevel::Info, LogLevel::Trace, Level::WARN, true)]
  #[case(LogLevel::Info, LogLevel::Trace, Level::ERROR, true)]
  fn test_lt(
    #[case] default: LogLevel,
    #[case] target_level: LogLevel,
    #[case] span_level: Level,
    #[case] expect_enabled: bool,
  ) {
    assert_eq!(
      opts(default, [TargetLevel::lt("wick", target_level)]).test_enabled("wick", span_level),
      expect_enabled
    );
  }

  #[rstest::rstest]
  #[case(LogLevel::Info, LogLevel::Trace, Level::TRACE, true)]
  #[case(LogLevel::Info, LogLevel::Trace, Level::DEBUG, true)]
  #[case(LogLevel::Info, LogLevel::Trace, Level::INFO, true)]
  #[case(LogLevel::Info, LogLevel::Trace, Level::WARN, true)]
  #[case(LogLevel::Info, LogLevel::Trace, Level::ERROR, true)]
  fn test_lte(
    #[case] default: LogLevel,
    #[case] target_level: LogLevel,
    #[case] span_level: Level,
    #[case] expect_enabled: bool,
  ) {
    assert_eq!(
      opts(default, [TargetLevel::lte("wick", target_level)]).test_enabled("wick", span_level),
      expect_enabled
    );
  }

  #[rstest::rstest]
  #[case(LogLevel::Info, LogLevel::Info, Level::TRACE, true)]
  #[case(LogLevel::Info, LogLevel::Info, Level::DEBUG, true)]
  #[case(LogLevel::Info, LogLevel::Info, Level::INFO, true)]
  #[case(LogLevel::Info, LogLevel::Info, Level::WARN, false)]
  #[case(LogLevel::Info, LogLevel::Info, Level::ERROR, false)]
  fn test_gte(
    #[case] default: LogLevel,
    #[case] target_level: LogLevel,
    #[case] span_level: Level,
    #[case] expect_enabled: bool,
  ) {
    assert_eq!(
      opts(default, [TargetLevel::gte("wick", target_level)]).test_enabled("wick", span_level),
      expect_enabled
    );
  }

  #[rstest::rstest]
  #[case(LogLevel::Info, LogLevel::Info, Level::TRACE, true)]
  #[case(LogLevel::Info, LogLevel::Info, Level::DEBUG, true)]
  #[case(LogLevel::Info, LogLevel::Info, Level::INFO, false)]
  #[case(LogLevel::Info, LogLevel::Info, Level::WARN, false)]
  #[case(LogLevel::Info, LogLevel::Info, Level::ERROR, false)]
  fn test_gt(
    #[case] default: LogLevel,
    #[case] target_level: LogLevel,
    #[case] span_level: Level,
    #[case] expect_enabled: bool,
  ) {
    assert_eq!(
      opts(default, [TargetLevel::gt("wick", target_level)]).test_enabled("wick", span_level),
      expect_enabled
    );
  }

  #[rstest::rstest]
  #[case(LogLevel::Info, LogLevel::Info, Level::TRACE, false)]
  #[case(LogLevel::Info, LogLevel::Info, Level::DEBUG, false)]
  #[case(LogLevel::Info, LogLevel::Info, Level::INFO, true)]
  #[case(LogLevel::Info, LogLevel::Info, Level::WARN, false)]
  #[case(LogLevel::Info, LogLevel::Info, Level::ERROR, false)]
  fn test_eq(
    #[case] default: LogLevel,
    #[case] target_level: LogLevel,
    #[case] span_level: Level,
    #[case] expect_enabled: bool,
  ) {
    assert_eq!(
      opts(default, [TargetLevel::is("wick", target_level)]).test_enabled("wick", span_level),
      expect_enabled
    );
  }
}
