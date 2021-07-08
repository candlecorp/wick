use std::net::Ipv4Addr;
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

  /// The address to bind to
  #[structopt(short, long, default_value = "127.0.0.1")]
  pub address: Ipv4Addr,

  /// The port to bind to
  #[structopt(short, long, default_value = "8060")]
  pub port: u16,

  /// Path to pem file for TLS
  #[structopt(long)]
  pub pem: Option<PathBuf>,

  /// Path to key file for TLS
  #[structopt(long)]
  pub key: Option<PathBuf>,

  /// Path to ca pem file for TLS
  #[structopt(long)]
  pub ca: Option<PathBuf>,
}

pub async fn handle_command(command: StartCommand) -> Result<String> {
  crate::utils::init_logger(&command.logging)?;

  let config = match command.manifest {
    Some(file) => vino_host::HostDefinition::load_from_file(&file)?,
    None => HostDefinition::default(),
  };

  let config = merge_runconfig(config, command.nats, command.host);

  let host_builder = HostBuilder::new();

  let mut host = host_builder.build();

  debug!("Starting host");
  match host.start().await {
    Ok(_) => {
      debug!("Applying manifest");
      host.start_network(config.network).await?;
      info!("Manifest applied");
      let addr = host
        .start_rpc_server(
          command.address,
          if command.port == 0 {
            None
          } else {
            Some(command.port)
          },
          command.pem,
          command.key,
          command.ca,
        )
        .await?;
      info!("Bound to {}", addr);
    }
    Err(e) => {
      error!("Failed to start host: {}", e);
    }
  }

  actix_rt::signal::ctrl_c().await.unwrap();
  info!("Ctrl-C received, shutting down");
  host.stop().await;
  Ok("Done".to_owned())
}
