use std::collections::{
  HashMap,
  VecDeque,
};
use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::mpsc::{
  unbounded_channel,
  UnboundedReceiver,
  UnboundedSender,
};

use crate::dev::prelude::*;
use crate::schematic_service::handlers::component_payload::ComponentPayload;
use crate::schematic_service::handlers::schematic_output::SchematicOutput;
use crate::schematic_service::handlers::transaction_update::TransactionUpdate;
type Result<T> = std::result::Result<T, TransactionError>;

#[derive(Debug)]
pub(crate) struct TransactionExecutor {
  model: Arc<Mutex<SchematicModel>>,
  // A list of references that don't have inputs and thus won't be triggered
  // by an upstream's output message
  autorun: Vec<String>,
}

impl TransactionExecutor {
  pub(crate) fn new(model: Arc<Mutex<SchematicModel>>) -> Self {
    let locked = model.lock();
    // components that have no inputs need to be run now or lazily later
    let autorun_instances = locked
      .get_instances()
      .filter_map(|instance| {
        locked
          .get_upstream_connections_by_instance(instance)
          .next()
          .is_none()
          .then(|| instance.clone())
      })
      .collect();
    drop(locked);
    debug!("Instances {:?} are lazy", autorun_instances);

    Self {
      model,
      autorun: autorun_instances,
    }
  }
  pub(crate) fn new_transaction(
    &mut self,
    tx_id: String,
  ) -> (
    UnboundedReceiver<TransactionUpdate>,
    UnboundedSender<TransactionUpdate>,
  ) {
    let mut transaction = Transaction::new(tx_id.clone(), self.model.clone());

    let (inbound_tx, inbound_rx) = unbounded_channel::<TransactionUpdate>();
    let (outbound_tx, mut outbound_rx) = unbounded_channel::<TransactionUpdate>();

    // TODO: Instances with no ports should be run lazily, not automatically.
    for instance in &self.autorun {
      ok_or_log!(
        inbound_tx.send(TransactionUpdate::Transition(ComponentPayload {
          tx_id: tx_id.clone(),
          instance: instance.clone(),
          payload_map: HashMap::new(),
        }))
      );
    }

    tokio::spawn(async move {
      let log_prefix = format!("TX:{}:", tx_id);
      trace!("{}:Start", log_prefix);
      while let Some(msg) = outbound_rx.recv().await {
        match msg {
          TransactionUpdate::Transition(_) => todo!(),
          TransactionUpdate::Result(_) => todo!(),
          TransactionUpdate::Done(_) => todo!(),
          TransactionUpdate::Update(input) => {
            trace!("{}:Update for {}", log_prefix, input.connection,);
            transaction.receive(input.connection.clone(), input.payload);
            let target = &input.connection.to;
            let port = input.connection.to.get_port();

            if target.matches_instance(SCHEMATIC_OUTPUT) {
              trace!("{}:Result for port {}", log_prefix, port,);

              if let Some(payload) = transaction.take_from_port(target) {
                ok_or_log!(inbound_tx.send(TransactionUpdate::Result(SchematicOutput {
                  tx_id: tx_id.clone(),
                  port: port.to_owned(),
                  payload,
                })));
              }

              for port in &transaction.output_ports {
                if transaction.has_active_upstream(port)? {
                  trace!("{}:Waiting", log_prefix);
                  continue;
                }
              }
              // If all connections to the schematic outputs are closed, then finish up.
              outbound_rx.close();
              ok_or_log!(inbound_tx.send(TransactionUpdate::Done(tx_id.clone())));
            } else if transaction.is_target_ready(target) {
              trace!("{}:Transitioning to {}", log_prefix, target);

              let map = transaction.take_inputs(target)?;
              ok_or_log!(
                inbound_tx.send(TransactionUpdate::Transition(ComponentPayload {
                  tx_id: tx_id.clone(),
                  instance: target.get_instance_owned(),
                  payload_map: map
                }))
              );
            }
          }
        };
      }
      debug!("{}:Done", log_prefix);
      Ok!(())
    });

    (inbound_rx, outbound_tx)
  }
}

#[derive(Debug, PartialEq, Eq)]
enum PortStatus {
  Open,
  Closed,
}

#[derive(Debug)]
struct Transaction {
  tx_id: String,
  buffermap: BufferMap,
  model: Arc<Mutex<SchematicModel>>,
  port_statuses: HashMap<ConnectionTargetDefinition, PortStatus>,
  output_ports: Vec<ConnectionTargetDefinition>,
}

