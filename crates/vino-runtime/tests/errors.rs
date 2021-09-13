#[path = "./runtime_utils/mod.rs"]
mod utils;
use tokio_stream::StreamExt;
use utils::*;
use vino_entity::Entity;
use vino_runtime::prelude::TransportWrapper;

#[test_logger::test(actix_rt::test)]
async fn bad_wapc_component() -> Result<()> {
  let (network, _) = init_network_from_yaml("./manifests/v0/bad-wapc-component.yaml").await?;

  let data = hashmap! {
      "input" => "1234567890",
  };

  let result = network
    .request("schematic", Entity::test("bad_wapc_component"), &data)
    .await?;

  let mut messages: Vec<TransportWrapper> = result.collect().await;
  println!("{:?}", messages);
  assert_eq!(messages.len(), 1);

  let output: TransportWrapper = messages.pop().unwrap();

  println!("output: {:?}", output);
  assert!(output.payload.is_err());
  Ok(())
}
