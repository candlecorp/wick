use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use tokio_stream::StreamExt;
use uuid::Uuid;
use vino_entity::Entity;
use vino_schematic_graph::{ComponentIndex, ComponentKind, ExternalReference, PortDirection, PortReference};
use vino_transport::{Invocation, MessageSignal, MessageTransport, TransportMap, TransportWrapper};

use self::port::port_handler::{BufferAction, PortHandler};
use self::port::{InputPorts, OutputPorts, PortStatus};
use crate::graph::types::*;
use crate::interpreter::channel::Event;
use crate::interpreter::error::StateError;
use crate::interpreter::provider::core_provider::CORE_PROVIDER_NS;
use crate::interpreter::provider::internal_provider::INTERNAL_PROVIDER_NS;
use crate::interpreter::provider::provider_provider::PROVIDERPROVIDER_NAMESPACE;
use crate::interpreter::provider::schematic_provider::SELF_NAMESPACE;
use crate::{ExecutionError, InterpreterDispatchChannel, Provider, Providers};
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
  providers: Arc<Providers>,
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

#[derive(Debug)]
#[must_use]
struct Reference {
  name: String,
  namespace: String,
}

impl From<&ExternalReference> for Reference {
  fn from(v: &ExternalReference) -> Self {
    Self {
      name: v.name().to_owned(),
      namespace: v.namespace().to_owned(),
    }
  }
}

impl InstanceHandler {
  pub(super) fn new(
    schematic: Arc<Schematic>,
    component: &Component,
    providers: Arc<Providers>,
    self_provider: Arc<dyn Provider + Send + Sync>,
  ) -> Self {
    let inputs = component.inputs().to_vec();
    let outputs = component.outputs().to_vec();
    let reference = match component.kind() {
      ComponentKind::Input => Reference {
        name: component.name().to_owned(),
        namespace: INTERNAL_PROVIDER_NS.to_owned(),
      },
      ComponentKind::Output => Reference {
        name: component.name().to_owned(),
        namespace: INTERNAL_PROVIDER_NS.to_owned(),
      },
      ComponentKind::External(comp) => comp.into(),
      ComponentKind::Inherent(comp) => comp.into(),
    };

    Self {
      schematic,
      inputs: InputPorts::new(inputs),
      outputs: OutputPorts::new(outputs),
      reference,
      index: component.index(),
      identifier: component.name().to_owned(),
      providers,
      pending: AtomicU32::new(0),
      self_provider,
    }
  }

  pub(crate) fn entity(&self) -> Entity {
    Entity::component(&self.reference.namespace, &self.reference.name)
  }

  pub(crate) fn namespace(&self) -> &str {
    &self.reference.namespace
  }

  pub(crate) fn index(&self) -> ComponentIndex {
    self.index
  }

  pub(crate) fn id(&self) -> &str {
    &self.identifier
  }

  pub(crate) fn is_core_component(&self, name: &str) -> bool {
    self.reference.namespace == CORE_PROVIDER_NS && self.reference.name == name
  }

  pub(crate) fn is_generator(&self) -> bool {
    self.reference.namespace == PROVIDERPROVIDER_NAMESPACE
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
      if !payload.contains(input.name()) {
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
      trace!(?connection);
      let connection = &self.schematic.connections()[*connection];
      let upstream_instance = &instances[connection.from().component_index()];
      upstream_instance.outputs.get_handler(connection.from())
    });
    trace!(?upstream_ports);

    let statuses = check_statuses(upstream_ports);

