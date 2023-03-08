use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use flow_graph::{NodeIndex, PortReference};
use tokio_stream::StreamExt;
use tracing_futures::Instrument;
use uuid::Uuid;
use wasmrs_rx::{FluxChannel, Observer};
use wick_packet::{Entity, Invocation, Packet, PacketError, PacketPayload, PacketSender, PacketStream};

use self::port::port_handler::BufferAction;
use self::port::{InputPorts, OutputPorts, PortStatus};
use crate::constants::*;
use crate::graph::types::*;
use crate::graph::Reference;
use crate::interpreter::channel::InterpreterDispatchChannel;
use crate::interpreter::error::StateError;
use crate::interpreter::executor::error::ExecutionError;
use crate::{Collection, HandlerMap};
type Result<T> = std::result::Result<T, ExecutionError>;

pub(crate) mod port;

#[derive()]
#[must_use]
pub(crate) struct InstanceHandler {
  reference: Reference,
  identifier: String,
  index: NodeIndex,
  sender: PacketSender,
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
    operation: &Operation,
    collections: Arc<HandlerMap>,
    self_collection: Arc<dyn Collection + Send + Sync>,
  ) -> Self {
    let inputs = operation.inputs().to_vec();
    let outputs = operation.outputs().to_vec();
    let reference = operation.kind().cref().into();

    Self {
      schematic,
      inputs: InputPorts::new(inputs),
      outputs: OutputPorts::new(outputs),
      reference,
      index: operation.index(),
      identifier: operation.id().to_owned(),
      collections,
      sender: FluxChannel::new(),
      pending: AtomicU32::new(0),
      self_collection,
    }
  }

  pub(crate) fn entity(&self) -> Entity {
    Entity::operation(self.reference.namespace(), self.reference.name())
  }

  pub(crate) fn namespace(&self) -> &str {
    self.reference.namespace()
  }

  pub(crate) fn index(&self) -> NodeIndex {
    self.index
  }

  pub(crate) fn id(&self) -> &str {
    &self.identifier
  }

  // pub(crate) fn is_core_operation(&self, name: &str) -> bool {
  //   self.reference.is_core_operation(name)
  // }

  // pub(crate) fn is_static(&self) -> bool {
  //   self.reference.is_static()
  // }

  pub(crate) fn done(&self) -> bool {
    for port in self.inputs.iter() {
      if port.status() != PortStatus::DoneClosed {
        return false;
      }
    }
    true
  }

  pub(super) fn take_packets(&self) -> Result<Vec<Packet>> {
    self.inputs.take_packets()
  }

  pub(super) fn take_output(&self, port: &PortReference) -> Option<Packet> {
    self.outputs.take(port)
  }

  // pub(super) fn take_input(&self, port: &PortReference) -> Option<Packet> {
  //   self.inputs.take(port)
  // }

  // pub(crate) fn validate_payload(&self, payload: &TransportMap) -> Result<()> {
  //   for input in self.inputs.iter() {
  //     if !payload.contains_key(input.name()) {
  //       return Err(ExecutionError::MissingInput(input.name().to_owned()));
  //     }
  //   }
  //   Ok(())
  // }

  pub(crate) fn buffer_in(&self, port: &PortReference, value: Packet) -> BufferAction {
    trace!(%port, ?value, "buffering input message");

    self.inputs.receive(port, value)
  }

  pub(crate) fn buffer_out(&self, port: &PortReference, value: Packet) -> BufferAction {
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

  // pub(crate) fn get_port_status(&self, port: &PortReference) -> PortStatus {
  //   match port.direction() {
  //     PortDirection::In => self.inputs.get_handler(port).status(),
  //     PortDirection::Out => self.outputs.get_handler(port).status(),
  //   }
  // }

  // pub(crate) fn buffered_packets(&self, port: &PortReference) -> usize {
  //   match port.direction() {
  //     PortDirection::In => self.inputs.get_handler(port).len(),
  //     PortDirection::Out => self.outputs.get_handler(port).len(),
  //   }
  // }

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
          // Note: A port can still be "Open" after a call if the operation panics.
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

  pub(crate) fn accept_packets(self: Arc<Self>, packets: Vec<Packet>) -> Result<()> {
    for packet in packets {
      self.sender.send(packet)?;
    }
    let still_open = self.inputs.iter().any(|p| p.get_status() != PortStatus::DoneClosed);
    if !still_open {
      self.sender.complete();
    }
    Ok(())
  }

  pub(crate) async fn start(
    self: Arc<Self>,
    tx_id: Uuid,
    invocation: Invocation,
    channel: InterpreterDispatchChannel,
  ) -> Result<()> {
    let span = trace_span!("op-start", otel.name=%self.entity());

    let identifier = self.id().to_owned();
    let entity = self.entity();
    let namespace = self.namespace().to_owned();

    let associated_data = self.schematic.nodes()[self.index()].data();

    let associated_data = associated_data.clone();

    self.increment_pending();
    let stream = PacketStream::new(Box::new(self.sender.take_rx().unwrap()));

    let fut = if namespace == NS_SELF {
      let clone = self.self_collection.clone();
      tokio::spawn(async move {
        clone
          .handle(invocation, stream, associated_data)
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
          .handle(invocation, stream, associated_data)
          .await
          .map_err(ExecutionError::CollectionError)
      })
    };

    let outer_result = fut
      .instrument(span)
      .await
      .map_err(|e| ExecutionError::CollectionError(Box::new(e)));

    let stream = match outer_result {
      Ok(Ok(result)) => result,
      Ok(Err(error)) | Err(error) => {
        warn!(%error, "component error");
        channel
          .dispatch_op_err(
            tx_id,
            self.index(),
            PacketPayload::Err(PacketError::new(error.to_string())),
          )
          .await;
        return Ok(());
      }
    };
    let span = trace_span!(
      "output_task", component = %format!("{} ({})", identifier, entity)
    );
    tokio::spawn(async move {
      if let Err(error) = output_handler(tx_id, &self, stream, channel).instrument(span).await {
        error!(%error, "error in output handler");
      }
    });
    Ok(())
  }

  // pub(crate) fn clone_buffer(&self, port: &PortReference) -> Vec<Packet> {
  //   match port.direction() {
  //     PortDirection::In => self.inputs.get_handler(port).clone_buffer(),
  //     PortDirection::Out => self.outputs.get_handler(port).clone_buffer(),
  //   }
  // }

  pub(crate) fn num_pending(&self) -> u32 {
    self.pending.load(Ordering::Relaxed)
  }
}

