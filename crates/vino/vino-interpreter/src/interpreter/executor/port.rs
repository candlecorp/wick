use std::collections::HashMap;

use vino_schematic_graph::{PortReference, Schematic};
use vino_transport::{TransportMap, TransportWrapper};

use super::buffer::PacketBuffer;
use super::error::ExecutionError;
use crate::interpreter::error::StateError;
type Result<T> = std::result::Result<T, ExecutionError>;

#[derive(Debug)]
#[must_use]
pub(super) struct PortList {
  inner: HashMap<PortReference, PortHandler>,
}

impl PortList {
  pub(super) fn new(schematic: &Schematic, ports: &[PortReference]) -> Self {
    Self {
      inner: ports
        .iter()
        .map(|port| (*port, PortHandler::new(schematic, *port)))
        .collect(),
    }
  }

  #[allow(unused)]
  pub(super) fn len(&self) -> usize {
    self.inner.len()
  }

  pub(super) fn iter(&self) -> impl Iterator<Item = (&PortReference, &PortHandler)> {
    self.inner.iter()
  }
}

#[derive(Debug)]
#[must_use]
pub(crate) struct PortHandler {
  name: String,
  buffer: PacketBuffer,
}

impl PortHandler {
  pub(super) fn new(schematic: &Schematic, port: PortReference) -> Self {
    let component_port = schematic.get_port(&port);

    Self {
      buffer: Default::default(),
      name: component_port.name().to_owned(),
    }
  }

  pub(super) fn name(&self) -> &str {
    &self.name
  }

  pub(super) async fn accept(&self, value: TransportWrapper) -> Result<()> {
    self.buffer.push(value).await?;
    Ok(())
  }

  pub(super) async fn shift(&self) -> Result<TransportWrapper> {
    self
      .buffer
      .receive()
      .await?
      .ok_or_else(|| ExecutionError::InvalidState(StateError::PayloadMissing(self.name().to_owned())))
  }
}

#[derive(Debug)]
#[must_use]
pub(super) struct OutputPorts {
  inner: PortList,
}

impl OutputPorts {
  pub(super) fn new(ports: PortList) -> Self {
    Self { inner: ports }
  }

  pub(super) fn iter(&self) -> impl Iterator<Item = (&PortReference, &PortHandler)> {
    self.inner.iter()
  }
}

impl Ports for OutputPorts {
  fn inner(&self) -> &PortList {
    &self.inner
  }
  fn inner_mut(&mut self) -> &mut PortList {
    &mut self.inner
  }
}
#[derive(Debug)]
#[must_use]
pub(super) struct InputPorts {
  inner: PortList,
}
impl InputPorts {
  pub(super) fn new(ports: PortList) -> Self {
    Self { inner: ports }
  }

  pub(super) async fn receive(&self, port: &PortReference, value: TransportWrapper) -> Result<()> {
    match self.inner.inner.get(port) {
      Some(port) => port.accept(value).await,
      None => Err(StateError::MissingPort(*port).into()),
    }
  }

  pub(super) fn iter(&self) -> impl Iterator<Item = (&PortReference, &PortHandler)> {
    self.inner.iter()
  }

  pub(super) fn is_ready(&self) -> bool {
    trace!("is ready?");
    for port in self.inner.inner.values() {
      trace!("port '{:?}' has_data: {:?}", port, port.buffer.has_data());
      if !port.buffer.has_data().unwrap() {
        return false;
      }
    }
    true
  }

  pub(super) async fn shift(&self) -> Result<TransportMap> {
    let mut map = TransportMap::default();
    for (_, handler) in self.inner.iter() {
      let value = handler.shift().await?;
      map.insert(value.port, value.payload);
    }
    Ok(map)
  }
}

impl Ports for InputPorts {
  fn inner(&self) -> &PortList {
    &self.inner
  }
  fn inner_mut(&mut self) -> &mut PortList {
    &mut self.inner
  }
}

pub(super) trait Ports {
  fn inner(&self) -> &PortList;
  fn inner_mut(&mut self) -> &mut PortList;

  fn len(&self) -> usize {
    self.inner().len()
  }

  fn find(&self, name: &str) -> Option<&PortReference> {
    for (port, handler) in &self.inner().inner {
      if handler.name == name {
        return Some(port);
      }
    }
    None
  }
}
