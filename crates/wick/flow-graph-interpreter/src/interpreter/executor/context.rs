use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use flow_component::RuntimeCallback;
use flow_graph::{NodeIndex, PortReference, SCHEMATIC_OUTPUT_INDEX};
use futures::StreamExt;
use parking_lot::Mutex;
use seeded_random::{Random, Seed};
use uuid::Uuid;
use wasmrs_rx::Observer;
use wick_packet::{Entity, Invocation, Packet, PacketError, PacketSender, PacketStream, RuntimeConfig};

use self::operation::{FutureInvocation, InstanceHandler};
use super::error::ExecutionError;
use crate::graph::types::*;
use crate::interpreter::channel::InterpreterDispatchChannel;
use crate::interpreter::components::self_component::SelfComponent;
use crate::interpreter::error::StateError;
use crate::interpreter::executor::context::operation::port::PortStatus;
use crate::{HandlerMap, InterpreterOptions};

pub(crate) mod operation;

pub(crate) mod statistics;
pub(crate) use statistics::ExecutionStatistics;

type Result<T> = std::result::Result<T, ExecutionError>;

#[derive(Debug, Clone, Copy)]
pub(crate) enum TxState {
  OutputPending,
  Finished,
  CompleteWithTasksPending,
}

#[derive()]
#[must_use]
pub struct ExecutionContext {
  schematic: Arc<Schematic>,
  output: (Option<PacketSender>, Option<PacketStream>),
  channel: InterpreterDispatchChannel,
  invocation: Invocation,
  instances: Vec<Arc<InstanceHandler>>,
  id: Uuid,
  start_time: Instant,
  finished: AtomicBool,
  span: tracing::Span,
  callback: Arc<RuntimeCallback>,
  root_config: Option<RuntimeConfig>,
  op_config: Option<RuntimeConfig>,
  options: Option<InterpreterOptions>,
  pub(crate) last_access_time: Mutex<SystemTime>,
  pub(crate) stats: ExecutionStatistics,
}

impl std::fmt::Debug for ExecutionContext {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ExecutionContext").field("id", &self.id).finish()
  }
}

#[allow(clippy::too_many_arguments)]
impl ExecutionContext {
  pub(crate) fn new(
    schematic: Arc<Schematic>,
    invocation: Invocation,
    channel: InterpreterDispatchChannel,
    components: &Arc<HandlerMap>,
    self_component: &SelfComponent,
    callback: Arc<RuntimeCallback>,
    root_config: Option<RuntimeConfig>,
    op_config: Option<RuntimeConfig>,
    seed: Seed,
  ) -> Self {
    let rng = Random::from_seed(seed);
    let id = invocation.id;

    let instances: Vec<_> = schematic
      .nodes()
      .iter()
      .map(|op_node| {
        Arc::new(InstanceHandler::new(
          schematic.clone(),
          FutureInvocation::next(
            &invocation,
            Entity::operation(op_node.cref().component_id(), op_node.cref().name()),
            rng.gen(),
          ),
          op_node,
          components.clone(),
          self_component.clone(),
        ))
      })
      .collect();

    let stats = ExecutionStatistics::new(invocation.id);
    stats.mark("new");
    let span = info_span!(parent:&invocation.span,"execution_flow",ctx_id=%id);
    let channel = channel.with_span(span.clone());

    let (tx, rx) = invocation.make_response();

    Self {
      channel,
      options: None,
      invocation,
      schematic,
      root_config,
      op_config,
      output: (Some(tx), Some(rx)),
      instances,
      start_time: Instant::now(),
      stats,
      last_access_time: Mutex::new(SystemTime::now()),
      id,
      span,
      finished: AtomicBool::new(false),
      callback,
    }
  }

  pub fn run(self) {
    let channel = self.channel.clone();
    channel.dispatch_start(Box::new(self));
  }

  pub fn in_scope<F: FnOnce() -> T, T>(&self, f: F) -> T {
    self.span.in_scope(f)
  }

