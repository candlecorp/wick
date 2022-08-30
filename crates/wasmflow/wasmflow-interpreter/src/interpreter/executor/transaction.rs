use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use parking_lot::Mutex;
use seeded_random::{Random, Seed};
use uuid::Uuid;
use wasmflow_schematic_graph::iterators::SchematicHop;
use wasmflow_schematic_graph::{ComponentIndex, PortReference};
use wasmflow_sdk::v1::transport::{Failure, MessageTransport, TransportMap, TransportStream, TransportWrapper};
use wasmflow_sdk::v1::{Entity, InherentData, Invocation};

use self::component::port::port_handler::BufferAction;
use self::component::InstanceHandler;
use super::error::ExecutionError;
use super::output_channel::OutputChannel;
use crate::constants::*;
use crate::graph::types::*;
use crate::interpreter::channel::Event;
use crate::interpreter::error::StateError;
use crate::interpreter::executor::transaction::component::port::PortStatus;
use crate::{Collection, HandlerMap, InterpreterDispatchChannel};

pub(crate) mod component;

pub(crate) mod statistics;
pub(crate) use statistics::TransactionStatistics;

type Result<T> = std::result::Result<T, ExecutionError>;

#[derive()]
#[must_use]
pub struct Transaction {
  schematic: Arc<Schematic>,
  output: OutputChannel,
  channel: InterpreterDispatchChannel,
  invocation: Invocation,
  instances: Vec<Arc<InstanceHandler>>,
  id: Uuid,
  start_time: Instant,
  rng: Random,
  finished: AtomicBool,
  pub(crate) last_access_time: Mutex<SystemTime>,
  pub(crate) stats: TransactionStatistics,
}

impl std::fmt::Debug for Transaction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Transaction").field("id", &self.id).finish()
  }
}

