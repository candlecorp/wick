use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use flow_component::{Component, ComponentError, RuntimeCallback};
use flow_graph::{NodeIndex, PortReference};
use parking_lot::Mutex;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use tracing::Span;
use tracing_futures::Instrument;
use uuid::Uuid;
use wasmrs_rx::{FluxChannel, Observer};
use wick_packet::{Entity, Invocation, Packet, PacketError, PacketPayload, PacketSender, PacketStream};

use self::port::{InputPorts, OutputPorts, PortStatus};
use crate::graph::types::*;
use crate::graph::{LiquidOperationConfig, Reference};
use crate::interpreter::channel::InterpreterDispatchChannel;
use crate::interpreter::components::self_component::SelfComponent;
use crate::interpreter::error::StateError;
use crate::interpreter::executor::error::ExecutionError;
use crate::utils::Bucket;
use crate::{HandlerMap, InterpreterOptions};
type Result<T> = std::result::Result<T, ExecutionError>;

pub(crate) mod port;

#[derive()]
#[must_use]
pub(crate) struct InstanceHandler {
  reference: Reference,
  identifier: String,
  invocation: Bucket<Invocation>,
  index: NodeIndex,
  sender: PacketSender,
  inputs: InputPorts,
  outputs: OutputPorts,
  schematic: Arc<Schematic>,
  pending: AtomicU32,
  components: Arc<HandlerMap>,
  task: InstanceTask,
  self_component: SelfComponent,
  span: Span,
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
    invocation: Invocation,
    op_node: &OperationNode,
    components: Arc<HandlerMap>,
    self_component: SelfComponent,
  ) -> Self {
    let inputs = op_node.inputs().to_vec();
    let outputs = op_node.outputs().to_vec();
    let reference: Reference = op_node.kind().cref().into();

    let span = invocation.following_span(debug_span!("instance", entity = %invocation.target));

    Self {
      schematic,
      inputs: InputPorts::new(op_node.id(), inputs),
      outputs: OutputPorts::new(op_node.id(), outputs),
      invocation: Bucket::new(invocation),
      reference,
      index: op_node.index(),
      identifier: op_node.id().to_owned(),
      components,
      sender: FluxChannel::new(),
      pending: AtomicU32::new(0),
      self_component,
      task: Default::default(),
      span,
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

  pub(super) fn drain_inputs(&self) -> Result<Vec<Packet>> {
    self.inputs.drain_packets()
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

  pub(super) fn is_running(&self) -> bool {
    self.task.has_started() && !self.task.is_done()
  }

  pub(super) fn has_started(&self) -> bool {
    self.task.has_started()
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
        PortStatus::Open => PortStatus::UpstreamComplete,
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
    channel: InterpreterDispatchChannel,
    options: &InterpreterOptions,
    callback: Arc<RuntimeCallback>,
    config: LiquidOperationConfig,
  ) -> Result<()> {
    if self.task.has_started() {
      #[cfg(debug_assertions)]
      self
        .span
        .in_scope(|| warn!("BUG: trying to start instance when one is already running"));
      return Ok(());
    }

    self.span.in_scope(|| debug!("instance:starting"));

    let identifier = self.id().to_owned();

    let Some(mut invocation) = self.invocation.take() else {
      return Err(StateError::InvocationMissing(identifier).into());
    };

    let entity = self.entity();
    let namespace = self.namespace().to_owned();

    let mut associated_data = self.schematic.nodes()[self.index()].data().clone();

    if associated_data.config.root().is_none() {
      associated_data.config.set_root(config.root().cloned());
    }

    if associated_data.config.value().is_none() {
      associated_data.config.set_value(config.value().cloned());
    }

    let config = associated_data
      .config
      .render(&invocation.inherent)
      .map_err(|e| ExecutionError::ComponentError(ComponentError::new(e)))?;

    let timeout = associated_data
      .settings
      .as_ref()
      .and_then(|v| v.timeout)
      .unwrap_or(options.output_timeout);

    let span = invocation.following_span(trace_span!(
      "next", operation = %format!("{} ({})", identifier, entity)
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

    let fut = if namespace == SelfComponent::ID {
      let clone = self.self_component.clone();
      tokio::spawn(async move {
        clone
          .handle(invocation, config, cb)
          .await
          .map_err(ExecutionError::ComponentError)
      })
    } else {
      let clone = self
        .components
        .get(&namespace)
        .ok_or_else(|| ExecutionError::InvalidState(StateError::MissingComponent(self.namespace().to_owned())))?
        .component
        .clone();
      tokio::spawn(async move {
        clone
          .handle(invocation, config, cb)
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

        span.in_scope(|| warn!(%msg, "component error"));

        channel.dispatch_op_err(tx_id, self.index(), PacketPayload::Err(PacketError::new(msg)));
        return Ok(());
      }
    };

    self
      .task
      .start(tx_id, self.clone(), stream, channel, timeout, span.clone());

    Ok(())
  }

  pub(crate) fn num_pending(&self) -> u32 {
    self.pending.load(Ordering::Relaxed)
  }
}

#[derive(Default, Clone)]
struct InstanceTask {
  task: Arc<Mutex<Option<JoinHandle<Result<()>>>>>,
  start_time: Arc<Mutex<Option<Instant>>>,
  end_time: Arc<Mutex<Option<Instant>>>,
}

impl InstanceTask {
  fn start(
    &self,
    tx_id: Uuid,
    instance: Arc<InstanceHandler>,
    stream: PacketStream,
    channel: InterpreterDispatchChannel,
    timeout: Duration,
    span: Span,
  ) {
    if self.has_started() {
      #[cfg(debug_assertions)]
      span.in_scope(|| warn!("BUG: trying to start instance task when one is already running"));
      return;
    }
    self.start_time.lock().replace(Instant::now());

    let end_time = self.end_time.clone();

    span.in_scope(|| debug!(instance = instance.id(), "task:start"));
    let task = tokio::spawn(async move {
      let result = output_handler(tx_id, &instance, stream, channel, timeout, span.clone()).await;
      if let Err(error) = &result {
        span.in_scope(|| error!(%error, "error in output handler"));
      }
      let now = Instant::now();
      span.in_scope(|| {
        let elapsed = now - instance.task.start_time.lock().unwrap();
        debug!(instance = instance.id(), elapsed_ms = elapsed.as_millis(), "task:end");
      });

      end_time.lock().replace(now);
      result
    });
    self.task.lock().replace(task);
  }

  fn has_started(&self) -> bool {
    self.task.lock().is_some()
  }

  #[allow(unused)]
  fn is_done(&self) -> bool {
    self.end_time.lock().is_some()
  }

  #[allow(unused)]
  async fn join(&self) -> Result<()> {
    let task = self.task.lock().take();
    if let Some(task) = task {
      return task.await.map_err(ExecutionError::OperationFailure)?;
    }
    Ok(())
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
          channel.dispatch_op_err(tx_id, instance.index(), PacketPayload::fatal_error(e.to_string()));
          break CompletionStatus::Error;
        }
        let message = message.unwrap();

        span.in_scope(
          || trace!(op=instance.id(),port=%message.port(),flags=message.flags(),payload=?message.payload(),"received output packet"),
        );

        if message.is_fatal_error() {
          span.in_scope(|| warn!(error=?message,"component-wide error"));
          channel.dispatch_op_err(tx_id, instance.index(), message.payload);
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

        instance.buffer_out(&port, message);
        channel.dispatch_data(tx_id, port);
      }
      Err(error) => {
        span.in_scope(|| warn!(%error,"timeout"));
        let msg = format!(
          "Transaction timed out waiting for output from operation {} ({})",
          instance.id(),
          instance.entity()
        );
        channel.dispatch_op_err(tx_id, instance.index(), PacketPayload::fatal_error(msg));
        break CompletionStatus::Timeout;
      }
      Ok(None) => {
        if num_received == 0 && instance.outputs().len() > 0 {
          let err = "operation produced no output, likely due to a panic or misconfiguration";
          span.in_scope(|| warn!(error = err, "stream complete"));
          channel.dispatch_op_err(tx_id, instance.index(), PacketPayload::fatal_error(err));
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
  channel.dispatch_call_complete(tx_id, instance.index());
  Ok(())
}

#[derive(Clone, Copy)]
pub(crate) enum CompletionStatus {
  Finished,
  Timeout,
  Error,
}
