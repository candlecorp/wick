use std::sync::Arc;

use anyhow::Result;
use clap::Args;
use futures::TryFutureExt;
use seeded_random::Seed;
use serde_json::{json, Value};
use structured_output::StructuredOutput;
use wick_component_cli::options::DefaultCliOptions;
use wick_config::config::UninitializedConfiguration;
use wick_config::WickConfiguration;
use wick_host::ComponentHostBuilder;
use wick_oci_utils::OciOptions;
use wick_test::{ComponentFactory, SharedComponent, TestSuite};

use crate::utils::merge_config;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
#[group(skip)]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) oci: crate::options::oci::OciOptions,

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
  #[clap(long = "filter", short = 'F', action)]
  filter: Vec<String>,
}

pub(crate) async fn handle(
  opts: Options,
  _settings: wick_settings::Settings,
  span: tracing::Span,
) -> Result<StructuredOutput> {
  let oci_opts: OciOptions = opts.oci.clone().into();
  let root_manifest = WickConfiguration::fetch(&opts.location, oci_opts.clone())
    .await?
    .into_inner()
    .try_component_config()?;

  let mut suite = TestSuite::from_configuration(root_manifest.tests())?;

  let test_files: Vec<_> = futures::future::join_all(opts.tests.iter().map(|path| {
    WickConfiguration::fetch(path, oci_opts.clone())
      .and_then(|config| futures::future::ready(config.finish().and_then(|c| c.try_test_config())))
  }))
  .await
  .into_iter()
  .collect::<Result<_, _>>()?;

  for config in &test_files {
    suite.add_configuration(config)?;
  }

  let server_options = DefaultCliOptions { ..Default::default() };

  let manifest = merge_config(&root_manifest, &opts.oci, Some(server_options));

  let factory: ComponentFactory = Box::new(move |config| {
    let mut builder = UninitializedConfiguration::new(WickConfiguration::Component(manifest.clone()));
    let span = span.clone();

    let task = async move {
      builder.set_root_config(config);
      let manifest = builder
        .finish()
        .map_err(|e| wick_test::TestError::Factory(e.to_string()))?
        .try_component_config()
        .unwrap();

      let mut host = ComponentHostBuilder::default()
        .manifest(manifest)
        .span(span)
        .build()
        .map_err(|e| wick_test::TestError::Factory(e.to_string()))?;
      host
        .start_runtime(opts.seed.map(Seed::unsafe_new))
        .await
        .map_err(|e| wick_test::TestError::Factory(e.to_string()))?;
      let component: SharedComponent = Arc::new(wick_host::HostComponent::new(host));
      Ok(component)
    };
    Box::pin(task)
  });

  let runners = suite.run(factory, opts.filter).await?;

  let mut lines: Vec<String> = Vec::new();
  let mut output: Vec<Value> = Vec::new();
  let mut num_failed = 0;

  for harness in runners {
    lines.extend(harness.get_tap_lines().clone().into_iter());
    output.push(json!({"tap_output":harness.get_tap_lines()}));

    num_failed += harness.num_failed();
  }

  let output = StructuredOutput::new(
    lines.join("\n"),
    json!({"success": num_failed ==0, "failures": num_failed, "output": output}),
  );

  Ok(output)
}
