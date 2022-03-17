use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use super::ports::PortStatus;
use super::Result;
use crate::dev::prelude::*;

pub(crate) enum ConnectionEvent<'a> {
  Data(&'a ConnectionDefinition, MessageTransport),
  Take(&'a ConnectionDefinition),
}

#[derive(Debug)]
pub(crate) struct ActiveConnections {
  buffermap: BufferMap,
  tx_id: String,
  schematic_name: String,
  inner: RefCell<HashMap<ConnectionDefinition, ConnectionStatus>>,
}

impl ActiveConnections {
  pub(crate) fn new<T: AsRef<str>>(tx_id: T, model: &SharedModel) -> Self {
    let readable = model.read();

    let connections = readable.get_connections().clone();
    let schematic_name = readable.get_name();
    Self {
      inner: RefCell::new(
        connections
          .into_iter()
          .map(|conn| (conn, ConnectionStatus::new()))
          .collect(),
      ),
      tx_id: tx_id.as_ref().to_owned(),
      schematic_name,
      buffermap: BufferMap::default(),
    }
  }

  pub(crate) fn is_waiting(&self, connection: &ConnectionDefinition) -> bool {
    self
      .inner
      .borrow()
      .get(connection)
      .map_or(false, |conn| conn.is_waiting())
  }

  pub(crate) fn is_done(&self) -> bool {
    for (connection, status) in self.inner.borrow().iter() {
      if connection.to.matches_instance(SCHEMATIC_OUTPUT) && status.state != ConnectionState::Closed {
        return false;
      }
    }
    true
  }

  fn log_prefix(&self) -> String {
    format!("TX:{}({}):", self.tx_id, self.schematic_name)
  }

  pub(crate) fn buffer(&self, connection: &ConnectionDefinition, payload: MessageTransport) -> &Self {
    // let ports = self.get_upstream_connections(connection.to.get_instance());
    self.buffermap.push(connection, payload);

    self
  }

  pub(crate) fn get_upstream_connections(&self, instance: &str) -> Vec<ConnectionDefinition> {
    let mut connections = Vec::new();
    for conn in self.inner.borrow().keys() {
      if conn.to.matches_instance(instance) {
        connections.push(conn.clone());
      }
    }
    connections
  }

  pub(crate) fn has_open_connections(&self, instance: &str) -> bool {
    let connections = self.get_upstream_connections(instance);
    if connections.is_empty() {
      false
    } else {
      all(connections, |conn| self.upstream_status(&conn) != PortStatus::Closed)
    }
  }

  pub(crate) fn upstream_status(&self, connection: &ConnectionDefinition) -> PortStatus {
    self
      .inner
      .borrow()
      .get(connection)
      .map_or(PortStatus::Invalid, |c| c.upstream_status())
  }

  pub(crate) fn has_data(&self, connection: &ConnectionDefinition) -> bool {
    self.buffermap.has_data(connection)
  }

  pub(crate) fn is_target_ready(&self, connection: &ConnectionDefinition) -> bool {
    let instance = connection.to.get_instance();
    let connections = self.get_upstream_connections(instance);
    all(connections, |conn| self.is_connection_ready(&conn))
  }

  pub(crate) fn is_connection_ready(&self, connection: &ConnectionDefinition) -> bool {
    self.buffermap.has_data(connection)
  }

  pub(crate) fn dispatch<'a>(&'a self, event: ConnectionEvent<'a>) -> &Self {
    match event {
      ConnectionEvent::Data(connection, msg) => {
        match msg {
          MessageTransport::Success(_) => {
            self.buffer(connection, msg);
            self.transition_state(connection, &ConnectionState::HasData);
            let connections = self.get_upstream_connections(connection.to.get_instance());
            if connection.to.matches_instance(SCHEMATIC_OUTPUT) {
            } else {
              for conn in connections {
                if &conn != connection {
                  self.transition_state(&conn, &ConnectionState::Waiting);
                }
              }
            }
          }
          MessageTransport::Failure(ref failure) => match failure {
            Failure::Invalid => {
              self.buffer(connection, msg);
              self.transition_state(connection, &ConnectionState::Error);
            }
            Failure::Exception(_) => {
              self.buffer(connection, msg);
              self.transition_state(connection, &ConnectionState::HasData);
            }
            Failure::Error(_) => {
              self.buffer(connection, msg);
              self.transition_state(connection, &ConnectionState::HasData);
            }
          },
          MessageTransport::Signal(signal) => match signal {
            MessageSignal::Done => {
              // Propagate "Done" messages outside a schematic
              let has_open_inputs = self.has_open_connections(connection.from.get_instance());
              if has_open_inputs {
                if connection.to.matches_instance(SCHEMATIC_OUTPUT) {
                  self.buffer(connection, msg);
                  self.transition_state(connection, &ConnectionState::Closed);
                } else {
                  self.transition_state(connection, &ConnectionState::Idle);
                }
              } else {
                self.transition_state(connection, &ConnectionState::Closed);
              }
            }
            MessageSignal::OpenBracket => unimplemented!(),
            MessageSignal::CloseBracket => unimplemented!(),
          },
        };
      }
      ConnectionEvent::Take(connection) => {
        if self.has_data(connection) {
          self.transition_state(connection, &ConnectionState::HasData);
        } else {
          self.transition_state(connection, &ConnectionState::Idle);
        }
      }
    }
    self
  }

  pub(crate) fn take<'a, 'b>(&'a self, connection: &'b ConnectionDefinition) -> Option<MessageTransport>
  where
    'b: 'a,
  {
    let result = self.buffermap.take(connection);
    self.dispatch(ConnectionEvent::Take(connection));
    result
  }

  pub(crate) fn take_inputs<'a, 'b>(&'a mut self, target: &'b ConnectionDefinition) -> Result<TransportMap>
  where
    'b: 'a,
  {
    let connections = self.get_upstream_connections(target.to.get_instance());

    let mut map = HashMap::new();
    for connection in connections {
      let message = self.take(&connection).ok_or(InternalError::E9005)?;
      map.insert(connection.to.get_port_owned(), message);
    }
    Ok(TransportMap::from_map(map))
  }

  #[allow(clippy::too_many_lines)]
  pub(crate) fn transition_state<'a, 'b>(
    &'a self,
    conn: &'b ConnectionDefinition,
    new_state: &ConnectionState,
  ) -> &Self {
    let has_data = self.has_data(conn);

    let mut inner = self.inner.borrow_mut();
    let current_state = &mut inner.get_mut(conn).unwrap().state;

    let invalid_state = || {
      trace!(
        "{}PORT[{}]:TRANSITION:{}=>{}",
        self.log_prefix(),
        conn,
        current_state,
        new_state
      );
      panic!("Invalid state transition {}=>{}", current_state, new_state);
    };

    if new_state == current_state {
      return self;
    }

    let actual_state = match new_state {
      ConnectionState::Idle => match current_state {
        ConnectionState::Closing => {
          if !has_data {
            ConnectionState::Closed
          } else {
            ConnectionState::Closing
          }
        }
        ConnectionState::HasData => ConnectionState::Idle,
        ConnectionState::Waiting => {
          if has_data {
            ConnectionState::Idle
          } else {
            ConnectionState::Waiting
          }
        }
        _ => invalid_state(),
      },
      ConnectionState::Waiting => match current_state {
        ConnectionState::HasData => ConnectionState::HasData,
        ConnectionState::Idle => ConnectionState::Waiting,
        ConnectionState::Closing => {
          if !has_data {
            ConnectionState::Closed
          } else {
            ConnectionState::Closing
          }
        }
        _ => invalid_state(),
      },
      ConnectionState::Closed => match current_state {
        ConnectionState::Idle | ConnectionState::HasData => {
          if has_data {
            ConnectionState::Closing
          } else {
            ConnectionState::Closed
          }
        }
        _ => invalid_state(),
      },
      ConnectionState::Closing => invalid_state(),

      ConnectionState::HasData => match current_state {
        ConnectionState::Idle => ConnectionState::HasData,
        ConnectionState::Waiting => ConnectionState::HasData,
        ConnectionState::HasData => ConnectionState::HasData,
        ConnectionState::Closed => invalid_state(),
        ConnectionState::Error => ConnectionState::Error,
        ConnectionState::Closing => ConnectionState::Closing,
      },
      ConnectionState::Error => ConnectionState::Error,
    };

    if new_state != &actual_state {
      trace!(
        "{}PORT[{}]:TRANSITION:{}=>{}[WANTED={}]",
        self.log_prefix(),
        conn,
        current_state,
        actual_state,
        new_state
      );
    } else {
      trace!(
        "{}PORT[{}]:TRANSITION:{}=>{}",
        self.log_prefix(),
        conn,
        current_state,
        actual_state
      );
    }
    *current_state = actual_state;
    self
  }
}

