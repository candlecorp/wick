use futures::prelude::*;
use log::debug;
use test_vino_provider::Provider;
use vino_macros::*;
use vino_rpc::RpcHandler;

#[test_logger::test(tokio::test)]
async fn request() -> anyhow::Result<()> {
  let provider = Provider::default();
  let input = "some_input";
  let job_payload = transport_map! {
    "input" => input,
  };

  let mut outputs = provider
    .invoke(
      vino_entity::Entity::component_direct("test-component"),
      job_payload,
    )
    .await?;
  let output = outputs.next().await.unwrap();
  println!("Received payload from [{}]", output.port);
  let payload: String = output.payload.try_into()?;

  println!("outputs: {:?}", payload);
  assert_eq!(payload, "TEST: some_input");

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn list() -> anyhow::Result<()> {
  let provider = Provider::default();

  let response = provider.get_list().await?;
  debug!("list response : {:?}", response);

  Ok(())
}
