use std::io::Read;

use structopt::StructOpt;
use tokio_stream::StreamExt;
use vino_provider::native::prelude::{
  Entity,
  TransportMap,
};
use vino_provider_wasm::provider::Provider;
use vino_rpc::RpcHandler;

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

  let payload = TransportMap::from_json_str(&data)?;

  debug!("Loading wasm {}", opts.wasm);
  let component =
    vino_provider_wasm::helpers::load_wasm(&opts.wasm, opts.pull.latest, &opts.pull.insecure)
      .await?;

  let provider = Provider::new(component, 1);

  let mut response = provider
    .invoke(Entity::Component(opts.component_name), payload)
    .await?;

  let mut map = serde_json::Map::new();
  while let Some(message) = response.next().await {
    if message.payload.is_signal() {
      debug!(
        "Skipping signal '{}' on port '{}'",
        message.payload, message.port
      );
    } else {
      map.insert(message.port, message.payload.into_json());
    }
  }

  println!("{}", serde_json::to_string(&map)?);

  Ok(())
}
