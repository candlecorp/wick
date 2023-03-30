use futures::stream::StreamExt;
use tracing::debug;
use wick_config::WickConfiguration;
use wick_packet::{Entity, InherentData, Invocation, Packet, PacketStream};
use wick_runtime::{Network, NetworkBuilder};

pub async fn init_network_from_yaml(path: &str) -> anyhow::Result<(Network, uuid::Uuid)> {
  let host_def = WickConfiguration::load_from_file(path).await?.try_component_config()?;
  debug!("Manifest loaded");

  let builder = NetworkBuilder::from_definition(host_def)?;

  let network = builder.build().await?;

  let nuid = network.uid;
  Ok((network, nuid))
}

#[allow(unused)]
pub async fn tester(path: &str, stream: PacketStream, target: &str, mut expected: Vec<Packet>) -> anyhow::Result<()> {
  let (network, _) = init_network_from_yaml(path).await?;
  let inherent = InherentData::new(1, 1000);

  let result = network
    .invoke(
      Invocation::new(Entity::test("simple schematic"), Entity::local(target), Some(inherent)),
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
