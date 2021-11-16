pub(crate) mod run;
pub(crate) mod serve;
pub(crate) mod test;

use structopt::StructOpt;
use vino_provider_wasm::provider::WasiParams;

#[derive(Debug, Clone, StructOpt)]
pub(crate) enum CliCommand {
  /// Execute a component in the target WASM module.
  #[structopt(name = "run")]
  Run(run::RunCommand),
  /// Start a persistent RPC, HTTP, or Lattice host for the target WASM module.
  #[structopt(name = "serve")]
  Serve(Box<serve::ServeCommand>),
  /// Run a test file against the passed WASM module.
  #[structopt(name = "test")]
  Test(test::TestCommand),
}

#[derive(Debug, Clone, StructOpt)]
pub(crate) struct PullOptions {
  /// Allow ':latest' tag if pulling from an OCI registry.
  #[structopt(long)]
  pub(crate) latest: bool,

  /// Registries to connect via HTTP vs HTTPS.
  #[structopt(long)]
  pub(crate) insecure: Vec<String>,
}

#[derive(Debug, Clone, StructOpt)]
pub(crate) struct WasiOptions {
  /// Directories to expose to the WASM module via WASI.
  #[structopt(long = "wasi-dir")]
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
