use std::collections::{
  HashMap,
  VecDeque,
};
use std::sync::{
  Arc,
  Mutex,
};

use tokio::sync::mpsc::{
  unbounded_channel,
  UnboundedReceiver,
};
use vino_transport::message_transport::MessageSignal;
use vino_transport::MessageTransport;

use crate::error::SchematicError;
use crate::schematic::{
  PayloadReceived,
  ReferenceReady,
  SchematicOutputReceived,
  TransactionUpdate,
};
use crate::schematic_model::SchematicModel;
use crate::{
  PortEntity,
  Result,
};

type TransactionId = String;

#[derive(Debug)]
pub(crate) struct TransactionMap {
  model: SchematicModel,
  map: HashMap<TransactionId, Arc<Mutex<Transaction>>>,
}

impl TransactionMap {
  pub(crate) fn new(model: SchematicModel) -> Self {
    Self {
      model,
      map: HashMap::new(),
    }
  }
  pub(crate) fn new_transaction(
    &mut self,
    tx_id: String,
    mut rx: UnboundedReceiver<PayloadReceived>,
  ) -> UnboundedReceiver<TransactionUpdate> {
    let transaction = Arc::new(Mutex::new(Transaction::new(
      tx_id.clone(),
      self.model.clone(),
    )));
    self.map.insert(tx_id.clone(), transaction.clone());
    let (ready_tx, ready_rx) = unbounded_channel::<TransactionUpdate>();
    tokio::spawn(async move {
      trace!("Waiting for outputs on tx {}", tx_id);
      while let Some(output) = rx.recv().await {
        trace!(
          ">>> Received message from {} to {} on tx {} : {:?} ",
          output.origin,
          output.target,
          tx_id,
          output.payload
        );
        let mut locked = transaction.lock()?;
        locked.receive(output.origin.clone(), output.target.clone(), output.payload);

        if output.target.reference == crate::SCHEMATIC_OUTPUT {
          debug!("Received schematic output, pushing immediately...");
          if let Some(payload) = locked.take_from_port(&output.target) {
            meh!(ready_tx.send(TransactionUpdate::SchematicOutput(
              SchematicOutputReceived {
                tx_id: tx_id.clone(),
                port: output.target.name,
                payload,
              }
            )));
          }
          for port in &locked.output_ports {
            if locked.has_active_upstream(port)? {
              debug!("Schematic still waiting for upstreams to finish");
              continue;
            }
          }
          // If all connections to the schematic outputs are closed, then finish up.
          debug!("Sending schematic done");
          meh!(ready_tx.send(TransactionUpdate::SchematicDone(tx_id.clone())));
          rx.close();
          continue;
        }

        if locked.is_reference_ready(&output.target) {
          debug!("Reference {} is ready, continuing...", output.target);
          let map = locked.take_inputs(&output.target.reference)?;
          drop(locked);
          meh!(
            ready_tx.send(TransactionUpdate::ReferenceReady(ReferenceReady {
              tx_id: tx_id.clone(),
              reference: output.target.reference,
              payload_map: map
            }))
          );
        } else {
          debug!("Reference is not ready, waiting...")
        }
      }
      debug!("Transaction {} finishing up", tx_id);
      Ok!(())
    });
    ready_rx
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
  model: SchematicModel,
  port_statuses: HashMap<PortEntity, PortStatus>,
  output_ports: Vec<PortEntity>,
}

impl Transaction {
  fn new(tx_id: TransactionId, model: SchematicModel) -> Self {
    let port_statuses = model
      .connections
      .iter()
      .flat_map(|conn| {
        [
          (conn.to.clone(), PortStatus::Open),
          (conn.from.clone(), PortStatus::Open),
        ]
      })
      .collect();
    let output_ports = model
      .get_schematic_outputs()
      .into_iter()
      .map(|n| PortEntity {
        reference: crate::SCHEMATIC_OUTPUT.to_string(),
        name: n,
      })
      .collect();
    Self {
      tx_id,
      buffermap: BufferMap::default(),
      model,
      port_statuses,
      output_ports,
    }
  }
  pub(crate) fn has_active_upstream(&self, port: &PortEntity) -> Result<bool> {
    let upstream = self
      .model
      .get_upstream(port)
      .ok_or_else(|| SchematicError::UpstreamNotFound(port.clone()))?;
    Ok(self.has_data(port) || !self.is_closed(upstream))
  }
  fn receive(&mut self, from: PortEntity, to: PortEntity, payload: MessageTransport) {
    if let MessageTransport::Signal(signal) = payload {
      match signal {
        MessageSignal::Close => {
          trace!("Port {} closing", from);
          self.port_statuses.insert(from, PortStatus::Closed);
        }
        MessageSignal::OpenBracket => panic!("Not implemented"),
        MessageSignal::CloseBracket => panic!("Not implemented"),
      }
    } else {
      debug!("Pushing to port buffer {}", to);
      self.buffermap.push(to, payload);
    }
  }
  fn has_data(&self, port: &PortEntity) -> bool {
    self.buffermap.has_data(port)
  }
  fn is_closed(&self, port: &PortEntity) -> bool {
    self
      .port_statuses
      .get(port)
      .map(|status| status == &PortStatus::Closed)
      .unwrap_or(true)
  }
  fn take_from_port(&mut self, port: &PortEntity) -> Option<MessageTransport> {
    self.buffermap.take(port)
  }
  fn get_connected_ports(&self, reference: &str) -> Vec<PortEntity> {
    self
      .model
      .connections
      .iter()
      .filter(|conn| conn.to.reference == reference)
      .map(|conn| conn.to.clone())
      .collect()
  }
  fn take_inputs(&mut self, reference: &str) -> Result<HashMap<String, MessageTransport>> {
    let ports = self.get_connected_ports(reference);

    let mut map = HashMap::new();
    for port in ports {
      let message = self.take_from_port(&port).ok_or_else(|| {
        crate::Error::SchematicError("Tried to take from port that had none".to_string())
      })?;
      map.insert(port.name, message);
    }
    Ok(map)
  }
  fn is_reference_ready(&self, port: &PortEntity) -> bool {
    let reference = port.reference.clone();
    let ports = self.get_connected_ports(&reference);
    debug!("ref: {}, ports: {:?}", reference, ports);
    self.are_ports_ready(&ports)
  }
  fn is_port_ready(&self, port: &PortEntity) -> bool {
    debug!("Is port {} ready? {}", port, self.buffermap.has_data(port));
    self.buffermap.has_data(port)
  }
  fn are_ports_ready(&self, ports: &[PortEntity]) -> bool {
    itertools::all(ports, |ent| self.is_port_ready(ent))
  }
}

#[derive(Debug, Default)]
struct BufferMap {
  map: HashMap<PortEntity, PortBuffer>,
}

impl BufferMap {
  fn push(&mut self, port: PortEntity, payload: MessageTransport) {
    let queue = self.map.entry(port).or_insert_with(PortBuffer::default);
    queue.push_back(payload);
  }