  pub fn id(&self) -> Uuid {
    self.id
  }

  pub fn schematic_name(&self) -> &str {
    self.schematic.name()
  }

  pub(crate) fn schematic(&self) -> &Schematic {
    &self.schematic
  }

  pub(crate) fn output_handler(&self) -> &InstanceHandler {
    &self.instances[self.schematic.output().index()]
  }

  pub(crate) fn instance(&self, index: NodeIndex) -> &Arc<InstanceHandler> {
    &self.instances[index]
  }

  pub(crate) fn active_instances(&self) -> Vec<&Arc<InstanceHandler>> {
    self.instances.iter().filter(|i| i.is_running()).collect()
  }

  pub(crate) fn done(&self) -> bool {
    let output_handler = self.output_handler();
    let outputs_done = output_handler
      .inputs()
      .iter()
      .all(|p| p.status() == PortStatus::DoneClosed && p.is_empty());

    outputs_done
  }

  pub(crate) async fn start(&mut self, options: &InterpreterOptions) -> Result<()> {
    self.stats.mark("start");
    self.stats.start("execution");
    self.span.in_scope(|| trace!("starting execution"));

    self.options = Some(options.clone());

    self.start_time = Instant::now();

    for instance in self.instances.iter() {
      if instance.index() == SCHEMATIC_OUTPUT_INDEX {
        continue;
      }
      // start operations that have no inputs.
      if instance.inputs().is_empty() {
        instance
          .clone()
          .start(
            self.id(),
            self.channel.clone(),
            options,
            self.callback.clone(),
            self.root_config.clone(),
            self.op_config.clone(),
          )
          .await?;
      }
    }

    let incoming = self.invocation.eject_stream();

    self.prime_input_ports(self.schematic.input().index(), incoming)?;

    self.stats.mark("start_done");
    Ok(())
  }

  pub(crate) async fn start_instance(&self, instance: Arc<InstanceHandler>) -> Result<()> {
    instance
      .start(
        self.id(),
        self.channel.clone(),
        self.options.as_ref().unwrap(),
        self.callback.clone(),
        self.root_config.clone(),
        self.op_config.clone(),
      )
      .await?;

    Ok(())
  }

  fn prime_input_ports(&self, index: NodeIndex, mut payloads: PacketStream) -> Result<()> {
    let input = self.instance(index).clone();
    let channel = self.channel.clone();
    let ctx_id = self.id();

    tokio::spawn(async move {
      while let Some(Ok(packet)) = payloads.next().await {
        if let Ok(port) = input.find_input(packet.port()) {
          accept_input(ctx_id, port, &input, &channel, packet);
        } else if packet.is_noop() {
          // TODO: propagate this and/or its context if it becomes an issue.
        } else {
          debug!(port = packet.port(), "dropping packet for unconnected port");
        }
      }
    });
    Ok(())
  }

  pub(crate) fn update_last_access(&self) {
    let now = SystemTime::now();
    *self.last_access_time.lock() = now;
  }

  pub(crate) fn last_access(&self) -> SystemTime {
    *self.last_access_time.lock()
  }

  // Run when the execution context has finished delivering output to its output ports.
  //
  // A context may still be executing operations with side effects after this point.
  pub(crate) fn finish(&mut self) -> Result<&ExecutionStatistics> {
    self.span.in_scope(|| trace!("finishing execution output"));

    // drop our output sender;
    drop(self.output.0.take());

    // mark our end of execution
    self.stats.end("execution");

    // print stats if we're in tests.
    #[cfg(test)]
    self.stats.print();

    self.span.in_scope(|| trace!(statistics=?self.stats));

    Ok(&self.stats)
  }

