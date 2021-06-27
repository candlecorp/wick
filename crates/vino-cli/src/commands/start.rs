use std::path::PathBuf;

use structopt::StructOpt;
use vino_host::{
  HostBuilder,
  HostDefinition,
};

use crate::utils::merge_runconfig;
use crate::Result;
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct StartCommand {
  #[structopt(flatten)]
  pub logging: super::LoggingOptions,

  #[structopt(flatten)]
  pub nats: super::NatsOptions,

  #[structopt(flatten)]
  pub host: super::HostOptions,

  /// Specifies a manifest file to apply to the host once started
  #[structopt(parse(from_os_str))]
  pub manifest: Option<PathBuf>,
}

pub async fn handle_command(command: StartCommand) -> Result<String> {
  crate::utils::init_logger(&command.logging)?;

  let config = match command.manifest {
    Some(file) => vino_host::HostDefinition::load_from_file(&file)?,
    None => HostDefinition::default(),
  };

  let config = merge_runconfig(config, command.nats, command.host);

  // debug!("Attempting connection to NATS server");
  // let nats_url = &format!("{}:{}", config.config.rpc_host, config.config.rpc_port);
  // let nc_rpc = nats_connection(
  //     nats_url,
  //     config.config.rpc_jwt,
  //     config.config.rpc_seed,
  //     config.config.rpc_credsfile,
  // );
  // let nc_control = nats_connection(
  //     nats_url,
  //     config.config.control_jwt,
  //     config.config.control_seed,
  //     config.config.control_credsfile,
  // );

  let host_builder = HostBuilder::new();

  // match try_join!(nc_rpc, nc_control) {
  //     Ok((nc_rpc, nc_control)) => {
  //         host_builder = host_builder
  //             .with_rpc_client(nc_rpc)
  //             .with_control_client(nc_control);
  //     }
  //     Err(e) => warn!("Could not connect to NATS, operating locally ({})", e),
  // }

  // if config.config.allow_oci_latest {
  //     debug!("Enabling :latest tag");
  //     host_builder = host_builder.oci_allow_latest();
  // }

  // if !config.config.allowed_insecure.is_empty() {
  //     debug!(
  //         "Allowing insecure registries: {}",
  //         config.config.allowed_insecure.join(", ")
  //     );
  //     host_builder = host_builder.oci_allow_insecure(config.config.allowed_insecure);
  // }

  let mut host = host_builder.build();

  debug!("Starting host");
  match host.start().await {
    Ok(_) => {
      debug!("Applying manifest");
      host.start_network(config.network).await?;
      info!("Manifest applied");
    }
    Err(e) => {
      error!("Failed to start host: {}", e);
    }
  }

  actix_rt::signal::ctrl_c().await.unwrap();
  info!("Ctrl-C received, shutting down");
  host.stop().await;
  Ok("Done".to_string())
}