impl Transaction {
  fn new(tx_id: String, model: Arc<Mutex<SchematicModel>>) -> Self {
    let locked = model.lock();
    let port_statuses = locked
      .get_connections()
      .iter()
      .flat_map(|conn| {
        [
          (conn.to.clone(), PortStatus::Open),
          (conn.from.clone(), PortStatus::Open),
        ]
      })
      .collect();
    let output_ports = locked.get_schematic_outputs().cloned().collect();
    drop(locked);
    Self {
      tx_id,
      buffermap: BufferMap::default(),
      model,
      port_statuses,
      output_ports,
    }
  }
  pub(crate) fn has_active_upstream(&self, port: &ConnectionTargetDefinition) -> Result<bool> {
    let locked = self.model.lock();

    let upstream = locked
      .get_upstream(port)
      .ok_or_else(|| TransactionError::UpstreamNotFound(port.clone()))?;
    Ok(self.has_data(port) || !self.is_closed(upstream))
  }
  fn receive(&mut self, connection: ConnectionDefinition, payload: MessageTransport) {
    if let MessageTransport::Signal(signal) = payload {
      match signal {
        MessageSignal::Close => {
          self
            .port_statuses
            .insert(connection.from, PortStatus::Closed);
        }
        MessageSignal::OpenBracket => panic!("Not implemented"),
        MessageSignal::CloseBracket => panic!("Not implemented"),
      }
    } else {
      self.buffermap.push(connection.to, payload);
    }
  }
  fn has_data(&self, port: &ConnectionTargetDefinition) -> bool {
    self.buffermap.has_data(port)
  }
  fn is_closed(&self, port: &ConnectionTargetDefinition) -> bool {
    self
      .port_statuses
      .get(port)
      .map_or(true, |status| status == &PortStatus::Closed)
  }
  fn take_from_port(&mut self, port: &ConnectionTargetDefinition) -> Option<MessageTransport> {
    self.buffermap.take(port)
  }
  fn get_connected_ports(&self, reference: &str) -> Vec<ConnectionTargetDefinition> {
    let locked = self.model.lock();
    locked
      .get_connections()
      .iter()
      .filter(|conn| conn.to.matches_instance(reference))
      .map(|conn| conn.to.clone())
      .collect()
  }
  fn take_inputs(
    &mut self,
    target: &ConnectionTargetDefinition,
  ) -> Result<HashMap<String, MessageTransport>> {
    let ports = self.get_connected_ports(target.get_instance());

    let mut map = HashMap::new();
    for port in ports {
      let message = self.take_from_port(&port).ok_or(InternalError(7001))?;
      map.insert(port.get_port_owned(), message);
    }
    Ok(map)
  }
  fn is_target_ready(&self, port: &ConnectionTargetDefinition) -> bool {
    let ports = self.get_connected_ports(port.get_instance());
    self.are_ports_ready(&ports)
  }
  fn is_port_ready(&self, port: &ConnectionTargetDefinition) -> bool {
    self.buffermap.has_data(port)
  }
  fn are_ports_ready(&self, ports: &[ConnectionTargetDefinition]) -> bool {
    all(ports, |ent| self.is_port_ready(ent))
  }
}

#[derive(Debug, Default)]
struct BufferMap {
  map: HashMap<ConnectionTargetDefinition, PortBuffer>,
}

impl BufferMap {
  fn push(&mut self, port: ConnectionTargetDefinition, payload: MessageTransport) {
    let queue = self.map.entry(port).or_insert_with(PortBuffer::default);
    queue.push_back(payload);
  }

  fn has_data(&self, port: &ConnectionTargetDefinition) -> bool {
    self.map.get(port).map_or(false, PortBuffer::has_data)
  }
  fn take(&mut self, port: &ConnectionTargetDefinition) -> Option<MessageTransport> {
    self.map.get_mut(port).and_then(PortBuffer::pop_front)
  }
}

#[derive(Debug, Default)]
struct PortBuffer {
  buffer: VecDeque<MessageTransport>,
}

impl PortBuffer {
  fn push_back(&mut self, payload: MessageTransport) {
    self.buffer.push_back(payload);
  }
  fn has_data(&self) -> bool {
    !self.buffer.is_empty()
  }

  fn pop_front(&mut self) -> Option<MessageTransport> {
    self.buffer.pop_front()
  }
}
#[cfg(test)]
mod tests {
  use vino_component::packet::v0::Payload;
  use vino_component::Packet;
  use vino_transport::message_transport::MessageSignal;

