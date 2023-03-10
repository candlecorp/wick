use futures::StreamExt;
use wick_packet::{fan_out, Observer, Packet, PacketStream};

use crate::request_response;

request_response!(job, minijob => {
  inputs: {
    input => String,
  },
  output: "output",
});

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn minijob(_input: String) -> Result<String, wick_packet::Error> {
  panic!("This component will always panic");
}
