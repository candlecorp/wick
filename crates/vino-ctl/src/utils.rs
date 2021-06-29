use logger::LoggingOptions;

fn this_or_that_option<T>(a: Option<T>, b: Option<T>) -> Option<T> {
  if a.is_some() {
    a
  } else {
    b
  }
}

pub fn init_logger(opts: &LoggingOptions) -> crate::Result<()> {
  logger::Logger::init(
    opts,
    &["logger", "vino", "wasmcloud", "wasmcloud_host", "wapc"],
    &[],
  )?;
  Ok(())
}
