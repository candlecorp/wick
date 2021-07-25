use std::collections::HashMap;
use std::io::Read;

use structopt::StructOpt;
use tokio_stream::StreamExt;
use vino_codec::messagepack::serialize;
use vino_provider::entity::Entity;
use vino_provider_wasm::provider::Provider;
use vino_rpc::RpcHandler;
use vino_transport::MessageTransport;

use crate::Result;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub(crate) struct RunCommand {
  /// Path or URL to WebAssembly binary
  wasm: String,

  /// The name of the component to execute
  component_name: String,

  /// JSON data
  data: Option<String>,

  #[structopt(flatten)]
  pub(crate) logging: super::LoggingOptions,

  #[structopt(flatten)]
  pub(crate) pull: super::PullOptions,
}

pub(crate) async fn handle_command(opts: RunCommand) -> Result<()> {
  logger::Logger::init(&opts.logging)?;
  let data = match opts.data {
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
    .map(|(name, val)| Ok((name, serialize(val)?)))
    .filter_map(Result::ok)
    .collect();

  debug!("Loading wasm {}", opts.wasm);
  let component =
    vino_provider_wasm::helpers::load_wasm(&opts.wasm, opts.pull.latest, &opts.pull.insecure)
      .await?;

  let provider = Provider::new(component, 5);

  let mut response = provider
    .invoke(Entity::Component(opts.component_name), multibytes)
    .await?;

  let mut map = serde_json::Map::new();
  while let Some(message) = response.next().await {
    let transport: MessageTransport = message.packet.into();
    map.insert(message.port, transport.into_json());
  }

  println!("{}", serde_json::to_string(&map)?);

  Ok(())
}
