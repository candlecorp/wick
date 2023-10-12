use std::path::{Path, PathBuf};

use futures::stream::StreamExt;
use tracing::debug;
use wick_config::WickConfiguration;
use wick_packet::{Entity, InherentData, Invocation, Packet, PacketExt, PacketStream, RuntimeConfig};
use wick_runtime::{Runtime, RuntimeBuilder};

#[allow(unused)]
pub fn example(file: &str) -> PathBuf {
  let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let workspace_dir = path.join("..").join("..").join("..").join("examples");
  workspace_dir.join(file)
}

pub async fn init_engine_from_yaml(
  path: impl AsRef<Path>,
  config: Option<RuntimeConfig>,
) -> anyhow::Result<(Runtime, uuid::Uuid)> {
  let mut host_def = WickConfiguration::fetch(path.as_ref(), Default::default()).await?;
  host_def.set_root_config(config);
  let host_def = host_def.finish()?.try_component_config()?;
  debug!("manifest loaded");

  let builder = RuntimeBuilder::from_definition(host_def).namespace("__TEST__");

  let engine = builder.build(None).await?;

  let nuid = engine.uid;
  Ok((engine, nuid))
}

#[allow(unused)]
pub async fn common_test(
  path: impl AsRef<Path>,
  stream: PacketStream,
  target: &str,
  mut expected: Vec<Packet>,
) -> anyhow::Result<()> {
  base_test(path, stream, Entity::local(target), expected, None, None).await
}

#[allow(unused)]
pub async fn test_with_config(
  path: impl AsRef<Path>,
  stream: PacketStream,
  target: &str,
  mut expected: Vec<Packet>,
  root_config: Option<RuntimeConfig>,
  config: Option<RuntimeConfig>,
) -> anyhow::Result<()> {
  base_test(path, stream, Entity::local(target), expected, root_config, config).await
}

#[allow(unused)]
pub async fn base_test(
  path: impl AsRef<Path>,
  stream: PacketStream,
  target: Entity,
  mut expected: Vec<Packet>,
  root_config: Option<RuntimeConfig>,
  config: Option<RuntimeConfig>,
) -> anyhow::Result<()> {
  let cwd = std::env::current_dir()?;
  let (engine, _) = init_engine_from_yaml(path, root_config).await?;
  let inherent = InherentData::new(1, 1000);

  let target = if target.component_id() == Entity::LOCAL {
    Entity::operation(engine.namespace(), target.operation_id())
  } else {
    target
  };

  let result = engine
    .invoke(
      Invocation::test("simple schematic", target, stream, Some(inherent))?,
      config,
    )
    .await?;

  let messages: Vec<_> = result.collect().await;
  println!("Result: {:?}", messages);
  assert_eq!(messages.len(), expected.len());
  expected.reverse();
  for packet in messages {
    let expected = expected.pop().unwrap();
    let actual = packet.unwrap();
    if actual.has_data() {
      let actual = actual.decode_value()?;
      let expected = expected.decode_value()?;
      assert_eq!(
        actual, expected,
        "actual packet value should be equal to expected packet value"
      );
    } else {
      assert_eq!(actual, expected, "actual packet should be equal to expected packet");
    }
  }

  Ok(())
}
