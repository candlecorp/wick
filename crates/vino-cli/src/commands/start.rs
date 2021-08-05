use std::path::PathBuf;

use structopt::StructOpt;
use vino_host::{
  HostBuilder,
  HostDefinition,
};
use vino_provider_cli::cli::DefaultCliOptions;

use crate::utils::merge_runconfig;
use crate::Result;
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub(crate) struct StartCommand {
  #[structopt(flatten)]
  pub(crate) host: super::HostOptions,

  /// Specifies a manifest file to apply to the host once started.
  #[structopt(parse(from_os_str))]
  pub(crate) manifest: Option<PathBuf>,

  #[structopt(flatten)]
  pub(crate) server_options: DefaultCliOptions,
}

pub(crate) async fn handle_command(command: StartCommand) -> Result<String> {
  let config = match command.manifest {
    Some(file) => vino_host::HostDefinition::load_from_file(&file)?,
    None => HostDefinition::default(),
  };

  let config = merge_runconfig(config, command.host);

  let host_builder = HostBuilder::new();

  let mut host = host_builder.build();

  vino_provider_cli::init_logging(&command.server_options.logging)?;

  debug!("Starting host");
  match host.start().await {
    Ok(_) => {
      debug!("Applying manifest");
      host.start_network(config.network).await?;
      info!("Manifest applied");
      let metadata = host
        .start_rpc_server(Some(command.server_options.into()))
        .await?;
      let addr = metadata.rpc_addr.unwrap();
      info!("Server bound to {} on port {}", addr.ip(), addr.port());
    }
    Err(e) => {
      error!("Failed to start host: {}", e);
    }
  }

  info!("Waiting for Ctrl-C");
  let _ = tokio::signal::ctrl_c().await;
  info!("Ctrl-C received, shutting down");
  host.stop().await;
  Ok("Done".to_owned())
}
