use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::Mutex;
use uuid::Uuid;
use vino_schematic_graph::iterators::{SchematicHop, WalkDirection};
use vino_schematic_graph::{ComponentIndex, PortReference, Schematic};
use vino_transport::{TransportMap, TransportStream, TransportWrapper};

use super::buffer::PacketBuffer;
use super::component::ComponentHandler;
use super::error::ExecutionError;
use crate::interpreter::channel::{InterpreterEvent, PortData};
use crate::interpreter::error::{missing_port, StateError};
use crate::InterpreterDispatchChannel;

pub(crate) mod statistics;
pub use statistics::TransactionStatistics;

type Result<T> = std::result::Result<T, ExecutionError>;

#[derive(Debug, PartialEq, Eq)]
enum TransactionStatus {
  New,
}

#[derive()]
#[must_use]
pub(crate) struct Transaction {
  schematic: Arc<Schematic>,
  output: PacketBuffer,
  state: TransactionStatus,
  channel: InterpreterDispatchChannel,
  inputs: Option<TransportMap>,
  components: HashMap<ComponentIndex, ComponentHandler>,
  port_status: Mutex<HashMap<PortReference, PortStatus>>,
  id: Uuid,
  start_time: Instant,
  pub(crate) stats: TransactionStatistics,
}

impl std::fmt::Debug for Transaction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Transaction")
      .field("state", &self.state)
      .field("id", &self.id)
      .finish()
  }
}

impl Transaction {
  pub(crate) fn new(
    id: Uuid,
    graph: Arc<Schematic>,
    inputs: TransportMap,

    channel: InterpreterDispatchChannel,
  ) -> Self {
    let components = graph
      .components()
      .iter()
      .map(|component| (component.index(), ComponentHandler::new(&graph, component)))
      .collect();
    let port_status = graph
      .get_ports()
      .into_iter()
      .map(|port| (port, PortStatus::Open))
      .collect();
    let stats = TransactionStatistics::new(id);
    stats.mark("new");
    Self {
      state: TransactionStatus::New,
      channel,
      inputs: Some(inputs),
      schematic: graph,
      output: PacketBuffer::default(),
      components,
      port_status: Mutex::new(port_status),
      start_time: Instant::now(),
      stats,
      id,
    }
  }

  pub(crate) async fn emit_data(&self, data: TransportWrapper) -> Result<()> {
    self.output.push(data).await?;
    Ok(())
  }

  pub(crate) fn id(&self) -> Uuid {
    self.id
  }

  pub(crate) fn get_stream(&mut self) -> Option<TransportStream> {
    self.output.detach().map(|rx| TransportStream::new(rx.into_stream()))
  }

  pub(crate) fn get_graph(&self) -> Arc<Schematic> {
    self.schematic.clone()
  }

  pub(crate) fn get_handler(&self, index: ComponentIndex) -> Result<&ComponentHandler> {
    self
      .components
      .get(&index)
      .ok_or(ExecutionError::InvalidState(StateError::MissingComponent(index)))
  }

  pub(crate) async fn start(&mut self) -> Result<()> {
    self.stats.start("execution");
    let span = trace_span!("transaction", id = self.id.to_string().as_str());
    let _guard = span.enter();
    trace!("starting transaction");
    self.start_time = Instant::now();
    let inputs = if self.state == TransactionStatus::New {
      self.inputs.take().unwrap()
    } else {
      return Err(ExecutionError::AlreadyStarted);
    };

    let input_component = self.schematic.input();

    let input = self.get_handler(input_component.index())?;
    input.validate_payload(&inputs)?;

    for transport in inputs {
      let port = *input.find_input(&transport.port).unwrap();
      trace!(
        "priming port '{}' (component:{},port:{})",
        transport.port,
        port.component_index(),
        port.port_index()
      );
      let done_message = TransportWrapper::done(&transport.port);
      self
        .channel
        .dispatch(InterpreterEvent::PortData(PortData::new(self.id, port, transport)))
        .await?;
      self
        .channel
        .dispatch(InterpreterEvent::PortData(PortData::new(self.id, port, done_message)))
        .await?;
    }

    trace!("transaction started");
    Ok(())
  }

  pub(crate) fn finish(mut self) -> Result<TransactionStatistics> {
    self.stats.end("execution");
    self.stats.print();

    Ok(self.stats)
  }

  #[instrument(skip_all, name="port_status", fields(port= ?port.as_ref()))]
  pub(crate) fn set_port_done<T: AsRef<PortReference>>(&self, port: T) -> Result<()> {
    if self.upstreams_closed(&port)? {
      debug!("setting to done_closed");
      self.port_status.lock().insert(*port.as_ref(), PortStatus::DoneClosed);
    } else {
      debug!("setting to done_open");
      self.port_status.lock().insert(*port.as_ref(), PortStatus::DoneOpen);
    }
    Ok(())
  }

  #[instrument( skip_all, fields(port= ?port.as_ref()))]
  pub(crate) fn upstreams_closed<T: AsRef<PortReference>>(&self, port: T) -> Result<bool> {
    let port = port.as_ref();

    for hop in self.schematic.walk_from_port(*port, WalkDirection::Up).skip(1) {
      match hop {
        SchematicHop::Port(port) => {
          if !self.is_done_closed(port.as_ref())? {
            trace!("has open upstream");
            return Ok(false);
          }
        }
        _ => continue,
      }
    }
    trace!("has no open upstreams");

    Ok(true)
  }

  pub(crate) fn is_done_closed<T: AsRef<PortReference>>(&self, port: T) -> Result<bool> {
    let lock = self.port_status.lock();
    let status = lock.get(port.as_ref()).ok_or_else(|| missing_port(port.as_ref()))?;
    Ok(status == &PortStatus::DoneClosed)
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum PortStatus {
  Open,
  DoneOpen,
  DoneClosed,
}
