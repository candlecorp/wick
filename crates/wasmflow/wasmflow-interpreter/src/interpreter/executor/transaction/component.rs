use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio_stream::StreamExt;
use tracing_futures::Instrument;
use uuid::Uuid;
use wasmflow_schematic_graph::{ComponentIndex, PortDirection, PortReference};
use wasmflow_transport::{MessageSignal, MessageTransport, TransportMap, TransportStream, TransportWrapper};
use wasmflow_entity::Entity;
use wasmflow_invocation::Invocation;

use self::port::port_handler::{BufferAction, PortHandler};
use self::port::{InputPorts, OutputPorts, PortStatus};
use crate::constants::*;
use crate::graph::types::*;
use crate::graph::Reference;
use crate::interpreter::channel::Event;
use crate::interpreter::error::StateError;
use crate::{ExecutionError, HandlerMap, InterpreterDispatchChannel, Provider};
type Result<T> = std::result::Result<T, ExecutionError>;

pub(crate) mod port;

#[derive()]
#[must_use]
pub(crate) struct InstanceHandler {
  reference: Reference,
  identifier: String,
  index: ComponentIndex,
  inputs: InputPorts,
  outputs: OutputPorts,
  schematic: Arc<Schematic>,
  pending: AtomicU32,
  providers: Arc<HandlerMap>,
  self_provider: Arc<dyn Provider + Send + Sync>,
}

impl std::fmt::Debug for InstanceHandler {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("InstanceHandler")
      .field("reference", &self.reference)
      .field("identifier", &self.identifier)
      .field("index", &self.index)
      .field("inputs", &self.inputs)
      .field("outputs", &self.outputs)
      .finish()
  }
}

impl InstanceHandler {
  pub(super) fn new(
    schematic: Arc<Schematic>,
    component: &Component,
    providers: Arc<HandlerMap>,
    self_provider: Arc<dyn Provider + Send + Sync>,
  ) -> Self {
    let inputs = component.inputs().to_vec();
    let outputs = component.outputs().to_vec();
    let reference = component.kind().cref().into();

    Self {
      schematic,
      inputs: InputPorts::new(inputs),
      outputs: OutputPorts::new(outputs),
      reference,
      index: component.index(),
      identifier: component.id().to_owned(),
      providers,
      pending: AtomicU32::new(0),
      self_provider,
    }
  }

  pub(crate) fn entity(&self) -> Entity {
    Entity::component(self.reference.namespace(), &self.reference.name())
  }

  pub(crate) fn namespace(&self) -> &str {
    self.reference.namespace()
  }

  pub(crate) fn index(&self) -> ComponentIndex {
    self.index
  }

  pub(crate) fn id(&self) -> &str {
    &self.identifier
  }

  pub(crate) fn is_core_component(&self, name: &str) -> bool {
    self.reference.is_core_component(name)
  }

  pub(crate) fn is_schematic_output(&self) -> bool {
    self.reference.is_schematic_output()
  }

  pub(crate) fn is_static(&self) -> bool {
    self.reference.is_static()
  }

  pub(crate) fn done(&self) -> bool {
    for port in self.inputs.iter() {
      if port.status() != PortStatus::DoneClosed {
        return false;
      }
    }
    true
  }

  pub(super) fn take_payload(&self) -> Result<Option<TransportMap>> {
    self.inputs.collect_payload()
  }

  pub(super) fn take_output(&self, port: &PortReference) -> Option<TransportWrapper> {
    self.outputs.take(port)
  }

  pub(super) fn take_input(&self, port: &PortReference) -> Option<TransportWrapper> {
    self.inputs.take(port)
  }

  pub(crate) fn validate_payload(&self, payload: &TransportMap) -> Result<()> {
    for input in self.inputs.iter() {
      if !payload.contains_key(input.name()) {
        return Err(ExecutionError::MissingInput(input.name().to_owned()));
      }
    }

    Ok(())
  }

