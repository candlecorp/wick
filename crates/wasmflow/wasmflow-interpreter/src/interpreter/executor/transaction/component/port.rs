use wasmflow_schematic_graph::PortReference;
use wasmflow_sdk::v1::transport::{TransportMap, TransportWrapper};

mod port_buffer;
pub(crate) mod port_handler;

use self::port_handler::{BufferAction, PortHandler};
use crate::graph::types::ComponentPort;
use crate::interpreter::error::StateError;
use crate::ExecutionError;
type Result<T> = std::result::Result<T, ExecutionError>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum PortStatus {
  Open,
  DoneOpen,
  DoneYield,
  DoneClosing,
  DoneClosed,
}

impl std::fmt::Display for PortStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        PortStatus::Open => "Open",
        PortStatus::DoneOpen => "DoneOpen",
        PortStatus::DoneYield => "DoneYield",
        PortStatus::DoneClosing => "DoneClosing",
        PortStatus::DoneClosed => "DoneClosed",
      }
    )
  }
}

#[derive(Debug)]
#[must_use]
pub(crate) struct PortList {
  inner: Vec<PortHandler>,
}

impl PortList {
  pub(super) fn new(ports: Vec<ComponentPort>) -> Self {
    let ports = ports.into_iter().map(PortHandler::new).collect();
    Self { inner: ports }
  }

  pub(crate) fn refs(&self) -> impl Iterator<Item = PortReference> + '_ {
    self.inner.iter().map(|p| p.port_ref())
  }

  pub(crate) fn values(&self) -> impl Iterator<Item = &PortHandler> {
    self.inner.iter()
  }

  pub(super) fn receive(&self, port: &PortReference, value: TransportWrapper) -> Result<BufferAction> {
    self.inner[port.port_index()].buffer(value)
  }

  pub(super) fn take(&self, port: &PortReference) -> Option<TransportWrapper> {
    self.inner[port.port_index()].take()
  }

  pub(super) fn get_handler(&self, port: &PortReference) -> &PortHandler {
    &self.inner[port.port_index()]
  }

  pub(crate) fn find_ref(&self, name: &str) -> Option<PortReference> {
    for handler in &self.inner {
      if handler.name() == name {
        return Some(handler.port_ref());
      }
    }
    None
  }
}

#[derive(Debug)]
#[must_use]
pub(crate) struct OutputPorts {
  inner: PortList,
}

impl OutputPorts {
  pub(super) fn new(ports: Vec<ComponentPort>) -> Self {
    Self {
      inner: PortList::new(ports),
    }
  }

  pub(crate) fn refs(&self) -> impl Iterator<Item = PortReference> + '_ {
    self.inner.refs()
  }

  pub(super) fn receive(&self, port: &PortReference, value: TransportWrapper) -> Result<BufferAction> {
    self.inner.receive(port, value)
  }

  pub(super) fn take(&self, port: &PortReference) -> Option<TransportWrapper> {
    self.inner.take(port)
  }

  pub(crate) fn iter(&self) -> impl Iterator<Item = &PortHandler> {
    self.inner.inner.iter()
  }

  pub(super) fn get_handler(&self, port: &PortReference) -> &PortHandler {
    self.inner.get_handler(port)
  }

  pub(crate) fn find_ref(&self, name: &str) -> Option<PortReference> {
    self.inner.find_ref(name)
  }
}

#[derive(Debug)]
#[must_use]
pub(crate) struct InputPorts {
  inner: PortList,
}
impl InputPorts {
  pub(super) fn new(ports: Vec<ComponentPort>) -> Self {
    Self {
      inner: PortList::new(ports),
    }
  }

  pub(crate) fn refs(&self) -> impl Iterator<Item = PortReference> + '_ {
    self.inner.refs()
  }

  pub(super) fn receive(&self, port: &PortReference, value: TransportWrapper) -> Result<BufferAction> {
    self.inner.receive(port, value)
  }

  pub(super) fn take(&self, port: &PortReference) -> Option<TransportWrapper> {
    self.inner.take(port)
  }

  pub(crate) fn iter(&self) -> impl Iterator<Item = &PortHandler> {
    self.inner.inner.iter()
  }

  pub(super) fn get_handler(&self, port: &PortReference) -> &PortHandler {
    self.inner.get_handler(port)
  }

  pub(crate) fn find_ref(&self, name: &str) -> Option<PortReference> {
    self.inner.find_ref(name)
  }

  pub(crate) fn handlers(&self) -> impl Iterator<Item = &PortHandler> {
    self.inner.values()
  }

  pub(super) fn collect_payload(&self) -> Result<Option<TransportMap>> {
    let mut map = TransportMap::default();
    for handler in &self.inner.inner {
      if handler.is_empty() {
        return Ok(None);
      }
    }
    for handler in &self.inner.inner {
      let value = handler
        .take()
        .ok_or_else(|| ExecutionError::InvalidState(StateError::PayloadMissing(handler.name().to_owned())))?;
      map.insert(value.port, value.payload);
    }
    Ok(Some(map))
  }
}
