use std::sync::Arc;

use anyhow::Result;
use seeded_random::Seed;
use wasmflow_collection_cli::options::DefaultCliOptions;
use wasmflow_host::HostBuilder;
use wasmflow_manifest::host_definition::HostDefinition;
use wasmflow_test::TestSuite;

use crate::utils::merge_config;

#[allow(clippy::future_not_send, clippy::too_many_lines)]
pub(crate) async fn handle_command(opts: super::TestCommand, bytes: Vec<u8>) -> Result<()> {
  let manifest = HostDefinition::load_from_bytes(Some(opts.location), &bytes)?;

  let server_options = DefaultCliOptions {
    mesh: opts.mesh,
    ..Default::default()
  };

  let config = merge_config(manifest, &opts.fetch, Some(server_options));

  let host_builder = HostBuilder::from_definition(config);

  let mut host = host_builder.build();
  host.connect_to_mesh().await?;
  host.start_network(opts.seed.map(Seed::unsafe_new)).await?;

  let provider: wasmflow_host::Provider = host.into();

  let file = opts.data_path.to_string_lossy().to_string();
  let mut suite = TestSuite::try_from_file(opts.data_path.clone())?
    .filter(opts.filter)
    .name(format!("{} test for : {}", crate::BIN_NAME, file));

  let harness = suite.run(Arc::new(provider)).await?;

  harness.print();
  let num_failed = harness.num_failed();
  if num_failed > 0 {
    Err(anyhow!("{} tests failed", num_failed))
  } else {
    Ok(())
  }
}
