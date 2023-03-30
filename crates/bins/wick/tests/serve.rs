use std::panic;
mod utils;

use tracing::{debug, error};
use utils::*;
use wick_packet::Packet;

#[test_logger::test(tokio::test)]
#[ignore]
async fn test_wick_serve() -> utils::TestResult<()> {
  debug!("Starting collection");
  let (p_tx, p_handle, port) = start_collection(
    "wick",
    "component rpc server",
    &[
      "serve",
      "../../integration/test-baseline-component/component.yaml",
      "--trace",
      "--rpc",
    ],
    &[],
  )
  .await?;
  let input_data = "test input";

  let args = vec![format!("input=\"{}\"", input_data)];
  let actual = wick_invoke(&port, "validate", args).await?;
  let expected = vec![Packet::encode("output", input_data).to_json()];

  let result = panic::catch_unwind(|| {
    assert_eq!(actual, expected);
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
