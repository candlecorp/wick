use structopt::StructOpt;
use vino_host::HostBuilder;
use vino_manifest::host_definition::HostDefinition;
use vino_provider_cli::cli::{
  print_info,
  DefaultCliOptions,
};

use crate::utils::merge_config;
use crate::Result;
#[derive(Debug, Clone, StructOpt)]
pub(crate) struct StartCommand {
  #[structopt(flatten)]
  pub(crate) host: super::HostOptions,

  /// Manifest file path or OCI url.
  pub(crate) manifest: String,

  #[structopt(flatten)]
  pub(crate) server_options: DefaultCliOptions,
}

pub(crate) async fn handle_command(opts: StartCommand) -> Result<String> {
  vino_provider_cli::init_logging(&opts.server_options.logging)?;

  let manifest_src = vino_loader::get_bytes(
    &opts.manifest,
    opts.host.allow_latest,
    &opts.host.insecure_registries,
  )
  .await
  .map_err(|e| crate::error::VinoError::ManifestLoadFail(e.to_string()))?;

  let manifest = HostDefinition::load_from_bytes(&manifest_src)?;

  let config = merge_config(manifest, opts.host, Some(opts.server_options));

  let host_builder = HostBuilder::from_definition(config);

  let mut host = host_builder.build();

  host.start().await?;
  info!("Host started");
  match host.get_server_info() {
    Some(info) => {
      print_info(info);
    }
    None => {
      warn!("No server information available, did you intend to start a host without GRPC or a lattice connection?");
    }
  };
  info!("Waiting for Ctrl-C");
  let _ = tokio::signal::ctrl_c().await;
  info!("Ctrl-C received, shutting down");
  host.stop().await;
  Ok("Done".to_owned())
}
