use anyhow::{Context, Result};
use clap::Args;
use logger::LoggingOptions;
use wasmflow_collection_cli::options::LatticeCliOptions;
mod manifest;
mod wasm;

#[derive(Debug, Clone, Args)]
pub(crate) struct ListCommand {
  #[clap(flatten)]
  pub(crate) fetch: super::FetchOptions,

  /// The path or OCI URL to a wafl manifest or wasm file.
  pub(crate) location: String,

  #[clap(flatten)]
  pub(crate) logging: LoggingOptions,

  #[clap(flatten)]
  pub(crate) lattice: LatticeCliOptions,

  #[clap(long = "json")]
  pub(crate) json: bool,
}

pub(crate) async fn handle_command(opts: ListCommand) -> Result<()> {
  let _guard = logger::init(&opts.logging.name(crate::BIN_NAME));

  let bytes = wasmflow_loader::get_bytes(&opts.location, opts.fetch.allow_latest, &opts.fetch.insecure_registries)
    .await
    .context("Could not load from location")?;

  if crate::wasm::is_wasm(&bytes) {
    todo!()
    // wasm::handle_command(opts, bytes).await
  } else {
    manifest::handle_command(opts, bytes).await
  }
}
