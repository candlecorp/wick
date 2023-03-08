use std::path::PathBuf;

use opentelemetry::trace::TracerProvider;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::filter::FilterFn;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{filter, Layer};

use crate::error::LoggerError;
use crate::LoggingOptions;

#[derive(Debug, PartialEq)]
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
  #![allow(clippy::trivially_copy_pass_by_ref, clippy::needless_borrow)]
  match try_init(&opts, &Environment::Prod) {
    Ok(guard) => guard,
    Err(e) => panic!("Error initializing logger: {}", e),
  }
}

/// Initialize a logger for tests
#[must_use]
pub fn init_test(opts: &LoggingOptions) -> Option<LoggingGuard> {
  #![allow(clippy::trivially_copy_pass_by_ref, clippy::needless_borrow)]
  try_init(&opts, &Environment::Test).map_or_else(|_e| None, Some)
}

fn hushed_modules(module: &str) -> bool {
  ["h2", "tokio_util", "tower", "tonic", "hyper", "wasi_common"].contains(&module)
}

fn silly_modules(module: &str) -> bool {
  [
    "flow_graph_interpreter",
    "wasmtime_provider",
    "wapc",
    "wick_wascap",
    "flow_graph",
    "wick_config_component",
  ]
  .contains(&module)
}

#[must_use]
fn wick_filter(opts: &LoggingOptions) -> FilterFn {
  // This is split up into an if/else because FilterFn needs an fn type.
  // If the closure captures opts.silly then it won't be coercable to an fn.
  if opts.silly {
    FilterFn::new(move |e| {
      let module = &e
        .module_path()
        .unwrap_or_default()
        .split("::")
        .next()
        .unwrap_or_default();
      !hushed_modules(module)
    })
  } else {
    FilterFn::new(move |e| {
      let module = &e
        .module_path()
        .unwrap_or_default()
        .split("::")
        .next()
        .unwrap_or_default();
      !hushed_modules(module) && !silly_modules(module)
    })
  }
}

#[must_use]
#[derive(Debug)]
/// Guard that - when dropped - flushes all log messages and drop I/O handles.
pub struct LoggingGuard {
  #[allow(unused)]
  logfile: Option<WorkerGuard>,
  #[allow(unused)]
  console: WorkerGuard,
  #[allow(unused)]
  tracer_provider: Option<opentelemetry::sdk::trace::TracerProvider>,
}

impl LoggingGuard {
  fn new(
    logfile: Option<WorkerGuard>,
    console: WorkerGuard,
    tracer_provider: Option<opentelemetry::sdk::trace::TracerProvider>,
  ) -> Self {
    Self {
      logfile,
      console,
      tracer_provider,
    }
  }
  /// Call this function when you are done with the logger.
  pub fn teardown(&self) {
    // noop right now
  }
}

impl Drop for LoggingGuard {
  fn drop(&mut self) {
    if let Some(provider) = &self.tracer_provider {
      for result in provider.force_flush() {
        if let Err(_err) = result {
          println!("error flushing");
        }
      }
    }
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
      match xdg::BaseDirectories::with_prefix("wick") {
        Ok(xdg) => xdg.get_state_home(),
        Err(_) => std::env::current_dir()?,
      }
      #[cfg(target_os = "windows")]
      match std::env::var("LOCALAPPDATA") {
        Ok(localappdata) => PathBuf::from(format!("{}/wick", localappdata)),
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

#[allow(clippy::too_many_lines)]
fn try_init(opts: &LoggingOptions, environment: &Environment) -> Result<LoggingGuard, LoggerError> {
  #[cfg(windows)]
  let with_color = ansi_term::enable_ansi_support().is_ok();
  #[cfg(not(windows))]
  let with_color = true;

  let timer = UtcTime::new(time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]").unwrap());
  let (stderr_writer, console_guard) = get_stderr_writer(opts);

  let app_name = opts.app_name.clone();

  let (log_dir, logfile_writer, logfile_guard) = get_logfile_writer(opts)?;
  let file_layer = BunyanFormattingLayer::new(app_name, logfile_writer).with_filter(wick_filter(opts));

  let needs_simple_tracer = tokio::runtime::Handle::try_current().is_err() || environment == &Environment::Test;

  // Configure a jaeger tracer if we have a configured endpoint.
  let (otel_layer, tracer_provider) = opts.jaeger_endpoint.as_ref().map_or_else(
    || (None, None),
    |jaeger_endpoint| {
      let tracer_provider = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("wick")
        .with_endpoint(jaeger_endpoint);

      let tracer_provider = if needs_simple_tracer {
        tracer_provider.build_simple().unwrap()
      } else {
        tracer_provider.build_batch(opentelemetry::runtime::Tokio).unwrap()
      };

      let tracer = tracer_provider.versioned_tracer("wick", Some(env!("CARGO_PKG_VERSION")), None);
      let _ = opentelemetry::global::set_tracer_provider(tracer_provider.clone());

      let layer = Some(
        tracing_opentelemetry::layer()
          .with_tracer(tracer)
          .with_filter(get_levelfilter(opts))
          .with_filter(wick_filter(opts)),
      );
      (layer, Some(tracer_provider))
    },
  );

  // This is ugly. If you can improve it, go for it, but
  // start here to understand why it's laid out like this: https://github.com/tokio-rs/tracing/issues/575
  let (verbose_layer, normal_layer, json_layer, file_layer, logfile_guard, otel_layer, test_layer) = match environment {
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
              .with_filter(wick_filter(opts)),
          ),
          None,
          Some(JsonStorageLayer),
          Some(file_layer),
          Some(logfile_guard),
          Some(otel_layer),
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
              .with_filter(wick_filter(opts)),
          ),
          Some(JsonStorageLayer),
          Some(file_layer),
          Some(logfile_guard),
          Some(otel_layer),
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
      Some(otel_layer),
      Some(
        tracing_subscriber::fmt::layer()
          .with_writer(stderr_writer)
          .with_ansi(with_color)
          .without_time()
          .with_target(true)
          .with_test_writer()
          .with_filter(get_levelfilter(opts))
          .with_filter(wick_filter(opts)),
      ),
    ),
  };

  let subscriber = tracing_subscriber::registry()
    .with(test_layer)
    .with(verbose_layer)
    .with(normal_layer)
    .with(json_layer)
    .with(otel_layer)
    .with(file_layer);
  tracing::subscriber::set_global_default(subscriber)?;

  trace!("Logger initialized");
  debug!("Writing logs to {}", log_dir.to_string_lossy());
  Ok(LoggingGuard::new(logfile_guard, console_guard, tracer_provider))
}
