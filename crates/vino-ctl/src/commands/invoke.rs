use std::path::PathBuf;

use structopt::StructOpt;
use vino_host::{
  HostBuilder,
  HostDefinition,
};

use crate::Result;
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct InvokeCommand {
  #[structopt(flatten)]
  pub logging: super::LoggingOptions,

  #[structopt(flatten)]
  pub connection: super::ConnectOptions,
}

pub async fn handle_command(command: InvokeCommand) -> Result<String> {
  crate::utils::init_logger(&command.logging)?;

  Ok("Done".to_string())
}
