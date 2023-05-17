use clap::Args;
use wick_settings::Credential;

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
  #[clap(long = "jaeger-endpoint", short = 'j', env = "OTEL_EXPORTER_JAEGER_ENDPOINT", action)]
  pub(crate) jaeger_endpoint: Option<String>,

  /// The application doing the logging.
  #[clap(skip)]
  pub(crate) app_name: String,
}

impl LoggingOptions {
  /// Set the name of the application doing the logging.
  pub(crate) fn name(&self, name: impl AsRef<str>) -> Self {
    Self {
      app_name: name.as_ref().to_owned(),
      ..self.clone()
    }
  }
}

impl From<LoggingOptions> for wick_logger::LoggingOptions {
  fn from(value: LoggingOptions) -> Self {
    Self {
      verbose: value.verbose == 1,
      silly: value.verbose == 2,
      level: if value.quiet {
        wick_logger::LogLevel::Quiet
      } else if value.debug {
        wick_logger::LogLevel::Debug
      } else if value.trace {
        wick_logger::LogLevel::Trace
      } else {
        wick_logger::LogLevel::Info
      },
      log_json: false,
      log_dir: None,
      jaeger_endpoint: value.jaeger_endpoint,
      app_name: value.app_name,
    }
  }
}

pub(crate) fn apply_log_settings(settings: &wick_settings::Settings, options: &mut LoggingOptions) {
  if settings.trace.level == wick_settings::LogLevel::Debug {
    options.debug = true;
  }
  if settings.trace.level == wick_settings::LogLevel::Trace {
    options.trace = true;
  }
  if settings.trace.level == wick_settings::LogLevel::Off {
    options.quiet = true;
  }
  if settings.trace.modifier == wick_settings::LogModifier::Verbose {
    options.verbose = 1;
  }
  if settings.trace.modifier == wick_settings::LogModifier::Silly {
    options.verbose = 2;
  }
  if options.jaeger_endpoint.is_none() {
    options.jaeger_endpoint = settings.trace.jaeger.clone();
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
