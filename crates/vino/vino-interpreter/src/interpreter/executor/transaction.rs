use std::sync::Arc;
use std::time::{Instant, SystemTime};

use rand::Rng;
use uuid::Uuid;
use vino_entity::Entity;
use vino_schematic_graph::iterators::{SchematicHop, WalkDirection};
use vino_schematic_graph::{ComponentIndex, PortDirection, PortReference};
use vino_transport::{InherentData, Invocation, MessageTransport, TransportMap, TransportStream, TransportWrapper};

use self::component::port::port_handler::BufferAction;
use self::component::InstanceHandler;
use super::error::ExecutionError;
use super::output_channel::OutputChannel;
use crate::graph::types::*;
use crate::interpreter::channel::Event;
use crate::interpreter::error::StateError;
use crate::interpreter::executor::transaction::component::check_statuses;
use crate::interpreter::executor::transaction::component::port::PortStatus;
use crate::interpreter::provider::core_provider::SENDER_ID;
use crate::{InterpreterDispatchChannel, Provider, Providers};

pub(crate) mod component;

pub(crate) mod statistics;
pub use statistics::TransactionStatistics;

type Result<T> = std::result::Result<T, ExecutionError>;

#[derive()]
#[must_use]
pub(crate) struct Transaction {
  schematic: Arc<Schematic>,
  output: OutputChannel,
  channel: InterpreterDispatchChannel,
  invocation: Invocation,
  instances: Vec<Arc<InstanceHandler>>,
  id: Uuid,
  start_time: Instant,
  pub(crate) stats: TransactionStatistics,
}

impl std::fmt::Debug for Transaction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Transaction").field("id", &self.id).finish()
  }
}

