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
async fn test_collection() -> utils::TestResult<()> {
  debug!("Starting provider");
  let (p_tx, p_handle, port) = start_provider("vino-collection-inmemory").await?;

  debug!("Starting host");
  let (h_tx, h_handle) = start_vino(
    "./tests/manifests/collection-inmemory.yaml",
    vec![("TEST_PORT", &port)],
  )
  .await?;

  let collection_id = "some_collection";
  let doc_id = "some_document";
  let document = "this is a document";

  let args = json!({ "collection_id" : collection_id, "document_id": doc_id, "document": document});
  println!("Storing document: {}", args);
  let result_add = vinoc_invoke("add", args).await?;

  let expected_add = JsonOutput {
    error_msg: None,
    error_kind: JsonError::None,
    value: json!("some_document"),
  };

  let args = json!({ "collection_id" : collection_id, "document_id": doc_id});
  println!("Storing document: {}", args);
  let result_get = vinoc_invoke("get", args).await?;

  let expected_get = JsonOutput {
    error_msg: None,
    error_kind: JsonError::None,
    value: json!(document),
  };

  let args = json!({ "collection_id": collection_id });
  println!("Storing document: {}", args);
  let result_list = vinoc_invoke("list", args).await?;

  let expected_list = JsonOutput {
    error_msg: None,
    error_kind: JsonError::None,
    value: json!([doc_id]),
  };

  let result = panic::catch_unwind(|| {
    equals!(
      result_add,
      hashmap! {"document_id".to_owned() => expected_add}
    );
    equals!(result_get, hashmap! {"document".to_owned() => expected_get});
    equals!(
      result_list,
      hashmap! {"document_ids".to_owned() => expected_list}
    );
  });

  h_tx.send(Signal::Kill).await?;
  h_handle.await??;
  println!("Host shut down");

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
