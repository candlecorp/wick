use std::convert::TryInto;

use structopt::StructOpt;
use vino_rpc;
use vino_rpc::rpc::ListRequest;
use vino_types::signatures::HostedType;

use crate::rpc_client::rpc_client;
use crate::Result;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct ListCommand {
  #[structopt(flatten)]
  pub logging: super::LoggingOptions,

  #[structopt(flatten)]
  pub connection: super::ConnectOptions,
}

pub async fn handle_command(opts: ListCommand) -> Result<()> {
  crate::utils::init_logger(&opts.logging)?;
  let mut client = rpc_client(
    opts.connection.address,
    opts.connection.port,
    opts.connection.pem,
    opts.connection.key,
    opts.connection.ca,
    opts.connection.domain,
  )
  .await?;

  let list = client.list(ListRequest {}).await?;

  let mut converted: Vec<HostedType> = Vec::new();

  for item in list.schemas {
    converted.push(item.try_into()?);
  }

  println!("{}", serde_json::to_string(&converted)?);

  Ok(())
}
