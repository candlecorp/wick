use log::debug;
use test_native_collection::Collection;
use wasmflow_packet::PacketMap;
use wasmflow_rpc::RpcHandler;

#[test_logger::test(tokio::test)]
async fn request() -> anyhow::Result<()> {
  let collection = Collection::default();
  let input = "some_input";
  let job_payload: PacketMap = vec![("input", input)].into();
  let invocation = wasmflow_invocation::Invocation::new_test(
    file!(),
    wasmflow_entity::Entity::local("test-component"),
    job_payload,
    None,
  );

  let mut outputs = collection.invoke(invocation).await?;
  let packets: Vec<_> = outputs.drain_port("output").await?;
  let output = packets[0].clone();
  println!("Received payload from [{}]", output.port);
  let payload: String = output.payload.deserialize()?;

  println!("outputs: {:?}", payload);
  assert_eq!(payload, "TEST: some_input");

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn list() -> anyhow::Result<()> {
  let collection = Collection::default();

  let response = collection.get_list()?;
  debug!("list response : {:?}", response);

  Ok(())
}
