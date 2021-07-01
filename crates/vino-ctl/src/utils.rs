use logger::LoggingOptions;

pub fn init_logger(opts: &LoggingOptions) -> crate::Result<()> {
  logger::Logger::init(
    opts,
    &["logger", "vino", "wasmcloud", "wasmcloud_host", "wapc"],
    &[],
  )?;
  Ok(())
}
