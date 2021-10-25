use std::panic;

use log::{
  debug,
  error,
};
use serde_json::json;
use utils::*;
use vino_transport::{
  JsonError,
  TransportJson,
};

mod test {
  use super::*;
  #[test_logger::test(tokio::test)]
  async fn test_collection() -> utils::TestResult<()> {
    debug!("Starting vow");
    let (p_tx, p_handle, _) = start_provider(
      "vow",
      "component RPC server",
      &[
        "serve",
        "./crates/integration/test-wapc-component/build/test_component_s.wasm",
        "--lattice",
        "--trace",
        "--rpc",
        "--nats",
        "nats.vinodev.com",
        "--id",
        "lattice_wapc",
      ],
      &[],
    )
    .await?;

    let (h_tx, h_handle, h_port) = start_provider(
      "vino",
      "vino",
      &[
        "serve",
        "./tests/manifests/ns-link-lattice.yaml",
        "--rpc",
        "--trace",
        "--nats",
        "nats.vinodev.com",
      ],
      &[],
    )
    .await?;

    let input = "hello world";

    let args = vec![format!("input=\"{}\"", input)];
    let result = vinoc_invoke(&h_port, "ns-link", args).await?;
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
    println!("Provider shut down");

    match result {
      Ok(_) => Ok(()),
      Err(e) => {
        error!("{:?}", e);
        Err(anyhow!("Failed"))
      }
    }
  }
}
