use std::collections::HashMap;

use anyhow::anyhow;
use flow_component::{ComponentError, Context, Operation, RenderConfiguration};
use futures::FutureExt;
use wasmrs_rx::Observer;
use wick_interface_types::{Field, OperationSignature, StructDefinition, Type};
use wick_packet::{InherentData, Invocation, Packet, PacketStream, RuntimeConfig, StreamMap};

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
      node.add_input(field.name());
    }
    node.add_output("output");
    Ok(())
  }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub(crate) struct Config {
  inputs: Vec<Field>,
}

fn gen_signature(id: String, config: Config) -> (OperationSignature, StructDefinition) {
  let mut signature = OperationSignature::new_named(&id);
  let output_type = Vec::new();
  let mut output_signature = StructDefinition::new(&id, output_type, None);
  for field in config.inputs {
    output_signature.fields.push(field.clone());
    signature = signature.add_input(field.name, field.ty);
  }

  signature = signature.add_output("output", Type::Named(id));

  (signature, output_signature)
}

impl Op {
  pub(crate) const fn new() -> Self {
    Self {}
  }
  pub(crate) fn gen_signature(id: String, config: Config) -> (OperationSignature, StructDefinition) {
    gen_signature(id, config)
  }
}

impl Operation for Op {
  const ID: &'static str = "merge";
  type Config = Config;
  fn handle(
    &self,
    invocation: Invocation,
    context: Context<Self::Config>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let (tx, rx) = invocation.make_response();
    let stream = invocation.into_stream();
    let mut map = StreamMap::from_stream(stream, self.input_names(&context.config));
    tokio::spawn(async move {
      while let Ok(next) = map.next_set().await {
        if next.is_none() {
          break;
        }
        let next = next.unwrap();
        let output = if next.values().all(|p| p.has_data()) {
          next
            .into_iter()
            .map(|(k, v)| Ok((k, v.decode_value()?)))
            .collect::<Result<HashMap<_, _>, wick_packet::Error>>()
            .map(|map| Packet::encode("output", map))
        } else {
          let outlier = next.into_values().find(|x| !x.has_data()).unwrap();
          Ok(outlier.to_port("output"))
        };
        let _ = tx.send_result(output);
      }
    });

    async move { Ok(rx) }.boxed()
  }

  fn get_signature(&self, _config: Option<&Self::Config>) -> &OperationSignature {
    panic!("{} operation has a dynamic signature", Self::ID);
  }

  fn input_names(&self, config: &Self::Config) -> Vec<String> {
    config.inputs.iter().map(|n| n.name.clone()).collect()
  }
}

impl RenderConfiguration for Op {
  type Config = Config;
  type ConfigSource = RuntimeConfig;

  fn decode_config(data: Option<Self::ConfigSource>) -> Result<Self::Config, ComponentError> {
    let config =
      data.ok_or_else(|| anyhow!("Merge component requires configuration, please specify configuration."))?;

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
  use wick_packet::{packet_stream, Entity, InherentData};

  use super::*;

  #[tokio::test]
  async fn test_basic() -> Result<()> {
    let inputs = vec![Field::new("input_a", Type::String), Field::new("input_b", Type::U32)];
    let op = Op::new();
    let config = HashMap::from([("inputs".to_owned(), json!(inputs))]);
    let config = Op::decode_config(Some(config.into()))?;
    let stream = packet_stream!(("input_a", "hello"), ("input_b", 1000));
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
    let expected = json!({"input_a":"hello", "input_b": 1000});
    assert_eq!(actual, expected);

    Ok(())
  }
}
