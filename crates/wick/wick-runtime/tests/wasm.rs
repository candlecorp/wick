mod utils;
use serde_json::json;
use utils::*;
use wick_packet::{packet_stream, Packet};

type Result<T> = anyhow::Result<T, anyhow::Error>;

#[test_logger::test(tokio::test)]
async fn good_wasm_component_v0() -> Result<()> {
  common_test(
    "./manifests/v0/wasmrs-component.yaml",
    packet_stream!(("input", "1234567890")),
    "test",
    vec![Packet::encode("output", "1234567890"), Packet::done("output")],
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn good_wasm_component_v1() -> Result<()> {
  test_with_config(
    "../../integration/test-baseline-component/component.yaml",
    packet_stream!(("left", 10), ("right", 1001)),
    "add",
    vec![Packet::encode("output", 1011), Packet::done("output")],
    json!({"default_err":"custom error"}).try_into()?,
  )
  .await
}

#[test_logger::test(tokio::test)]
async fn flow_with_inputless_component() -> Result<()> {
  common_test(
    "./manifests/v1/flow_with_inputless_component.yaml",
    packet_stream!(("input", "hello world")),
    "test",
    vec![
      Packet::encode("uuid", "aa38fc21-01bd-ade2-254b-185bf88a15f7"),
      Packet::done("uuid"),
      Packet::encode("output", "hello world"),
      Packet::done("output"),
    ],
  )
  .await
}

#[test_logger::test(tokio::test)]
#[ignore = "signature check needs to be re-enabled for this test to pass"]
async fn bad_wasm_component_v1() -> Result<()> {
  let path = "./manifests/v1/bad-wasmrs-component.yaml";
  let result = init_engine_from_yaml(path, None, std::time::Duration::from_secs(1)).await;
  assert!(result.is_err());
  println!("Error: {:?}", result.err().unwrap());
  // let result = result.err().unwrap().source().unwrap();
  // assert_eq!(result, anyhow::anyhow!("hey"));
  Ok(())
}

#[test_logger::test(tokio::test)]
async fn bad_wasm_component() -> Result<()> {
  common_test(
    "./manifests/v0/bad-wasmrs-component.yaml",
    packet_stream!(("input", "1234567890")),
    "test",
    vec![
      Packet::err(
        "output",
        "Operation wick://wapc/error timed out waiting for upstream data.",
      ),
      Packet::done("output"),
    ],
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
//   let (network, _) = init_network_from_yaml("./manifests/v0/wasi-component.yaml").await?;
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
//   let (network, _) = init_network_from_yaml("./manifests/v0/subnetwork-ns-link.yaml").await?;

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