  fn has_data(&self, port: &PortEntity) -> bool {
    self
      .map
      .get(port)
      .map(|buf| buf.has_data())
      .unwrap_or(false)
  }
  fn take(&mut self, port: &PortEntity) -> Option<MessageTransport> {
    self
      .map
      .get_mut(port)
      .map(|buff| buff.pop_front())
      .flatten()
  }
}

#[derive(Debug, Default)]
struct PortBuffer {
  buffer: VecDeque<MessageTransport>,
}

impl PortBuffer {
  fn push_back(&mut self, payload: MessageTransport) {
    self.buffer.push_back(payload)
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
  use tokio::sync::mpsc::unbounded_channel;
  use vino_component::v0::Payload;
  use vino_component::Packet;
  use vino_transport::message_transport::MessageSignal;

  use super::*;
  use crate::prelude::*;
  use crate::schematic_definition::*;

  fn make_model() -> SchematicModel {
    let schematic_name = "Test";
    let mut schematic_def = SchematicDefinition::new(schematic_name.to_string());
    schematic_def.providers.push(ProviderDefinition {
      namespace: "test-namespace".to_string(),
      kind: ProviderKind::Native,
      reference: "internal".to_string(),
      data: HashMap::new(),
    });
    schematic_def.components.insert(
      "REF_ID_LOGGER".to_string(),
      ComponentDefinition {
        metadata: None,
        id: "test-namespace::log".to_string(),
      },
    );
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: SCHEMATIC_INPUT.to_string(),
        port: "input".to_string(),
      },
      to: ConnectionTargetDefinition {
        instance: "REF_ID_LOGGER".to_string(),
        port: "input".to_string(),
      },
    });
    schematic_def.connections.push(ConnectionDefinition {
      from: ConnectionTargetDefinition {
        instance: "REF_ID_LOGGER".to_string(),
        port: "output".to_string(),
      },
      to: ConnectionTargetDefinition {
        instance: SCHEMATIC_OUTPUT.to_string(),
        port: "output".to_string(),
      },
    });
    SchematicModel::new(schematic_def)
  }

  #[test_env_log::test]
  fn test_transaction() -> Result<()> {
    let tx_id = Invocation::uuid();
    let model = make_model();

    let mut transaction = Transaction::new(tx_id, model);
    let from = PortEntity {
      reference: "REF_ID_LOGGER1".into(),
      name: "vino::log".into(),
    };
    let to = PortEntity {
      reference: "REF_ID_LOGGER2".into(),
      name: "vino::log".into(),
    };
    trace!("pushing to port");
    transaction.receive(
      from.clone(),
      to.clone(),
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
      from,
      to,
      Packet::V0(Payload::Exception("oh no".into())).into(),
    );
    Ok(())
  }

  #[test_env_log::test(tokio::test)]
  async fn test_transaction_map() -> Result<()> {
    let model = make_model();
    // TODO NEED TO MAKE THIS BORROWED
    let mut map = TransactionMap::new(model);
    let tx_id = Invocation::uuid();
    let (tx, rx) = unbounded_channel::<PayloadReceived>();
    let mut ready_rx = map.new_transaction(tx_id.clone(), rx);

    // First message sends from the schematic input to the component
    meh!(tx.send(PayloadReceived {
      origin: PortEntity {
        reference: SCHEMATIC_INPUT.to_string(),
        name: "input".to_string(),
      },
      target: PortEntity {
        reference: "REF_ID_LOGGER".to_string(),
        name: "input".to_string(),
      },
      payload: MessageTransport::Test("input payload".to_string()),
      tx_id: Invocation::uuid(),
    }));
    // Second closes the schematic input
    meh!(tx.send(PayloadReceived {
      origin: PortEntity {
        reference: SCHEMATIC_INPUT.to_string(),
        name: "input".to_string(),
      },
      target: PortEntity {
        reference: "REF_ID_LOGGER".to_string(),
        name: "input".to_string(),
      },
      payload: MessageTransport::Signal(MessageSignal::Close),
      tx_id: Invocation::uuid(),
    }));
    // Third simulates output from the component
    meh!(tx.send(PayloadReceived {
      origin: PortEntity {
        reference: "REF_ID_LOGGER".to_string(),
        name: "output".to_string(),
      },
      target: PortEntity {
        reference: SCHEMATIC_OUTPUT.to_string(),
        name: "output".to_string(),
      },
      payload: MessageTransport::Test("output payload".to_string()),
      tx_id: Invocation::uuid(),
    }));
    meh!(tx.send(PayloadReceived {
      origin: PortEntity {
        reference: "REF_ID_LOGGER".to_string(),
        name: "output".to_string(),
      },
      target: PortEntity {
        reference: SCHEMATIC_OUTPUT.to_string(),
        name: "output".to_string(),
      },
      payload: MessageTransport::Signal(MessageSignal::Close),
      tx_id: Invocation::uuid(),
    }));
    // Transaction should close automatically after this because all ports
    // are drained and closed.
    let handle = tokio::spawn(async move {
      let mut msgs = vec![];
      while let Some(payloadmsg) = ready_rx.recv().await {
        msgs.push(payloadmsg);
      }
      msgs
    });
    let msgs = handle.await.unwrap();

    assert_eq!(msgs.len(), 4);

    Ok(())
  }
}
