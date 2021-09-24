use std::collections::VecDeque;
use std::time::Duration;

use vino_transport::message_transport::TransportMap;

use self::executor::SchematicOutput;
use self::ports::PortStatuses;
use crate::dev::prelude::*;
use crate::schematic_service::handlers::component_payload::ComponentPayload;
use crate::schematic_service::input_message::InputMessage;
type Result<T> = std::result::Result<T, TransactionError>;

pub(crate) mod executor;
pub(crate) mod ports;

#[derive(Debug)]
pub enum TransactionUpdate {
  Drained,
  Error(String),
  Timeout(Duration),
  Transition(ConnectionTargetDefinition),
  Execute(ComponentPayload),
  Result(SchematicOutput),
  Done(String),
  Update(InputMessage),
}

impl std::fmt::Display for TransactionUpdate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let name = match self {
      TransactionUpdate::Drained => "drained",
      TransactionUpdate::Error(_) => "error",
      TransactionUpdate::Timeout(_) => "timeout",
      TransactionUpdate::Transition(_) => "transition",
      TransactionUpdate::Execute(_) => "execute",
      TransactionUpdate::Result(_) => "result",
      TransactionUpdate::Done(_) => "done",
      TransactionUpdate::Update(_) => "update",
    };
    f.write_str(name)
  }
}

#[derive(Debug)]
struct Transaction {
  tx_id: String,
  model: SharedModel,
  ports: PortStatuses,
  output_ports: Vec<ConnectionTargetDefinition>,
  schematic_name: String,
  senders: Vec<ConnectionDefinition>,
  generators: Vec<ConnectionDefinition>,
}

impl Transaction {
  fn new(tx_id: String, model: SharedModel) -> Self {
    let ports = PortStatuses::new(tx_id.clone(), &model);
    let readable = model.read();
    let senders: Vec<_> = readable.get_senders().cloned().collect();
    let generators: Vec<_> = readable.get_generators().cloned().collect();
    let schematic_name = readable.get_name();
    let output_ports = readable.get_schematic_outputs().cloned().collect();
    drop(readable);
    Self {
      tx_id,
      model,
      ports,
      output_ports,
      schematic_name,
      senders,
      generators,
    }
  }

  pub(crate) fn log_prefix(&self) -> String {
    format!("TX:{}({}):", self.tx_id, self.schematic_name)
  }

  pub(crate) fn has_active_upstream(&self, port: &ConnectionTargetDefinition) -> bool {
    let model = self.model.read();
    let upstream = some_or_bail!(model.get_upstream(port), false);
    let has_data = self.ports.has_data(port);
    let is_closed = self.ports.is_closed(upstream);
    let is_sender = upstream.is_sender();
    let is_generator = self.ports.is_generator(upstream.get_instance());
    let active = has_data || (!is_closed && !is_sender && !is_generator);
    trace!(
      "TX:PORT:[{}]:HAS_DATA[{}]:IS_OPEN[{}]:IS_SENDER[{}]:IS_GENERATOR[{}]:IS_ACTIVE[{}]",
      upstream,
      has_data,
      !is_closed,
      is_sender,
      is_generator,
      active
    );
    active
  }

  fn is_done(&self) -> bool {
    for port in &self.output_ports {
      if !self.ports.is_closed(port) {
        return false;
      }
    }
    true
  }

