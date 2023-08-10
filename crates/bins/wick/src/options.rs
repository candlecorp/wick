use std::path::PathBuf;

use clap::Args;
use wick_logger::{FilterOptions, TargetLevel};
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

  /// The inclusion filter to apply to events posted to STDERR.
  #[clap(long = "log-keep", env = "LOG_INCLUDE", global = true, action)]
  pub(crate) log_include: Option<String>,

  /// The exclusion filter to apply to events posted to STDERR.
  #[clap(long = "log-filter", env = "LOG_EXCLUDE", global = true, action)]
  pub(crate) log_exclude: Option<String>,

  /// The inclusion filter to apply to events posted to the OTLP endpoint.
  #[clap(long = "otel-keep", env = "OTEL_INCLUDE", global = true, action)]
  pub(crate) otel_include: Option<String>,

  /// The exclusion filter to apply to events posted to the OTLP endpoint.
  #[clap(long = "otel-filter", env = "OTEL_EXCLUDE", global = true, action)]
  pub(crate) otel_exclude: Option<String>,

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

fn parse_logstr(value: &str) -> Vec<TargetLevel> {
  value
    .split(',')
    .filter_map(|s| {
      if s.is_empty() || !s.contains('=') {
        return None;
      }
      let mut parts = s.split('=');
      let target = parts.next()?;
      let level = parts.next().unwrap_or("info");
      Some(TargetLevel {
        target: target.to_owned(),
        level: level.parse().ok()?,
      })
    })
    .collect()
}

impl From<&LoggingOptions> for wick_logger::LoggingOptions {
  fn from(value: &LoggingOptions) -> Self {
    let stderr_inc = value.log_include.as_deref().map_or_else(Vec::new, parse_logstr);
    let stderr_exc = value.log_exclude.as_deref().map_or_else(Vec::new, parse_logstr);
    let otel_inc = value.otel_include.as_deref().map_or_else(Vec::new, parse_logstr);
    let otel_exc = value.otel_exclude.as_deref().map_or_else(Vec::new, parse_logstr);

    let global_level = if value.quiet {
      wick_logger::LogLevel::Quiet
    } else if value.trace {
      wick_logger::LogLevel::Trace
    } else if value.debug {
      wick_logger::LogLevel::Debug
    } else {
      wick_logger::LogLevel::Info
    };

    let default_inc = vec![TargetLevel::new("flow", wick_logger::LogLevel::Warn)];
    let default_exc = vec![
      TargetLevel::new("wasmrs", wick_logger::LogLevel::Error),
      TargetLevel::new("wasmrs_runtime", wick_logger::LogLevel::Error),
      TargetLevel::new("wasmrs_wasmtime", wick_logger::LogLevel::Error),
    ];
    Self {
      verbose: value.verbose == 1,
      log_json: false,
      log_dir: None,
      otlp_endpoint: value.otlp_endpoint.clone(),
      app_name: value.app_name.clone(),
      global: false,
      levels: wick_logger::LogFilters {
        telemetry: FilterOptions {
          level: global_level,
          include: [otel_inc, default_inc.clone()].concat(),
          exclude: [otel_exc, default_exc.clone()].concat(),
        },
        stderr: FilterOptions {
          level: global_level,
          include: [stderr_inc, default_inc].concat(),
          exclude: [stderr_exc, default_exc].concat(),
        },
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
    options.otel_include = otel_settings.include.clone();
    options.otel_exclude = otel_settings.exclude.clone();
  }
  if let Some(log_settings) = &settings.trace.stderr {
    options.log_include = log_settings.include.clone();
    options.log_exclude = log_settings.exclude.clone();
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
