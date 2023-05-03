use flow_component::{ComponentError, Context, Operation};
use futures::{FutureExt, StreamExt};
use wick_interface_types::{operation, OperationSignature};
use wick_packet::{Packet, PacketStream, StreamMap};

use crate::BoxFuture;
pub(crate) struct Op {
  signature: OperationSignature,
}

impl std::fmt::Debug for Op {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct(Op::ID).field("signature", &self.signature).finish()
  }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
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
    stream: PacketStream,
    context: Context<Self::Config>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let mut map = StreamMap::from_stream(stream, self.input_names(&context.config));
    let mapped = map.take("input").map_err(ComponentError::new).map(|input| {
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

  fn get_signature(&self, _config: Option<&Self::Config>) -> &OperationSignature {
    &self.signature
  }

  fn input_names(&self, _config: &Self::Config) -> Vec<String> {
    self.signature.inputs.iter().map(|n| n.name.clone()).collect()
  }

  fn decode_config(data: Option<wick_packet::OperationConfig>) -> Result<Self::Config, ComponentError> {
    let config = data.ok_or_else(|| {
      ComponentError::message("Merge component requires configuration, please specify configuration.")
    })?;
    Ok(Self::Config {
      field: config.get_into("field").map_err(ComponentError::new)?,
    })
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use flow_component::panic_callback;
  use wick_packet::packet_stream;

  use super::*;

  #[tokio::test]
  async fn test_basic() -> Result<()> {
    let op = Op::new();
    let config = serde_json::json!({
      "field": "pluck_this"
    });
    let config = Op::decode_config(Some(config.try_into()?))?;
    let stream = packet_stream!((
      "input",
      serde_json::json!({
        "pluck_this": "hello",
        "dont_pluck_this": "unused",
      })
    ));
    let mut packets = op
      .handle(stream, Context::new(config, panic_callback()))
      .await?
      .collect::<Vec<_>>()
      .await;
    println!("{:?}", packets);
    let _ = packets.pop().unwrap()?;
    let packet = packets.pop().unwrap()?;
    assert_eq!(packet.deserialize::<String>()?, "hello");

    Ok(())
  }
}
