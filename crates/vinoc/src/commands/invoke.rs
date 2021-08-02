use std::convert::TryInto;
use std::io::Read;

use nkeys::KeyPair;
use structopt::StructOpt;
use vino_entity::entity::Entity;
use vino_runtime::prelude::Invocation;
use vino_transport::{
  MessageTransport,
  TransportMap,
};

use crate::rpc_client::rpc_client;
use crate::{
  Error,
  Result,
};
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct InvokeCommand {
  #[structopt(flatten)]
  pub logging: super::LoggingOptions,

  #[structopt(flatten)]
  pub connection: super::ConnectOptions,

  /// Schematic to invoke
  pub schematic: String,

  /// JSON map of data to send to each input port
  data: Option<String>,
}

pub async fn handle_command(command: InvokeCommand) -> Result<()> {
  crate::utils::init_logger(&command.logging)?;
  let mut client = rpc_client(
    command.connection.address,
    command.connection.port,
    command.connection.pem,
    command.connection.key,
    command.connection.ca,
    command.connection.domain,
  )
  .await?;

  let data = match command.data {
    None => {
      eprintln!("No input passed, reading from <STDIN>");
      let mut data = String::new();
      std::io::stdin().read_to_string(&mut data)?;
      data
    }
    Some(i) => i,
  };

  let payload = TransportMap::from_json_str(&data)?;

  let kp = KeyPair::new_server();

  let rpc_invocation: vino_rpc::rpc::Invocation = Invocation::new(
    &kp,
    Entity::Client(kp.public_key()),
    Entity::Component(command.schematic),
    payload,
  )
  .try_into()?;

  debug!("Making invocation request");
  let response = client
    .invoke(rpc_invocation)
    .await
    .map_err(|e| Error::InvocationError(e.to_string()))?;
  debug!("Invocation response: {:?}", response);
  let mut stream = response.into_inner();

  while let Some(message) = stream.message().await? {
    let transport: MessageTransport = message.payload.unwrap().into();
    if transport.is_signal() {
      debug!("Skipping signal {}", transport);
    } else {
      println!("{}", transport.into_json());
    }
  }

  Ok(())
}
