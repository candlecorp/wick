//! Vino's logger crate

// !!START_LINTS
// Vino lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![deny(
    clippy::expect_used,
    clippy::explicit_deref_methods,
    clippy::option_if_let_else,
    clippy::await_holding_lock,
    clippy::cloned_instead_of_copied,
    clippy::explicit_into_iter_loop,
    clippy::flat_map_option,
    clippy::fn_params_excessive_bools,
    clippy::implicit_clone,
    clippy::inefficient_to_string,
    clippy::large_types_passed_by_value,
    clippy::manual_ok_or,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::must_use_candidate,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::redundant_else,
    clippy::semicolon_if_nothing_returned,
    // clippy::too_many_lines,
    clippy::trivially_copy_pass_by_ref,
    clippy::unnested_or_patterns,
    clippy::future_not_send,
    clippy::useless_let_if_seq,
    clippy::str_to_string,
    clippy::inherent_to_string,
    clippy::let_and_return,
    clippy::string_to_string,
    clippy::try_err,
    clippy::if_then_some_else_none,
    bad_style,
    clashing_extern_declarations,
    const_err,
    // dead_code,
    deprecated,
    explicit_outlives_requirements,
    improper_ctypes,
    invalid_value,
    missing_copy_implementations,
    missing_debug_implementations,
    mutable_transmutes,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements ,
    patterns_in_fns_without_body,
    private_in_public,
    trivial_bounds,
    trivial_casts,
    trivial_numeric_casts,
    type_alias_bounds,
    unconditional_recursion,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    // unused,
    unused_allocation,
    unused_comparisons,
    unused_import_braces,
    unused_parens,
    unused_qualifications,
    while_true,
    // missing_docs
)]
// !!END_LINTS
// Add exceptions here
#![allow(clippy::trivially_copy_pass_by_ref, clippy::needless_borrow)]

/// Error module for Logger
pub mod error;

/// Logger options
pub mod options;

use std::time::SystemTime;

use anyhow::Result;
use chrono::{
  DateTime,
  Utc,
};
use colored::Colorize;
use env_logger::filter::{
  Builder,
  Filter,
};
use log::{
  Level,
  LevelFilter,
  Log,
  Metadata,
  Record,
  SetLoggerError,
};
pub use options::LoggingOptions;
use serde_json::json;

use self::error::LoggerError;

const FILTER_ENV: &str = "VINO_LOG";

/// The logger instance
#[derive(Debug)]
pub struct Logger {
  inner: Filter,
  verbose: bool,
  json: bool,
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
      builder.filter_level(log::LevelFilter::Off);
      if opts.quiet {
        set_level(&mut builder, &priority_modules, log::LevelFilter::Error);
      } else if opts.trace {
        set_level(&mut builder, &priority_modules, log::LevelFilter::Trace);
      } else if opts.debug {
        set_level(&mut builder, &priority_modules, log::LevelFilter::Debug);
      } else {
        set_level(&mut builder, &priority_modules, log::LevelFilter::Info);
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
  /// Initialize a logger with the passed options, the modules to log by default, and the modules to silence.
  pub fn init(opts: &LoggingOptions) -> Result<(), LoggerError> {
    let logger = Self::new(&opts);

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
