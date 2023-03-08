use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use flow_graph::{NodeIndex, PortReference, SCHEMATIC_OUTPUT_INDEX};
use futures::StreamExt;
use parking_lot::Mutex;
use seeded_random::{Random, Seed};
use uuid::Uuid;
use wasmrs_rx::{FluxChannel, Observer};
use wick_packet::{InherentData, Invocation, Packet, PacketPayload, PacketStream};

use self::operation::port::port_handler::BufferAction;
use self::operation::InstanceHandler;
use super::error::ExecutionError;
use crate::graph::types::*;
use crate::interpreter::channel::InterpreterDispatchChannel;
use crate::interpreter::error::StateError;
use crate::interpreter::executor::transaction::operation::port::PortStatus;
use crate::{Collection, HandlerMap};

pub(crate) mod operation;

pub(crate) mod statistics;
pub(crate) use statistics::TransactionStatistics;

type Result<T> = std::result::Result<T, ExecutionError>;

#[derive()]
#[must_use]
pub struct Transaction {
  schematic: Arc<Schematic>,
  output: FluxChannel<Packet, wick_packet::Error>,
  channel: InterpreterDispatchChannel,
  invocation: Invocation,
  incoming: Option<PacketStream>,
  instances: Vec<Arc<InstanceHandler>>,
  id: Uuid,
  start_time: Instant,
  rng: Random,
  finished: AtomicBool,
  span: tracing::Span,
  pub(crate) last_access_time: Mutex<SystemTime>,
  pub(crate) stats: TransactionStatistics,
}

impl std::fmt::Debug for Transaction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Transaction").field("id", &self.id).finish()
  }
}

