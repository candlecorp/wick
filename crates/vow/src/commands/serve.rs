use std::sync::Arc;

use once_cell::sync::OnceCell;
use structopt::StructOpt;
use vino_provider_cli::cli::DefaultCliOptions;
use vino_provider_wasm::provider::{
  Provider,
  WasiParams,
};
use vino_rpc::BoxedRpcHandler;

use super::WasiOptions;
use crate::Result;
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub(crate) struct ServeCommand {
  #[structopt(flatten)]
  cli: DefaultCliOptions,

  #[structopt(flatten)]
  pull: super::PullOptions,

  /// Path or URL to WebAssembly binary.
  wasm: String,

  #[structopt(flatten)]
  wasi: WasiOptions,

  /// The number of threads to start.
  #[structopt(long, default_value = "2")]
  #[allow(unused)]
  threads: u8,
}

static PROVIDER: OnceCell<BoxedRpcHandler> = OnceCell::new();

pub(crate) async fn handle_command(opts: ServeCommand) -> Result<()> {
  vino_provider_cli::init_logging(&opts.cli.logging)?;
  debug!("Loading wasm {}", opts.wasm);
  let component =
    vino_provider_wasm::helpers::load_wasm(&opts.wasm, opts.pull.latest, &opts.pull.insecure)
      .await?;

  let wasi: WasiParams = (&opts.wasi).into();
  let provider = PROVIDER.get_or_init(|| {
    Arc::new(
      match Provider::try_load(&component, None, Some(wasi.clone()), None) {
        Ok(provider) => provider,
        Err(e) => {
          error!("Error starting WebAssembly provider: {}", e);
          panic!();
        }
      },
    )
  });
  vino_provider_cli::init_cli(provider.clone(), Some(opts.cli.into())).await?;

  Ok(())
}
