use std::panic;

use log::{
  debug,
  error,
};
use serde_json::json;
use utils::*;
use vino_transport::message_transport::{
  JsonError,
  JsonOutput,
};

#[test_env_log::test(tokio::test)]
async fn test_collection() -> utils::TestResult<()> {
  debug!("Starting provider");
  let (p_tx, p_handle, port) = start_provider("vino-collection-inmemory", &[], &[]).await?;
  debug!("Starting host");
  let (h_tx, h_handle, h_port) = start_provider(
    "vino",
    &[
      "start",
      "./tests/manifests/collection-inmemory.yaml",
      "--trace",
    ],
    &[("TEST_PORT", &port)],
  )
  .await?;
  // let (p_tx, p_handle, port) = start_provider("vino-collection-inmemory", &[]).await?;

  // debug!("Starting host");
  // let (h_tx, h_handle) = start_vino(
  //   "./tests/manifests/collection-inmemory.yaml",
  //   vec![("TEST_PORT", &port)],
  // )
  // .await?;

  let collection_id = "some_collection";
  let doc_id = "some_document";
  let document = "this is a document";

  let args = json!({ "collection_id" : collection_id, "document_id": doc_id, "document": document});
  println!("Storing document: {}", args);
  let result_add = vinoc_invoke(&h_port, "add", args).await?;
  println!("Result: {:?}", result_add);

  let expected_add = JsonOutput {
    error_msg: None,
    error_kind: JsonError::None,
    value: json!("some_document"),
  };

  let args = json!({ "collection_id" : collection_id, "document_id": doc_id});
  println!("Getting document: {}", args);
  let result_get = vinoc_invoke(&h_port, "get", args).await?;
  println!("Result: {:?}", result_get);

  let expected_get = JsonOutput {
    error_msg: None,
    error_kind: JsonError::None,
    value: json!(document),
  };

  let args = json!({ "collection_id": collection_id });
  println!("Listing documents: {}", args);
  let result_list = vinoc_invoke(&h_port, "list", args).await?;
  println!("Result: {:?}", result_list);

  let expected_list = JsonOutput {
    error_msg: None,
    error_kind: JsonError::None,
    value: json!([doc_id]),
  };

  let result = panic::catch_unwind(|| {
    equals!(result_add, vec![expected_add]);
    equals!(result_get, vec![expected_get]);
    equals!(result_list, vec![expected_list]);
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
