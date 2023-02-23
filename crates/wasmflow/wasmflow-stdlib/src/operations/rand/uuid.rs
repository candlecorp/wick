use futures::StreamExt;
use seeded_random::Random;
use wasmflow_packet_stream::{fan_out, Observer, Packet, PacketStream};

use crate::request_response;

request_response!(job, minijob => {
  inputs: {
    seed => u64,
  },
  output: "output",
});

#[allow(clippy::unused_async)]
pub(crate) async fn minijob(seed: u64) -> Result<String, wasmflow_packet_stream::Error> {
  let rng = Random::from_seed(seeded_random::Seed::unsafe_new(seed));
  Ok(rng.uuid().as_hyphenated().to_string())
}
