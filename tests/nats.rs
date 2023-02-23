// #[cfg(test)]
// mod test {
//   use std::panic;

//   use log::{debug, error, warn};
//   use serde_json::json;
//   use utils::*;
//   use wasmflow_packet_stream::Packet;

//   #[test_logger::test(tokio::test)]
//   async fn integration_test_mesh() -> utils::TestResult<()> {
//     debug!("Starting host 1");
//     let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| {
//       warn!("'NATS_URL' not present, defaulting to 127.0.0.1");
//       "127.0.0.1".to_owned()
//     });
//     let nats_arg = format!("--nats={}", nats_url);
//     let (p2_tx, p2_handle, _port2) = start_collection(
//       "wasmflow",
//       "network-two",
//       &[
//         "serve",
//         "./tests/manifests/mesh-two.wafl",
//         "--id=network-two",
//         "--trace",
//         &nats_arg,
//       ],
//       &[],
//     )
//     .await?;
//     let (p_tx, p_handle, port) = start_collection(
//       "wasmflow",
//       "network-one",
//       &[
//         "serve",
//         "./tests/manifests/mesh-one.wafl",
//         &nats_arg,
//         "--id=network-one",
//         "--trace",
//       ],
//       &[],
//     )
//     .await?;

//     let input_data = "test input";
//     let args = vec![format!("parent_input=\"{}\"", input_data)];
//     let actual = wasmflow_invoke(&port, "schematic-one", args).await?;

//     let expected = vec![Packet::encode("parent_output", input_data)];

//     let result = panic::catch_unwind(|| {
//       equals!(actual, expected);
//     });

//     p_tx.send(Signal::Kill).await?;
//     p_handle.await??;
//     println!("Collection 1 shut down");
//     p2_tx.send(Signal::Kill).await?;
//     p2_handle.await??;
//     println!("Collection 1 shut down");

//     match result {
//       Ok(_) => Ok(()),
//       Err(e) => {
//         error!("{:?}", e);
//         Err(anyhow!("Failed"))
//       }
//     }
//   }
// }