  use super::*;
  use crate::schematic_service::input_message::InputMessage;
  #[allow(unused_imports)]
  use crate::test::prelude::{
    assert_eq,
    *,
  };
  fn make_model() -> TestResult<Arc<Mutex<SchematicModel>>> {
    let schematic_name = "Test";
    let mut schematic_def = new_schematic(schematic_name);
    schematic_def.providers.push(ProviderDefinition {
      namespace: "test-namespace".to_owned(),
      kind: ProviderKind::Native,
      reference: "internal".to_owned(),
      data: HashMap::new(),
    });
    schematic_def.instances.insert(
      "REF_ID_LOGGER".to_owned(),
      ComponentDefinition::new("test-namespace", "log"),
    );
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition::new(SCHEMATIC_INPUT, "input"),
      to: ConnectionTargetDefinition::new("REF_ID_LOGGER", "input"),
      default: None,
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition::new("REF_ID_LOGGER", "output"),
      to: ConnectionTargetDefinition::new(SCHEMATIC_OUTPUT, "output"),
      default: None,
    });
    Ok(Arc::new(Mutex::new(SchematicModel::try_from(
      schematic_def,
    )?)))
  }

  #[test_env_log::test]
  fn test_transaction() -> TestResult<()> {
    let tx_id = get_uuid();
    let model = make_model()?;

    let mut transaction = Transaction::new(tx_id, model);
    let from = ConnectionTargetDefinition::new("REF_ID_LOGGER1", "vino::v0::log");
    let to = ConnectionTargetDefinition::new("REF_ID_LOGGER2", "vino::v0::log");

    trace!("pushing to port");
    transaction.receive(
      ConnectionDefinition::new(from.clone(), to.clone()),
      Packet::V0(Payload::MessagePack(vec![])).into(),
    );
    assert!(transaction.is_port_ready(&to));
    trace!("taking from port");
    let output = transaction.take_from_port(&to);
    assert_eq!(
      output,
      Some(Packet::V0(Payload::MessagePack(vec![])).into())
    );
    transaction.receive(
      ConnectionDefinition::new(from, to),
      Packet::V0(Payload::Exception("oh no".into())).into(),
    );
    Ok(())
  }

  fn conn(from_name: &str, from_port: &str, to_name: &str, to_port: &str) -> ConnectionDefinition {
    ConnectionDefinition {
      from: ConnectionTargetDefinition::new(from_name, from_port),
      to: ConnectionTargetDefinition::new(to_name, to_port),
      default: None,
    }
  }

  #[test_env_log::test(tokio::test)]
  async fn test_transaction_map() -> TestResult<()> {
    let model = make_model()?;

    let mut map = TransactionExecutor::new(model);
    let tx_id = get_uuid();
    let (mut ready_rx, tx) = map.new_transaction(tx_id.clone());

    // First message sends from the schematic input to the component
    ok_or_log!(tx.send(TransactionUpdate::Update(InputMessage {
      connection: conn(SCHEMATIC_INPUT, "input", "REF_ID_LOGGER", "input"),
      payload: MessageTransport::Test("input payload".to_owned()),
      tx_id: get_uuid(),
    })));

    // Second closes the schematic input
    ok_or_log!(tx.send(TransactionUpdate::Update(InputMessage {
      connection: conn(SCHEMATIC_INPUT, "input", "REF_ID_LOGGER", "input"),
      payload: MessageTransport::Signal(MessageSignal::Close),
      tx_id: get_uuid(),
    })));

    // Third simulates output from the component
    ok_or_log!(tx.send(TransactionUpdate::Update(InputMessage {
      connection: conn("REF_ID_LOGGER", "output", SCHEMATIC_OUTPUT, "output"),
      payload: MessageTransport::Test("output payload".to_owned()),
      tx_id: get_uuid(),
    })));

    // Fourth closes the output
    ok_or_log!(tx.send(TransactionUpdate::Update(InputMessage {
      connection: conn("REF_ID_LOGGER", "output", SCHEMATIC_OUTPUT, "output"),
      payload: MessageTransport::Signal(MessageSignal::Close),
      tx_id: get_uuid(),
    })));

    // Transaction should close automatically after this because all ports
    // are drained and closed.

    let handle = tokio::spawn(async move {
      let mut msgs = vec![];
      while let Some(payloadmsg) = ready_rx.recv().await {
        msgs.push(payloadmsg);
      }
      msgs
    });
    let msgs = handle.await?;

    assert_eq!(msgs.len(), 4);

    Ok(())
  }
}
