use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Args;
use seeded_random::Seed;
use wick_component_cli::options::DefaultCliOptions;
use wick_component_cli::LoggingOptions;
use wick_config::ComponentConfiguration;
use wick_host::ComponentHostBuilder;
use wick_test::TestSuite;

use crate::utils::merge_config;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct TestCommand {
  #[clap(flatten)]
  pub(crate) logging: LoggingOptions,

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

  /// The path or OCI URL to a component configuration with tests.
  #[clap(action)]
  pub(crate) location: String,

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

  let config = ComponentConfiguration::load_from_bytes(Some(opts.location), &bytes)?;
  let mut suite = TestSuite::from_test_cases(config.tests());
  let server_options = DefaultCliOptions { ..Default::default() };

  let config = merge_config(&config, &opts.fetch, Some(server_options));

  let mut host = ComponentHostBuilder::from_definition(config).build();
  host.start_network(opts.seed.map(Seed::unsafe_new)).await?;

  let component: wick_host::Component = host.into();

  let harness = suite.run(Some("__main"), Arc::new(component)).await?;

  harness.print();
  let num_failed = harness.num_failed();
  if num_failed > 0 {
    Err(anyhow!("{} tests failed", num_failed))
  } else {
    Ok(())
  }
}
