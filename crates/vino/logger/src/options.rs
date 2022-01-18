use structopt::StructOpt;

// fn parse_write_style(spec: &str) -> Result<WriteStyle, LoggerError> {
//   match spec {
//     "auto" => Ok(WriteStyle::Auto),
//     "always" => Ok(WriteStyle::Always),
//     "never" => Ok(WriteStyle::Never),
//     _ => Err(LoggerError::StyleParse),
//   }
// }

#[derive(StructOpt, Debug, Default, Clone)]
/// Logging options that can be used directly or via [StructOpt].
pub struct LoggingOptions {
  /// Disables logging.
  #[structopt(long = "quiet", short = "q")]
  pub quiet: bool,

  /// Turns on verbose logging.
  #[structopt(long = "verbose", short = "V")]
  pub verbose: bool,

  /// Turns on debug logging.
  #[structopt(long = "debug")]
  pub debug: bool,

  /// Turns on trace logging.
  #[structopt(long = "trace")]
  pub trace: bool,

  /// Log as JSON.
  #[structopt(long = "log-json")]
  pub log_json: bool,

  // /// The directory to store log files,
  // #[structopt(long = "log-dir")]
  // pub log_dir: String,
  /// The application doing the logging.
  #[structopt(skip)]
  pub app_name: String,
  // /// Print log output as color. Options are auto | always | never.
  // #[structopt(
  //       long = "color",
  //       env = "LOG_COLOR",
  //       parse(try_from_str = parse_write_style),
  //       default_value="auto"
  //   )]
  // pub color: WriteStyle,
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
