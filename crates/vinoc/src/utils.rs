use logger::LoggingOptions;

#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn init_logger(opts: &LoggingOptions) -> crate::Result<()> {
  logger::init(opts);
  Ok(())
}
