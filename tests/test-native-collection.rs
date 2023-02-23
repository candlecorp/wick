use futures::StreamExt;
use log::debug;
use test_native_collection::Collection;
use wasmflow_entity::Entity;
use wasmflow_packet_stream::{packet_stream, Invocation, Packet};
use wasmflow_rpc::RpcHandler;

#[test_logger::test(tokio::test)]
async fn request() -> anyhow::Result<()> {
  let collection = Collection::default();
  let input = "some_input";
  let stream = packet_stream![("input", input)];
  let invocation = Invocation::new(Entity::test(file!()), Entity::local("test-component"), None);

  let outputs = collection.invoke(invocation, stream).await?;
  let mut packets: Vec<_> = outputs.collect().await;

  let output = packets.pop().unwrap().unwrap();
  println!("Received payload [{:?}]", output);

  assert_eq!(output, Packet::encode("output", "TEST: some_input"));

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn list() -> anyhow::Result<()> {
  let collection = Collection::default();

  let response = collection.get_list()?;
  debug!("list response : {:?}", response);

  Ok(())
}