#[derive(Debug, Clone)]
pub(crate) struct ConnectionStatus {
  state: ConnectionState,
  from_status: PortStatus,
}

impl ConnectionStatus {
  pub(crate) fn new() -> Self {
    Self {
      state: ConnectionState::Idle,
      from_status: PortStatus::Idle,
    }
  }
  pub(crate) fn upstream_status(&self) -> PortStatus {
    self.from_status.clone()
  }

  pub(crate) fn is_waiting(&self) -> bool {
    self.state == ConnectionState::Waiting
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConnectionState {
  // Ports that have already served a purpose fall into an idle state and can be closed.
  Idle,
  // Schematic output ports start as "Waiting" and input ports flip to waiting
  // once sibling input is received.
  Waiting,
  // A port that has data.
  HasData,
  // Ports get closed when their upstreams close and they never reopen.
  Closed,
  // Ports that generated an error.
  Error,
  // Closed ports with data are in a Closing state.
  Closing,
}

impl std::fmt::Display for ConnectionState {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Idle => "Idle",
        Self::Waiting => "Waiting",
        Self::HasData => "HasData",
        Self::Error => "Error",
        Self::Closed => "Closed",
        Self::Closing => "Closing",
      }
    )
  }
}

#[derive(Debug, Default)]
struct BufferMap {
  map: RefCell<HashMap<ConnectionDefinition, PortBuffer>>,
}

impl BufferMap {
  fn push(&self, connection: &ConnectionDefinition, payload: MessageTransport) {
    let mut map = self.map.borrow_mut();
    let queue = if map.contains_key(connection) {
      map.get_mut(connection).unwrap()
    } else {
      map.entry(connection.clone()).or_insert_with(PortBuffer::default)
    };
    queue.push_back(payload);
  }

  fn has_data(&self, connection: &ConnectionDefinition) -> bool {
    self.map.borrow().get(connection).map_or(false, PortBuffer::has_data)
  }
  fn take(&self, connection: &ConnectionDefinition) -> Option<MessageTransport> {
    self
      .map
      .borrow_mut()
      .get_mut(connection)
      .and_then(PortBuffer::pop_front)
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