  pub(crate) fn update_input_status<'a>(
    &'_ self,
    port: &'a PortReference,
    instances: &'_ [Arc<InstanceHandler>],
  ) -> Option<&'a PortReference> {
    let current_status = self.get_port_status(port);

    let component = &self.schematic.components()[port.component_index()];
    let component_port = &component.inputs()[port.port_index()];

    let upstream_ports = component_port.connections().iter().map(|connection| {
      let connection = &self.schematic.connections()[*connection];
      let upstream_instance = &instances[connection.from().component_index()];
      upstream_instance.outputs.get_handler(connection.from())
    });

    let breakdown = check_statuses(upstream_ports);

    let num_buffered = self.buffered_packets(port);
    trace!(count = self.num_pending(), "pending executions");
    let new_status = if breakdown.has_open || breakdown.has_done_open {
      PortStatus::DoneOpen
    } else if breakdown.has_any_generator && !self.is_schematic_output() {
      PortStatus::DoneYield
    } else if num_buffered > 0 {
      PortStatus::DoneClosing
    } else {
      PortStatus::DoneClosed
    };

    if new_status != current_status {
      self.inputs().get_handler(port).set_status(new_status);
      Some(port)
    } else {
      None
    }
  }

  pub(crate) fn buffer_in(
    &self,
    port: &PortReference,
    value: TransportWrapper,
    instances: &[Arc<InstanceHandler>],
  ) -> Result<BufferAction> {
    trace!(%port, ?value, "buffering input message");

    if value.payload == MessageTransport::Signal(MessageSignal::Done) {
      self.update_input_status(port, instances);
      Ok(BufferAction::Consumed)
    } else {
      self.inputs.receive(port, value)
    }
  }

  pub(crate) fn buffer_out(&self, port: &PortReference, value: TransportWrapper) -> Result<BufferAction> {
    trace!(%port, ?value, "buffering output message");
    if value.payload == MessageTransport::Signal(MessageSignal::Done) {
      let breakdown = check_statuses(self.inputs.iter());

      trace!(count = self.num_pending(), "pending executions");
      let new_status = if self.is_static() || breakdown.has_all_generators && !self.is_schematic_output() {
        PortStatus::DoneYield
      } else if breakdown.has_any_open() || self.is_pending() {
        PortStatus::DoneOpen
      } else if self.buffered_packets(port) > 0 {
        PortStatus::DoneClosing
      } else {
        PortStatus::DoneClosed
      };
      self.outputs().get_handler(port).set_status(new_status);
    }
    self.outputs.receive(port, value)
  }

  pub(crate) fn find_input(&self, name: &str) -> Result<PortReference> {
    self
      .inputs
      .find_ref(name)
      .ok_or_else(|| StateError::MissingPortName(name.to_owned()).into())
  }

  pub(crate) fn find_output(&self, name: &str) -> Result<PortReference> {
    self
      .outputs
      .find_ref(name)
      .ok_or_else(|| StateError::MissingPortName(name.to_owned()).into())
  }

  pub(crate) fn outputs(&self) -> &OutputPorts {
    &self.outputs
  }

  pub(crate) fn inputs(&self) -> &InputPorts {
    &self.inputs
  }

  pub(crate) fn get_port_status(&self, port: &PortReference) -> PortStatus {
    match port.direction() {
      PortDirection::In => self.inputs.get_handler(port).status(),
      PortDirection::Out => self.outputs.get_handler(port).status(),
    }
  }

  pub(crate) fn is_port_empty(&self, port: &PortReference) -> bool {
    match port.direction() {
      PortDirection::In => self.inputs.get_handler(port).is_empty(),
      PortDirection::Out => self.outputs.get_handler(port).is_empty(),
    }
  }

  pub(crate) fn buffered_packets(&self, port: &PortReference) -> usize {
    match port.direction() {
      PortDirection::In => self.inputs.get_handler(port).len(),
      PortDirection::Out => self.outputs.get_handler(port).len(),
    }
  }

  pub(crate) fn increment_pending(&self) {
    self.pending.fetch_add(1, Ordering::Acquire);
  }

  pub(crate) fn decrement_pending(&self) -> Result<()> {
    self
      .pending
      .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
        if v > 0 {
          Some(v - 1)
        } else {
          None
        }
      })
      .map_err(|_| StateError::TooManyComplete)?;
    Ok(())
  }

  pub(crate) fn is_pending(&self) -> bool {
    let num = self.pending.load(Ordering::Relaxed);
    num > 0
  }

  pub(crate) fn handle_call_complete(&self, status: CompletionStatus) -> Result<Vec<PortReference>> {
    self.decrement_pending()?;
    Ok(self.update_output_statuses(status))
  }

  pub(super) fn update_output_statuses(&self, _status: CompletionStatus) -> Vec<PortReference> {
    let breakdown = check_statuses(self.inputs.handlers());
    let mut changed_statuses = Vec::new();
    for port in self.outputs.iter() {
      let current_status = port.status();
      let new_status = match current_status {
        PortStatus::DoneOpen | PortStatus::Open => {
          // Leave the port DoneOpen if we have any open inputs or we're still pending
          // otherwise close it.
          // Note: A port can still be "Open" after a call if the component panics.
          // Treat it the same way as DoneOpen
          if breakdown.has_any_open()
            || self.is_pending()
            || breakdown.has_pending_packets
            || breakdown.has_all_generators
          {
            PortStatus::DoneOpen
          } else {
            PortStatus::DoneClosed
          }
        }
        orig => orig,
      };

      if new_status != current_status {
        changed_statuses.push(port.port_ref());
        port.set_status(new_status);
      }
    }
    trace!(ports=?changed_statuses,"updated downstream ports");
    changed_statuses
  }

  pub(crate) async fn dispatch_invocation(
    self: Arc<Self>,
    _tx_id: Uuid,
    invocation: Invocation,
    channel: InterpreterDispatchChannel,
  ) -> Result<()> {
    channel.dispatch(Event::invocation(self.index(), invocation)).await?;
    Ok(())
  }

  pub(crate) async fn invoke(
    self: Arc<Self>,
    tx_id: Uuid,
    invocation: Invocation,
    channel: InterpreterDispatchChannel,
  ) -> Result<()> {
    debug!(?invocation, "invoking");
    let invocation_id = invocation.id;

    let identifier = self.id().to_owned();
    let entity = self.entity();
    let namespace = self.namespace().to_owned();

    let associated_data = self.schematic.components()[self.index()].data();

    let associated_data = associated_data.clone();

    self.increment_pending();

    let fut = if namespace == NS_SELF {
      let clone = self.self_provider.clone();
      tokio::spawn(async move {
        clone
          .handle(invocation, associated_data)
          .await
          .map_err(|e| ExecutionError::ProviderError(e.to_string()))
      })
    } else {
      let clone = self
        .providers
        .get(&namespace)
        .ok_or_else(|| ExecutionError::InvalidState(StateError::MissingProvider(self.namespace().to_owned())))?
        .provider
        .clone();
      tokio::spawn(async move {
        clone
          .handle(invocation, associated_data)
          .await
          .map_err(|e| ExecutionError::ProviderError(e.to_string()))
      })
    };

    let outer_result = fut.await.map_err(|e| ExecutionError::ProviderError(e.to_string()));

    let stream = match outer_result {
      Ok(Ok(result)) => result,
      Ok(Err(error)) | Err(error) => {
        warn!(%error, "component error");
        channel
          .dispatch(Event::call_err(
            tx_id,
            self.index(),
            MessageTransport::error(error.to_string()),
          ))
          .await?;
        return Ok(());
      }
    };

    tokio::spawn(async move {
      let span = trace_span!(
        "output_task", %invocation_id, component = %format!("{} ({})", identifier, entity)
      );
      if let Err(error) = output_handler(tx_id, &self, stream, channel).instrument(span).await {
        error!(%error, "error in output handler");
      }
    });
    Ok(())
  }

  pub(crate) fn clone_buffer(&self, port: &PortReference) -> Vec<TransportWrapper> {
    match port.direction() {
      PortDirection::In => self.inputs.get_handler(port).clone_buffer(),
      PortDirection::Out => self.outputs.get_handler(port).clone_buffer(),
    }
  }

  pub(crate) fn num_pending(&self) -> u32 {
    self.pending.load(Ordering::Relaxed)
  }
}

