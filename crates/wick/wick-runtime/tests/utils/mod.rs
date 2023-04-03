use std::time::Duration;

use futures::stream::StreamExt;
use tracing::debug;
use wick_config::WickConfiguration;
use wick_packet::{Entity, InherentData, Invocation, Packet, PacketStream};
use wick_runtime::{Engine, EngineBuilder};

pub async fn init_engine_from_yaml(path: &str, timeout: Duration) -> anyhow::Result<(Engine, uuid::Uuid)> {
  let host_def = WickConfiguration::load_from_file(path).await?.try_component_config()?;
  debug!("Manifest loaded");

  let builder = EngineBuilder::from_definition(host_def)?
    .namespace("__TEST__")
    .timeout(timeout);

  let engine = builder.build().await?;

  let nuid = engine.uid;
  Ok((engine, nuid))
}

#[allow(unused)]
pub async fn common_test(
  path: &str,
  stream: PacketStream,
  target: &str,
  mut expected: Vec<Packet>,
) -> anyhow::Result<()> {
  base_test(path, stream, Entity::local(target), expected).await
}

#[allow(unused)]
pub async fn base_test(
  path: &str,
  stream: PacketStream,
  target: Entity,
  mut expected: Vec<Packet>,
) -> anyhow::Result<()> {
  let (engine, _) = init_engine_from_yaml(path, Duration::from_secs(1)).await?;
  let inherent = InherentData::new(1, 1000);

  let target = if target.namespace() == Entity::LOCAL {
    Entity::operation(engine.namespace(), target.name())
  } else {
    target
  };

  let result = engine
    .invoke(
      Invocation::new(Entity::test("simple schematic"), target, Some(inherent)),
      stream,
    )
    .await?;

  let messages: Vec<_> = result.collect().await;
  println!("Result: {:?}", messages);
  assert_eq!(messages.len(), expected.len());
  expected.reverse();
  for packet in messages {
    let expected = expected.pop().unwrap();
    assert_eq!(packet.unwrap(), expected);
  }

  Ok(())
}
