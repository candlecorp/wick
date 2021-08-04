use std::time::SystemTime;

use chrono::{
  DateTime,
  Utc,
};
use colored::Colorize;
use env_logger::filter::{
  Builder,
  Filter,
};
use env_logger::WriteStyle;
use log::{
  set_boxed_logger,
  set_max_level,
  Level,
  LevelFilter,
  Log,
  Record,
};
use serde_json::json;

use crate::error::LoggerError;
use crate::{
  LoggingOptions,
  FILTER_ENV,
};

/// The logger instance
#[derive(Debug)]
pub struct Logger {
  inner: Filter,
  verbose: bool,
  json: bool,
  style: WriteStyle,
}

fn set_level(builder: &mut Builder, priority_modules: &[&str], level: LevelFilter) {
  for module in priority_modules.iter() {
    builder.filter_module(module.as_ref(), level);
  }
}

impl Logger {
  fn new(opts: &LoggingOptions) -> Logger {
    let mut builder = Builder::new();

    let priority_modules = [
      "logger",
      "vino",
      "vino_cli",
      "vinoc",
      "vino_macros",
      "vino_runtime",
      "vino_rpc",
      "vino_host",
      "vino_transport",
      "vino_codec",
      "vino_manifest",
      "vino_provider",
      "vino_native_provider",
      "vino_provider_cli",
      "vino_wascap",
      "wapc",
      "vow",
    ];

    let chatty_modules: [&str; 0] = [];

    if let Ok(ref filter) = std::env::var(FILTER_ENV) {
      builder.parse(filter);
    } else {
      builder.filter_level(LevelFilter::Off);
      if opts.quiet {
        set_level(&mut builder, &priority_modules, LevelFilter::Error);
      } else if opts.trace {
        set_level(&mut builder, &priority_modules, LevelFilter::Trace);
      } else if opts.debug {
        set_level(&mut builder, &priority_modules, LevelFilter::Debug);
      } else {
        set_level(&mut builder, &priority_modules, LevelFilter::Info);
      }

      for module in chatty_modules.iter() {
        builder.filter_module(module.as_ref(), LevelFilter::Off);
      }
    }

    Logger {
      inner: builder.build(),
      verbose: opts.verbose,
      json: opts.json,
      style: opts.color,
    }
  }
  /// Initialize a logger with the passed options, the modules to log by default, and the modules to silence.
  pub fn init(opts: &LoggingOptions) -> Result<(), LoggerError> {
    let logger = Self::new(&opts);

    set_max_level(logger.inner.filter());

    set_boxed_logger(Box::new(logger))?;
    log::trace!("logger initialized");
    Ok(())
  }

  fn format(&self, record: &Record) -> String {
    let datetime: DateTime<Utc> = SystemTime::now().into();
    let datestring = datetime.format("%Y-%m-%d %T");
    if self.json {
      json!({ "timestamp": datetime.to_rfc3339(), "level": record.level().to_string(), "msg": format!("{}", record.args()) }).to_string()
    } else {
      let level = match record.level() {
        Level::Error => "[E]",
        Level::Warn => "[W]",
        Level::Info => "[I]",
        Level::Debug => "[D]",
        Level::Trace => "[T]",
      };
      let msg = if self.verbose {
        format!(
          "[{}|{}]{} {}",
          datestring,
          level,
          record.module_path().unwrap_or_default(),
          record.args()
        )
      } else {
        format!("[{}]{} {}", datestring, level, record.args())
      };
      if self.style == WriteStyle::Auto || self.style == WriteStyle::Always {
        self.colorize(record.level(), msg)
      } else {
        msg
      }
    }
  }

  fn colorize(&self, level: Level, msg: String) -> String {
    match level {
      Level::Info => msg,
      Level::Debug => msg.blue().dimmed().to_string(),
      Level::Trace => msg.dimmed().to_string(),
      Level::Warn => msg.yellow().to_string(),
      Level::Error => msg.red().to_string(),
    }
  }
}

impl Log for Logger {
  fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
    self.inner.enabled(metadata)
  }

  fn log(&self, record: &Record) {
    // Check if the record is matched by the logger before logging
    if self.inner.matches(record) {
      eprintln!("{}", self.format(record));
    }
  }

  fn flush(&self) {}
}
