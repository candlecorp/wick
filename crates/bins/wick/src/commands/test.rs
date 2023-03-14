use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use wick_component_cli::options::MeshCliOptions;
use wick_component_cli::LoggingOptions;

mod manifest;
mod wasm;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct TestCommand {
  #[clap(flatten)]
  pub(crate) logging: LoggingOptions,

  #[clap(flatten)]
  pub(crate) mesh: MeshCliOptions,

  #[clap(flatten)]
  pub(crate) fetch: super::FetchOptions,

  #[clap(flatten)]
  wasi: crate::wasm::WasiOptions,

  /// Turn on info logging.
  #[clap(long = "info", action)]
  pub(crate) info: bool,

  /// Pass a seed along with the invocation.
  #[clap(long = "seed", short = 's', env = "WICK_SEED", action)]
  seed: Option<u64>,

  /// The path or OCI URL to a wick manifest or wasm file.
  #[clap(action)]
  pub(crate) location: String,

  /// The test data location.
  #[clap(action)]
  data_path: PathBuf,

  /// Filter which tests to run
  #[clap(action)]
  filter: Vec<String>,
}
#[allow(clippy::future_not_send, clippy::too_many_lines)]
pub(crate) async fn handle_command(opts: TestCommand) -> Result<()> {
  let _guard = logger::init(&opts.logging.name(crate::BIN_NAME));

  let bytes = wick_loader_utils::get_bytes(&opts.location, opts.fetch.allow_latest, &opts.fetch.insecure_registries)
    .await
    .context("Could not load from location")?;

  if wick_loader_utils::is_wasm(&bytes) {
    wasm::handle_command(opts, bytes).await
  } else {
    manifest::handle_command(opts, bytes).await
  }
}