impl Transaction {
  #[instrument(skip_all, name = "tx_new")]
  pub(crate) fn new(
    schematic: Arc<Schematic>,
    mut invocation: Invocation,
    stream: PacketStream,
    channel: InterpreterDispatchChannel,
    collections: &Arc<HandlerMap>,
    self_collection: &Arc<dyn Collection + Send + Sync>,
    seed: Seed,
  ) -> Self {
    let instances: Vec<_> = schematic
      .nodes()
      .iter()
      .map(|component| {
        Arc::new(InstanceHandler::new(
          schematic.clone(),
          component,
          collections.clone(),
          self_collection.clone(),
        ))
      })
      .collect();

    let rng = Random::from_seed(seed);
    let id = rng.uuid();
    invocation.tx_id = id;
    let stats = TransactionStatistics::new(id);
    stats.mark("new");
    let span = tracing::Span::current();
    span.record("tx_id", id.to_string());

    Self {
      channel,
      invocation,
      incoming: Some(stream),
      schematic,
      output: FluxChannel::new(),
      instances,
      start_time: Instant::now(),
      stats,
      last_access_time: Mutex::new(SystemTime::now()),
      id,
      span,
      finished: AtomicBool::new(false),
      rng,
    }
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

  pub(crate) fn done(&self) -> bool {
    let output_handler = self.output_handler();
    let outputs_done = output_handler
      .inputs()
      .iter()
      .all(|p| p.status() == PortStatus::DoneClosed && p.is_empty());

    outputs_done
  }

  #[instrument(parent = &self.span, skip_all, name = "tx_start", fields(id = %self.id()))]
  pub(crate) async fn start(&mut self) -> Result<()> {
    self.stats.mark("start");
    self.stats.start("execution");
    trace!("starting transaction");
    self.start_time = Instant::now();

    for instance in self.instances.iter() {
      if instance.index() == SCHEMATIC_OUTPUT_INDEX {
        continue;
      }
      let invocation = Invocation::next_tx(
        self.id(),
        self.invocation.origin.clone(),
        instance.entity(),
        self.invocation.inherent,
      );
      instance
        .clone()
        .start(self.id(), invocation, self.channel.clone())
        .await?;
    }

    let incoming = self.incoming.take().unwrap();

    self.prime_input_ports(self.schematic.input().index(), incoming)?;

    let inherent_data = self.invocation.inherent.unwrap_or_else(|| InherentData {
      seed: self.rng.gen(),
      timestamp: SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap(),
    });

    self.prime_inherent(inherent_data)?;

    self.stats.mark("start_done");
    Ok(())
  }

  fn prime_input_ports(&self, index: NodeIndex, mut payloads: PacketStream) -> Result<()> {
    let input = self.instance(index).clone();
    let channel = self.channel.clone();
    let tx_id = self.id();

    tokio::spawn(async move {
      while let Some(Ok(packet)) = payloads.next().await {
        let port = input.find_input(packet.port_name()).unwrap();
        accept_input(tx_id, port, &input, &channel, packet).await;
      }
    });
    Ok(())
  }

  fn prime_inherent(&self, inherent_data: InherentData) -> Result<()> {
    let inherent = self.instance(INHERENT_COMPONENT).clone();
    let seed_name = "seed";
    if let Ok(port) = inherent.find_input(seed_name) {
      trace!("priming inherent seed");

      let fut = accept_inputs(
        self.id(),
        port,
        inherent,
        self.channel.clone(),
        vec![Packet::encode(seed_name, inherent_data.seed), Packet::done(seed_name)],
      );
      tokio::spawn(fut);
    }
    Ok(())
  }

  pub(crate) fn update_last_access(&self) {
    let now = SystemTime::now();
    let elapsed = now.duration_since(self.last_access());
    trace!(?elapsed, "updating last access");
    *self.last_access_time.lock() = now;
  }

  pub(crate) fn last_access(&self) -> SystemTime {
    *self.last_access_time.lock()
  }

  pub(crate) fn finish(mut self) -> Result<TransactionStatistics> {
    self.stats.end("execution");
    #[cfg(test)]
    self.stats.print();

    Ok(self.stats)
  }

  pub(crate) async fn emit_output_message(&self, packets: Vec<Packet>) -> Result<()> {
    for packet in packets {
      trace!(?packet, "emitting tx output");
      self.output.send(packet).map_err(|_e| ExecutionError::ChannelSend)?;
    }

    if self.done() {
      self.emit_done().await?;
    }
    Ok(())
  }

  pub(crate) async fn emit_done(&self) -> Result<()> {
    if !self.finished.load(Ordering::Relaxed) {
      self.finished.store(true, Ordering::Relaxed);
      self.channel.dispatch_done(self.id()).await;
    }
    Ok(())
  }

  pub(crate) fn take_stream(&mut self) -> Option<PacketStream> {
    self.output.take_rx().ok().map(|s| PacketStream::new(Box::new(s)))
  }

  pub(crate) fn take_output(&self) -> Result<Vec<Packet>> {
    let output = self.output_handler();
    output
      .take_packets()
      .map_err(|_| ExecutionError::InvalidState(StateError::PayloadMissing(output.id().to_owned())))
  }

  pub(crate) fn take_packets(&self, instance: &InstanceHandler) -> Result<Vec<Packet>> {
    instance.take_packets()
  }

  pub(crate) fn take_component_output(&self, port: &PortReference) -> Option<Packet> {
    let instance = self.instance(port.node_index());
    instance.take_output(port)
  }

  pub(crate) async fn check_hung(&self) -> Result<bool> {
    if self.done() {
      self.channel.dispatch_done(self.id()).await;
      Ok(false)
    } else {
      warn!(tx_id = %self.id(), "transaction hung");
      self
        .emit_output_message(vec![Packet::component_error("Transaction hung")])
        .await?;
      Ok(true)
    }
  }

  pub(crate) fn push_packets(&self, index: NodeIndex, packets: Vec<Packet>) -> Result<()> {
    let instance = self.instance(index).clone();

    let _ = instance.accept_packets(packets);

    Ok(())
  }

  pub(crate) async fn handle_schematic_output(&self) -> Result<()> {
    debug!("schematic output");

    self.emit_output_message(self.take_output()?).await?;

    Ok(())
  }

  pub(crate) async fn handle_op_err(&self, index: NodeIndex, err: PacketPayload) -> Result<()> {
    self.stats.mark(format!("component:{}:op_err", index));
    let instance = self.instance(index);

    let graph = self.schematic();

    for port in instance.outputs().refs() {
      let downport_name = graph.get_port_name(&port);
      let down_instance = self.instance(port.node_index());
      accept_outputs(
        self.id(),
        port,
        down_instance.clone(),
        self.channel.clone(),
        vec![
          Packet::new_for_port(downport_name, err.clone()),
          Packet::done(downport_name),
        ],
      )
      .await;
    }
    Ok(())
  }
}

pub(crate) async fn accept_inputs(
  tx_id: Uuid,
  port: PortReference,
  instance: Arc<InstanceHandler>,
  channel: InterpreterDispatchChannel,
  msgs: Vec<Packet>,
) {
  for payload in msgs {
    accept_input(tx_id, port, &instance, &channel, payload).await;
  }
}

pub(crate) async fn accept_input<'a, 'b>(
  tx_id: Uuid,
  port: PortReference,
  instance: &'a Arc<InstanceHandler>,
  channel: &'b InterpreterDispatchChannel,
  payload: Packet,
) {
  trace!(?payload, "accepting input");
  let action = instance.buffer_in(&port, payload);
  match action {
    BufferAction::Consumed(packet) => {
      trace!(?packet, "consumed packet");
    }
    BufferAction::Buffered => {
      channel.dispatch_data(tx_id, port).await;
    }
  };
}

pub(crate) async fn accept_outputs(
  tx_id: Uuid,
  port: PortReference,
  instance: Arc<InstanceHandler>,
  channel: InterpreterDispatchChannel,
  msgs: Vec<Packet>,
) {
  for payload in msgs {
    accept_output(tx_id, port, &instance, &channel, payload).await;
  }
}
pub(crate) async fn accept_output<'a, 'b>(
  tx_id: Uuid,
  port: PortReference,
  instance: &'a Arc<InstanceHandler>,
  channel: &'b InterpreterDispatchChannel,
  payload: Packet,
) {
  trace!(?payload, "accepting output");
  let action = instance.buffer_out(&port, payload);
  if action == BufferAction::Buffered {
    channel.dispatch_data(tx_id, port).await;
  }
}
