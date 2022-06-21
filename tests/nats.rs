#[cfg(test)]
mod test {
  use std::panic;

  use log::{debug, error, warn};
  use serde_json::json;
  use utils::*;
  use wasmflow_sdk::v1::transport::{JsonError, TransportJson};

  #[test_logger::test(tokio::test)]
  async fn integration_test_mesh() -> utils::TestResult<()> {
    debug!("Starting host 1");
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| {
      warn!("'NATS_URL' not present, defaulting to 127.0.0.1");
      "127.0.0.1".to_owned()
    });
    let nats_arg = format!("--nats={}", nats_url);
    let (p2_tx, p2_handle, _port2) = start_collection(
      "wasmflow",
      "network-two",
      &[
        "serve",
        "./tests/manifests/mesh-two.yaml",
        "--id=network-two",
        "--trace",
        &nats_arg,
      ],
      &[],
    )
    .await?;
    let (p_tx, p_handle, port) = start_collection(
      "wasmflow",
      "network-one",
      &[
        "serve",
        "./tests/manifests/mesh-one.yaml",
        &nats_arg,
        "--id=network-one",
        "--trace",
      ],
      &[],
    )
    .await?;

    let input_data = "test input";
    let args = vec![format!("parent_input=\"{}\"", input_data)];
    let actual = wafl_invoke(&port, "schematic-one", args).await?;

    let expected = hashmap! { "parent_output".to_owned() => TransportJson {
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
    println!("Collection 1 shut down");
    p2_tx.send(Signal::Kill).await?;
    p2_handle.await??;
    println!("Collection 1 shut down");

    match result {
      Ok(_) => Ok(()),
      Err(e) => {
        error!("{:?}", e);
        Err(anyhow!("Failed"))
      }
    }
  }
}
