use anyhow::Result;
use tokio_stream::StreamExt;
use tracing::trace;
use wasmflow_sdk::v1::transport::{MessageTransport, TransportStream};

pub async fn print_stream_json(mut stream: TransportStream, filter: &[String], terse: bool, raw: bool) -> Result<()> {
  if !filter.is_empty() {
    trace!("filtering only {:?}", filter);
  }
  while let Some(wrapper) = stream.next().await {
    trace!(message=%wrapper, "output message");
    if (wrapper.payload.is_signal()) && !raw {
      continue;
    }
    if !filter.is_empty() && !filter.iter().any(|name| name == &wrapper.port) {
      tracing::debug!(port = %wrapper.port, "filtering out");
      continue;
    }
    if terse {
      if let MessageTransport::Failure(err) = &wrapper.payload {
        return Err(anyhow::Error::msg(err.message().to_owned()));
      }
      let mut json = wrapper.payload.as_json();

      json.as_object_mut().and_then(|o| o.remove("value")).map_or_else(
        || unreachable!("Message did not have an error nor a value: {}", json),
        |v| {
          println!(
            "{}",
            match v {
              serde_json::Value::String(s) => s,
              v => v.to_string(),
            }
          );
        },
      );
    } else {
      println!("{}", wrapper.as_json());
    }
  }
  trace!("stream complete");
  Ok(())
}
