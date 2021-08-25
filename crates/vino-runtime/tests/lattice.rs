use std::collections::HashMap;

#[path = "./runtime_utils/mod.rs"]
mod utils;
use tokio_stream::StreamExt;
use utils::*;
use vino_entity::Entity;
use vino_runtime::prelude::TransportWrapper;
use vino_transport::MessageTransport;

// #[test_env_log::test(actix_rt::test)]
// async fn multi_host_lattice() -> Result<()> {
//   let (network, _) = init_network_from_yaml("./manifests/v0/lattice-one.yaml").await?;

//   let data = hashmap! {
//       "input" => "simple string",
//   };

//   let mut result = network
//     .request("simple", Entity::test("simple schematic"), &data)
//     .await?;

//   println!("Result: {:?}", result);
//   let mut messages: Vec<TransportWrapper> = result.collect_port("output").await;
//   assert_eq!(result.buffered_size(), (0, 0));
//   assert_eq!(messages.len(), 1);

//   let msg: TransportWrapper = messages.pop().unwrap();
//   println!("Output: {:?}", msg);
//   let output: String = msg.payload.try_into()?;

//   equals!(output, "simple string");
//   Ok(())
// }
