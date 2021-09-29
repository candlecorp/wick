use std::path::PathBuf;

use structopt::StructOpt;
use vino_host::HostBuilder;
use vino_manifest::host_definition::HostDefinition;
use vino_provider_cli::cli::{
  DefaultCliOptions,
  LatticeCliOptions,
};
use vino_provider_cli::LoggingOptions;

use crate::utils::merge_config;
use crate::{
  Error,
  Result,
};

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub(crate) struct TestCommand {
  #[structopt(flatten)]
  pub(crate) logging: LoggingOptions,

  #[structopt(flatten)]
  pub(crate) lattice: LatticeCliOptions,

  #[structopt(flatten)]
  pub(crate) host: super::HostOptions,

  /// Turn on info logging.
  #[structopt(long = "info")]
  pub(crate) info: bool,

  /// Don't read input from STDIN.
  #[structopt(long = "no-input")]
  pub(crate) no_input: bool,

  /// A port=value string where value is JSON to pass as input.
  #[structopt(long, short)]
  data: Vec<String>,

  /// Skip additional I/O processing done for CLI usage.
  #[structopt(long, short)]
  raw: bool,

  /// Manifest file or OCI url.
  manifest: String,

  /// The test data.
  data_path: PathBuf,
}
#[allow(clippy::future_not_send, clippy::too_many_lines)]
pub(crate) async fn handle_command(opts: TestCommand) -> Result<()> {
  let mut logging = opts.logging;
  if !(opts.info || opts.logging.trace || opts.logging.debug) {
    logging.quiet = true;
  }
  logger::init(&logging);

  let manifest_src = vino_loader::get_bytes(
    &opts.manifest,
    opts.host.allow_latest,
    &opts.host.insecure_registries,
  )
  .await
  .map_err(|e| crate::error::VinoError::ManifestLoadFail(e.to_string()))?;

  let manifest = HostDefinition::load_from_bytes(&manifest_src)?;

  let server_options = DefaultCliOptions {
    lattice: opts.lattice,
    ..Default::default()
  };

  let config = merge_config(manifest, opts.host, Some(server_options));

  let host_builder = HostBuilder::from_definition(config);

  let mut host = host_builder.build();
  host.connect_to_lattice().await?;
  host.start_network().await?;

  let provider: vino_host::Provider = host.into();

  let file = opts.data_path.to_string_lossy().to_string();
  let name = format!("Vino test for : {}", file);
  let data = vino_test::read_data(opts.data_path.clone())
    .map_err(|e| Error::FileNotFound(file, e.to_string()))?;

  let harness = vino_test::run_test(name, data, Box::new(provider))
    .await
    .map_err(|e| Error::TestError(e.to_string()))?;

  harness.print();

  Ok(())
}
