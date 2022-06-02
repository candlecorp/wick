use anyhow::{Context, Result};
use clap::Args;
use wasmflow_collection_cli::options::LatticeCliOptions;
mod manifest;
mod wasm;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct RunCommand {
  #[clap(flatten)]
  pub(crate) logging: super::LoggingOptions,

  #[clap(flatten)]
  pub(crate) lattice: LatticeCliOptions,

  #[clap(flatten)]
  pub(crate) fetch: super::FetchOptions,

  /// The path or OCI URL to a wafl manifest or wasm file.
  location: String,

  /// Pass a seed along with the invocation.
  #[clap(long = "seed", short = 's', env = "WAFL_SEED")]
  seed: Option<u64>,

  /// Arguments to pass as inputs to a schematic.
  #[clap(last(true))]
  args: Vec<String>,
}

pub(crate) async fn handle_command(opts: RunCommand) -> Result<()> {
  let _guard = logger::init(&opts.logging.name(crate::BIN_NAME));

  debug!(args = ?opts.args, "rest args");

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
