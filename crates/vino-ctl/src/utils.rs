use logger::LoggingOptions;

pub fn init_logger(opts: &LoggingOptions) -> crate::Result<()> {
  logger::Logger::init(
    opts,
    &[
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
    ],
    &[],
  )?;
  Ok(())
}
