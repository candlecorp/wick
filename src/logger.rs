use chrono::{DateTime, Utc};
use colored::Colorize;
use env_logger::filter::{Builder, Filter};

use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use serde_json::json;
use structopt::lazy_static::lazy_static;

use crate::commands::LoggingOpts;
use anyhow::Result;

const FILTER_ENV: &str = "VINO_LOG";
use std::time::SystemTime;
pub struct Logger {
    inner: Filter,
    verbose: bool,
    json: bool,
}

lazy_static! {
    static ref RELEVANT_MODULES: Vec<&'static str> = vec!["vino", "wasmcloud", "wasmcloud_host"];
}
fn set_level(builder: &mut Builder, level: LevelFilter) {
    for module in RELEVANT_MODULES.iter() {
        builder.filter_module(module, level);
    }
}

impl Logger {
    fn new(opts: &LoggingOpts) -> Logger {
        let mut builder = Builder::new();

        builder.filter_level(log::LevelFilter::Off);

        if let Ok(ref filter) = std::env::var(FILTER_ENV) {
            builder.parse(filter);
        }

        if opts.quiet {
            set_level(&mut builder, log::LevelFilter::Off);
        } else if opts.trace {
            set_level(&mut builder, log::LevelFilter::Trace);
        } else if opts.debug {
            set_level(&mut builder, log::LevelFilter::Debug);
        } else {
            set_level(&mut builder, log::LevelFilter::Info);
        }

        Logger {
            inner: builder.build(),
            verbose: opts.verbose,
            json: opts.json,
        }
    }

    pub fn init(opts: &LoggingOpts) -> Result<(), SetLoggerError> {
        let logger = Self::new(opts);

        log::set_max_level(logger.inner.filter());
        log::set_boxed_logger(Box::new(logger))
    }

    fn format(&self, record: &Record) -> String {
        let datetime: DateTime<Utc> = SystemTime::now().into();
        let datestring = datetime.format("%Y-%m-%d %T");
        if self.json {
            json!({ "timestamp": datetime.to_rfc3339(), "level": record.level(), "msg": format!("{}", record.args()) }).to_string()
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
            println!("{}", self.format(record));
        }
    }

    fn flush(&self) {}
}
