use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::{
  Deserialize,
  Serialize,
};
use vino_codec::messagepack::{
  deserialize,
  serialize,
};
use vino_transport::{
  MessageTransport,
  TransportMap,
  TransportWrapper,
};

use super::host_call;
use super::prelude::ComponentError;
use crate::provider_link::ProviderLink;

/// Implementation of the ProviderLink for WASM modules.
pub trait WasmProviderLink {
  /// Get the link string for the call.
  fn get_link(&self, component: &str) -> String;

  /// Get the originating component entity URL.
  fn get_origin(&self) -> String;

  /// Make a call to the linked provider.
  fn call<T: From<ProviderOutput>>(
    &self,
    component: &str,
    payload: impl Into<TransportMap>,
  ) -> Result<T, super::Error> {
    let payload: TransportMap = payload.into();
    let result = host_call(
      "1",
      &self.get_origin(),
      &self.get_link(component),
      &serialize(&payload)?,
    )?;
    let packets: Vec<TransportWrapper> = deserialize(&result)?;
    let output = ProviderOutput::new(packets);
    Ok(output.into())
  }
}

impl WasmProviderLink for ProviderLink {
  fn get_link(&self, component: &str) -> String {
    self.get_component_url(component)
  }

  fn get_origin(&self) -> String {
    self.get_origin_url()
  }
}

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
    f.debug_struct("PortOutput")
      .field("iter", &self.name)
      .finish()
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
  pub fn try_next_into<T: DeserializeOwned>(&mut self) -> Result<T, ComponentError> {
    match self.iter.next() {
      Some(val) => Ok(
        val
          .try_into()
          .map_err(|e| ComponentError::new(e.to_string()))?,
      ),
      None => Err(ComponentError::new(format!(
        "No value to take from output for port '{}'",
        self.name
      ))),
    }
  }
}

impl Iterator for PortOutput {
  type Item = MessageTransport;

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next()
  }
}
