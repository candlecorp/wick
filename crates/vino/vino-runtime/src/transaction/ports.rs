use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use super::Result;
use crate::dev::prelude::*;

#[derive(Debug)]
pub(crate) struct PortStatuses {
  buffermap: BufferMap,
  tx_id: String,
  schematic_name: String,
  inner: RefCell<HashMap<ConnectionTargetDefinition, PortStatus>>,
  raw_ports: HashMap<String, RawPorts>,
}

impl PortStatuses {
  pub(crate) fn new<T: AsRef<str>>(tx_id: T, model: &SharedModel) -> Self {
    let raw_ports = get_incoming_ports(model);
    let readable = model.read();
    let port_statuses = readable
      .get_connections()
      .iter()
      .flat_map(|conn| {
        [
          (conn.from.clone(), PortStatus::Open),
          (conn.to.clone(), PortStatus::Waiting),
        ]
      })
      .collect();
    let schematic_name = readable.get_name();
    Self {
      buffermap: BufferMap::default(),
      inner: RefCell::new(port_statuses),
      raw_ports,
      schematic_name,
      tx_id: tx_id.as_ref().to_owned(),
    }
  }
  fn log_prefix(&self) -> String {
    format!("TX:{}({}):", self.tx_id, self.schematic_name)
  }

  pub(crate) fn update_port_status(&self, port: &ConnectionTargetDefinition, new_status: PortStatus) -> &Self {
    let prefix = self.log_prefix();
    let mut inner = self.inner.borrow_mut();
    let status = inner.get_mut(port).unwrap();
    // if the statuses are different and we aren't already closed
    if status != &new_status && status != &PortStatus::Closed {
      trace!("{}PORT_STATUS:[{}]:CHANGE:{}=>{}", prefix, port, status, new_status);
      *status = new_status;
    }
    self
  }
  pub(crate) fn set_waiting(&mut self, port: &ConnectionTargetDefinition) {
    if !self.has_data(port) {
      if self.check_port_status(port, &PortStatus::Closing) {
        self.update_port_status(port, PortStatus::Closed);
      } else {
        self.update_port_status(port, PortStatus::Waiting);
      }
    }
  }

  pub(crate) fn has_data(&self, port: &ConnectionTargetDefinition) -> bool {
    self.check_port_status(port, &PortStatus::HasData)
  }

  pub(crate) fn set_idle(&self, port: &ConnectionTargetDefinition) {
    if self.check_port_status(port, &PortStatus::Closing) {
      self.update_port_status(port, PortStatus::Closed);
    } else {
      self.update_port_status(port, PortStatus::Idle);
    }
  }

  pub(crate) fn check_port_status(&self, port: &ConnectionTargetDefinition, test: &PortStatus) -> bool {
    self.inner.borrow().get(port).map_or(false, |s| s == test)
  }

  pub(crate) fn is_closed(&self, port: &ConnectionTargetDefinition) -> bool {
    let a = self.inner.borrow();
    let status = a.get(port);
    status.map_or(true, |status| status == &PortStatus::Closed)
  }

  pub(crate) fn close(&mut self, port: &ConnectionTargetDefinition) -> &mut Self {
    if self.check_port_status(port, &PortStatus::HasData) {
      self.update_port_status(port, PortStatus::Closing);
    } else {
      self.update_port_status(port, PortStatus::Closed);
    }
    self
  }

  pub(crate) fn close_connection(&mut self, target: &ConnectionDefinition) -> &mut Self {
    self.close(&target.from);
    self.close(&target.to);
    self
  }

  pub(crate) fn get_incoming_ports(&self, instance: &str) -> Vec<ConnectionTargetDefinition> {
    self
      .raw_ports
      .get(instance)
      .map_or(vec![], |rp| rp.inputs.iter().cloned().collect())
  }

  pub(crate) fn buffer(&mut self, target: Cow<ConnectionTargetDefinition>, payload: MessageTransport) -> &mut Self {
    let ports = self.get_incoming_ports(target.get_instance());
    self.update_port_status(target.as_ref(), PortStatus::HasData);
    for p in ports.iter() {
      self.set_waiting(p);
    }

    self.buffermap.push(target, payload);

    self
  }

  pub(crate) fn take_from_port(&mut self, port: &ConnectionTargetDefinition) -> Option<MessageTransport> {
    let result = self.buffermap.take(port);
    if matches!(result, Some(MessageTransport::Signal(MessageSignal::Done))) {
      self.close(port);
    } else if !self.buffermap.has_data(port) {
      self.set_idle(port);
    };
    result
  }

