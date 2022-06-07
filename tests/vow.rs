use std::panic;

use log::{debug, error};
use serde_json::json;
use utils::*;
use wasmflow_transport::{JsonError, TransportJson};

#[test_logger::test(tokio::test)]
async fn test_vow_serve() -> utils::TestResult<()> {
  debug!("Starting collection");
  let (p_tx, p_handle, port) = start_collection(
    "wasmflow",
    "component rpc server",
    &[
      "serve",
      "./crates/integration/test-wasm-component/build/test_component_s.wasm",
      "--trace",
      "--rpc",
    ],
    &[],
  )
  .await?;
  let input_data = "test input";

  let args = vec![format!("input=\"{}\"", input_data)];
  let actual = wafl_invoke(&port, "validate", args).await?;

  let expected = hashmap! { "output".to_owned() => TransportJson {
      signal: None,
      error_msg: None,
      error_kind: JsonError::None,
      value: json!(input_data),
    }
  };

  let result = panic::catch_unwind(|| {
    equals!(actual, vec![expected]);
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
