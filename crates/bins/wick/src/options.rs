use std::path::PathBuf;

use clap::Args;
use once_cell::sync::Lazy;
use regex::Regex;
use wick_logger::{FilterOptions, LogLevel, LogModifier, TargetLevel};
use wick_oci_utils::{OciOptions, OnExisting};
use wick_settings::Credential;

#[derive(Args, Debug, Default, Clone)]
/// Global output options.
pub(crate) struct GlobalOptions {
  /// Print CLI output as JSON.
  #[clap(long = "json", short = 'j', global = true, action)]
  pub(crate) json: bool,
}

#[derive(Args, Debug, Default, Clone)]
/// Logging options that can be used directly or via [Args].
pub(crate) struct LoggingOptions {
  /// Silences log output.
  #[clap(long = "quiet", short = 'q', global = true, action)]
  pub(crate) quiet: bool,

  /// Turns on verbose logging. Repeat for increased verbosity.
  #[clap(long = "verbose", short = 'v', global = true, action = clap::ArgAction::Count)]
  pub(crate) verbose: u8,

  /// Turns on debug logging.
  #[clap(long = "debug", global = true, action)]
  pub(crate) debug: bool,

  /// Turns on trace logging.
  #[clap(long = "trace", global = true, action)]
  pub(crate) trace: bool,

  /// The endpoint to send jaeger-format traces.
  #[clap(long = "otlp", env = "OTLP_ENDPOINT", global = true, action)]
  pub(crate) otlp_endpoint: Option<String>,

  /// The filter to apply to events posted to STDERR.
  #[clap(long = "log-filter", env = "LOG_FILTER", global = true, action)]
  pub(crate) stderr_filter: Option<String>,

  /// The filter to apply to events posted to the OTLP endpoint.
  #[clap(long = "otel-filter", env = "OTEL_FILTER", global = true, action)]
  pub(crate) tel_filter: Option<String>,

  /// The application doing the logging.
  #[clap(skip)]
  pub(crate) app_name: String,
}

impl LoggingOptions {
  /// Set the name of the application doing the logging.
  pub(crate) fn name(&mut self, name: impl AsRef<str>) -> &mut Self {
    self.app_name = name.as_ref().to_owned();
    self
  }
}

static DEFAULT_FILTER: Lazy<Vec<TargetLevel>> = Lazy::new(|| {
  vec![
    TargetLevel::lte("flow", wick_logger::LogLevel::Warn),
    TargetLevel::lte("wasmrs", wick_logger::LogLevel::Error),
    TargetLevel::lte("wasmrs_runtime", wick_logger::LogLevel::Error),
    TargetLevel::lte("wasmrs_wasmtime", wick_logger::LogLevel::Error),
  ]
});

static RULE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("(\\w+)\\s*(<|<=|>|>=|!=|==|=)\\s*(\\w+)").unwrap());

fn parse_logstr(default_level: LogLevel, default_filter: &[TargetLevel], logstr: &str) -> FilterOptions {
  let parts: Vec<(LogModifier, Option<&str>, LogLevel)> = logstr
    .split(',')
    .filter_map(|s| {
      let s = s.trim();
      if s.is_empty() {
        return None;
      }

      if !s.contains(['<', '>', '=', '!']) {
        return Some((LogModifier::LessThanOrEqualTo, None, s.parse().ok()?));
      }
      let parts = RULE_REGEX.captures(s)?;

      let target = parts.get(1)?.as_str();
      let modifier: LogModifier = parts.get(2)?.as_str().parse().ok()?;
      let level: LogLevel = parts.get(3)?.as_str().parse().ok()?;

      Some((modifier, Some(target), level))
    })
    .collect();

  let global_level = parts
    .iter()
    .find(|(_, target, _)| target.is_none())
    .map_or_else(|| default_level, |(_, _, level)| *level);

  let filter = parts
    .iter()
    .filter_map(|(modifier, target, level)| target.map(|target| TargetLevel::new(target, *level, *modifier)))
    .collect::<Vec<_>>();

  FilterOptions {
    level: global_level,
    // If the filter had inclusion rules, use those. Otherwise, use the default.
    filter: [filter, default_filter.to_vec()].concat(),
  }
}

impl From<&LoggingOptions> for wick_logger::LoggingOptions {
  fn from(value: &LoggingOptions) -> Self {
    let global_level = if value.quiet {
      wick_logger::LogLevel::Quiet
    } else if value.trace {
      wick_logger::LogLevel::Trace
    } else if value.debug {
      wick_logger::LogLevel::Debug
    } else {
      wick_logger::LogLevel::Info
    };

    let stderr_opts = parse_logstr(
      global_level,
      &DEFAULT_FILTER,
      value.stderr_filter.as_deref().unwrap_or_default(),
    );

    let otel_opts = parse_logstr(
      global_level,
      &DEFAULT_FILTER,
      value.tel_filter.as_deref().unwrap_or_default(),
    );

    Self {
      verbose: value.verbose == 1,
      log_json: false,
      log_dir: None,
      otlp_endpoint: value.otlp_endpoint.clone(),
      app_name: value.app_name.clone(),
      global: false,
      levels: wick_logger::LogFilters {
        telemetry: otel_opts,
        stderr: stderr_opts,
      },
    }
  }
}

