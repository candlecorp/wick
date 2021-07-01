use structopt::StructOpt;

use crate::Result;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct StatsCommand {
  #[structopt(flatten)]
  pub logging: super::LoggingOptions,

  #[structopt(flatten)]
  pub connection: super::ConnectOptions,
}

pub async fn handle_command(command: StatsCommand) -> Result<String> {
  crate::utils::init_logger(&command.logging)?;

  Ok("Done".to_string())
}