  pub(crate) fn emit_output_message(&self, packets: Vec<Packet>) -> Result<()> {
    if let Some(ref output) = self.output.0 {
      for packet in packets {
        output.send(packet).map_err(|_e| ExecutionError::ChannelSend)?;
      }
    } else if packets.iter().any(|p| !p.is_done()) {
      self
        .span
        .in_scope(|| error!(ctx_id = %self.id(), "attempted to send output message after tx finished"));
    }

    if self.done() {
      self.emit_done()?;
    }
    Ok(())
  }

  pub(crate) fn emit_done(&self) -> Result<()> {
    if !self.finished.load(Ordering::Relaxed) {
      self.span.in_scope(|| trace!("tx finished, dispatching done"));
      self.finished.store(true, Ordering::Relaxed);
      self.channel.dispatch_done(self.id());
    }
    Ok(())
  }

  pub(crate) fn take_stream(&mut self) -> Option<PacketStream> {
    self.output.1.take()
  }

  pub(crate) fn take_tx_output(&self) -> Result<Vec<Packet>> {
    let output = self.output_handler();
    output
      .drain_inputs()
      .map_err(|_| ExecutionError::InvalidState(StateError::PayloadMissing(output.id().to_owned())))
  }

  pub(crate) fn take_instance_output(&self, port: &PortReference) -> Option<Packet> {
    let instance = self.instance(port.node_index());
    instance.take_output(port)
  }

  pub(crate) fn take_instance_input(&self, port: &PortReference) -> Option<Packet> {
    let instance = self.instance(port.node_index());
    instance.take_input(port)
  }

  pub(crate) fn check_stalled(&self) -> Result<TxState> {
    if self.done() {
      let active_instances = self.active_instances();
      if active_instances.is_empty() {
        Ok(TxState::Finished)
      } else {
        Ok(TxState::CompleteWithTasksPending)
      }
    } else {
      self.span.in_scope(|| warn!("execution hung"));
      self.emit_output_message(vec![Packet::component_error("execution hung")])?;
      Ok(TxState::OutputPending)
    }
  }

  pub(crate) async fn push_packets(&self, index: NodeIndex, packets: Vec<Packet>) -> Result<()> {
    let instance = self.instance(index).clone();
    if !instance.has_started() {
      self.start_instance(instance.clone()).await?;
    }

    let _ = instance.accept_packets(packets);

    Ok(())
  }

  pub(crate) fn handle_schematic_output(&self) -> Result<()> {
    self.emit_output_message(self.take_tx_output()?)?;

    Ok(())
  }

  pub(crate) fn handle_op_err(&self, index: NodeIndex, err: &PacketError) -> Result<()> {
    self.stats.mark(format!("component:{}:op_err", index));
    let instance = self.instance(index);

    let graph = self.schematic();

    for port in instance.outputs().refs() {
      let downport_name = graph.get_port_name(&port);
      let down_instance = self.instance(port.node_index());
      accept_outputs(
        self.id(),
        port,
        down_instance,
        &self.channel,
        vec![Packet::raw_err(downport_name, err.clone()), Packet::done(downport_name)],
      );
    }
    Ok(())
  }
}

pub(crate) fn accept_input(
  ctx_id: Uuid,
  port: PortReference,
  instance: &Arc<InstanceHandler>,
  channel: &InterpreterDispatchChannel,
  payload: Packet,
) {
  instance.buffer_in(&port, payload);
  channel.dispatch_data(ctx_id, port);
}

pub(crate) fn accept_outputs(
  ctx_id: Uuid,
  port: PortReference,
  instance: &Arc<InstanceHandler>,
  channel: &InterpreterDispatchChannel,
  msgs: Vec<Packet>,
) {
  for payload in msgs {
    accept_output(ctx_id, port, instance, channel, payload);
  }
}
pub(crate) fn accept_output(
  ctx_id: Uuid,
  port: PortReference,
  instance: &Arc<InstanceHandler>,
  channel: &InterpreterDispatchChannel,
  payload: Packet,
) {
  instance.buffer_out(&port, payload);
  channel.dispatch_data(ctx_id, port);
}
