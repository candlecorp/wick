use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use wick_config::config::TestCase;
use wick_packet::{InherentData, Packet, PacketStream, RuntimeConfig};

use crate::utils::gen_packet;
use crate::TestError;

#[derive(Debug, Clone)]
pub struct UnitTest<'a> {
  pub test: &'a TestCase,
  pub actual: Vec<Packet>,
}

pub(crate) fn get_payload(
  test: &UnitTest,
  root_config: Option<&RuntimeConfig>,
  op_config: Option<&RuntimeConfig>,
) -> Result<(PacketStream, InherentData), TestError> {
  let mut packets = Vec::new();
  let mut not_done = HashSet::new();
  if test.test.inputs().is_empty() {
    packets.push(Packet::no_input());
  } else {
    for packet in test.test.inputs() {
      let done = packet.flags().map_or(false, |f| f.done());
      if done {
        not_done.remove(packet.port());
      } else {
        not_done.insert(packet.port());
      }
      debug!("Test input for port {:?}", packet);
      packets.push(gen_packet(packet, root_config, op_config)?);
    }
    for port in not_done {
      packets.push(Packet::done(port));
    }
  }

  let (seed, timestamp) = test.test.inherent().map_or_else(
    || {
      (
        0,
        SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .unwrap()
          .as_millis()
          .try_into()
          .unwrap(),
      )
    },
    |inherent| {
      let seed = inherent.seed().unwrap_or(0);
      let timestamp = inherent.timestamp().unwrap_or(
        SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .unwrap()
          .as_millis()
          .try_into()
          .unwrap(),
      );
      (seed, timestamp)
    },
  );

  Ok((packets.into(), InherentData::new(seed, timestamp)))
}
