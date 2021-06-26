use std::collections::{
  HashMap,
  VecDeque,
};

use vino_transport::MessageTransport;

use crate::{
  PortEntity,
  Result,
};

type TransactionId = String;

#[derive(Debug)]
struct Transaction {
  tx_id: String,
  buffermap: BufferMap,
}

impl Transaction {
  fn new(tx_id: TransactionId) -> Self {
    Self {
      tx_id,
      buffermap: BufferMap::default(),
    }
  }
  fn push_to_port(&mut self, port: PortEntity, payload: MessageTransport) {
    self.buffermap.push(port, payload)
  }
  fn take_from_port(&mut self, port: &PortEntity) -> Option<MessageTransport> {
    self.buffermap.take(port)
  }
  fn is_port_ready(&self, port: &PortEntity) -> bool {
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
#[derive(Debug, Default)]
pub(crate) struct TransactionMap {
  map: HashMap<TransactionId, Transaction>,
}

impl TransactionMap {
  pub(crate) fn new() -> Self {
    Self::default()
  }
  pub(crate) fn new_transaction(&mut self, tx_id: String) {
    self.map.insert(tx_id.clone(), Transaction::new(tx_id));
  }
  pub(crate) fn push_to_port(
    &mut self,
    tx_id: &str,
    port: PortEntity,
    payload: MessageTransport,
  ) -> Result<()> {
    let transaction = self.map.get_mut(tx_id).ok_or_else(|| {
      crate::Error::SchematicError(format!(
        "Invalid state: no transaction map found for tx {}",
        tx_id
      ))
    })?;
    transaction.push_to_port(port, payload);
    Ok(())
  }
  pub(crate) fn are_ports_ready(&self, tx_id: &str, ports: &[PortEntity]) -> bool {
    self
      .map
      .get(tx_id)
      .map(|tx| tx.are_ports_ready(ports))
      .unwrap_or(false)
  }
  pub(crate) fn take_from_ports(
    &mut self,
    tx_id: &str,
    ports: Vec<PortEntity>,
  ) -> Result<HashMap<String, Vec<u8>>> {
    let tx = self.map.get_mut(tx_id).ok_or_else(|| {
      crate::Error::SchematicError(format!(
        "Invalid state: no transaction map found for tx {}",
        tx_id
      ))
    })?;
    let mut map = HashMap::new();
    for port in ports {
      let message = tx.take_from_port(&port).ok_or_else(|| {
        crate::Error::SchematicError("Tried to take from port that had none".to_string())
      })?;
      // TODO: Handle this better
      let bytes = message.into_bytes()?;
      map.insert(port.name, bytes);
    }
    Ok(map)
  }
}

#[cfg(test)]
mod tests {
  use vino_component::v0::Payload;
  use vino_component::Packet;

  use super::*;
  use crate::{
    Invocation,
    Result,
  };
  #[test_env_log::test]
  fn test_transaction() -> Result<()> {
    let tx_id = Invocation::uuid();

    let mut transaction = Transaction::new(tx_id);
    let port = PortEntity {
      schematic: "logger".into(),
      reference: "logger".into(),
      name: "vino::log".into(),
    };
    trace!("pushing to port");
    transaction.push_to_port(
      port.clone(),
      Packet::V0(Payload::MessagePack(vec![])).into(),
    );
    assert!(transaction.is_port_ready(&port));
    trace!("taking from port");
    let output = transaction.take_from_port(&port);
    assert_eq!(
      output,
      Some(Packet::V0(Payload::MessagePack(vec![])).into())
    );
    transaction.push_to_port(port, Packet::V0(Payload::Exception("oh no".into())).into());
    Ok(())
  }
}