impl From<&mut LoggingOptions> for wick_logger::LoggingOptions {
  fn from(value: &mut LoggingOptions) -> Self {
    let v: &LoggingOptions = value;
    v.into()
  }
}

pub(crate) fn apply_log_settings(settings: &wick_settings::Settings, options: &mut LoggingOptions) {
  if !(options.debug || options.trace) {
    options.debug = settings.trace.level == wick_settings::LogLevel::Debug;
    options.trace = settings.trace.level == wick_settings::LogLevel::Trace;
  }

  if settings.trace.level == wick_settings::LogLevel::Off {
    options.quiet = true;
  }
  if options.verbose == 0 && settings.trace.modifier == wick_settings::LogModifier::Verbose {
    options.verbose = 1;
  }
  if let Some(otel_settings) = &settings.trace.telemetry {
    options.tel_filter = otel_settings.filter.clone();
  }
  if let Some(log_settings) = &settings.trace.stderr {
    options.stderr_filter = log_settings.filter.clone();
  }
  if options.otlp_endpoint.is_none() {
    options.otlp_endpoint = settings.trace.otlp.clone();
  }
}

pub(crate) fn get_auth_for_scope(
  configured_creds: Option<&Credential>,
  override_username: Option<&str>,
  override_password: Option<&str>,
) -> (Option<String>, Option<String>) {
  let mut username = None;
  let mut password = None;

  if let Some(creds) = configured_creds {
    match &creds.auth {
      wick_settings::Auth::Basic(basic) => {
        debug!("using basic auth from configuration settings.");
        username = Some(basic.username.clone());
        password = Some(basic.password.clone());
      }
    };
  }

  if override_username.is_some() {
    debug!("overriding username from arguments.");
    username = override_username.map(|v| v.to_owned());
  }

  if override_password.is_some() {
    debug!("overriding password from arguments.");
    password = override_password.map(|v| v.to_owned());
  }

  (username, password)
}

pub(crate) fn reconcile_fetch_options(
  reference: &str,
  settings: &wick_settings::Settings,
  opts: crate::oci::Options,
  force: bool,
  output: Option<PathBuf>,
) -> OciOptions {
  let configured_creds = settings.credentials.iter().find(|c| reference.starts_with(&c.scope));

  let (username, password) = get_auth_for_scope(configured_creds, opts.username.as_deref(), opts.password.as_deref());

  let mut oci_opts = OciOptions::default();
  oci_opts
    .set_allow_insecure(opts.insecure_registries)
    .set_allow_latest(true)
    .set_username(username)
    .set_password(password)
    .set_on_existing(if force {
      OnExisting::Overwrite
    } else {
      OnExisting::Error
    });
  if let Some(output) = output {
    oci_opts.set_cache_dir(output);
  }
  oci_opts
}

#[cfg(test)]
mod test {
  use anyhow::Result;

  use super::*;

  type ExpectedLogRule = (LogLevel, Vec<TargetLevel>);

  #[rstest::rstest]
  #[case(LogLevel::Info, "trace", (LogLevel::Trace,vec![]))]
  #[case(LogLevel::Info, "wick<=trace", (LogLevel::Info, vec![TargetLevel::lte("wick", LogLevel::Trace)]))]
  #[case(LogLevel::Info, "debug,wick<=trace", (LogLevel::Debug, vec![TargetLevel::lte("wick", LogLevel::Trace)]))]
  #[case(LogLevel::Info, "wick<=trace,debug", (LogLevel::Debug, vec![TargetLevel::lte("wick", LogLevel::Trace)]))]
  #[case(LogLevel::Info, " wick<=trace , debug ,,, ,", (LogLevel::Debug, vec![TargetLevel::lte("wick", LogLevel::Trace)]))]
  #[case(LogLevel::Info, "wick<=trace,flow!=info", (LogLevel::Info, vec![TargetLevel::lte("wick", LogLevel::Trace),TargetLevel::not("flow", LogLevel::Info)]))]
  #[case(LogLevel::Info, "wick<=trace,flow!=info,wasmrs<=info", (LogLevel::Info, vec![TargetLevel::lte("wick", LogLevel::Trace),TargetLevel::not("flow", LogLevel::Info),TargetLevel::lte("wasmrs", LogLevel::Info)]))]
  #[case(LogLevel::Info, "wick <= trace, flow != info, wasmrs <= info",(LogLevel::Info, vec![TargetLevel::lte("wick", LogLevel::Trace),TargetLevel::not("flow", LogLevel::Info),TargetLevel::lte("wasmrs", LogLevel::Info)]))]
  fn test_log_rules(
    #[case] default_loglevel: LogLevel,
    #[case] filter: &str,
    #[case] expected: ExpectedLogRule,
  ) -> Result<()> {
    let filter = parse_logstr(default_loglevel, &DEFAULT_FILTER, filter);
    assert_eq!(filter.level, expected.0);

    // append the default exclusion filter so we don't need to include it in test cases above.
    let expected_filter = [expected.1, DEFAULT_FILTER.to_vec()].concat();

    assert_eq!(filter.filter, expected_filter);

    Ok(())
  }
}
