pub mod options;

use chrono::{DateTime, Utc};
use colored::Colorize;
use env_logger::filter::{Builder, Filter};

use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use serde_json::json;

use anyhow::Result;
pub use options::LoggingOptions;

const FILTER_ENV: &str = "VINO_LOG";
use std::time::SystemTime;
pub struct Logger {
    inner: Filter,
    verbose: bool,
    json: bool,
}

fn set_level<T: AsRef<str>>(builder: &mut Builder, priority_modules: &[T], level: LevelFilter) {
    for module in priority_modules.iter() {
        builder.filter_module(module.as_ref(), level);
    }
}

impl Logger {
    fn new<T: AsRef<str>>(
        opts: &LoggingOptions,
        priority_modules: &[T],
        chatty_modules: &[T],
    ) -> Logger {
        let mut builder = Builder::new();

        if let Ok(ref filter) = std::env::var(FILTER_ENV) {
            builder.parse(filter);
        } else {
            builder.filter_level(log::LevelFilter::Off);
            if opts.quiet {
                set_level(&mut builder, priority_modules, log::LevelFilter::Error);
            } else if opts.trace {
                set_level(&mut builder, priority_modules, log::LevelFilter::Trace);
            } else if opts.debug {
                set_level(&mut builder, priority_modules, log::LevelFilter::Debug);
            } else {
                set_level(&mut builder, priority_modules, log::LevelFilter::Info);
            }

            for module in chatty_modules.iter() {
                builder.filter_module(module.as_ref(), log::LevelFilter::Off);
            }
        }

        Logger {
            inner: builder.build(),
            verbose: opts.verbose,
            json: opts.json,
        }
    }

    pub fn init<T: AsRef<str>>(
        opts: &LoggingOptions,
        priority_modules: &[T],
        chatty_modules: &[T],
    ) -> Result<(), SetLoggerError> {
        let logger = Self::new(opts, priority_modules, chatty_modules);

        log::set_max_level(logger.inner.filter());
        log::set_boxed_logger(Box::new(logger))?;
        log::trace!("logger initialized");
        Ok(())
    }

    fn format(&self, record: &Record) -> String {
        let datetime: DateTime<Utc> = SystemTime::now().into();
        let datestring = datetime.format("%Y-%m-%d %T");
        if self.json {
            json!({ "timestamp": datetime.to_rfc3339(), "level": record.level().to_string(), "msg": format!("{}", record.args()) }).to_string()
        } else {
            let msg = if self.verbose {
                format!(
                    "[{}|{}] {}",
                    datestring,
                    record.module_path().unwrap_or_default(),
                    record.args()
                )
            } else {
                format!("[{}] {}", datestring, record.args())
            };
            Logger::colorize(record.level(), msg)
        }
    }

    fn colorize(level: Level, msg: String) -> String {
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
    fn enabled(&self, metadata: &Metadata) -> bool {
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
