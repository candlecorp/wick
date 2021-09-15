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

use crate::error::LoggerError;
use crate::{
  LoggingOptions,
  FILTER_ENV,
};

fn set_level(builder: &mut Builder, priority_modules: &[&str], level: LevelFilter) {
  for module in priority_modules.iter() {
    builder.filter_module(module.as_ref(), level);
  }
}

/// Initialize a logger or panic on failure
pub fn init_defaults() {
  if let Err(e) = try_init(&LoggingOptions::default()) {
    panic!("Error initializing logger: {}", e);
  }
}

/// Initialize a logger or panic on failure
pub fn init(opts: &LoggingOptions) {
  if let Err(e) = try_init(opts) {
    panic!("Error initializing logger: {}", e);
  }
}

/// Initialize a logger
pub fn try_init(opts: &LoggingOptions) -> Result<(), LoggerError> {
  let mut builder = Builder::new();

  let priority_modules = [
    "logger",
    "oci_utils",
    "vinoc",
    "vino",
    "vino_cli",
    "vino_host",
    "vino_invocation_server",
    "vino_lattice",
    "vino_loader",
    "vino_macros",
    "vino_manifest",
    "vino_provider_cli",
    "vino_provider_wasm",
    "vino_rpc",
    "vino_runtime",
    "vino_wascap",
    "vino_codec",
    "vino_entity",
    "vino_macros",
    "vino_packet",
    "vino_provider",
    "vino_transport",
    "vino_types",
    "vino_root",
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
    .try_init()?;
  log::trace!("Logger initialized");
  Ok(())
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
