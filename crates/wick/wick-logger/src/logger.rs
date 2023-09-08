use opentelemetry::global;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Layer;
mod otel;

use crate::error::LoggerError;
use crate::LoggingOptions;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Environment {
  Prod,
  Test,
}

/// Initialize a logger or panic on failure
pub fn init(opts: &LoggingOptions) -> LoggingGuard {
  #![allow(clippy::trivially_copy_pass_by_ref, clippy::needless_borrow)]
  match try_init(&opts, Environment::Prod) {
    Ok(guard) => guard,
    Err(e) => panic!("Error initializing logger: {}", e),
  }
}

/// Initialize a logger for tests
#[must_use]
pub fn init_test(opts: &LoggingOptions) -> Option<LoggingGuard> {
  #![allow(clippy::trivially_copy_pass_by_ref, clippy::needless_borrow)]
  try_init(&opts, Environment::Test).ok()
}

#[must_use]
#[derive(Debug)]
/// Guard that - when dropped - flushes all log messages and drop I/O handles.
pub struct LoggingGuard {
  #[allow(unused)]
  env: Environment,
  #[allow(unused)]
  logfile: Option<WorkerGuard>,
  #[allow(unused)]
  console: WorkerGuard,
  #[allow(unused)]
  tracer_provider: Option<opentelemetry::sdk::trace::TracerProvider>,
}

impl LoggingGuard {
  fn new(
    env: Environment,
    logfile: Option<WorkerGuard>,
    console: WorkerGuard,
    tracer_provider: Option<opentelemetry::sdk::trace::TracerProvider>,
  ) -> Self {
    Self {
      env,
      logfile,
      console,
      tracer_provider,
    }
  }
  /// Call this function when you are done with the logger.
  #[allow(clippy::missing_const_for_fn)]
  pub fn teardown(&self) {
    // noop right now
  }

  /// Flush any remaining logs.
  pub fn flush(&mut self) {
    let has_otel = self.tracer_provider.take().is_some();

    if has_otel {
      // Shut down the global tracer provider.
      // This has to be done in a separate thread because it will deadlock
      // if any of its requests have stalled.
      // See: https://github.com/open-telemetry/opentelemetry-rust/issues/868
      let (sender, receiver) = std::sync::mpsc::channel();
      let handle = std::thread::spawn(move || {
        opentelemetry::global::shutdown_tracer_provider();
        let _ = sender.send(());
      });

      // Wait a bit to see if the shutdown completes gracefully.
      let _ = receiver.recv_timeout(std::time::Duration::from_millis(200));

      // Otherwise, issue a warning because opentelemetry will complain
      // and we want to add context to the warning.
      if !handle.is_finished() {
        debug!("open telemetry tracer provider did not shut down in time, forcing shutdown");
      }
    }
  }
}

impl Drop for LoggingGuard {
  fn drop(&mut self) {
    self.flush();
  }
}

fn get_stderr_writer(_opts: &LoggingOptions) -> (NonBlocking, WorkerGuard) {
  let (stderr_writer, console_guard) = tracing_appender::non_blocking(std::io::stderr());

  (stderr_writer, console_guard)
}

#[allow(clippy::too_many_lines)]
fn try_init(opts: &LoggingOptions, environment: Environment) -> Result<LoggingGuard, LoggerError> {
  #[cfg(windows)]
  let with_color = ansi_term::enable_ansi_support().is_ok();
  #[cfg(not(windows))]
  let with_color = true;

  let timer = UtcTime::new(time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]").unwrap());
  let (stderr_writer, console_guard) = get_stderr_writer(opts);

  let needs_simple_tracer = tokio::runtime::Handle::try_current().is_err() || environment == Environment::Test;

  // Configure a jaeger tracer if we have a configured endpoint.
  let (otel_layer, tracer_provider) = opts.otlp_endpoint.as_ref().map_or_else(
    || (None, None),
    |otlp_endpoint| {
      let (tracer, provider) = if needs_simple_tracer {
        otel::build_simple(otlp_endpoint).unwrap()
      } else {
        otel::build_batch(otlp_endpoint).unwrap() // unwrap OK for now, this is infallible.
      };

      let _ = global::set_tracer_provider(provider.clone());

      let layer = Some(
        tracing_opentelemetry::layer()
          .with_tracer(tracer)
          .with_filter(opts.levels.telemetry.clone()),
      );
      (layer, Some(provider))
    },
  );

  // This is ugly. If you can improve it, go for it, but
  // start here to understand why it's laid out like this: https://github.com/tokio-rs/tracing/issues/575
  let (verbose_layer, normal_layer, logfile_guard, test_layer) = match environment {
    Environment::Prod => {
      if opts.verbose {
        (
          Some(
            tracing_subscriber::fmt::layer()
              .with_writer(stderr_writer)
              .with_ansi(with_color)
              .with_timer(timer)
              .with_thread_names(cfg!(debug_assertions))
              .with_target(cfg!(debug_assertions))
              .with_file(cfg!(debug_assertions))
              .with_line_number(cfg!(debug_assertions))
              .with_filter(opts.levels.stderr.clone()),
          ),
          None,
          None,
          None,
        )
      } else {
        (
          None,
          Some(
            tracing_subscriber::fmt::layer()
              .with_writer(stderr_writer)
              .with_thread_names(false)
              .with_ansi(with_color)
              .with_target(false)
              .with_timer(timer)
              .with_filter(opts.levels.stderr.clone()),
          ),
          None,
          None,
        )
      }
    }
    Environment::Test => (
      None,
      None,
      None,
      Some(
        tracing_subscriber::fmt::layer()
          .with_writer(stderr_writer)
          .with_ansi(with_color)
          .without_time()
          .with_target(true)
          .with_test_writer()
          .with_filter(opts.levels.stderr.clone()),
      ),
    ),
  };

  let subscriber = tracing_subscriber::registry()
    .with(otel_layer)
    .with(test_layer)
    .with(verbose_layer)
    .with(normal_layer);

  #[cfg(feature = "console")]
  let subscriber = subscriber.with(console_subscriber::spawn());

  tracing::subscriber::set_global_default(subscriber)?;
  let guards = Ok(LoggingGuard::new(
    environment,
    logfile_guard,
    console_guard,
    tracer_provider,
  ));
  trace!(options=?opts,"logger initialized");

  guards
}
