use std::io::Write;

use env_logger::fmt::{
  Color,
  Formatter,
};
use env_logger::Builder;
use log::{
  Level,
  LevelFilter,
  Record,
};
use serde_json::json;

use crate::{
  LoggingOptions,
  FILTER_ENV,
};

fn set_level(builder: &mut Builder, priority_modules: &[&str], level: LevelFilter) {
  for module in priority_modules.iter() {
    builder.filter_module(module.as_ref(), level);
  }
}

/// The logger instance.
pub fn init(opts: &LoggingOptions) {
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
    builder.parse_filters(filter);
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
  let json = opts.json;
  let verbose = opts.verbose;

  builder
    .format(move |buf, record| format(buf, record, json, verbose))
    .init();
  log::trace!("Logger initialized");
}

fn format(buf: &mut Formatter, record: &Record, json: bool, verbose: bool) -> std::io::Result<()> {
  let mut style = buf.style();
  let msg = if json {
    json!({ "timestamp": buf.timestamp().to_string(), "level": record.level().to_string(), "msg": format!("{}", record.args()) }).to_string()
  } else {
    let timestamp = buf.timestamp();

    let level = match record.level() {
      Level::Error => {
        style.set_color(Color::Red);
        "[E]"
      }
      Level::Warn => {
        style.set_color(Color::Yellow);
        "[W]"
      }
      Level::Info => "[I]",
      Level::Debug => {
        style.set_color(Color::Blue);
        "[D]"
      }
      Level::Trace => {
        style.set_color(Color::Magenta);
        "[T]"
      }
    };
    let msg = if verbose {
      format!(
        "[{}|{}]{} {}",
        timestamp,
        level,
        record.module_path().unwrap_or_default(),
        record.args()
      )
    } else {
      format!("[{}]{} {}", timestamp, level, record.args())
    };

    msg
  };
  writeln!(buf, "{}", style.value(msg))
}
