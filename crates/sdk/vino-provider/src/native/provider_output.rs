use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use vino_transport::{BoxedTransportStream, MessageTransport, TransportStream};

/// A wrapper object for the packets returned from the provider call.
#[allow(missing_debug_implementations)]
pub struct ProviderOutput {
  packets: TransportStream,
}

impl ProviderOutput {
  /// Initialize a [ProviderOutput] with a [Vec<TransportWrapper>]
  #[must_use]
  pub fn new(packets: BoxedTransportStream) -> Self {
    Self {
      packets: TransportStream::new(packets),
    }
  }

  /// Get a list of [MessageTransport] from the specified port.
  pub async fn take<T: AsRef<str> + Send>(&mut self, port: T) -> Vec<MessageTransport> {
    let packets: Vec<_> = self.packets.collect_port(port).await;
    packets.into_iter().map(|p| p.payload).collect()
  }
}

/// Iterator wrapper for a list of [MessageTransport]s
#[must_use]
pub struct PortOutput<T: DeserializeOwned> {
  name: String,
  iter: Box<dyn Iterator<Item = MessageTransport>>,
  _data: PhantomData<T>,
}

impl<T: DeserializeOwned> std::fmt::Debug for PortOutput<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("PortOutput")
      .field("iter", &self.name)
      .finish()
  }
}

impl<T: DeserializeOwned> PortOutput<T> {
  /// Constructor for [PortOutput] that takes a list of [MessageTransport]
  pub fn new(name: String, packets: Vec<MessageTransport>) -> Self {
    Self {
      name,
      iter: Box::new(packets.into_iter()),
      _data: PhantomData,
    }
  }

  /// Grab the next value and deserialize it in one method.
  pub fn try_next_into(&mut self) -> Result<T, super::Error> {
    match self.iter.next() {
      Some(val) => Ok(
        val
          .try_into()
          .map_err(|e| super::Error::Codec(e.to_string()))?,
      ),
      None => Err(super::Error::EndOfOutput(self.name.clone())),
    }
  }
}

impl<T: DeserializeOwned> Iterator for PortOutput<T> {
  type Item = MessageTransport;

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next()
  }
}
