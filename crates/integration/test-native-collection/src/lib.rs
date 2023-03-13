use async_trait::async_trait;
use futures::StreamExt;
use wick_interface_types::{component, HostedType};
use wick_packet::{fan_out, Invocation, Observer, Packet, PacketStream};
use wick_rpc::error::RpcError;
use wick_rpc::{dispatch, RpcHandler, RpcResult};

#[macro_use]
extern crate tracing;

#[derive(Clone, Debug)]
pub struct Context {}

#[derive(Clone, Default)]
pub struct Collection {}

#[async_trait]
impl RpcHandler for Collection {
  async fn invoke(&self, invocation: Invocation, stream: PacketStream) -> RpcResult<PacketStream> {
    let target = invocation.target_url();
    trace!("test collection invoke: {}", target);
    let stream = dispatch!(invocation, stream, {
      "error" => error,
      "test-component" => test_component,
    });
    trace!("test collection result: {}", target);

    Ok(stream)
  }

  fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    trace!("test collection get list");
    let signature = component! {
        name: "test-native-collection",
        version: "0.1.0",
        operations: {
          "error" => {
            inputs: {"input" => "string"},
            outputs: {"output" => "string"},
          },
          "test-component" => {
            inputs: {"input" => "string"},
            outputs: {"output" => "string"},
          }
        }

    };
    Ok(vec![HostedType::Collection(signature)])
  }
}

async fn error(_input: PacketStream) -> Result<PacketStream, anyhow::Error> {
  Err(anyhow::anyhow!("Always errors"))
}

async fn test_component(mut input: PacketStream) -> Result<PacketStream, anyhow::Error> {
  let (tx, stream) = PacketStream::new_channels();
  tokio::spawn(async move {
    let mut input = fan_out!(input, "input");
    while let Some(Ok(input)) = input.next().await {
      let input: String = input.payload.deserialize().unwrap();
      let output = Packet::encode("output", format!("TEST: {}", input));
      tx.send(output).unwrap();
    }
    tx.complete();
  });

  Ok(stream)
}

#[cfg(test)]
mod tests {

  use pretty_assertions::assert_eq;
  use tracing::*;
  use wick_interface_types::*;
  use wick_packet::{packet_stream, Entity};

  use super::*;

  #[test_logger::test(tokio::test)]
  async fn request() -> anyhow::Result<()> {
    let collection = Collection::default();
    let input = "some_input";
    let input_stream = packet_stream!(("input", input));

    let entity = Entity::local("test-component");
    let invocation = Invocation::new(Entity::test(file!()), entity, None);

    let outputs = collection.invoke(invocation, input_stream).await?;
    let mut packets: Vec<_> = outputs.collect().await;
    let output = packets.pop().unwrap().unwrap();

    println!("Received payload {:?}", output);
    assert_eq!(output, Packet::encode("output", format!("TEST: {}", input)));

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn list() -> anyhow::Result<()> {
    let collection = Collection::default();

    let response = collection.get_list()?;

    debug!("list response : {:?}", response);

    assert_eq!(response.len(), 1);
    let expected = ComponentSignature {
      name: Some("test-native-collection".to_owned()),
      operations: vec![
        OperationSignature {
          name: "error".to_string(),
          inputs: vec![Field::new("input", TypeSignature::String)],
          outputs: vec![Field::new("output", TypeSignature::String)],
        },
        OperationSignature {
          name: "test-component".to_string(),
          inputs: vec![Field::new("input", TypeSignature::String)],
          outputs: vec![Field::new("output", TypeSignature::String)],
        },
      ],
      ..Default::default()
    };
    assert_eq!(response[0], HostedType::Collection(expected));
    Ok(())
  }
}
