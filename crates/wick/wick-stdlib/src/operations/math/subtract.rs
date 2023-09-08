use futures::StreamExt;
use wick_packet::{fan_out, Observer, Packet, PacketStream};

use crate::request_response;

request_response!(job, minijob => {
  inputs: {
    left => u64,
    rightseed => u64,
  },
  output: "output",
});

pub(crate) const fn minijob(left: u64, right: u64) -> Result<u64, wick_packet::Error> {
  Ok(left - right)
}
