use flow_component::{BoxFuture, Component, ComponentError, LocalScope};
use futures::StreamExt;
use wick_interface_types::{component, ComponentSignature};
use wick_packet::{fan_out, Invocation, Observer, Packet, PacketExt, PacketStream, RuntimeConfig};
use wick_rpc::dispatch;

mod wick_component_cli;
mod wick_invocation_server;
mod wick_packet_test;
mod wick_test;

#[derive(Clone, Debug)]
pub struct Context {}

#[derive(Clone)]
pub struct NativeComponent {
  signature: ComponentSignature,
}

impl Default for NativeComponent {
  fn default() -> Self {
    let sig = component! {
      name: "test-native-component",
      version: Some("0.1.0"),
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
    Self { signature: sig }
  }
}

impl Component for NativeComponent {
  fn handle(
    &self,
    invocation: Invocation,
    _data: Option<RuntimeConfig>,
    _callback: LocalScope,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let target = invocation.target().url();
    trace!("test collection invoke: {}", target);
    Box::pin(async move {
      let stream = dispatch!(invocation, {
        "error" => error,
        "test-component" => test_component,
      });
      Ok(stream)
    })
  }

  fn signature(&self) -> &wick_interface_types::ComponentSignature {
    trace!("test collection get list");
    &self.signature
  }
}

async fn error(_input: Invocation) -> Result<PacketStream, ComponentError> {
  Err(anyhow::anyhow!("Always errors"))
}

async fn test_component(input: Invocation) -> Result<PacketStream, ComponentError> {
  let (tx, stream) = input.make_response();
  tokio::spawn(async move {
    let mut stream = input.into_stream();
    let mut input = fan_out!(stream, "input");
    while let Some(Ok(input)) = input.next().await {
      if input.is_done() {
        break;
      }
      let input: String = input.payload.decode().unwrap();
      let output = Packet::encode("output", format!("TEST: {}", input));
      tx.send(output).unwrap();
    }
    tx.complete();
  });

  Ok(stream)
}

mod tests {

  use pretty_assertions::assert_eq;
  use tracing::*;
  use wick_interface_types::*;
  use wick_packet::{packet_stream, Entity};

  use super::*;

  #[test_logger::test(tokio::test)]
  async fn request() -> anyhow::Result<()> {
    let collection = NativeComponent::default();
    let input = "some_input";
    let input_stream = packet_stream!(("input", input));

    let entity = Entity::local("test-component");
    let invocation = Invocation::test(file!(), entity, input_stream, None)?;

    let outputs = collection
      .handle(invocation, Default::default(), Default::default())
      .await?;
    let mut packets: Vec<_> = outputs.collect().await;
    let output = packets.pop().unwrap().unwrap();

    println!("Received payload {:?}", output);
    assert_eq!(output, Packet::encode("output", format!("TEST: {}", input)));

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn list() -> anyhow::Result<()> {
    let collection = NativeComponent::default();

    let sig = collection.signature();

    debug!("signature response : {:?}", sig);

    let expected = ComponentSignature::new(
      "test-native-component",
      Some("0.1.0".to_owned()),
      vec![
        OperationSignature::new(
          "error",
          vec![Field::new("input", Type::String)],
          vec![Field::new("output", Type::String)],
          Default::default(),
        ),
        OperationSignature::new(
          "test-component",
          vec![Field::new("input", Type::String)],
          vec![Field::new("output", Type::String)],
          Default::default(),
        ),
      ],
      Default::default(),
      Default::default(),
    );
    assert_eq!(sig, &expected);
    Ok(())
  }
}
