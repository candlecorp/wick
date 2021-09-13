#[path = "./runtime_utils/mod.rs"]
mod utils;
use utils::*;
use vino_entity::Entity;
use vino_runtime::prelude::TransportWrapper;

#[test_logger::test(actix_rt::test)]
async fn multi_host_lattice() -> Result<()> {
  let (_network2, _) = init_network_from_yaml("./manifests/v0/lattice-two.yaml").await?;
  let (network1, _) = init_network_from_yaml("./manifests/v0/lattice-one.yaml").await?;

  let data = hashmap! {
      "parent_input" => "simple string",
  };

  let mut result = network1
    .request("schematic-one", Entity::test("multi_host_lattice"), &data)
    .await?;

  println!("Result: {:?}", result);
  let mut messages: Vec<TransportWrapper> = result.collect_port("parent_output").await;
  assert_eq!(result.buffered_size(), (0, 0));
  assert_eq!(messages.len(), 1);

  let msg: TransportWrapper = messages.pop().unwrap();
  println!("Output: {:?}", msg);
  let output: String = msg.payload.try_into()?;

  equals!(output, "simple string");
  Ok(())
}
