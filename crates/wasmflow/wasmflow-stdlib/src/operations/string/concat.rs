use futures::StreamExt;
use wasmflow_packet_stream::{fan_out, Observer, Packet, PacketStream};

use crate::request_response;

request_response!(job, minijob => {
  inputs: {
    left => String,
    right => String,
  },
  output: "output",
});

#[allow(clippy::unused_async)]
async fn minijob(left: String, right: String) -> Result<String, wasmflow_packet_stream::Error> {
  Ok(format!("{}{}", left, right))
}
