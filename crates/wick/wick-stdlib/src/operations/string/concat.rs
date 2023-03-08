use futures::StreamExt;
use wick_packet::{fan_out, Observer, Packet, PacketStream};

use crate::request_response;

request_response!(job, minijob => {
  inputs: {
    left => String,
    right => String,
  },
  output: "output",
});

#[allow(clippy::unused_async)]
async fn minijob(left: String, right: String) -> Result<String, wick_packet::Error> {
  Ok(format!("{}{}", left, right))
}
