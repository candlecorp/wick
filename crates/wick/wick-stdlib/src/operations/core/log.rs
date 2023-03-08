use futures::StreamExt;
use wick_packet::{fan_out, Observer, Packet, PacketStream};

use crate::request_response;

request_response!(job, minijob => {
  inputs: {
    input => String,
  },
  output: "output",
});

#[allow(clippy::unused_async)]
pub(crate) async fn minijob(input: String) -> Result<String, wick_packet::Error> {
  println!("Logger: {}", input);
  Ok(input)
}