  fn check_senders(&mut self) -> VecDeque<TransactionUpdate> {
    let mut messages = VecDeque::new();

    for sender in &self.senders {
      if self.ports.is_waiting(&sender.to) {
        self.ports.set_idle(&sender.to);
        match sender.from.get_data() {
          Some(data) => {
            messages.push_back(TransactionUpdate::Update(InputMessage {
              connection: sender.clone(),
              payload: data.as_message(),
              tx_id: self.tx_id.clone(),
            }));
          }
          None => {
            debug!("{}{:?}", self.log_prefix(), sender);
            error!("Schematic '{}' has a sender defined for connection '{}' but has no data to send. This is likely a bug in the schematic.", self.schematic_name, sender);
          }
        }
      }
    }

    for generator in &self.generators {
      if self.ports.is_waiting(&generator.to) {
        self.ports.set_idle(&generator.to);
        messages.push_back(TransactionUpdate::Execute(ComponentPayload {
          tx_id: self.tx_id.clone(),
          instance: generator.from.get_instance_owned(),
          payload_map: TransportMap::new(),
        }));
      }
    }
    messages
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::sync::Arc;
  use std::time::Duration;

  use parking_lot::RwLock;
  use vino_packet::packet::v0::Payload;
  use vino_packet::Packet;

  use super::*;
  use crate::schematic_service::input_message::InputMessage;
  #[allow(unused_imports)]
  use crate::test::prelude::{
    assert_eq,
    *,
  };
  use crate::transaction::executor::TransactionExecutor;
  fn make_model() -> TestResult<Arc<RwLock<SchematicModel>>> {
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
    schematic_def
      .connections
      .push(ConnectionDefinition::from_v0_str(
        "<>=>REF_ID_LOGGER[input]",
      )?);
    schematic_def
      .connections
      .push(ConnectionDefinition::from_v0_str(
        "REF_ID_LOGGER[output]=><>",
      )?);
    Ok(Arc::new(RwLock::new(SchematicModel::try_from(
      schematic_def,
    )?)))
  }

  #[test_logger::test]
  fn test_transaction() -> TestResult<()> {
    let tx_id = get_uuid();
    let model = make_model()?;

    let mut transaction = Transaction::new(tx_id, model);
    let from = ConnectionTargetDefinition::new("<input>", "input");
    let to = ConnectionTargetDefinition::new("REF_ID_LOGGER", "input");

    println!("pushing to port");
    let connection = ConnectionDefinition::new(from, to.clone());
    transaction
      .ports
      .receive(&connection, Packet::V0(Payload::MessagePack(vec![])).into());
    assert!(transaction.ports.is_port_ready(&to));
    println!("taking from port");
    let output = transaction.ports.take_from_port(&to);
    assert_eq!(
      output,
      Some(MessageTransport::Success(Success::MessagePack(vec![])))
    );
    transaction.ports.receive(
      &connection,
      Packet::V0(Payload::Exception("!!".into())).into(),
    );
    let output = transaction.ports.take_from_port(&to);
    assert!(matches!(
      output,
      Some(MessageTransport::Failure(Failure::Exception(_)))
    ));

    Ok(())
  }

  fn conn(from_name: &str, from_port: &str, to_name: &str, to_port: &str) -> ConnectionDefinition {
    ConnectionDefinition {
      from: ConnectionTargetDefinition::new(from_name, from_port),
      to: ConnectionTargetDefinition::new(to_name, to_port),
      default: None,
    }
  }

  #[test_logger::test(tokio::test)]
  async fn test_transaction_map() -> TestResult<()> {
    let model = make_model()?;

    let mut map = TransactionExecutor::new(model, Duration::from_millis(100));
    let tx_id = get_uuid();
    let (mut ready_rx, tx) = map.new_transaction(tx_id.clone());

    // First message sends from the schematic input to the component
    tx.send(TransactionUpdate::Update(InputMessage {
      connection: conn(SCHEMATIC_INPUT, "input", "REF_ID_LOGGER", "input"),
      payload: MessageTransport::success(&"input payload"),
      tx_id: tx_id.clone(),
    }))?;

    // Second closes the schematic input
    tx.send(TransactionUpdate::Update(InputMessage {
      connection: conn(SCHEMATIC_INPUT, "input", "REF_ID_LOGGER", "input"),
      payload: MessageTransport::Signal(MessageSignal::Done),
      tx_id: tx_id.clone(),
    }))?;

    // Third simulates output from the component
    tx.send(TransactionUpdate::Update(InputMessage {
      connection: conn("REF_ID_LOGGER", "output", SCHEMATIC_OUTPUT, "output"),
      payload: MessageTransport::success(&"output payload"),
      tx_id: tx_id.clone(),
    }))?;

    // Second closes the schematic input
    tx.send(TransactionUpdate::Update(InputMessage {
      connection: conn("REF_ID_LOGGER", "output", SCHEMATIC_OUTPUT, "output"),
      payload: MessageTransport::Signal(MessageSignal::Done),
      tx_id: tx_id.clone(),
    }))?;

    // Transaction should close automatically after this because the schematic
    // is complete

    let handle = tokio::spawn(async move {
      let mut msgs = vec![];
      while let Some(payloadmsg) = ready_rx.recv().await {
        println!("Got message : {:?}", payloadmsg);
        msgs.push(payloadmsg);
      }
      msgs
    });
    let msgs = handle.await?;
    println!("Transaction Updates {:#?}", msgs);

    // 1 execute the component
    assert!(matches!(msgs[0], TransactionUpdate::Execute(_)));
    // 2 get result for schematic
    assert!(matches!(msgs[1], TransactionUpdate::Result(_)));
    // 3 get done signal for schematic port
    assert!(matches!(msgs[2], TransactionUpdate::Result(_)));
    // 4 get done update for schematic transaction
    assert!(matches!(msgs[3], TransactionUpdate::Done(_)));
    assert_eq!(msgs.len(), 4);

    Ok(())
  }
}
