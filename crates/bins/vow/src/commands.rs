pub(crate) mod run;
pub(crate) mod serve;
pub(crate) mod test_cmd;

use clap::{Args, Subcommand};
use vino_provider_wasm::provider::WasiParams;

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum CliCommand {
  /// Execute a component in the target WASM module.
  #[clap(name = "run")]
  Run(run::RunCommand),
  /// Start a persistent RPC, HTTP, or Lattice host for the target WASM module.
  #[clap(name = "serve")]
  Serve(Box<serve::ServeCommand>),
  /// Run a test file against the passed WASM module.
  #[clap(name = "test")]
  Test(test_cmd::TestCommand),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct PullOptions {
  /// Allow ':latest' tag if pulling from an OCI registry.
  #[clap(long = "latest")]
  pub(crate) latest: bool,

  /// Registries to connect via HTTP vs HTTPS.
  #[clap(long = "insecure")]
  pub(crate) insecure: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct WasiOptions {
  /// Directories to expose to the WASM module via WASI.
  #[clap(long = "wasi-dir")]
  wasi_dir: Vec<String>,
}

impl From<&WasiOptions> for WasiParams {
  fn from(opts: &WasiOptions) -> Self {
    WasiParams {
      preopened_dirs: opts.wasi_dir.clone(),
      ..Default::default()
    }
  }
}
