use std::sync::Arc;

use anyhow::Result;
use clap::Args;
use seeded_random::Seed;
use wick_component_cli::options::DefaultCliOptions;
use wick_component_cli::LoggingOptions;
use wick_config::WickConfiguration;
use wick_host::ComponentHostBuilder;
use wick_test::TestSuite;

use crate::utils::merge_config;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct TestCommand {
  #[clap(flatten)]
  pub(crate) logging: LoggingOptions,

  #[clap(flatten)]
  pub(crate) oci: crate::oci::Options,

  #[clap(flatten)]
  wasi: crate::wasm::WasiOptions,

  /// Turn on info logging.
  #[clap(long = "info", action)]
  pub(crate) info: bool,

  /// Pass a seed along with the invocation.
  #[clap(long = "seed", short = 's', env = "WICK_SEED", action)]
  seed: Option<u64>,

  /// The path or OCI URL to a component configuration with tests.
  #[clap(action)]
  pub(crate) location: String,

  /// Filter which tests to run
  #[clap(action)]
  filter: Vec<String>,
}

pub(crate) async fn handle_command(opts: TestCommand) -> Result<()> {
  let _guard = wick_logger::init(&opts.logging.name(crate::BIN_NAME));

  let fetch_options = wick_config::config::FetchOptions::new()
    .allow_latest(opts.oci.allow_latest)
    .allow_insecure(&opts.oci.insecure_registries);

  let config = WickConfiguration::fetch(&opts.location, fetch_options)
    .await?
    .try_component_config()?;

  let mut suite = TestSuite::from_test_cases(config.tests());
  let server_options = DefaultCliOptions { ..Default::default() };

  let config = merge_config(&config, &opts.oci, Some(server_options));

  let mut host = ComponentHostBuilder::from_definition(config).build();
  host.start_engine(opts.seed.map(Seed::unsafe_new)).await?;

  let component = Arc::new(wick_host::HostComponent::new(host));
  let id = component.id().to_owned();

  let harness = suite.run(Some(&id), component).await?;

  harness.print();
  let num_failed = harness.num_failed();
  if num_failed > 0 {
    Err(anyhow!("{} tests failed", num_failed))
  } else {
    Ok(())
  }
}