    let num_buffered = self.buffered_packets(port);
    trace!(?statuses, is_pending = self.is_pending());
    let new_status = if statuses.has_open || statuses.has_done_open {
      PortStatus::DoneOpen
    } else if statuses.has_any_generator {
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

  #[instrument(skip_all, name = "buffer_in")]
  pub(crate) fn buffer_in(
    &self,
    port: &PortReference,
    value: TransportWrapper,
    instances: &[Arc<InstanceHandler>],
  ) -> Result<BufferAction> {
    trace!(?port, ?value, "buffering input message");

    if value.payload == MessageTransport::Signal(MessageSignal::Done) {
      self.update_input_status(port, instances);
      Ok(BufferAction::Consumed)
    } else {
      self.inputs.receive(port, value)
    }
  }

  #[instrument(skip_all, name = "buffer_out")]
  pub(crate) fn buffer_out(&self, port: &PortReference, value: TransportWrapper) -> Result<BufferAction> {
    trace!(?port, ?value, "buffering output message");
    if value.payload == MessageTransport::Signal(MessageSignal::Done) {
      let status = check_statuses(self.inputs.iter());

      trace!(?status, is_pending = self.is_pending());
      let new_status = if self.is_generator() || status.has_all_generators {
        PortStatus::DoneYield
      } else if status.has_any_open() || self.is_pending() {
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

  #[instrument(skip_all, name = "call_complete")]
  pub(crate) fn handle_call_complete(&self) -> Result<Vec<PortReference>> {
    self.decrement_pending()?;
    Ok(self.update_output_statuses())
  }

  #[instrument(skip_all, name = "update_output_ports")]
  pub(super) fn update_output_statuses(&self) -> Vec<PortReference> {
    let input_status = check_statuses(self.inputs.handlers());
    debug!(?input_status);
    let mut changed_statuses = Vec::new();
    for port in self.outputs.iter() {
      let current_status = port.status();
      let new_status = match current_status {
        PortStatus::DoneOpen | PortStatus::Open => {
          // Leave the port DoneOpen if we have any open inputs or we're still pending
          // otherwise close it.
          // Note: A port can still be "Open" after a call if the component panics.
          // Treat it the same way as DoneOpen
          if input_status.has_any_open()
            || self.is_pending()
            || input_status.has_pending_packets
            || input_status.has_all_generators
          {
            PortStatus::DoneOpen
          } else {
            PortStatus::DoneClosed
          }
        }
        orig => orig,
      };
      trace!(?current_status, ?new_status);
      if new_status != current_status {
        changed_statuses.push(port.port_ref());
        port.set_status(new_status);
      }
    }
    trace!(ports=?changed_statuses,"updated downstream ports");
    changed_statuses
  }

  #[instrument(skip(self, invocation), name = "component_call")]
  pub(crate) async fn handle_component_call(
    self: Arc<Self>,
    tx_id: Uuid,
    invocation: Invocation,
    channel: InterpreterDispatchChannel,
  ) -> Result<()> {
    debug!(?invocation);

    let identifier = self.id().to_owned();
    let entity = self.entity();
    let namespace = self.namespace().to_owned();

    let associated_data = self.schematic.components()[self.index()].data();

    let associated_data = associated_data.clone();

    self.increment_pending();

    let fut = if namespace == SELF_NAMESPACE {
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

    let mut stream = match outer_result {
      Ok(Ok(result)) => result,
      Ok(Err(err)) | Err(err) => {
        warn!(error = ?err, "component error");
        channel
          .dispatch(Event::call_err(
            tx_id,
            self.index(),
            MessageTransport::error(err.to_string()),
          ))
          .await?;
        return Ok(());
      }
    };

    let index = self.index();
    tokio::spawn(async move {
      let span = trace_span!(
        "output_task",
        component = format!("{} ({})", identifier.as_str(), entity,).as_str()
      );
      let _guard = span.enter();
      trace!("starting output task");
      while let Some(wrapper) = stream.next().await {
        let port = match self.find_output(&wrapper.port) {
          Ok(port) => port,
          Err(e) => {
            error!(error = e.to_string().as_str());
            continue;
          }
        };

        trace!("received packet for {}", wrapper.port);
        let action = self.buffer_out(&port, wrapper).unwrap();
        if action == BufferAction::Buffered {
          if let Err(e) = channel.dispatch(Event::port_data(tx_id, port)).await {
            error!("could not send packet: {}", e);
          };
        }
      }
      let ports = self.handle_call_complete();
      if let Err(e) = ports {
        error!("error handling call complete: {}", e);
        return;
      }
      let ports = ports.unwrap();
      trace!(?ports, "ports to update");
      if let Err(e) = channel.dispatch(Event::call_complete(tx_id, index)).await {
        error!("could not send event: {}", e);
      };
      for port in ports {
        if let Err(e) = channel.dispatch(Event::port_status_change(tx_id, port)).await {
          error!("could not send port status change event: {}", e);
        };
      }
    });
    Ok(())
  }

  #[cfg(test)]
  pub(crate) fn clone_packets(&self, port: &PortReference) -> Vec<TransportWrapper> {
    match port.direction() {
      PortDirection::In => self.inputs.get_handler(port).clone_packets(),
      PortDirection::Out => self.outputs.get_handler(port).clone_packets(),
    }
  }

  #[cfg(test)]
  pub(crate) fn num_pending(&self) -> u32 {
    self.pending.load(Ordering::Relaxed)
  }
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
  let mut num_generators = 0;
  let mut total_ports = 0;
  for port in ports {
    total_ports += 1;
    info!(?port);
    let upstream_status = port.status();
    if !port.is_empty() {
      breakdown.has_pending_packets = true;
    }
    match upstream_status {
      PortStatus::Open => breakdown.has_open = true,
      PortStatus::DoneOpen => breakdown.has_done_open = true,
      PortStatus::DoneYield => {
        breakdown.has_any_generator = true;
        num_generators += 1;
      }
      PortStatus::DoneClosing => breakdown.has_done_closing = true,
      PortStatus::DoneClosed => breakdown.has_done_closed = true,
    }
  }
  breakdown.has_all_generators = num_generators == total_ports;
  breakdown
}
