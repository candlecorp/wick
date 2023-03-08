use anyhow::Result;
use runtime_testutils::*;
use wick_packet::{packet_stream, Packet};

#[test_logger::test(tokio::test)]
#[ignore = "TODO:FIX_SEND_AFTER_DONE"]
async fn simple_schematic() -> Result<()> {
  tester(
    "./manifests/v0/collections/native-component.wafl",
    packet_stream!(("left", 42), ("right", 302309)),
    "native_component",
    vec![Packet::encode("output", 42 + 302309), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn global_collections() -> Result<()> {
  tester(
    "./manifests/v0/global-collection-def.wafl",
    packet_stream!(("input", "some input")),
    "first_schematic",
    vec![Packet::encode("output", "some input"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn subnetworks() -> Result<()> {
  tester(
    "./manifests/v0/sub-network-parent.wafl",
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
//   let _ = make_rpc_server(socket, Arc::new(test_native_collection::Collection::default()));
//   env::set_var("TEST_PORT", port.to_string());

//   let (network, _) = init_network_from_yaml("./manifests/v0/collections/grpc.wafl").await?;
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
//   let (network, _) = init_network_from_yaml("./manifests/v0/collections/grpctar.wafl").await?;

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
