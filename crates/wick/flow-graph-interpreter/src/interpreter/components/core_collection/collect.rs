use std::collections::HashMap;

use flow_component::{ComponentError, Context, Operation, RenderConfiguration};
use futures::FutureExt;
use serde_json::Value;
use tokio_stream::StreamExt;
use wasmrs_rx::Observer;
use wick_interface_types::{OperationSignature, Type};
use wick_packet::{Invocation, Packet, PacketStream, RuntimeConfig};

use crate::BoxFuture;
pub(crate) struct Op {}

impl std::fmt::Debug for Op {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct(Op::ID).finish()
  }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub(crate) struct Config {
  inputs: Vec<String>,
}

fn gen_signature(id: &str, config: Config) -> OperationSignature {
  let mut signature = OperationSignature::new(id);
  for field in config.inputs {
    signature = signature.add_input(&field, Type::Object);
  }
  signature.add_output("output", Type::Object)
}

impl Op {
  pub(crate) fn new() -> Self {
    Self {}
  }
  pub(crate) fn gen_signature(id: &str, config: Config) -> OperationSignature {
    gen_signature(id, config)
  }
}

impl Operation for Op {
  const ID: &'static str = "collect";
  type Config = Config;
  fn handle(
    &self,
    mut invocation: Invocation,
    context: Context<Self::Config>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let (tx, rx) = invocation.make_response();

    tokio::spawn(async move {
      let mut ports: HashMap<String, Vec<Value>> = context.config.inputs.iter().map(|n| (n.clone(), vec![])).collect();
      ports.insert(Packet::FATAL_ERROR.to_owned(), vec![]);
      while let Some(next) = invocation.packets.next().await {
        let next = next.unwrap();
        let port = next.port();
        if next.is_done() || next.is_close_bracket() || next.is_open_bracket() {
          continue;
        }
        let Some(value) = ports.get_mut(port) else {
          let _ = tx.send(Packet::err("output", "received value for invalid port"));
          return;
        };

        value.push(
          next
            .to_json()
            .as_object_mut()
            .and_then(|o| o.remove("payload"))
            .unwrap(),
        );
      }
      let _ = tx.send(Packet::encode("output", ports));
      let _ = tx.send(Packet::done("output"));
    });

    async move { Ok(rx) }.boxed()
  }

  fn get_signature(&self, _config: Option<&Self::Config>) -> &OperationSignature {
    panic!("Merge component has a dynamic signature");
  }

  fn input_names(&self, config: &Self::Config) -> Vec<String> {
    config.inputs.clone()
  }
}

impl RenderConfiguration for Op {
  type Config = Config;
  type ConfigSource = RuntimeConfig;

  fn decode_config(data: Option<Self::ConfigSource>) -> Result<Self::Config, ComponentError> {
    let config = data.ok_or_else(|| {
      ComponentError::message("Merge component requires configuration, please specify configuration.")
    })?;

    Ok(Self::Config {
      inputs: config.coerce_key("inputs").map_err(ComponentError::new)?,
    })
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use flow_component::panic_callback;
  use serde_json::json;
  use tokio_stream::StreamExt;
  use wick_packet::{Entity, InherentData};

  use super::*;

  #[tokio::test]
  async fn test_basic() -> Result<()> {
    let inputs = vec!["output_a".to_owned(), "output_b".to_owned()];
    let op = Op::new();
    let config = HashMap::from([("inputs".to_owned(), json!(inputs))]);
    let config = Op::decode_config(Some(config.into()))?;
    let mut stream = vec![
      Packet::encode("output_a", "hello"),
      Packet::encode("output_a", "hello2"),
      Packet::encode("output_b", 1000),
    ];
    stream.push(Packet::err("output_b", "Should collect errors too"));
    stream.push(Packet::done("output_a"));
    stream.push(Packet::done("output_b"));

    let inv = Invocation::test(file!(), Entity::test("noop"), stream, None)?;
    let mut packets = op
      .handle(
        inv,
        Context::new(config, &InherentData::unsafe_default(), panic_callback()),
      )
      .await?
      .collect::<Vec<_>>()
      .await;
    println!("{:?}", packets);
    let _ = packets.pop().unwrap()?;
    let packet = packets.pop().unwrap()?;
    let actual = packet.decode_value()?;
    let expected = json!({
      "output_a":[{"value":"hello"},{"value":"hello2"}],
      "output_b": [{"value":1000}, {"error":"Should collect errors too"}],
      "<error>": [],
    });
    assert_eq!(actual, expected);

    Ok(())
  }
}
