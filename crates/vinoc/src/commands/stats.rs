use structopt::StructOpt;
use vino_rpc::rpc::StatsRequest;
use vino_rpc::Statistics;

use crate::Result;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct StatsCommand {
  #[structopt(flatten)]
  pub logging: super::LoggingOptions,

  #[structopt(flatten)]
  pub connection: super::ConnectOptions,
}

pub async fn handle_command(opts: StatsCommand) -> Result<()> {
  crate::utils::init_logger(&opts.logging)?;
  let mut client = vino_rpc::make_rpc_client(
    opts.connection.address,
    opts.connection.port,
    opts.connection.pem,
    opts.connection.key,
    opts.connection.ca,
    opts.connection.domain,
  )
  .await?;

  let list = client.stats(StatsRequest {}).await?;

  let mut converted: Vec<Statistics> = Vec::new();

  for item in list.stats {
    converted.push(item.into());
  }

  println!("{}", serde_json::to_string(&converted)?);

  Ok(())
}
