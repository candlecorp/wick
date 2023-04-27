use flow_component::{ComponentError, Context, Operation};
use futures::{FutureExt, StreamExt};
use wick_interface_types::{operation, OperationSignature};
use wick_packet::{Packet, PacketStream};

use crate::BoxFuture;
pub(crate) struct Op {
  signature: OperationSignature,
}

impl std::fmt::Debug for Op {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct(Op::ID).field("signature", &self.signature).finish()
  }
}

#[derive(serde::Deserialize, Debug)]
pub(crate) struct Config {
  field: String,
}

impl Op {
  pub(crate) fn new() -> Self {
    Self {
      signature: operation!(Op::ID=>{
        inputs: {
          "input" => "object"
        },
        outputs: {
          "output" => "object"
        },
      }),
    }
  }
}

impl Operation for Op {
  const ID: &'static str = "pluck";
  type Config = Config;
  fn handle(
    &self,
    mut payload: wick_packet::StreamMap,
    context: Context<Self::Config>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let mapped = payload.take("input").map_err(ComponentError::new).map(|input| {
      input
        .map(move |next| {
          let field = context.config.field.clone();
          next.and_then(move |packet| {
            if packet.has_data() {
              let obj = packet.deserialize_generic()?;
              let value = obj.get(&field).map_or_else(
                || Packet::err("output", format!("Field {} not found", field)),
                |value| Packet::encode("output", value),
              );

              Ok(value)
            } else {
              Ok(packet.set_port("output"))
            }
          })
        })
        .boxed()
        .into()
    });
    async move { mapped }.boxed()
  }

  fn signature(&self, _config: Option<&Self::Config>) -> &OperationSignature {
    &self.signature
  }

  fn input_names(&self, _config: &Self::Config) -> Vec<String> {
    self.signature.inputs.iter().map(|n| n.name.clone()).collect()
  }

  fn decode_config(data: Option<flow_component::Value>) -> Result<Self::Config, ComponentError> {
    serde_json::from_value(data.ok_or_else(|| ComponentError::message("Empty configuration passed"))?)
      .map_err(ComponentError::new)
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use wick_packet::{packet_stream, StreamMap};

  use super::*;

  #[tokio::test]
  async fn test_basic() -> Result<()> {
    let op = Op::new();
    let config = serde_json::json!({
      "field": "pluck_this"
    });
    let config = Op::decode_config(Some(config))?;
    let stream = packet_stream!((
      "input",
      serde_json::json!({
        "pluck_this": "hello",
        "dont_pluck_this": "unused",
      })
    ));
    let map = StreamMap::from_stream(stream, ["input".to_owned()]);
    let mut packets = op.handle(map, Context::new(config)).await?.collect::<Vec<_>>().await;
    println!("{:?}", packets);
    let _ = packets.pop().unwrap()?;
    let packet = packets.pop().unwrap()?;
    assert_eq!(packet.deserialize::<String>()?, "hello");

    Ok(())
  }
}