  pub(crate) fn receive(&mut self, connection: &ConnectionDefinition, payload: MessageTransport) -> &Self {
    let from = &connection.from;
    let to = Cow::Borrowed(&connection.to);
    match &payload {
      MessageTransport::Success(_) => self.buffer(to, payload),
      MessageTransport::Failure(failure) => match failure {
        Failure::Invalid => self.close_connection(connection).buffer(to, payload),
        Failure::Exception(_) => self.buffer(to, payload),
        Failure::Error(_) => self.close_connection(connection).buffer(to, payload),
      },
      MessageTransport::Signal(signal) => match signal {
        MessageSignal::Done => {
          let is_schem_input = connection.from.matches_instance(SCHEMATIC_INPUT);
          let is_schem_output = connection.to.matches_instance(SCHEMATIC_OUTPUT);
          let has_no_open_inputs = !self.has_open_inputs(connection.from.get_instance());
          if is_schem_input || has_no_open_inputs {
            if is_schem_output {
              // If it's the schematic output, send then signal onward for the consumer.
              self.close_connection(connection).buffer(to, payload)
            } else {
              // Otherwise we're the intended consumer so eat the signal and move on.
              self.close_connection(connection)
            }
          } else {
            if is_schem_output {
              // If it's the schematic output, send then signal onward for the consumer.
              self.close_connection(connection).buffer(to, payload);
            } else {
              // Otherwise the port sits idle until it receives something.
              self.set_idle(from);
            }
            self
          }
        }
        MessageSignal::OpenBracket => panic!("Not implemented"),
        MessageSignal::CloseBracket => panic!("Not implemented"),
      },
    }
  }

  pub(crate) fn take_inputs(&mut self, target: &ConnectionTargetDefinition) -> Result<TransportMap> {
    let ports = self.get_incoming_ports(target.get_instance());

    let mut map = HashMap::new();
    for port in ports {
      let message = self.take_from_port(&port).ok_or(InternalError::E9005)?;
      map.insert(port.get_port_owned(), message);
    }
    Ok(TransportMap::from_map(map))
  }

  pub(crate) fn is_target_ready(&self, port: &ConnectionTargetDefinition) -> bool {
    let instance = port.get_instance();
    let ports = self.get_incoming_ports(instance);
    all(ports, |port| self.is_port_ready(&port))
  }

  pub(crate) fn has_open_inputs(&self, instance: &str) -> bool {
    let ports = self.get_incoming_ports(instance);
    if ports.is_empty() {
      false
    } else {
      all(ports, |port| !self.check_port_status(&port, &PortStatus::Closed))
    }
  }

  pub(crate) fn is_port_ready(&self, port: &ConnectionTargetDefinition) -> bool {
    self.buffermap.has_data(port)
  }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PortStatus {
  Open,
  Idle,
  HasData,
  Waiting,
  Closed,
  Closing,
}

impl std::fmt::Display for PortStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        // All input ports start off as Open.
        PortStatus::Open => "Open",
        // Ports that have already served a purpose fall into an idle state and can be closed.
        PortStatus::Idle => "Idle",
        // Any input port that is sitting on buffered data is in a HasData state.
        PortStatus::HasData => "HasData",
        // Schematic output ports start as "Waiting" and input ports flip to waiting once any input is received.
        PortStatus::Waiting => "Waiting",
        // Ports get closed when their upstreams close and they never reopen.
        PortStatus::Closed => "Closed",
        // Closed ports with data are in a Closing state.
        PortStatus::Closing => "Closing",
      }
    )
  }
}

#[derive(Debug, Default)]
struct BufferMap {
  map: HashMap<ConnectionTargetDefinition, PortBuffer>,
}

impl BufferMap {
  fn push(&mut self, port: Cow<ConnectionTargetDefinition>, payload: MessageTransport) {
    let queue = if self.map.contains_key(port.as_ref()) {
      self.map.get_mut(port.as_ref()).unwrap()
    } else {
      self.map.entry(port.into_owned()).or_insert_with(PortBuffer::default)
    };
    queue.push_back(payload);
  }

  fn has_data(&self, port: &ConnectionTargetDefinition) -> bool {
    self.map.get(port).map_or(false, PortBuffer::has_data)
  }
  fn take(&mut self, port: &ConnectionTargetDefinition) -> Option<MessageTransport> {
    self.map.get_mut(port).and_then(PortBuffer::pop_front)
  }
}

#[derive(Debug, Default)]
struct PortBuffer {
  buffer: VecDeque<MessageTransport>,
}

impl PortBuffer {
  fn push_back(&mut self, payload: MessageTransport) {
    self.buffer.push_back(payload);
  }
  fn has_data(&self) -> bool {
    !self.buffer.is_empty()
  }

  fn pop_front(&mut self) -> Option<MessageTransport> {
    self.buffer.pop_front()
  }
}
