#![allow(dead_code)]
mod observer;
mod test_component;

use anyhow::Result;
use flow_component::{panic_callback, Component};
use flow_graph_interpreter::graph::from_def;
use flow_graph_interpreter::Interpreter;
pub use observer::JsonWriter;
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub use test_component::TestComponent;
use wick_packet::{Entity, Packet, RuntimeConfig};

use crate::test;

pub async fn common_setup(
  manifest: &str,
  op: &str,
  packets: Vec<Packet>,
) -> anyhow::Result<(Interpreter, Vec<Result<Packet, wick_packet::Error>>)> {
  base_setup(
    manifest,
    Entity::local(op),
    packets,
    Default::default(),
    Default::default(),
  )
  .await
}

pub async fn base_setup(
  manifest: &str,
  entity: Entity,
  packets: Vec<Packet>,
  component_config: Option<RuntimeConfig>,
  config: Option<RuntimeConfig>,
) -> anyhow::Result<(Interpreter, Vec<Result<Packet, wick_packet::Error>>)> {
  use flow_graph_interpreter::{HandlerMap, InterpreterOptions, NamespaceHandler};
  use tokio_stream::StreamExt;
  use wick_packet::Invocation;
  let options = Some(InterpreterOptions { ..Default::default() });
  let mut def = wick_config::WickConfiguration::fetch(manifest, Default::default()).await?;
  def.set_root_config(component_config);
  let mut def = def.finish()?.try_component_config()?;

  let collections = HandlerMap::new(vec![NamespaceHandler::new(
    "test",
    Box::new(test::TestComponent::new()),
  )])
  .unwrap();
  let network = from_def(&mut def, &collections)?;

  let mut interpreter = Interpreter::new(
    network,
    None,
    def.root_config().cloned(),
    Some(collections),
    panic_callback(),
    &tracing::Span::current(),
  )?;

  interpreter.start(options, None).await;
  let stream = wick_packet::PacketStream::new(Box::new(futures::stream::iter(packets.into_iter().map(Ok))));
  let invocation = Invocation::test("test", entity, stream, None)?;
  let stream = interpreter.invoke(invocation, config).await?;
  let outputs: Vec<_> = stream.collect().await;
  println!("{:#?}", outputs);
  Ok((interpreter, outputs))
}

pub fn from_packet_file(file: &str) -> Result<Vec<Packet>> {
  let file = std::fs::read_to_string(file).unwrap();
  let mut packets = Vec::new();
  for line in file.lines() {
    let json = serde_json::from_str::<JsonPacket>(line)?;
    let packet: Packet = json.into();
    packets.push(packet);
  }

  Ok(packets)
}

pub async fn first_packet_test(file: &str, packets: Vec<Packet>, expected: &str) -> Result<()> {
  first_packet_test_op("test", file, packets, expected).await
}

pub async fn first_packet_test_op(op_name: &str, file: &str, packets: Vec<Packet>, expected: &str) -> Result<()> {
  let (interpreter, mut outputs) = test::common_setup(file, op_name, packets).await?;

  assert_eq!(outputs.len(), 2);

  let _ = outputs.pop();
  let wrapper = outputs.pop().unwrap().unwrap();
  let actual: String = wrapper.decode()?;
  assert_eq!(actual, expected);
  println!("shutting down interpreter");
  interpreter.shutdown().await?;
  println!("done");

  Ok(())
}

pub async fn first_packet_test_config(
  file: &str,
  root_config: Option<RuntimeConfig>,
  config: Option<RuntimeConfig>,
  packets: Vec<Packet>,
  expected: impl Into<Value>,
) -> Result<()> {
  let (interpreter, mut outputs) = test::base_setup(file, Entity::local("test"), packets, root_config, config).await?;

  assert_eq!(outputs.len(), 2);

  let _ = outputs.pop();
  let wrapper = outputs.pop().unwrap().unwrap();
  let actual: Value = wrapper.decode()?;
  assert_eq!(actual, expected.into());
  interpreter.shutdown().await?;

  Ok(())
}

pub async fn test_config(
  file: &str,
  root_config: Option<RuntimeConfig>,
  config: Option<RuntimeConfig>,
  packets: Vec<Packet>,
  expected: Vec<Packet>,
) -> Result<()> {
  let (interpreter, outputs) =
    super::test::base_setup(file, Entity::local("test"), packets, root_config, config).await?;

  for (i, expected) in expected.into_iter().enumerate() {
    let actual_packet = outputs.get(i).cloned().unwrap().unwrap();
    println!("actual[{}] raw: {:?}", i, actual_packet);
    println!("expected[{}] raw: {:?}", i, expected);

    if actual_packet.has_data() {
      let actual: Value = actual_packet.decode()?;
      let expected: Value = expected.decode()?;
      println!("actual[{}] value: {}", i, actual);
      println!("expected[{}] value: {}", i, expected);
      assert_eq!(actual, expected);
    } else {
      assert_eq!(actual_packet, expected);
    }
  }

  interpreter.shutdown().await?;

  Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SuccessPacket {
  #[serde(default)]
  flags: u8,
  #[serde(default)]
  payload: Option<Value>,
  port: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ErrorPacket {
  port: String,
  error: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum JsonPacket {
  Error(ErrorPacket),
  Success(SuccessPacket),
}

impl From<JsonPacket> for Packet {
  fn from(value: JsonPacket) -> Self {
    match value {
      JsonPacket::Error(v) => Packet::err(v.port, v.error),
      JsonPacket::Success(v) => {
        if v.payload.is_some() {
          Packet::encode(v.port, v.payload)
        } else if v.flags == 64 {
          Packet::open_bracket(v.port)
        } else if v.flags == 32 {
          Packet::close_bracket(v.port)
        } else {
          Packet::done(v.port)
        }
      }
    }
  }
}
