use std::collections::{HashSet, VecDeque};

use wasmflow_packet::Packet;
type PortBuffer = VecDeque<(String, Packet)>;

#[derive(Debug, Default)]
pub(crate) struct Transaction {
  pub(crate) buffer: PortBuffer,
  pub(crate) ports: HashSet<String>,
}
