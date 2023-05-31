use std::sync::Arc;

use anyhow::Result;
use clap::Args;
use futures::TryFutureExt;
use seeded_random::Seed;
use wick_component_cli::options::DefaultCliOptions;
use wick_config::WickConfiguration;
use wick_host::ComponentHostBuilder;
use wick_test::{ComponentFactory, SharedComponent, TestSuite};

use crate::utils::merge_config;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct TestCommand {
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

  /// Paths to external test files.
  #[clap(action)]
  pub(crate) tests: Vec<String>,

  /// Filter which tests to run
  #[clap(long = "filter", short = 'f', action)]
  filter: Vec<String>,
}

pub(crate) async fn handle(opts: TestCommand, _settings: wick_settings::Settings, span: tracing::Span) -> Result<()> {
  let fetch_options = wick_config::config::FetchOptions::new()
    .allow_latest(opts.oci.allow_latest)
    .allow_insecure(&opts.oci.insecure_registries);

  let root_manifest = WickConfiguration::fetch_all(&opts.location, fetch_options.clone())
    .await?
    .try_component_config()?;

  let mut suite = TestSuite::from_configuration(root_manifest.tests());

  let test_files: Vec<_> = futures::future::join_all(opts.tests.iter().map(|path| {
    WickConfiguration::fetch_all(path, fetch_options.clone())
      .and_then(|config| futures::future::ready(config.try_test_config()))
  }))
  .await
  .into_iter()
  .collect::<Result<_, _>>()?;

  for config in &test_files {
    suite.add_configuration(config);
  }

  let server_options = DefaultCliOptions { ..Default::default() };

  let manifest = merge_config(&root_manifest, &opts.oci, Some(server_options));

  let factory: ComponentFactory = Box::new(move |config| {
    let manifest = manifest.clone();
    let span = span.clone();
    let task = async move {
      let mut host = ComponentHostBuilder::default()
        .manifest(manifest)
        .config(config)
        .span(span)
        .build()
        .map_err(|e| wick_test::TestError::Factory(e.to_string()))?;
      host
        .start_engine(opts.seed.map(Seed::unsafe_new))
        .await
        .map_err(|e| wick_test::TestError::Factory(e.to_string()))?;
      let component: SharedComponent = Arc::new(wick_host::HostComponent::new(host));
      Ok(component)
    };
    Box::pin(task)
  });

  let runners = suite.run(factory).await?;

  for harness in runners {
    harness.print();
    let num_failed = harness.num_failed();
    if num_failed > 0 {
      return Err(anyhow!("{} tests failed", num_failed));
    }
  }

  Ok(())
}
