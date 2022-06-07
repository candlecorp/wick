#[cfg(test)]
mod test {
  use std::panic;

  use log::{debug, error};
  use serde_json::json;
  use utils::*;
  use wasmflow_transport::{JsonError, TransportJson};

  #[test_logger::test(tokio::test)]
  async fn integration_test_collection() -> utils::TestResult<()> {
    debug!("Starting vow");
    let (p_tx, p_handle, _) = start_collection(
      "wasmflow",
      "component RPC server",
      &[
        "serve",
        "./crates/integration/test-wasm-component/build/test_component_s.wasm",
        "--mesh",
        "--trace",
        "--rpc",
        "--nats",
        "127.0.0.1",
        "--id",
        "mesh_wapc",
      ],
      &[],
    )
    .await?;

    let (h_tx, h_handle, h_port) = start_collection(
      "wasmflow",
      "wasmflow",
      &[
        "serve",
        "./tests/manifests/ns-link-mesh.yaml",
        "--rpc",
        "--trace",
        "--nats",
        "127.0.0.1",
      ],
      &[],
    )
    .await?;

    let input = "hello world";

    let args = vec![format!("input=\"{}\"", input)];
    let result = wafl_invoke(&h_port, "ns-link", args).await?;
    println!("Result: {:?}", result);

    let expected = hashmap! {
      "output".to_owned() => TransportJson {
        error_msg: None,
        error_kind: JsonError::None,
        signal: None,
        value: json!("DLROW OLLEH"),
      }
    };

    let result = panic::catch_unwind(|| {
      equals!(result, vec![expected]);
    });

    h_tx.send(Signal::Kill).await?;
    h_handle.await??;
    println!("Host shut down");

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
}
