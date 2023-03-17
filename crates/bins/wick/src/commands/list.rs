use anyhow::{Context, Result};
use clap::Args;
use logger::LoggingOptions;
mod manifest;
mod wasm;

#[derive(Debug, Clone, Args)]
pub(crate) struct ListCommand {
  #[clap(flatten)]
  pub(crate) fetch: super::FetchOptions,

  /// The path or OCI URL to a wick manifest or wasm file.
  #[clap(action)]
  pub(crate) location: String,

  #[clap(flatten)]
  pub(crate) logging: LoggingOptions,

  #[clap(long = "json", action)]
  pub(crate) json: bool,
}

pub(crate) async fn handle_command(opts: ListCommand) -> Result<()> {
  let _guard = logger::init(&opts.logging.name(crate::BIN_NAME));

  let bytes = wick_loader_utils::get_bytes(&opts.location, opts.fetch.allow_latest, &opts.fetch.insecure_registries)
    .await
    .context("Could not load from location")?;

  if wick_loader_utils::is_wasm(&bytes) {
    todo!("The list command is not yet enabled for wasm modules.");
  } else {
    manifest::handle_command(opts, bytes).await
  }
}
