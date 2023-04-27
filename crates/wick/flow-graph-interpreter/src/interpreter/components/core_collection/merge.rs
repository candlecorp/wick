use std::collections::HashMap;

use flow_component::{ComponentError, Context, Operation};
use futures::FutureExt;
use wasmrs_rx::Observer;
use wick_interface_types::{Field, OperationSignature, StructSignature, TypeSignature};
use wick_packet::{Packet, PacketStream};

use crate::BoxFuture;
pub(crate) struct Op {}

impl std::fmt::Debug for Op {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct(Op::ID).finish()
  }
}

#[derive(serde::Deserialize, Debug, Clone)]
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
    mut payload: wick_packet::StreamMap,
    _context: Context<Self::Config>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let (tx, rx) = PacketStream::new_channels();
    tokio::spawn(async move {
      while let Ok(next) = payload.next_set().await {
        if next.is_none() {
          // let _ = tx.send(Packet::done("output"));
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

  fn signature(&self, _config: Option<&Self::Config>) -> &OperationSignature {
    panic!("Merge component has a dynamic signature");
  }

  fn input_names(&self, config: &Self::Config) -> Vec<String> {
    config.inputs.iter().map(|n| n.name.clone()).collect()
  }

  fn decode_config(data: Option<flow_component::Value>) -> Result<Self::Config, ComponentError> {
    serde_json::from_value(data.ok_or_else(|| ComponentError::message("Empty configuration passed"))?)
      .map_err(ComponentError::new)
  }
}

// #[cfg(test)]
// mod test {
//   use anyhow::Result;
//   use wick_packet::{packet_stream, StreamMap};

//   use super::*;

//   #[tokio::test]
//   async fn test_basic() -> Result<()> {
//     let operation = operation!({"test"=>{inputs: {"input"=>"object"}, outputs: {"output"=>"object"}}}}})
//     let op = Op::new();
//     let config = serde_json::json!({
//       "field": "pluck_this"
//     });
//     let config = Op::decode_config(Some(config))?;
//     let stream = packet_stream!((
//       "input",
//       serde_json::json!({
//         "pluck_this": "hello",
//         "dont_pluck_this": "unused",
//       })
//     ));
//     let map = StreamMap::from_stream(stream, ["input".to_owned()].iter());
//     let mut packets = op.handle(map, Context::new(config)).await?.collect::<Vec<_>>().await;
//     println!("{:?}", packets);
//     let _ = packets.pop().unwrap()?;
//     let packet = packets.pop().unwrap()?;
//     assert_eq!(packet.deserialize::<String>()?, "hello");

//     Ok(())
//   }
// }
