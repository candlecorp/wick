use std::path::PathBuf;
use std::sync::Arc;

use structopt::StructOpt;
use vino_provider_cli::LoggingOptions;
use vino_provider_wasm::provider::Provider;
use vino_test::TestSuite;

use super::WasiOptions;
use crate::Result;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub(crate) struct TestCommand {
  #[structopt(flatten)]
  logging: LoggingOptions,

  #[structopt(flatten)]
  pull: super::PullOptions,

  #[structopt(flatten)]
  wasi: WasiOptions,

  /// Path or URL to WebAssembly binary.
  wasm: String,

  /// The path to the data file.
  data_path: PathBuf,
}
#[allow(clippy::future_not_send, clippy::too_many_lines)]
pub(crate) async fn handle_command(opts: TestCommand) -> Result<()> {
  let _guard = vino_provider_cli::init_logging(&opts.logging.name("vow"));

  debug!("Loading wasm {}", opts.wasm);
  let component =
    vino_provider_wasm::helpers::load_wasm(&opts.wasm, opts.pull.latest, &opts.pull.insecure)
      .await?;

  let provider = Provider::try_load(&component, 1, None, Some((&opts.wasi).into()), None)?;

  let mut suite = TestSuite::try_from_file(opts.data_path.clone())?;

  let runner = suite.run(Arc::new(provider)).await?;
  runner.print();

  Ok(())
}
