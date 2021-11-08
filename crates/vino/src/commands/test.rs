use std::path::PathBuf;
use std::sync::Arc;

use structopt::StructOpt;
use vino_host::HostBuilder;
use vino_manifest::host_definition::HostDefinition;
use vino_provider_cli::options::{DefaultCliOptions, LatticeCliOptions};
use vino_provider_cli::LoggingOptions;
use vino_test::TestSuite;

use crate::error::VinoError;
use crate::utils::merge_config;
use crate::Result;

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

  /// Manifest file or OCI url.
  manifest: String,

  /// The test data.
  data_path: PathBuf,

  /// Filter which tests to run
  filter: Vec<String>,
}
#[allow(clippy::future_not_send, clippy::too_many_lines)]
pub(crate) async fn handle_command(opts: TestCommand) -> Result<()> {
  let mut logging = opts.logging;
  if !(opts.info || logging.trace || logging.debug) {
    logging.quiet = true;
  }
  let _guard = logger::init(&logging.name("vino"));

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
  let mut suite = TestSuite::try_from_file(opts.data_path.clone())?
    .filter(opts.filter)
    .name(format!("Vino test for : {}", file));

  let harness = suite
    .run(Arc::new(provider))
    .await
    .map_err(|e| VinoError::TestError(e.to_string()))?;

  harness.print();
  let num_failed = harness.num_failed();
  if num_failed > 0 {
    Err(VinoError::TestFailed(num_failed))
  } else {
    Ok(())
  }
}
