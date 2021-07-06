use std::convert::TryInto;

use structopt::StructOpt;
use vino_rpc;
use vino_rpc::rpc::ListRequest;
use vino_rpc::HostedType;

use crate::rpc_client::rpc_client;
use crate::{
  Error,
  Result,
};

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct ListCommand {
  #[structopt(flatten)]
  pub logging: super::LoggingOptions,

  #[structopt(flatten)]
  pub connection: super::ConnectOptions,
}

pub async fn handle_command(command: ListCommand) -> Result<String> {
  crate::utils::init_logger(&command.logging)?;
  let mut client = rpc_client(command.connection.address, command.connection.port).await?;

  let list = client.list(ListRequest {});
  debug!("Making list request");
  let list = list.await.map_err(|e| Error::Other(e.to_string()))?;
  debug!("Component list: {:?}", list);
  let list = list.into_inner();

  let mut converted: Vec<HostedType> = Vec::new();

  for item in list.components {
    converted.push(item.try_into()?)
  }

  println!("{}", serde_json::to_string(&converted)?);

  Ok("Done".to_string())
}