async fn output_handler(
  tx_id: Uuid,
  instance: &InstanceHandler,
  mut stream: TransportStream,
  channel: InterpreterDispatchChannel,
) -> Result<()> {
  trace!("starting output task");

  // TODO: make this timeout configurable from instance configuration.
  let timeout = Duration::from_millis(1000);

  let reason = loop {
    let response = tokio::time::timeout(timeout, stream.next());
    match response.await {
      Ok(Some(message)) => {
        if message.is_component_error() {
          warn!(message=?message,"component-wide error");
          let error = message.error().unwrap();
          channel
            .dispatch(Event::call_err(tx_id, instance.index(), MessageTransport::error(error)))
            .await?;
          break CompletionStatus::Error;
        }
        if message.is_component_state() {
          // TODO
          continue;
        }
        let port = instance.find_output(&message.port)?;

        trace!(port=%message.port,"received packet");
        let action = instance.buffer_out(&port, message).unwrap();
        if action == BufferAction::Buffered {
          channel.dispatch(Event::port_data(tx_id, port)).await?;
        }
      }
      Err(error) => {
        error!(%error,"timeout");
        break CompletionStatus::Timeout;
      }
      Ok(None) => {
        trace!("stream complete");
        break CompletionStatus::Finished;
      }
    }
  };
  let ports = instance.handle_call_complete(reason)?;
  trace!(?ports, "ports to update");
  for port in ports {
    channel.dispatch(Event::port_status_change(tx_id, port)).await?;
  }
  channel.dispatch(Event::call_complete(tx_id, instance.index())).await?;
  Ok(())
}

