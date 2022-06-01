use anyhow::Result;
use vino_host::HostBuilder;
use vino_manifest::host_definition::HostDefinition;
use vino_provider_cli::options::DefaultCliOptions;
use vino_random::Seed;

use crate::utils::merge_config;

pub(crate) async fn handle_command(opts: super::RunCommand, bytes: Vec<u8>) -> Result<()> {
  debug!(args = ?opts.args, "rest args");
  let manifest = HostDefinition::load_from_bytes(Some(opts.location.clone()), &bytes)?;

  let server_options = DefaultCliOptions {
    lattice: opts.lattice.clone(),
    ..Default::default()
  };

  let config = merge_config(manifest, &opts.fetch, Some(server_options));

  let code = exec_main(&opts, config).await?;
  if code > 0 {
    return Err(anyhow!(format!("Error code {}", code)));
  }

  Ok(())
}

async fn exec_main(opts: &super::RunCommand, config: HostDefinition) -> Result<u32> {
  let host_builder = HostBuilder::from_definition(config);

  let mut host = host_builder.build();
  host.connect_to_lattice().await?;
  host.start_network(opts.seed.map(Seed::unsafe_new)).await?;

  let code = host.exec_main(opts.args.clone()).await?;

  host.stop().await;

  Ok(code)
}
