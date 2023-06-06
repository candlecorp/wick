use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio_stream::wrappers::UnboundedReceiverStream;
use wick_config::config::TestCase;
use wick_packet::{InherentData, Packet, PacketStream};

use crate::utils::gen_packet;

#[derive(Debug, Clone)]
pub struct UnitTest<'a> {
  pub test: &'a TestCase,
  pub actual: Vec<Packet>,
}

pub(crate) fn get_payload(test: &UnitTest) -> (PacketStream, InherentData) {
  let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
  let mut not_done = HashSet::new();
  for packet in test.test.inputs() {
    let done = packet.flags().map_or(false, |f| f.done());
    if done {
      not_done.remove(packet.port());
    } else {
      not_done.insert(packet.port());
    }
    debug!("Test input for port {:?}", packet);
    tx.send(
      gen_packet(packet)
        .map_err(|e| wick_packet::Error::Component(format!("could not convert test packet to real packet: {}", e))),
    )
    .unwrap();
  }
  for port in not_done {
    tx.send(Ok(Packet::done(port))).unwrap();
  }
  let stream = PacketStream::new(Box::new(UnboundedReceiverStream::new(rx)));
  let (seed, timestamp) = if let Some(inherent) = test.test.inherent() {
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
  } else {
    (
      0,
      SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap(),
    )
  };
  (stream, InherentData::new(seed, timestamp))
}