impl Transaction {
  pub(crate) fn new(
    schematic: Arc<Schematic>,
    mut invocation: Invocation,
    channel: InterpreterDispatchChannel,
    collections: &Arc<HandlerMap>,
    self_collection: &Arc<dyn Collection + Send + Sync>,
    seed: Seed,
  ) -> Self {
    let instances: Vec<_> = schematic
      .components()
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
    Self {
      channel,
      invocation,
      schematic,
      output: OutputChannel::default(),
      instances,
      start_time: Instant::now(),
      stats,
      last_access_time: Mutex::new(SystemTime::now()),
      id,
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

  pub(crate) fn instance(&self, index: ComponentIndex) -> &Arc<InstanceHandler> {
    &self.instances[index]
  }

  pub(crate) fn senders(&self) -> impl Iterator<Item = &Arc<InstanceHandler>> {
    self.instances.iter().filter(|i| i.is_core_component(CORE_ID_SENDER))
  }

  pub(crate) fn generators(&self) -> impl Iterator<Item = &Arc<InstanceHandler>> {
    self.instances.iter().filter(|i| i.is_static() || i.inputs().len() == 0)
  }

  pub(crate) fn done(&self) -> bool {
    let output_handler = self.output_handler();
    let outputs_done = output_handler
      .inputs()
      .iter()
      .all(|p| p.status() == PortStatus::DoneClosed && p.is_empty());

    outputs_done
  }

  pub(crate) async fn start(&mut self) -> Result<()> {
    self.stats.mark("start");
    self.stats.start("execution");
    let span = trace_span!("transaction", id = %self.id);
    let _guard = span.enter();
    trace!("starting transaction");
    self.start_time = Instant::now();

    self
      .prime_input_ports(self.schematic.input().index(), &self.invocation.payload)
      .await?;
    trace!("primed input ports");

    let inherent_data = self.invocation.inherent.unwrap_or_else(|| InherentData {
      seed: self.rng.gen(),
      timestamp: SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap(),
    });

    self.prime_inherent(inherent_data).await?;

    self.kick_senders().await?;
    self.kick_generators().await?;

    trace!("transaction started");
    self.stats.mark("start_done");
    Ok(())
  }

  async fn prime_input_ports(&self, index: ComponentIndex, payload: &TransportMap) -> Result<()> {
    let input = self.instance(index);
    input.validate_payload(payload)?;
    for (name, payload) in payload.inner() {
      let port = input.find_input(name)?;
      trace!("priming input port '{}'", name);
      self
        .accept_inputs(
          &port,
          vec![
            TransportWrapper::new(name, payload.clone()),
            TransportWrapper::done(name),
          ],
        )
        .await?;
    }
    Ok(())
  }

  async fn prime_inherent(&self, inherent_data: InherentData) -> Result<()> {
    let inherent = self.instance(INHERENT_COMPONENT);
    let seed_name = "seed";
    if let Ok(port) = inherent.find_input(seed_name) {
      trace!("priming inherent seed");
      self
        .accept_inputs(
          &port,
          vec![
            TransportWrapper::new(seed_name, MessageTransport::success(&inherent_data.seed)),
            TransportWrapper::done(seed_name),
          ],
        )
        .await?;
    }
    Ok(())
  }

  async fn kick_senders(&self) -> Result<()> {
    for instance in self.senders() {
      trace!("readying sender '{}'", instance.id());
      self
        .dispatch_invocation(instance.index(), TransportMap::default())
        .await?;
    }
    Ok(())
  }

  async fn kick_generators(&self) -> Result<()> {
    for instance in self.generators() {
      self.kick_generator(instance).await?;
    }
    Ok(())
  }

  async fn kick_generator(&self, instance: &InstanceHandler) -> Result<()> {
    self
      .dispatch_invocation(instance.index(), TransportMap::default())
      .await
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

  pub(crate) async fn emit_output_message(&self, message: TransportWrapper) -> Result<()> {
    debug!(%message, "emitting tx output");
    self.output.push(message).await?;
    if self.done() {
      self.emit_done().await?;
    }
    Ok(())
  }

  pub(crate) async fn emit_done(&self) -> Result<()> {
    if !self.finished.load(Ordering::Relaxed) {
      self.finished.store(true, Ordering::Relaxed);
      self.channel.dispatch(Event::tx_done(self.id())).await?;
    }
    Ok(())
  }

  pub(crate) fn take_stream(&mut self) -> Option<TransportStream> {
    self.output.detach().map(|rx| TransportStream::new(rx.into_stream()))
  }

  pub(crate) fn take_output(&self, port: &PortReference) -> Result<TransportWrapper> {
    let output = self.output_handler();
    output
      .take_input(port)
      .ok_or_else(|| ExecutionError::InvalidState(StateError::PayloadMissing(output.id().to_owned())))
  }

  #[allow(clippy::unused_async)]
  pub(crate) async fn take_payload(&self, instance: &InstanceHandler) -> Result<Option<TransportMap>> {
    let payload = instance.take_payload()?;
    match payload {
      Some(_) => {
        trace!("payload collected");
      }
      None => {
        trace!("payload not ready");
      }
    };
    Ok(payload)
  }

  pub(crate) fn take_component_output(&self, port: &PortReference) -> Option<TransportWrapper> {
    let instance = self.instance(port.component_index());
    instance.take_output(port)
  }

  #[instrument(name = "downstream-input", skip_all, fields(%port))]
  pub(crate) async fn accept_inputs(&self, port: &PortReference, msgs: Vec<TransportWrapper>) -> Result<()> {
    for payload in msgs {
      let instance = self.instance(port.component_index());
      let action = instance.buffer_in(port, payload)?;
      match action {
        BufferAction::Consumed(packet) => {
          trace!(?packet, "consumed packet");
        }
        BufferAction::Buffered => {
          self.channel.dispatch(Event::port_data(self.id, *port)).await?;
        }
      };
    }

    Ok(())
  }

  pub(crate) async fn accept_outputs(&self, port: &PortReference, msgs: Vec<TransportWrapper>) -> Result<()> {
    let instance = self.instance(port.component_index());
    for payload in msgs {
      let action = instance.buffer_out(port, payload)?;
      if action == BufferAction::Buffered {
        self.channel.dispatch(Event::port_data(self.id, *port)).await?;
      }
    }
    Ok(())
  }

  pub(crate) async fn check_hung(&self) -> Result<bool> {
    if self.done() {
      self.channel.dispatch(Event::tx_done(self.id())).await?;
      Ok(false)
    } else {
      warn!(tx_id = %self.id(), "transaction hung");
      self
        .emit_output_message(TransportWrapper::component_error(MessageTransport::Failure(
          Failure::Error("Transaction hung".to_owned()),
        )))
        .await?;
      Ok(true)
    }
  }

  #[instrument(skip(self, err), name = "short_circuit")]
  pub(crate) async fn handle_short_circuit(&self, index: ComponentIndex, err: MessageTransport) -> Result<()> {
    self.stats.mark(format!("component:{}:short_circuit", index));
    let instance = self.instance(index);

    let graph = self.schematic();

    for port in instance.outputs().refs() {
      let downport_name = graph.get_port_name(&port);
      self
        .accept_outputs(
          &port,
          vec![
            TransportWrapper::new(downport_name, err.clone()),
            TransportWrapper::done(downport_name),
          ],
        )
        .await?;
    }
    Ok(())
  }

  #[instrument(skip(self, payload), name = "dispatch-invocation")]
  pub(crate) async fn dispatch_invocation(&self, index: ComponentIndex, payload: TransportMap) -> Result<()> {
    let tx_id = self.id();

    let instance = self.instance(index).clone();
    debug!(id = instance.id(), ?payload);

    if payload.has_error() {
      let err = payload.take_error().unwrap();
      return self.handle_short_circuit(instance.index(), err).await;
    }

    let invocation = Invocation::next(
      self.id(),
      Entity::local(self.schematic_name()),
      instance.entity(),
      payload,
      self.invocation.inherent,
    );

    instance
      .dispatch_invocation(tx_id, invocation, self.channel.clone())
      .await
  }

  #[instrument(skip(self, invocation), name = "invoke")]
  pub(crate) async fn invoke(&self, index: ComponentIndex, invocation: Invocation) -> Result<()> {
    let tx_id = self.id();

    let instance = self.instance(index).clone();

    instance.invoke(tx_id, invocation, self.channel.clone()).await
  }

  pub(crate) async fn handle_schematic_output(&self, port: &PortReference) -> Result<()> {
    debug!("schematic output");

    let message = self.take_output(port)?;

    self.emit_output_message(message).await?;

    Ok(())
  }

  pub(crate) fn json_status(&self) -> Vec<serde_json::Value> {
    let graph = self.schematic();
    let mut lines = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for hop in graph.walk_from_output() {
      match hop {
        SchematicHop::Component(c) => {
          if !seen.contains(c.name()) {
            lines.push(serde_json::json!({"type":"component","name":c.name(),"component_index":c.inner().index()}));
            seen.insert(c.name().to_owned());
          }
        }
        SchematicHop::Port(p) => {
          if !seen.contains(&p.to_string()) {
            let instance = self.instance(p.component().index());

            let status = instance.get_port_status(p.as_ref());

            let component = p.component();
            let component = component.name();
            let port_ref = p.inner().detached();
            let pending = instance.buffered_packets(&port_ref);
            let packets = instance.clone_buffer(&port_ref);
            lines.push(serde_json::json!({
              "type":"port",
              "direction":p.direction().to_string(),
              "port":p.name(),
              "component":component,
              "port_index":port_ref.port_index(),
              "component_index":port_ref.component_index(),
              "pending":pending,
              "packets":packets,
              "status":status.to_string()
            }));
            seen.insert(p.to_string());
          }
        }
        SchematicHop::Connection(c) => {
          if !seen.contains(&c.to_string()) {
            lines.push(serde_json::json!({"type":"connection","connection":c.to_string()}));
            seen.insert(c.to_string());
          }
        }
        _ => {}
      }
    }
    let mut lines: Vec<_> = lines.into_iter().rev().collect();

    let output = self.output_handler();
    for instance in output.outputs().iter() {
      let port_ref = instance.port_ref();
      let status = instance.status();
      let pending = instance.len();
      let packets = instance.clone_buffer();

      lines.push(serde_json::json!({
        "type":"port",
        "direction":port_ref.direction().to_string(),
        "port":instance.name(),
        "component":output.id(),
        "port_index":port_ref.port_index(),
        "component_index":port_ref.component_index(),
        "pending":pending,
        "packets":packets,
        "status":status.to_string(),
      }));
    }

    for instance in &self.instances {
      lines.push(serde_json::json!({
        "type":"pending",
        "component_index":instance.index(),
        "num":instance.num_pending()
      }));
    }
    lines
  }
}
