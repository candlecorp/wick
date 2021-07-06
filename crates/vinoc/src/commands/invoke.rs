use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Read;

use nkeys::KeyPair;
use structopt::StructOpt;
use vino_codec::messagepack::serialize;
use vino_entity::entity::Entity;
use vino_runtime::prelude::Invocation;
use vino_transport::MessageTransport;

use crate::rpc_client::rpc_client;
use crate::Result;
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

pub async fn handle_command(command: InvokeCommand) -> Result<String> {
  crate::utils::init_logger(&command.logging)?;
  let mut client = rpc_client(command.connection.address, command.connection.port).await?;

  let data = match command.data {
    None => {
      eprintln!("No input passed, reading from <STDIN>");
      let mut data = String::new();
      std::io::stdin().read_to_string(&mut data)?;
      data
    }
    Some(i) => i,
  };

  let json: HashMap<String, serde_json::value::Value> = serde_json::from_str(&data)?;
  let multibytes: HashMap<String, Vec<u8>> = json
    .into_iter()
    .map(|(name, val)| Ok!((name, serialize(val)?)))
    .filter_map(Result::ok)
    .collect();
  let kp = KeyPair::new_server();

  let rpc_invocation: vino_rpc::rpc::Invocation = Invocation::new(
    &kp,
    Entity::Client(kp.public_key()),
    Entity::Schematic(command.schematic),
    MessageTransport::MultiBytes(multibytes),
  )
  .try_into()?;

  debug!("Making invocation request");
  let response = client.invoke(rpc_invocation).await?;
  debug!("Invocation response: {:?}", response);
  let mut stream = response.into_inner();

  let mut map = serde_json::Map::new();
  while let Some(message) = stream.message().await? {
    let transport: MessageTransport = message.payload.unwrap().into();
    map.insert(message.port, transport.into_json());
  }

  println!("{}", serde_json::to_string(&map)?);

  Ok("Done".to_string())
}
