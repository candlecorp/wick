use anyhow::Context;
use logger::LoggingOptions;

pub fn init_logger(opts: &LoggingOptions) -> crate::Result<()> {
    logger::Logger::init(
        &opts,
        &[
            "logger",
            "vino",
            "vinoctl",
            "wasmcloud",
            "wasmcloud_host",
            "wapc",
        ],
        &[],
    )
    .context("Could not initialize logger")?;
    Ok(())
}
