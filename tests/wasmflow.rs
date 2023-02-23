use std::panic;

use log::{debug, error};
use utils::*;
use wasmflow_packet_stream::Packet;

#[test_logger::test(tokio::test)]
async fn test_wasmflow_serve() -> utils::TestResult<()> {
  debug!("Starting collection");
  let (p_tx, p_handle, port) = start_collection(
    "wasmflow",
    "component rpc server",
    &[
      "serve",
      "./crates/integration/test-baseline-component/build/baseline.signed.wasm",
      "--trace",
      "--rpc",
    ],
    &[],
  )
  .await?;
  let input_data = "test input";

  let args = vec![format!("input=\"{}\"", input_data)];
  let actual = wasmflow_invoke(&port, "validate", args).await?;
  let expected = vec![Packet::encode("output", input_data)];

  let result = panic::catch_unwind(|| {
    equals!(actual, expected);
  });

  p_tx.send(Signal::Kill).await?;
  p_handle.await??;
  println!("Collection shut down");

  match result {
    Ok(_) => Ok(()),
    Err(e) => {
      error!("{:?}", e);
      Err(anyhow!("Failed"))
    }
  }
}