impl Transaction {
  pub(crate) fn new(
    id: Uuid,
    schematic: Arc<Schematic>,
    invocation: Invocation,
    channel: InterpreterDispatchChannel,
    providers: &Arc<Providers>,
    self_provider: &Arc<dyn Provider + Send + Sync>,
  ) -> Self {
    let instances: Vec<_> = schematic
      .components()
      .iter()
      .map(|component| {
        Arc::new(InstanceHandler::new(
          schematic.clone(),
          component,
          providers.clone(),
          self_provider.clone(),
        ))
      })
      .collect();

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
      id,
    }
  }

  pub(crate) fn id(&self) -> Uuid {
    self.id
  }

  pub(crate) fn schematic_name(&self) -> &str {
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
    self.instances.iter().filter(|i| i.is_core_component(SENDER_ID))
  }

  pub(crate) fn generators(&self) -> impl Iterator<Item = &Arc<InstanceHandler>> {
    self.instances.iter().filter(|i| i.is_generator())
  }

  pub(crate) fn done(&self) -> bool {
    let output_handler = self.instance(self.schematic.output().index());
    let status = check_statuses(output_handler.inputs().handlers());
    let ports_look_done = !status.has_any_open();

    let any_pending = self.instances.iter().any(|instance| instance.is_pending());
    trace!(any_pending, ?status, ?ports_look_done, "checking done");
    ports_look_done && !any_pending
  }

  pub(crate) async fn start(&mut self) -> Result<()> {
    self.stats.mark("start");
    self.stats.start("execution");
    let span = trace_span!("transaction", id = self.id.to_string().as_str());
    let _guard = span.enter();
    trace!("starting transaction");
    self.start_time = Instant::now();

    self
      .prime_input_ports(self.schematic.input().index(), &self.invocation.payload)
      .await?;

    let inherent_data = self.invocation.inherent.unwrap_or_else(|| InherentData {
      seed: rand::thread_rng().gen(),
      timestamp: SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs(),
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
    for (name, payload) in payload.iter() {
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
      self.invoke_component(instance.index(), TransportMap::default()).await?;
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
    trace!("readying provider ref '{}'", instance.id());
    self.invoke_component(instance.index(), TransportMap::default()).await
  }

  pub(crate) fn finish(mut self) -> Result<TransactionStatistics> {
    self.stats.end("execution");
    #[cfg(test)]
    self.stats.print();

    Ok(self.stats)
  }

  pub(crate) async fn emit_output_message(&self, data: TransportWrapper) -> Result<()> {
    debug!(?data, "emitting tx output");
    self.output.push(data).await?;
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

  pub(crate) async fn take_payload(&self, instance: &InstanceHandler) -> Result<Option<TransportMap>> {
    for port in instance.inputs().refs() {
      // check if any of this component's input ports are empty.
      if instance.is_port_empty(&port) {
        let walker = self.schematic.walk_from_port(port, WalkDirection::Up);
        // If any are, walk back up and kick any generators in our upstream.
        for hop in walker {
          match hop {
            SchematicHop::Component(c) => {
              let instance = self.instance(c.index());
              if instance.is_generator() {
                self.kick_generator(instance).await?;
              }
            }
            _ => continue,
          }
        }
      }
    }
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

  pub(crate) async fn accept_inputs(&self, port: &PortReference, msgs: Vec<TransportWrapper>) -> Result<()> {
    for payload in msgs {
      let instance = self.instance(port.component_index());
      let action = instance.buffer_in(port, payload, &self.instances)?;
      if action == BufferAction::Buffered {
        self.channel.dispatch(Event::port_data(self.id, *port)).await?;
      }
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

  pub(crate) fn is_output_port_done<T: AsRef<PortReference>>(&self, port: T) -> Result<bool> {
    let port_ref = port.as_ref();
    let instance = self.instance(port_ref.component_index());
    let status = instance.get_port_status(port_ref);
    trace!(?status, "port status");
    Ok(status == PortStatus::DoneClosed || status == PortStatus::DoneYield)
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

  #[instrument(skip(self, payload), name = "invoke_component")]
  pub(crate) async fn invoke_component(&self, index: ComponentIndex, payload: TransportMap) -> Result<()> {
    let tx_id = self.id();

    let instance = self.instance(index).clone();
    debug!(id = instance.id(), ?payload);

    if payload.has_error() {
      let err = payload.take_error().unwrap();
      return self.handle_short_circuit(instance.index(), err).await;
    }

    let invocation = Invocation::new(
      Entity::schematic(self.schematic_name()),
      instance.entity(),
      payload,
      self.invocation.inherent,
    );

    instance
      .handle_component_call(tx_id, invocation, self.channel.clone())
      .await
  }

  #[instrument(skip(self), name = "schematic_output")]
  pub(crate) async fn handle_schematic_output(&self, port: &PortReference) -> Result<()> {
    let name = self.schematic.get_port_name(port);
    debug!( port=?port, name=name);

    let message = self.take_output(port)?;

    self.emit_output_message(message).await?;

    if self.is_output_port_done(port)? {
      self.emit_output_message(TransportWrapper::done(name)).await?;
    }

    if self.done() {
      self.channel.dispatch(Event::tx_done(self.id())).await?;
    }

    Ok(())
  }

  #[async_recursion::async_recursion]
  pub(crate) async fn propagate_status(&self, port: PortReference) -> Result<()> {
    let walker = Port::new(&self.schematic, port);

    match port.direction() {
      PortDirection::In => {
        let instance = self.instance(port.component_index());
        let updated = instance.update_output_statuses();
        let output_index = self.output_handler().index();

        for port in updated {
          if port.component_index() != output_index {
            self.propagate_status(port).await?;
          }
        }
      }
      PortDirection::Out => {
        for connection in walker.connections() {
          let downport = connection.to();
          let instance = self.instance(downport.component().index());
          let updated = instance.update_input_status(downport.as_ref(), &self.instances);
          let output_index = self.output_handler().index();

          if let Some(updated) = updated {
            if updated.component_index() == output_index {
              // If we updated a schematic output, then we need to generate
              // a done message explicitly.
              trace!(?port, "closed output port");
              let name = self.schematic.get_port_name(updated);
              self.emit_output_message(TransportWrapper::done(name)).await?;
            } else {
              self.propagate_status(port).await?;
            }
          }
        }
      }
    }
    Ok(())
  }

  #[cfg(test)]
  pub(crate) fn debug_status(&self) -> Vec<serde_json::Value> {
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
            let packets = instance.clone_packets(&port_ref);
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
      let packets = instance.clone_packets();

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
      lines.push(serde_json::json!({"type":"pending","component_index":instance.index(),"num":instance.num_pending()}));
    }
    lines
  }
}
