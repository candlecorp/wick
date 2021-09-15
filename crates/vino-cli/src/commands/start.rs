use std::path::PathBuf;

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

  /// Specifies a manifest file to apply to the host once started.
  #[structopt(parse(from_os_str))]
  pub(crate) manifest: Option<PathBuf>,

  #[structopt(flatten)]
  pub(crate) server_options: DefaultCliOptions,
}

pub(crate) async fn handle_command(command: StartCommand) -> Result<String> {
  let config = match command.manifest {
    Some(file) => HostDefinition::load_from_file(&file)?,
    None => HostDefinition::default(),
  };

  vino_provider_cli::init_logging(&command.server_options.logging)?;

  let config = merge_config(config, command.host, Some(command.server_options));

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
