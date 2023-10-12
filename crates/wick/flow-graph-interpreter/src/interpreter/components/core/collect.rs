use std::collections::HashMap;

use anyhow::{anyhow, bail};
use flow_component::{ComponentError, Context, Operation, RenderConfiguration};
use futures::FutureExt;
use serde_json::{json, Value};
use tokio_stream::StreamExt;
use wasmrs_rx::Observer;
use wick_interface_types::{OperationSignature, Type};
use wick_packet::{InherentData, Invocation, Packet, PacketExt, PacketStream, RuntimeConfig};

use crate::BoxFuture;
pub(crate) struct Op {}

impl std::fmt::Debug for Op {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct(Op::ID).finish()
  }
}

impl crate::graph::NodeDecorator for Op {
  fn decorate(node: &mut crate::graph::types::Node) -> Result<(), String> {
    let Ok(config) = node.data().config.render(&InherentData::unsafe_default()) else {
      return Err(format!("Could not render config for {}", Op::ID));
    };
    let config = match Op::decode_config(config) {
      Ok(c) => c,
      Err(e) => {
        return Err(e.to_string());
      }
    };
    for field in config.inputs {
      node.add_input(&field);
    }
    node.add_output("output");
    Ok(())
  }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub(crate) struct Config {
  inputs: Vec<String>,
}

fn gen_signature(id: &str, config: Config) -> OperationSignature {
  let mut signature = OperationSignature::new_named(id);
  for field in config.inputs {
    signature = signature.add_input(&field, Type::Object);
  }
  signature.add_output("output", Type::Object)
}

impl Op {
  pub(crate) const fn new() -> Self {
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
    invocation: Invocation,
    context: Context<Self::Config>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let (tx, rx) = invocation.make_response();

    tokio::spawn(async move {
      let mut ports: HashMap<String, Vec<Value>> = context.config.inputs.iter().map(|n| (n.clone(), vec![])).collect();
      let mut array_levels: HashMap<String, i16> = HashMap::new();
      let mut stream = invocation.into_stream();

      while let Some(next) = stream.next().await {
        if let Err(e) = next {
          ports
            .entry(Packet::FATAL_ERROR.to_owned())
            .or_insert_with(Vec::new)
            .push(json!({"error": e.to_string()}));
          continue;
        }

        let next = next.unwrap();
        let level = array_levels.entry(next.port().to_owned()).or_insert(0);
        if next.is_fatal_error() {
          ports
            .entry(Packet::FATAL_ERROR.to_owned())
            .or_insert_with(Vec::new)
            .push(json!({"error": next.unwrap_err().msg()}));
          continue;
        }

        let port = next.port();
        if next.is_done() {
          continue;
        }

        let Some(value) = ports.get_mut(port) else {
          let _ = tx.send(Packet::err("output", "received value for invalid port"));
          return;
        };

        if next.is_open_bracket() {
          *level += 1;
          value.push(Value::Array(vec![]));
          continue;
        }
        if next.is_close_bracket() {
          *level -= 0;
          assert!(*level >= 0, "Received close bracket without open bracket");
          continue;
        }

        let next = next
          .to_json()
          .as_object_mut()
          .and_then(|o| o.remove("payload"))
          .unwrap();

        if *level > 0 {
          // push a value onto the last array created by an open_bracket
          let inner_array = get_inner_array(value.last_mut().unwrap(), *level - 1).unwrap();
          inner_array.as_array_mut().unwrap().push(next);
        } else {
          value.push(next);
        }
      }
      let _ = tx.send(Packet::encode("output", ports));
      let _ = tx.send(Packet::done("output"));
    });

    async move { Ok(rx) }.boxed()
  }

  fn get_signature(&self, _config: Option<&Self::Config>) -> &OperationSignature {
    panic!("{} operation has a dynamic signature", Self::ID);
  }

  fn input_names(&self, config: &Self::Config) -> Vec<String> {
    config.inputs.clone()
  }
}

fn get_inner_array(value: &mut Value, depth: i16) -> Result<&mut Value, ComponentError> {
  if depth == 0 {
    return Ok(value);
  }
  match value {
    Value::Array(ref mut array) => {
      let inner = array
        .last_mut()
        .ok_or_else(|| anyhow!("Invalid structure in bracketed streams"))?;
      get_inner_array(inner, depth - 1)
    }
    _ => bail!("Value {} is not an array", value),
  }
}

impl RenderConfiguration for Op {
  type Config = Config;
  type ConfigSource = RuntimeConfig;

  fn decode_config(data: Option<Self::ConfigSource>) -> Result<Self::Config, ComponentError> {
    let config =
      data.ok_or_else(|| anyhow!("Collect component requires configuration, please specify configuration."))?;

    Ok(Self::Config {
      inputs: config.coerce_key("inputs")?,
    })
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
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
      Packet::open_bracket("output_a"),
      Packet::encode("output_a", "hello"),
      Packet::encode("output_a", "hello2"),
      Packet::close_bracket("output_a"),
      Packet::encode("output_b", 1000),
    ];
    stream.push(Packet::err("output_b", "Should collect errors too"));
    stream.push(Packet::done("output_a"));
    stream.push(Packet::done("output_b"));

    let inv = Invocation::test(file!(), Entity::test("noop"), stream, None)?;
    let mut packets = op
      .handle(
        inv,
        Context::new(config, &InherentData::unsafe_default(), Default::default()),
      )
      .await?
      .collect::<Vec<_>>()
      .await;
    println!("{:?}", packets);
    let _ = packets.pop().unwrap()?;
    let packet = packets.pop().unwrap()?;
    let actual = packet.decode_value()?;
    let expected = json!({
      "output_a":[[{"value":"hello"},{"value":"hello2"}]],
      "output_b": [{"value":1000}, {"error":"Should collect errors too"}],
    });
    assert_eq!(actual, expected);

    Ok(())
  }
}
