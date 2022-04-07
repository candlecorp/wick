use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use vino_transport::{MessageTransport, TransportWrapper};

use super::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A wrapper object for the packets returned from the provider call.
#[must_use]
pub struct ProviderOutput {
  packets: HashMap<String, Vec<MessageTransport>>,
}

impl ProviderOutput {
  /// Initialize a [ProviderOutput] with a [Vec<TransportWrapper>]
  pub fn new(packets: Vec<TransportWrapper>) -> Self {
    let mut map = HashMap::new();
    for packet in packets {
      let list = map.entry(packet.port).or_insert_with(Vec::new);
      list.push(packet.payload);
    }
    Self { packets: map }
  }

  /// Get a list of [MessageTransport] from the specified port.
  pub fn take<T: AsRef<str>>(&mut self, port: T) -> Option<Vec<MessageTransport>> {
    self.packets.remove(port.as_ref())
  }
}

/// Iterator wrapper for a list of [MessageTransport]s
#[must_use]
pub struct PortOutput {
  name: String,
  iter: Box<dyn Iterator<Item = MessageTransport>>,
}

impl std::fmt::Debug for PortOutput {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("PortOutput").field("iter", &self.name).finish()
  }
}

impl PortOutput {
  /// Constructor for [PortOutput] that takes a list of [MessageTransport]
  pub fn new(name: String, packets: Vec<MessageTransport>) -> Self {
    Self {
      name,
      iter: Box::new(packets.into_iter()),
    }
  }

  /// Grab the next value and deserialize it in one method.
  pub fn try_next_into<T: DeserializeOwned>(&mut self) -> Result<T, Error> {
    match self.iter.next() {
      Some(val) => Ok(val.deserialize().map_err(|e| Error::Codec(e.to_string()))?),
      None => Err(Error::EndOfOutput(self.name.clone())),
    }
  }
}

impl Iterator for PortOutput {
  type Item = MessageTransport;

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next()
  }
}
