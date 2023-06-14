use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use flow_component::{Component, RuntimeCallback};
use flow_graph::{NodeIndex, PortReference};
use tokio_stream::StreamExt;
use tracing::Span;
use tracing_futures::Instrument;
use uuid::Uuid;
use wasmrs_rx::{FluxChannel, Observer};
use wick_packet::{Entity, Invocation, Packet, PacketError, PacketPayload, PacketSender, PacketStream};

use self::port::{InputPorts, OutputPorts, PortStatus};
use crate::constants::*;
use crate::graph::types::*;
use crate::graph::Reference;
use crate::interpreter::channel::InterpreterDispatchChannel;
use crate::interpreter::error::StateError;
use crate::interpreter::executor::error::ExecutionError;
use crate::{HandlerMap, InterpreterOptions};
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
  self_collection: Arc<dyn Component + Send + Sync>,
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

impl std::fmt::Display for InstanceHandler {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}::{}", self.namespace(), self.id())
  }
}

impl InstanceHandler {
  pub(super) fn new(
    schematic: Arc<Schematic>,
    operation: &Operation,
    collections: Arc<HandlerMap>,
    self_collection: Arc<dyn Component + Send + Sync>,
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

  pub(super) fn take_packets(&self) -> Result<Vec<Packet>> {
    self.inputs.take_packets()
  }

  pub(super) fn take_output(&self, port: &PortReference) -> Option<Packet> {
    self.outputs.take(port)
  }

  pub(super) fn take_input(&self, port: &PortReference) -> Option<Packet> {
    self.inputs.take(port)
  }

  pub(crate) fn buffer_in(&self, port: &PortReference, value: Packet) {
    trace!(operation=%self, port=self.inputs.get_handler(port).name(), ?value, "buffering input message");

    self.inputs.receive(port, value);
  }

  pub(crate) fn buffer_out(&self, port: &PortReference, value: Packet) {
    trace!(operation=%self, port=self.outputs.get_handler(port).name(), ?value, "buffering output message");

    self.outputs.receive(port, value);
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

  pub(crate) fn handle_stream_complete(&self, status: CompletionStatus) -> Result<Vec<PortReference>> {
    self.decrement_pending()?;
    Ok(self.set_outputs_closed(status))
  }

  pub(super) fn set_outputs_closed(&self, _status: CompletionStatus) -> Vec<PortReference> {
    let mut changed_statuses = Vec::new();
    for port in self.outputs.iter() {
      let current_status = port.status();

      let new_status = match current_status {
        PortStatus::Open => {
          // Note: A port can still be "Open" after a call if the operation panics.
          // if port.is_empty() {
          //   PortStatus::DoneClosed
          // } else {
          PortStatus::DoneClosing
          // }
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
    let all_closed = self.inputs.iter().all(|p| p.get_status().is_closed());
    if all_closed {
      self.sender.complete();
    }
    Ok(())
  }

  pub(crate) async fn start(
    self: Arc<Self>,
    tx_id: Uuid,
    mut invocation: Invocation,
    channel: InterpreterDispatchChannel,
    options: &InterpreterOptions,
    callback: Arc<RuntimeCallback>,
  ) -> Result<()> {
    let identifier = self.id().to_owned();
    let entity = self.entity();
    let namespace = self.namespace().to_owned();

    let associated_data = self.schematic.nodes()[self.index()].data();

    let associated_data = associated_data.clone();

    let span = invocation.following_span(trace_span!(
      "operation exec", component = %format!("{} ({})", identifier, entity)
    ));

    self.increment_pending();
    let stream = if self.inputs.is_empty() {
      invocation.trace(|| debug!(%entity, "operation has no inputs, starting with noop packet"));
      PacketStream::noop()
    } else {
      PacketStream::new(Box::new(self.sender.take_rx().unwrap()))
    };
    invocation.attach_stream(stream);
    let cb = callback.clone();

    let fut = if namespace == NS_SELF {
      let clone = self.self_collection.clone();
      tokio::spawn(async move {
        clone
          .handle(invocation, associated_data, cb)
          .await
          .map_err(ExecutionError::ComponentError)
      })
    } else {
      let clone = self
        .collections
        .get(&namespace)
        .ok_or_else(|| ExecutionError::InvalidState(StateError::MissingComponent(self.namespace().to_owned())))?
        .component
        .clone();
      tokio::spawn(async move {
        clone
          .handle(invocation, associated_data, cb)
          .await
          .map_err(ExecutionError::ComponentError)
      })
    };

    let outer_result = fut
      .instrument(span.clone())
      .await
      .map_err(ExecutionError::OperationFailure);

    let stream = match outer_result {
      Ok(Ok(result)) => result,
      Ok(Err(error)) | Err(error) => {
        let msg = if let ExecutionError::OperationFailure(e) = error {
          if e.is_panic() {
            format!("Operation {} panicked", entity)
          } else {
            format!("Operation {} cancelled", entity)
          }
        } else {
          format!("Operation {} failed: {}", entity, error)
        };

        warn!(%msg, "component error");

        channel
          .dispatch_op_err(tx_id, self.index(), PacketPayload::Err(PacketError::new(msg)))
          .await;
        return Ok(());
      }
    };

    let timeout = options.output_timeout;
    tokio::spawn(async move {
      if let Err(error) = output_handler(tx_id, &self, stream, channel, timeout, span.clone()).await {
        span.in_scope(|| error!(%error, "error in output handler"));
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
  timeout: Duration,
  span: Span,
) -> Result<()> {
  span.in_scope(|| trace!("starting output task"));

  let mut num_received = 0;
  let reason = loop {
    let response = tokio::time::timeout(timeout, stream.next());
    let mut hanging = HashMap::new();
    match response.await {
      Ok(Some(message)) => {
        num_received += 1;
        if let Err(e) = message {
          span.in_scope(|| warn!(error=?e,"component-wide error"));
          channel
            .dispatch_op_err(tx_id, instance.index(), PacketPayload::fatal_error(e.to_string()))
            .await;
          break CompletionStatus::Error;
        }
        let message = message.unwrap();
        if message.is_fatal_error() {
          span.in_scope(|| warn!(error=?message,"component-wide error"));
          channel.dispatch_op_err(tx_id, instance.index(), message.payload).await;
          break CompletionStatus::Error;
        }

        if message.is_noop() {
          continue;
        }

        let port = match instance.find_output(message.port()) {
          Ok(port) => port,
          Err(e) => {
            span.in_scope(|| warn!(error=?e,port=message.port(),data=?message.payload(),"invalid port name, this is likely due to a misconfigured or broken component"));
            return Err(e);
          }
        };

        if message.is_done() {
          hanging.remove(&port);
        } else {
          hanging.insert(port, message.port().to_owned());
        }

        span.in_scope(|| trace!(port=%message.port(),"received output packet"));

        instance.buffer_out(&port, message);
        channel.dispatch_data(tx_id, port).await;
      }
      Err(error) => {
        span.in_scope(|| warn!(%error,"timeout"));
        let msg = format!("Operation {} timed out waiting for upstream data.", instance.entity());
        channel
          .dispatch_op_err(tx_id, instance.index(), PacketPayload::fatal_error(msg))
          .await;
        break CompletionStatus::Timeout;
      }
      Ok(None) => {
        if num_received == 0 && instance.outputs().len() > 0 {
          let err = "component failed to produce output";
          span.in_scope(|| warn!(error = err, "stream complete"));
          channel
            .dispatch_op_err(tx_id, instance.index(), PacketPayload::fatal_error(err))
            .await;
          break CompletionStatus::Error;
        }
        for (portref, port) in hanging {
          span.in_scope(|| debug!(%port,"auto-closing port"));
          instance.buffer_out(&portref, Packet::done(port));
        }
        span.in_scope(|| trace!("stream complete"));
        break CompletionStatus::Finished;
      }
    }
  };
  instance.handle_stream_complete(reason)?;
  channel.dispatch_call_complete(tx_id, instance.index()).await;
  Ok(())
}

#[derive(Clone, Copy)]
pub(crate) enum CompletionStatus {
  Finished,
  Timeout,
  Error,
}
