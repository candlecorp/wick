use std::panic;

use serde_json::json;
use tracing::{
  debug,
  error,
};
use utils::*;
use vino_transport::message_transport::{
  JsonError,
  JsonOutput,
};

#[test_env_log::test(tokio::test)]
async fn test_vow_serve() -> utils::TestResult<()> {
  debug!("Starting provider");
  let (p_tx, p_handle, _port) = start_provider(
    "vow",
    &[
      "serve",
      "--port=8060",
      "./crates/integration/test-wapc-component/build/test_component_s.wasm",
    ],
  )
  .await?;

  let args = json!({ "input" : "test input"});
  let actual = vinoc_invoke("validate", args).await?;

  let expected = JsonOutput {
    error_msg: None,
    error_kind: JsonError::None,
    value: json!("test input"),
  };

  let result = panic::catch_unwind(|| {
    equals!(actual, hashmap! {"output".to_owned() => expected});
  });

  p_tx.send(Signal::Kill).await?;
  p_handle.await??;
  println!("Provider shut down");

  match result {
    Ok(_) => Ok(()),
    Err(e) => {
      error!("{:?}", e);
      Err(anyhow!("Failed"))
    }
  }
}
