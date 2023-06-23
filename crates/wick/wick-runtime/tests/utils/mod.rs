use futures::stream::StreamExt;
use tracing::debug;
use wick_config::WickConfiguration;
use wick_packet::{Entity, InherentData, Invocation, Packet, PacketStream, RuntimeConfig};
use wick_runtime::{Runtime, RuntimeBuilder};

pub async fn init_engine_from_yaml(path: &str, config: Option<RuntimeConfig>) -> anyhow::Result<(Runtime, uuid::Uuid)> {
  let mut host_def = WickConfiguration::load_from_file(path).await?;
  host_def.set_root_config(config);
  let host_def = host_def.finish()?.try_component_config()?;
  debug!("Manifest loaded");

  let builder = RuntimeBuilder::from_definition(host_def).namespace("__TEST__");

  let engine = builder.build(None).await?;

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
  base_test(path, stream, Entity::local(target), expected, None).await
}

#[allow(unused)]
pub async fn test_with_config(
  path: &str,
  stream: PacketStream,
  target: &str,
  mut expected: Vec<Packet>,
  config: RuntimeConfig,
) -> anyhow::Result<()> {
  base_test(path, stream, Entity::local(target), expected, Some(config)).await
}

#[allow(unused)]
pub async fn base_test(
  path: &str,
  stream: PacketStream,
  target: Entity,
  mut expected: Vec<Packet>,
  config: Option<RuntimeConfig>,
) -> anyhow::Result<()> {
  let cwd = std::env::current_dir()?;
  let (engine, _) = init_engine_from_yaml(path, config).await?;
  let inherent = InherentData::new(1, 1000);

  let target = if target.component_id() == Entity::LOCAL {
    Entity::operation(engine.namespace(), target.operation_id())
  } else {
    target
  };

  let result = engine
    .invoke(
      Invocation::test("simple schematic", target, stream, Some(inherent))?,
      Default::default(),
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