async fn output_handler(
  tx_id: Uuid,
  instance: &InstanceHandler,
  mut stream: PacketStream,
  channel: InterpreterDispatchChannel,
) -> Result<()> {
  trace!("starting output task");

  // TODO: make this timeout configurable from instance configuration.
  let timeout = Duration::from_millis(1000);

  let reason = loop {
    let response = tokio::time::timeout(timeout, stream.next());
    match response.await {
      Ok(Some(message)) => {
        if let Err(e) = message {
          warn!(error=?e,"component-wide error");
          channel
            .dispatch_op_err(tx_id, instance.index(), PacketPayload::fatal_error(e.to_string()))
            .await;
          break CompletionStatus::Error;
        }
        let message = message.unwrap();

        let port = instance.find_output(message.port_name())?;

        trace!(port=%message.port_name(),"received packet");

        let action = instance.buffer_out(&port, message);
        if action == BufferAction::Buffered {
          channel.dispatch_data(tx_id, port).await;
        }
      }
      Err(error) => {
        warn!(%error,"timeout");
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
    channel.dispatch_status_change(tx_id, port).await;
  }
  channel.dispatch_call_complete(tx_id, instance.index()).await;
  Ok(())
}

#[derive(Clone, Copy)]
pub(crate) enum CompletionStatus {
  Finished,
  Timeout,
  Error,
}
