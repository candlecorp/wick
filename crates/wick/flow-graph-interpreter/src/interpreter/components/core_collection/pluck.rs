use flow_component::{ComponentError, Context, Operation};
use futures::{FutureExt, StreamExt};
use serde_json::Value;
use wick_interface_types::{operation, OperationSignature};
use wick_packet::{Invocation, Packet, PacketStream, StreamMap};

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
  #[serde(alias = "field")]
  field: Vec<String>,
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

fn _pluck<'a>(val: &'a Value, path: &[String], idx: usize) -> Option<&'a Value> {
  if idx == path.len() {
    Some(val)
  } else {
    let part = &path[idx];
    match val {
      Value::Object(map) => map.get(part).and_then(|next| _pluck(next, path, idx + 1)),
      Value::Array(list) => {
        let i: Result<usize, _> = part.parse();
        i.map_or(None, |i| list.get(i).and_then(|next| _pluck(next, path, idx + 1)))
      }
      _ => None,
    }
  }
}

fn pluck<'a>(val: &'a Value, path: &[String]) -> Option<&'a Value> {
  _pluck(val, &path, 0)
}

impl Operation for Op {
  const ID: &'static str = "pluck";
  type Config = Config;
  fn handle(
    &self,
    invocation: Invocation,
    context: Context<Self::Config>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let mut map = StreamMap::from_stream(invocation.packets, self.input_names(&context.config));
    let mapped = map.take("input").map_err(ComponentError::new).map(|input| {
      input
        .map(move |next| {
          let field = context.config.field.clone();
          next.and_then(move |packet| {
            if packet.has_data() {
              let obj = packet.decode_value()?;
              let value = pluck(&obj, &field).map_or_else(
                || {
                  Packet::err(
                    "output",
                    format!("could not retrieve data from object path [{}]", field.join(",")),
                  )
                },
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

  fn decode_config(data: Option<wick_packet::GenericConfig>) -> Result<Self::Config, ComponentError> {
    let config = data.ok_or_else(|| {
      ComponentError::message("Pluck component requires configuration, please specify configuration.")
    })?;
    for (k, v) in config.into_iter() {
      if k == "field" {
        let field: String = serde_json::from_value(v).map_err(ComponentError::new)?;
        warn!("pluck should be configured with 'path' as an array of strings, 'field' is deprecated and will be removed in a future release.");
        return Ok(Self::Config {
          field: field.split(".").map(|s| s.to_string()).collect(),
        });
      }
      if k == "path" {
        let field: Vec<String> = serde_json::from_value(v).map_err(ComponentError::new)?;
        return Ok(Self::Config { field });
      }
    }
    Err(ComponentError::message(
      "invalid configuration for pluck, 'path' field is required",
    ))
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use flow_component::panic_callback;
  use wick_packet::{packet_stream, Entity, InherentData};

  use super::*;

  #[tokio::test]
  async fn test_deprecated() -> Result<()> {
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
    assert_eq!(packet.decode::<String>()?, "hello");

    Ok(())
  }

  #[tokio::test]
  async fn test_basic() -> Result<()> {
    let op = Op::new();
    let config = serde_json::json!({
      "path": ["pluck_this"]
    });
    let config = Op::decode_config(Some(config.try_into()?))?;
    let stream = packet_stream!((
      "input",
      serde_json::json!({
        "pluck_this": "hello",
        "dont_pluck_this": "unused",
      })
    ));
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
    assert_eq!(packet.decode::<String>()?, "hello");

    Ok(())
  }

  #[tokio::test]
  async fn test_pluck_fn() -> Result<()> {
    let json = serde_json::json!({
      "first": {
        "second": {
          "third" : [
            {"fourth": "first element"},
            {"fourth": "second element"}
          ]
        }
      }
    });

    let val = pluck(
      &json,
      &[
        "first".to_owned(),
        "second".to_owned(),
        "third".to_owned(),
        "0".to_owned(),
        "fourth".to_owned(),
      ],
    );
    assert_eq!(val, Some(&serde_json::json!("first element")));

    Ok(())
  }
}
