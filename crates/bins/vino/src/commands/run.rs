use clap::Args;
use vino_host::HostBuilder;
use vino_manifest::host_definition::HostDefinition;
use vino_provider_cli::options::{DefaultCliOptions, LatticeCliOptions};
use vino_random::Seed;

use crate::utils::merge_config;
use crate::Result;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct RunCommand {
  #[clap(flatten)]
  pub(crate) logging: super::LoggingOptions,

  #[clap(flatten)]
  pub(crate) lattice: LatticeCliOptions,

  #[clap(flatten)]
  pub(crate) host: super::HostOptions,

  /// Manifest file or OCI url.
  manifest: String,

  /// Pass a seed along with the invocation.
  #[clap(long = "seed", short = 's', env = "VINO_SEED")]
  seed: Option<u64>,

  /// Arguments to pass as inputs to a schematic.
  #[clap(last(true))]
  args: Vec<String>,
}

pub(crate) async fn handle_command(mut opts: RunCommand) -> Result<()> {
  let logging = &mut opts.logging;

  let _guard = logger::init(&logging.name("vino"));

  debug!(args = ?opts.args, "rest args");

  let manifest_src = vino_loader::get_bytes(&opts.manifest, opts.host.allow_latest, &opts.host.insecure_registries)
    .await
    .map_err(|e| crate::error::VinoError::ManifestLoadFail(e.to_string()))?;

  let manifest = HostDefinition::load_from_bytes(Some(opts.manifest.clone()), &manifest_src)?;

  let server_options = DefaultCliOptions {
    lattice: opts.lattice.clone(),
    ..Default::default()
  };

  let config = merge_config(manifest, &opts.host, Some(server_options));

  let code = exec_main(&opts, config).await?;
  if code > 0 {
    return Err(crate::Error::Other(format!("{}", code)));
  }

  Ok(())
}

async fn exec_main(opts: &RunCommand, config: HostDefinition) -> Result<u32> {
  let host_builder = HostBuilder::from_definition(config);

  let mut host = host_builder.build();
  host.connect_to_lattice().await?;
  host.start_network(opts.seed.map(Seed::unsafe_new)).await?;

  let code = host.exec_main(opts.args.clone()).await?;

  host.stop().await;

  Ok(code)
}
