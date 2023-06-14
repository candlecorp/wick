use clap::Args;
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
  #[clap(long = "otlp", env = "OTLP_ENDPOINT", action)]
  pub(crate) otlp_endpoint: Option<String>,

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

impl From<&LoggingOptions> for wick_logger::LoggingOptions {
  fn from(value: &LoggingOptions) -> Self {
    Self {
      verbose: value.verbose == 1,
      silly: value.verbose >= 2,
      level: if value.quiet {
        wick_logger::LogLevel::Quiet
      } else if value.trace {
        wick_logger::LogLevel::Trace
      } else if value.debug {
        wick_logger::LogLevel::Debug
      } else {
        wick_logger::LogLevel::Info
      },
      log_json: false,
      log_dir: None,
      otlp_endpoint: value.otlp_endpoint.clone(),
      app_name: value.app_name.clone(),
      global: false,
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
  if options.verbose == 0 {
    if settings.trace.modifier == wick_settings::LogModifier::Verbose {
      options.verbose = 1;
    }
    if settings.trace.modifier == wick_settings::LogModifier::Silly {
      options.verbose = 2;
    }
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
