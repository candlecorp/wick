use std::path::PathBuf;

use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::filter::FilterFn;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{filter, Layer};

use crate::error::LoggerError;
use crate::LoggingOptions;

#[derive(Debug)]
enum Environment {
  Prod,
  Test,
}

/// Initialize a logger or panic on failure
pub fn init_defaults() -> LoggingGuard {
  match try_init(&LoggingOptions::default(), &Environment::Prod) {
    Ok(guard) => guard,
    Err(e) => panic!("Error initializing logger: {}", e),
  }
}

/// Initialize a logger or panic on failure
pub fn init(opts: &LoggingOptions) -> LoggingGuard {
  match try_init(&opts, &Environment::Prod) {
    Ok(guard) => guard,
    Err(e) => panic!("Error initializing logger: {}", e),
  }
}

/// Initialize a logger for tests
#[must_use]
pub fn init_test(opts: &LoggingOptions) -> Option<LoggingGuard> {
  match try_init(&opts, &Environment::Test) {
    Ok(guard) => Some(guard),
    Err(_) => None,
  }
}

fn priority_module(module: &str) -> bool {
  [
    "logger",
    "vow",
    "cli_common",
    "wapc",
    "wafl",
    "test_native_collection",
    "wasmtime_provider",
  ]
  .contains(&module)
}

#[must_use]
fn wasmflow_filter() -> FilterFn {
  FilterFn::new(|e| {
    let module = &e
      .module_path()
      .unwrap_or_default()
      .split("::")
      .next()
      .unwrap_or_default();
    priority_module(module) || module.starts_with("wafl") || module.starts_with("wasmflow")
  })
}

#[must_use]
#[derive(Debug)]
/// Guard that - when dropped - flushes all log messages and drop I/O handles.
pub struct LoggingGuard {
  #[allow(unused)]
  logfile: Option<WorkerGuard>,
  #[allow(unused)]
  console: WorkerGuard,
}

impl LoggingGuard {
  fn new(logfile: Option<WorkerGuard>, console: WorkerGuard) -> Self {
    Self { logfile, console }
  }
}

fn get_stderr_writer(_opts: &LoggingOptions) -> (NonBlocking, WorkerGuard) {
  let (stderr_writer, console_guard) = tracing_appender::non_blocking(std::io::stderr());

  (stderr_writer, console_guard)
}

fn get_logfile_writer(opts: &LoggingOptions) -> Result<(PathBuf, NonBlocking, WorkerGuard), LoggerError> {
  let logfile_prefix = format!("{}.{}.log", opts.app_name, std::process::id());

  let mut log_dir = match &opts.log_dir {
    Some(dir) => dir.clone(),
    None => {
      #[cfg(not(target_os = "windows"))]
      match xdg::BaseDirectories::with_prefix("wafl") {
        Ok(xdg) => xdg.get_state_home(),
        Err(_) => std::env::current_dir()?,
      }
      #[cfg(target_os = "windows")]
      match std::env::var("LOCALAPPDATA") {
        Ok(localappdata) => PathBuf::from(format!("{}/wafl", localappdata)),
        Err(_) => std::env::current_dir()?,
      }
    }
  };
  log_dir.push("logs");

  let (writer, guard) =
    tracing_appender::non_blocking(tracing_appender::rolling::daily(log_dir.clone(), logfile_prefix));

  Ok((log_dir, writer, guard))
}

fn get_levelfilter(opts: &LoggingOptions) -> tracing::level_filters::LevelFilter {
  if opts.quiet {
    filter::LevelFilter::ERROR
  } else if opts.trace {
    filter::LevelFilter::TRACE
  } else if opts.debug {
    filter::LevelFilter::DEBUG
  } else {
    filter::LevelFilter::INFO
  }
}

fn try_init(opts: &LoggingOptions, environment: &Environment) -> Result<LoggingGuard, LoggerError> {
  #[cfg(windows)]
  let with_color = ansi_term::enable_ansi_support().is_ok();
  #[cfg(not(windows))]
  let with_color = true;

  let timer = UtcTime::new(time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]").unwrap());
  let (stderr_writer, console_guard) = get_stderr_writer(opts);

  let app_name = opts.app_name.clone();

  let (log_dir, logfile_writer, logfile_guard) = get_logfile_writer(opts)?;
  let file_layer = BunyanFormattingLayer::new(app_name, logfile_writer).with_filter(wasmflow_filter());

  // This is ugly. If you can improve it, go for it, but
  // start here to understand why it's laid out like this: https://github.com/tokio-rs/tracing/issues/575
  let (verbose_layer, normal_layer, json_layer, file_layer, logfile_guard, test_layer) = match environment {
    Environment::Prod => {
      if opts.verbose {
        (
          Some(
            tracing_subscriber::fmt::layer()
              .with_writer(stderr_writer)
              .with_thread_names(true)
              .with_ansi(with_color)
              .with_target(true)
              .with_filter(get_levelfilter(opts))
              .with_filter(wasmflow_filter()),
          ),
          None,
          Some(JsonStorageLayer),
          Some(file_layer),
          Some(logfile_guard),
          None,
        )
      } else {
        (
          None,
          Some(
            tracing_subscriber::fmt::layer()
              .with_writer(stderr_writer)
              .with_ansi(with_color)
              .with_target(false)
              .with_thread_names(false)
              .with_timer(timer)
              .with_filter(get_levelfilter(opts))
              .with_filter(wasmflow_filter()),
          ),
          Some(JsonStorageLayer),
          Some(file_layer),
          Some(logfile_guard),
          None,
        )
      }
    }
    Environment::Test => (
      None,
      None,
      Some(JsonStorageLayer),
      Some(file_layer),
      Some(logfile_guard),
      Some(
        tracing_subscriber::fmt::layer()
          .with_writer(stderr_writer)
          .with_ansi(with_color)
          .without_time()
          .with_target(false)
          .with_test_writer()
          .with_filter(get_levelfilter(opts))
          .with_filter(wasmflow_filter()),
      ),
    ),
  };

  let subscriber = tracing_subscriber::registry()
    .with(test_layer)
    .with(verbose_layer)
    .with(normal_layer)
    .with(json_layer)
    .with(file_layer);
  tracing::subscriber::set_global_default(subscriber)?;

  trace!("Logger initialized");
  debug!("Writing logs to {}", log_dir.to_string_lossy());
  Ok(LoggingGuard::new(logfile_guard, console_guard))
}