#[derive(Clone, Copy)]
pub(crate) enum CompletionStatus {
  Finished,
  Timeout,
  Error,
  Deferred,
}

#[derive(Default, Debug)]
pub(super) struct StatusBreakdown {
  has_open: bool,
  has_any_generator: bool,
  has_all_generators: bool,
  has_done_open: bool,
  has_done_closing: bool,
  has_done_closed: bool,
  has_pending_packets: bool,
}

impl StatusBreakdown {
  pub(super) fn has_any_open(&self) -> bool {
    self.has_open || self.has_done_open || self.has_done_closing || self.has_pending_packets
  }
}

pub(super) fn check_statuses<'a>(ports: impl Iterator<Item = &'a PortHandler>) -> StatusBreakdown {
  let mut breakdown = StatusBreakdown::default();
  let mut total_ports = 0;

  let mut ports_open = Vec::new();
  let mut ports_doneopen = Vec::new();
  let mut ports_doneyield = Vec::new();
  let mut ports_doneclosing = Vec::new();
  let mut ports_doneclosed = Vec::new();

  for port in ports {
    total_ports += 1;
    let upstream_status = port.status();
    trace!(port = port.name(), status = %upstream_status, "port status");
    // if port is not empty (and is not a generator)
    if !port.is_empty() && !matches!(upstream_status, PortStatus::DoneYield) {
      breakdown.has_pending_packets = true;
    }
    match upstream_status {
      PortStatus::Open => ports_open.push(port.name()),
      PortStatus::DoneOpen => ports_doneopen.push(port.name()),
      PortStatus::DoneYield => ports_doneyield.push(port.name()),
      PortStatus::DoneClosing => ports_doneclosing.push(port.name()),
      PortStatus::DoneClosed => ports_doneclosed.push(port.name()),
    }
  }
  breakdown.has_all_generators = ports_doneyield.len() == total_ports;
  trace!(
    open = %ports_open.join(", "),
    done_open = %ports_doneopen.join(", "),
    done_yield = %ports_doneyield.join(", "),
    done_closing = %ports_doneclosing.join(", "),
    done_closed = %ports_doneclosed.join(", "),
    all_generators = breakdown.has_all_generators,
    "port statuses");
  breakdown.has_open = !ports_open.is_empty();
  breakdown.has_done_open = !ports_doneopen.is_empty();
  breakdown.has_any_generator = !ports_doneyield.is_empty();
  breakdown.has_done_closing = !ports_doneclosing.is_empty();
  breakdown.has_done_closed = !ports_doneclosed.is_empty();
  breakdown
}
