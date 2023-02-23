use futures::StreamExt;
use wasmflow_packet_stream::{fan_out, Observer, Packet, PacketStream};

use crate::request_response;

request_response!(job, minijob => {
  inputs: {
    left => u64,
    rightseed => u64,
  },
  output: "output",
});

#[allow(clippy::unused_async)]
pub(crate) async fn minijob(left: u64, right: u64) -> Result<u64, wasmflow_packet_stream::Error> {
  Ok(left - right)
}
