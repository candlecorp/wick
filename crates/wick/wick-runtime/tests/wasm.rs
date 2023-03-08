use runtime_testutils::*;
use wick_packet::{packet_stream, Packet};

type Result<T> = anyhow::Result<T, anyhow::Error>;

#[test_logger::test(tokio::test)]
async fn good_wasm_component() -> Result<()> {
  tester(
    "./manifests/v0/wasmrs-component.wafl",
    packet_stream!(("input", "1234567890")),
    "test",
    vec![Packet::encode("output", "1234567890"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
#[ignore = "TODO:FIX TRANSACTION_MISSING"]
async fn bad_wasm_component() -> Result<()> {
  tester(
    "./manifests/v0/bad-wasmrs-component.wafl",
    packet_stream!(("input", "1234567890")),
    "test",
    vec![Packet::err("output", "wat")],
  )
  .await
}

// #[test_logger::test(tokio::test)]
// async fn good_wasi_component() -> Result<()> {
//   let tempdir = std::env::temp_dir();
//   let tempfile = tempdir.join("test_file.txt");
//   let now = SystemTime::now();
//   let time = now.duration_since(UNIX_EPOCH).unwrap().as_millis().to_string();
//   debug!("Writing '{}' to test file {:?}", time, tempfile);
//   std::fs::write(&tempfile, &time).unwrap();
//   std::env::set_var("TEST_TEMPDIR", tempdir);
//   let (network, _) = init_network_from_yaml("./manifests/v0/wasi-component.wafl").await?;
//   std::env::remove_var("TEST_TEMPDIR");

//   let data = hashmap! {
//       "filename" => "/test_file.txt",
//   };

//   println!("Requesting first run");
//   let mut result = network
//     .invoke(Invocation::new(
//       Entity::test("wapc_component"),
//       Entity::local("wasi_component"),
//       data.try_into()?,
//       None,
//     ))
//     .await?;

//   let mut messages: Vec<TransportWrapper> = result.drain_port("contents").await?;
//   assert_eq!(messages.len(), 1);

//   let output: TransportWrapper = messages.pop().unwrap();
//   let result: String = output.payload.deserialize()?;
//   println!("Output for first run: {:?}", result);
//   assert_eq!(result, time);

//   Ok(())
// }

// #[test_logger::test(tokio::test)]
// async fn subnetwork_link_call() -> Result<()> {
//   let (network, _) = init_network_from_yaml("./manifests/v0/subnetwork-ns-link.wafl").await?;

//   let data = hashmap! {
//       "input" => "hello world",
//   };

//   let mut result = network
//     .invoke(Invocation::new(
//       Entity::test("ns-link"),
//       Entity::local("test"),
//       data.try_into()?,
//       None,
//     ))
//     .await?;

//   let mut messages = result.drain_port("output").await?;
//   println!("{:#?}", messages);
//   assert_eq!(messages.len(), 1);

//   let output = messages.pop().unwrap();
//   let result: String = output.payload.deserialize()?;
//   println!("Output for first run: {:?}", result);
//   assert_eq!(result, "DLROW OLLEH");

//   Ok(())
// }
