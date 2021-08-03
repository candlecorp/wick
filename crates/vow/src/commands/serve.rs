use std::sync::Arc;

use structopt::StructOpt;
use tokio::sync::Mutex;
use vino_provider_cli::cli::DefaultCliOptions;
use vino_provider_wasm::provider::Provider;

use crate::Result;
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub(crate) struct ServeCommand {
  /// Path or URL to WebAssembly binary
  pub(crate) wasm: String,

  #[structopt(flatten)]
  pub(crate) cli: DefaultCliOptions,

  #[structopt(flatten)]
  pub(crate) pull: super::PullOptions,
}

pub(crate) async fn handle_command(opts: ServeCommand) -> Result<()> {
  vino_provider_cli::init_logging(&opts.cli.logging)?;
  debug!("Loading wasm {}", opts.wasm);
  let component =
    vino_provider_wasm::helpers::load_wasm(&opts.wasm, opts.pull.latest, &opts.pull.insecure)
      .await?;

  vino_provider_cli::init_cli(
    Arc::new(Mutex::new(Provider::try_from_module(component, 5)?)),
    Some(opts.cli.into()),
  )
  .await?;

  Ok(())
}
