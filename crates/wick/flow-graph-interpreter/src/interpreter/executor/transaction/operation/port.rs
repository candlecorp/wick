use flow_graph::PortReference;
use wick_packet::Packet;

mod port_buffer;
pub(crate) mod port_handler;

use self::port_handler::PortHandler;
use crate::graph::types::OperationPort;
use crate::interpreter::executor::error::ExecutionError;
type Result<T> = std::result::Result<T, ExecutionError>;
type PacketType = Packet;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum PortStatus {
  Open,
  DoneClosing,
  DoneClosed,
  UpstreamComplete,
}

impl PortStatus {
  #[allow(unused)]
  #[must_use]
  pub(crate) fn is_open(self) -> bool {
    !matches!(self, PortStatus::DoneClosed)
  }
  #[allow(unused)]
  #[must_use]
  pub(crate) fn is_closed(self) -> bool {
    matches!(self, PortStatus::DoneClosed)
  }

  #[allow(unused)]
  #[must_use]
  pub(crate) fn is_done(self) -> bool {
    matches!(self, PortStatus::DoneClosed | PortStatus::DoneClosing)
  }
}

impl std::fmt::Display for PortStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        PortStatus::Open => "Open",
        PortStatus::DoneClosing => "DoneClosing",
        PortStatus::DoneClosed => "DoneClosed",
        PortStatus::UpstreamComplete => "UpstreamComplete",
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
  pub(super) fn new(operation_instance: impl AsRef<str>, ports: Vec<OperationPort>) -> Self {
    let ports = ports
      .into_iter()
      .map(|p| PortHandler::new(operation_instance.as_ref(), p))
      .collect();
    Self { inner: ports }
  }

  pub(crate) fn refs(&self) -> impl Iterator<Item = PortReference> + '_ {
    self.inner.iter().map(|p| p.port_ref())
  }

  #[allow(unused)]
  pub(crate) fn len(&self) -> usize {
    self.inner.len()
  }

  pub(crate) fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  pub(super) fn receive(&self, port: &PortReference, value: PacketType) {
    self.inner[port.port_index()].buffer(value);
  }

  pub(super) fn take(&self, port: &PortReference) -> Option<PacketType> {
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
  pub(super) fn new(operation_instance: impl AsRef<str>, ports: Vec<OperationPort>) -> Self {
    Self {
      inner: PortList::new(operation_instance, ports),
    }
  }

  pub(crate) fn refs(&self) -> impl Iterator<Item = PortReference> + '_ {
    self.inner.refs()
  }

  pub(crate) fn len(&self) -> usize {
    self.inner.inner.len()
  }

  pub(super) fn receive(&self, port: &PortReference, value: PacketType) {
    self.inner.receive(port, value);
  }

  pub(super) fn take(&self, port: &PortReference) -> Option<PacketType> {
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
  pub(super) fn new(operation_instance: impl AsRef<str>, ports: Vec<OperationPort>) -> Self {
    Self {
      inner: PortList::new(operation_instance, ports),
    }
  }

  pub(super) fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  pub(super) fn receive(&self, port: &PortReference, value: PacketType) {
    self.inner.receive(port, value);
  }

  pub(super) fn take(&self, port: &PortReference) -> Option<PacketType> {
    self.inner.take(port)
  }

  pub(crate) fn iter(&self) -> impl Iterator<Item = &PortHandler> {
    self.inner.inner.iter()
  }

  #[allow(unused)]
  pub(crate) fn len(&self) -> usize {
    self.inner.inner.len()
  }

  pub(super) fn get_handler(&self, port: &PortReference) -> &PortHandler {
    self.inner.get_handler(port)
  }

  pub(crate) fn find_ref(&self, name: &str) -> Option<PortReference> {
    self.inner.find_ref(name)
  }

  pub(super) fn take_packets(&self) -> Result<Vec<Packet>> {
    let mut vec = Vec::new();

    for handler in &self.inner.inner {
      let mut drain = handler.drain(0..);
      vec.append(&mut drain);
    }
    Ok(vec)
  }
}
