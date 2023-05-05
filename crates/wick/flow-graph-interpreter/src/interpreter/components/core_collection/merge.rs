use std::collections::HashMap;

use flow_component::{ComponentError, Context, Operation};
use futures::FutureExt;
use wasmrs_rx::Observer;
use wick_interface_types::{Field, OperationSignature, StructSignature, TypeSignature};
use wick_packet::{Packet, PacketStream, StreamMap};

use crate::BoxFuture;
pub(crate) struct Op {}

impl std::fmt::Debug for Op {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct(Op::ID).finish()
  }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub(crate) struct Config {
  inputs: Vec<Field>,
}

fn gen_signature(id: String, config: Config) -> (OperationSignature, StructSignature) {
  let mut signature = OperationSignature::new(&id);
  let output_type = Vec::new();
  let mut output_signature = StructSignature::new(&id, output_type);
  for field in config.inputs {
    output_signature.fields.push(field.clone());
    signature = signature.add_input(field.name, field.ty);
  }

  signature = signature.add_output("output", TypeSignature::Ref { reference: id });

  (signature, output_signature)
}

impl Op {
  pub(crate) fn new() -> Self {
    Self {}
  }
  pub(crate) fn gen_signature(id: String, config: Config) -> (OperationSignature, StructSignature) {
    gen_signature(id, config)
  }
}

impl Operation for Op {
  const ID: &'static str = "merge";
  type Config = Config;
  fn handle(
    &self,
    stream: PacketStream,
    context: Context<Self::Config>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let mut map = StreamMap::from_stream(stream, self.input_names(&context.config));
    let (tx, rx) = PacketStream::new_channels();
    tokio::spawn(async move {
      while let Ok(next) = map.next_set().await {
        if next.is_none() {
          break;
        }
        let next = next.unwrap();
        let output = if next.values().all(|p| p.has_data()) {
          next
            .into_iter()
            .map(|(k, v)| Ok((k, v.deserialize_generic()?)))
            .collect::<Result<HashMap<_, _>, wick_packet::Error>>()
            .map(|map| Packet::encode("output", map))
        } else {
          let outlier = next.into_values().find(|x| !x.has_data()).unwrap();
          Ok(outlier.set_port("output"))
        };
        let _ = tx.send_result(output);
      }
    });

    async move { Ok(rx) }.boxed()
  }

  fn get_signature(&self, _config: Option<&Self::Config>) -> &OperationSignature {
    panic!("Merge component has a dynamic signature");
  }

  fn input_names(&self, config: &Self::Config) -> Vec<String> {
    config.inputs.iter().map(|n| n.name.clone()).collect()
  }

  fn decode_config(data: Option<wick_packet::OperationConfig>) -> Result<Self::Config, ComponentError> {
    let config = data.ok_or_else(|| {
      ComponentError::message("Merge component requires configuration, please specify configuration.")
    })?;
    Ok(Self::Config {
      inputs: config.get_into("inputs").map_err(ComponentError::new)?,
    })
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use flow_component::panic_callback;
  use serde_json::json;
  use tokio_stream::StreamExt;
  use wick_packet::packet_stream;

  use super::*;

  #[tokio::test]
  async fn test_basic() -> Result<()> {
    let inputs = vec![
      Field::new("input_a", TypeSignature::String),
      Field::new("input_b", TypeSignature::U32),
    ];
    let op = Op::new();
    let config = serde_json::json!({ "inputs": inputs });
    let config = Op::decode_config(Some(config.try_into()?))?;
    let stream = packet_stream!(("input_a", "hello"), ("input_b", 1000));
    let mut packets = op
      .handle(stream, Context::new(config, None, panic_callback()))
      .await?
      .collect::<Vec<_>>()
      .await;
    println!("{:?}", packets);
    let _ = packets.pop().unwrap()?;
    let packet = packets.pop().unwrap()?;
    let actual = packet.deserialize_generic()?;
    let expected = json!({"input_a":"hello", "input_b": 1000});
    assert_eq!(actual, expected);

    Ok(())
  }
}
