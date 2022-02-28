#[cfg(test)]
mod test {
  use std::panic;

  use log::{debug, error, warn};
  use serde_json::json;
  use utils::*;
  use vino_transport::{JsonError, TransportJson};

  use super::*;
  #[test_logger::test(tokio::test)]
  async fn integration_test_lattice() -> utils::TestResult<()> {
    debug!("Starting host 1");
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| {
      warn!("'NATS_URL' not present, defaulting to nats.vinodev.com");
      "nats.vinodev.com".to_owned()
    });
    let nats_arg = format!("--nats={}", nats_url);
    let (p2_tx, p2_handle, _port2) = start_provider(
      "vino",
      "network-two",
      &[
        "serve",
        "./tests/manifests/lattice-two.yaml",
        "--id=network-two",
        "--trace",
        &nats_arg,
      ],
      &[],
    )
    .await?;
    let (p_tx, p_handle, port) = start_provider(
      "vino",
      "network-one",
      &[
        "serve",
        "./tests/manifests/lattice-one.yaml",
        &nats_arg,
        "--id=network-one",
        "--trace",
      ],
      &[],
    )
    .await?;

    let input_data = "test input";
    let args = vec![format!("parent_input=\"{}\"", input_data)];
    let actual = vinoc_invoke(&port, "schematic-one", args).await?;

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
    println!("Provider 1 shut down");
    p2_tx.send(Signal::Kill).await?;
    p2_handle.await??;
    println!("Provider 1 shut down");

    match result {
      Ok(_) => Ok(()),
      Err(e) => {
        error!("{:?}", e);
        Err(anyhow!("Failed"))
      }
    }
  }
}
