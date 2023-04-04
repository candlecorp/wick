use anyhow::Result;
mod utils;
use utils::*;
use wick_packet::{packet_stream, Packet};

#[test_logger::test(tokio::test)]
async fn simple_schematic() -> Result<()> {
  common_test(
    "./manifests/v0/collections/native-component.yaml",
    packet_stream!(("left", 42), ("right", 302309)),
    "native_component",
    vec![Packet::encode("output", 42 + 302309 + 302309), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn global_collections() -> Result<()> {
  common_test(
    "./manifests/v0/global-collection-def.yaml",
    packet_stream!(("input", "some input")),
    "first_schematic",
    vec![Packet::encode("output", "some input"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn subnetworks() -> Result<()> {
  common_test(
    "./manifests/v0/sub-network-parent.yaml",
    packet_stream!(("input", "some input")),
    "parent",
    vec![Packet::encode("output", "some input"), Packet::done("output")],
  )
  .await
}

// #[test_logger::test(tokio::test)]
// async fn grpc() -> Result<()> {
//   let socket = bind_new_socket()?;
//   let port = socket.local_addr()?.port();
//   let _ = make_rpc_server(socket, Arc::new(test_native_component::Collection::default()));
//   env::set_var("TEST_PORT", port.to_string());

//   let (network, _) = init_network_from_yaml("./manifests/v0/collections/grpc.yaml").await?;
//   let user_data = "Hello world";

//   let data = hashmap! {
//       "input" => user_data,
//   };

//   let mut result = network
//     .invoke(Invocation::new(
//       Entity::test("grpc"),
//       Entity::local("grpc"),
//       data.try_into()?,
//       None,
//     ))
//     .await?;

//   let mut messages: Vec<TransportWrapper> = result.drain_port("output").await?;
//   assert_eq!(messages.len(), 1);

//   let output: String = messages.pop().unwrap().payload.deserialize()?;

//   assert_eq!(output, format!("TEST: {}", user_data));

//   Ok(())
// }

// #[test_logger::test(tokio::test)]
// #[ignore] // Need to automate the creation of GrpcTar bundles
// async fn grpctar() -> Result<()> {
//   let (network, _) = init_network_from_yaml("./manifests/v0/collections/grpctar.yaml").await?;

//   let data = hashmap! {
//       "left" => 32,
//       "right" => 43,
//   };

//   let mut result = network
//     .invoke(Invocation::new(
//       Entity::test("grpctar"),
//       Entity::local("grpctar"),
//       data.try_into()?,
//       None,
//     ))
//     .await?;

//   let mut messages: Vec<TransportWrapper> = result.drain_port("output").await?;
//   assert_eq!(messages.len(), 1);

//   let output: u32 = messages.pop().unwrap().payload.deserialize()?;

//   assert_eq!(output, 75);

//   Ok(())
// }
