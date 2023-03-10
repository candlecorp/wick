use futures::StreamExt;
use seeded_random::Random;
use wick_packet::{fan_out, Observer, Packet, PacketStream};

use crate::request_response;

request_response!(job, minijob => {
  inputs: {
    seed => u64,
  },
  output: "output",
});

pub(crate) fn minijob(seed: u64) -> Result<String, wick_packet::Error> {
  let rng = Random::from_seed(seeded_random::Seed::unsafe_new(seed));
  Ok(rng.uuid().as_hyphenated().to_string())
}
