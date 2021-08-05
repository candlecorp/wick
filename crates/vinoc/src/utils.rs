use logger::LoggingOptions;

pub fn init_logger(opts: &LoggingOptions) -> crate::Result<()> {
  logger::init(opts);
  Ok(())
}
