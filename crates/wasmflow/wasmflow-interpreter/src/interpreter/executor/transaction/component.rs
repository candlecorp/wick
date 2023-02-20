use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio_stream::StreamExt;
use tracing_futures::Instrument;
use uuid::Uuid;
use wasmflow_schematic_graph::{ComponentIndex, PortDirection, PortReference};
use wasmflow_sdk::v1::transport::{MessageTransport, TransportMap, TransportStream, TransportWrapper};
use wasmflow_sdk::v1::{Entity, Invocation};

use self::port::port_handler::BufferAction;
use self::port::{InputPorts, OutputPorts, PortStatus};
use crate::constants::*;
use crate::graph::types::*;
use crate::graph::Reference;
use crate::interpreter::channel::Event;
use crate::interpreter::error::StateError;
use crate::{Collection, ExecutionError, HandlerMap, InterpreterDispatchChannel};
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
  collections: Arc<HandlerMap>,
  self_collection: Arc<dyn Collection + Send + Sync>,
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
    collections: Arc<HandlerMap>,
    self_collection: Arc<dyn Collection + Send + Sync>,
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
      collections,
      pending: AtomicU32::new(0),
      self_collection,
    }
  }

  pub(crate) fn entity(&self) -> Entity {
    Entity::component(self.reference.namespace(), self.reference.name())
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

  pub(crate) fn buffer_in(&self, port: &PortReference, value: TransportWrapper) -> Result<BufferAction> {
    trace!(%port, ?value, "buffering input message");

    self.inputs.receive(port, value)
  }

  pub(crate) fn buffer_out(&self, port: &PortReference, value: TransportWrapper) -> Result<BufferAction> {
    trace!(%port, ?value, "buffering output message");

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

  pub(crate) fn handle_call_complete(&self, status: CompletionStatus) -> Result<Vec<PortReference>> {
    self.decrement_pending()?;
    Ok(self.update_output_statuses(status))
  }

  pub(super) fn update_output_statuses(&self, _status: CompletionStatus) -> Vec<PortReference> {
    let mut changed_statuses = Vec::new();
    for port in self.outputs.iter() {
      let current_status = port.status();

      let new_status = match current_status {
        PortStatus::Open => {
          // Note: A port can still be "Open" after a call if the component panics.
          if port.is_empty() {
            PortStatus::DoneClosed
          } else {
            PortStatus::DoneClosing
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
      let clone = self.self_collection.clone();
      tokio::spawn(async move {
        clone
          .handle(invocation, associated_data)
          .await
          .map_err(ExecutionError::CollectionError)
      })
    } else {
      let clone = self
        .collections
        .get(&namespace)
        .ok_or_else(|| ExecutionError::InvalidState(StateError::MissingCollection(self.namespace().to_owned())))?
        .collection
        .clone();
      tokio::spawn(async move {
        clone
          .handle(invocation, associated_data)
          .await
          .map_err(ExecutionError::CollectionError)
      })
    };

    let outer_result = fut.await.map_err(|e| ExecutionError::CollectionError(Box::new(e)));

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
}
