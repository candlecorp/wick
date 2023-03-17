use anyhow::{Context, Result};
use clap::Args;

mod manifest;
mod wasm;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct InvokeCommand {
  #[clap(flatten)]
  pub(crate) logging: super::LoggingOptions,

  #[clap(flatten)]
  wasi: crate::wasm::WasiOptions,

  #[clap(flatten)]
  pub(crate) fetch: super::FetchOptions,

  /// Turn on info logging.
  #[clap(long = "info", action)]
  pub(crate) info: bool,

  /// Path or OCI url to manifest or wasm file.
  #[clap(action)]
  location: String,

  // *****************************************************************
  // Everything below is copied from common-cli-options::RunOptions
  // Flatten doesn't work with positional args...
  //
  // TODO: Eliminate the need for copy/pasting
  // *****************************************************************
  /// Name of the component to execute.
  #[clap(default_value = "default", action)]
  component: String,

  /// Don't read input from STDIN.
  #[clap(long = "no-input", action)]
  no_input: bool,

  /// Skip additional I/O processing done for CLI usage.
  #[clap(long = "raw", short = 'r', action)]
  raw: bool,

  /// Filter the outputs by port name.
  #[clap(long = "filter", action)]
  filter: Vec<String>,

  /// A port=value string where value is JSON to pass as input.
  #[clap(long = "data", short = 'd', action)]
  data: Vec<String>,

  /// Print values only and exit with an error code and string on any errors.
  #[clap(long = "values", short = 'o', action)]
  short: bool,

  /// Pass a seed along with the invocation.
  #[clap(long = "seed", short = 's', env = "WICK_SEED", action)]
  seed: Option<u64>,

  /// Arguments to pass as inputs to a component.
  #[clap(last(true), action)]
  args: Vec<String>,
}

pub(crate) async fn handle_command(mut opts: InvokeCommand) -> Result<()> {
  let mut logging = &mut opts.logging;
  if !(opts.info || logging.trace || logging.debug) {
    logging.quiet = true;
  }
  let _guard = logger::init(&logging.name(crate::BIN_NAME));

  let bytes = wick_loader_utils::get_bytes(&opts.location, opts.fetch.allow_latest, &opts.fetch.insecure_registries)
    .await
    .context("Could not load from location")?;

  if wick_loader_utils::is_wasm(&bytes) {
    wasm::handle_command(opts, bytes).await
  } else {
    manifest::handle_command(opts, bytes).await
  }
}
