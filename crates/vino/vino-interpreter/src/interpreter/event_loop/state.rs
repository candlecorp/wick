use std::collections::HashMap;

use uuid::Uuid;
use vino_schematic_graph::{PortDirection, PortReference};
use vino_transport::{MessageTransport, TransportWrapper};

use crate::default::make_default_transport;
use crate::interpreter::channel::CallComplete;
use crate::interpreter::executor::error::ExecutionError;
use crate::interpreter::executor::transaction::Transaction;

#[derive(Debug)]
pub(super) struct State {
  transactions: TransactionMap,
}

impl State {
  pub(super) fn new() -> Self {
    Self {
      transactions: TransactionMap::default(),
    }
  }

  fn get_tx(&self, uuid: &Uuid) -> Result<&Transaction, ExecutionError> {
    self.transactions.get(uuid).ok_or(ExecutionError::MissingTx(*uuid))
  }

  pub(super) async fn handle_transaction_start(&mut self, mut transaction: Transaction) -> Result<(), ExecutionError> {
    match transaction.start().await {
      Ok(_) => {
        self.transactions.init_tx(transaction.id(), transaction);
        Ok(())
      }
      Err(e) => {
        error!(tx_error = ?e);
        Err(e)
      }
    }
  }

  pub(super) async fn handle_transaction_done(&mut self, tx_id: Uuid) -> Result<(), ExecutionError> {
    let tx = self
      .transactions
      .remove(&tx_id)
      .ok_or(ExecutionError::MissingTx(tx_id))?;

    let statistics = tx.finish()?;
    trace!(?statistics);
    Ok(())
  }

  pub(super) async fn handle_port_status_change(
    &mut self,
    tx_id: Uuid,
    port: PortReference,
  ) -> Result<(), ExecutionError> {
    let tx = self.get_tx(&tx_id)?;
    tx.propagate_status(port).await?;

    Ok(())
  }

  async fn handle_input_data(&mut self, tx_id: Uuid, port: PortReference) -> Result<(), ExecutionError> {
    debug!(
      port = ?port
    );
    let tx = self.get_tx(&tx_id)?;
    let graph = tx.schematic();
    let port_name = graph.get_port_name(&port);

    let span = trace_span!("input", port = port_name, component = port.component_index());
    let _guard = span.enter();

    tx.stats
      .mark(format!("input:{}:{}:ready", port.component_index(), port.port_index()));

    let instance = tx.instance(port.component_index());

    let is_schematic_output = port.component_index() == graph.output().index();

    if is_schematic_output {
      tx.handle_schematic_output(&port).await?;
    } else if let Some(payload) = tx.take_payload(instance).await? {
      tx.invoke_component(port.component_index(), payload).await?;
    }

    Ok(())
  }

  async fn handle_output_data(&mut self, tx_id: Uuid, port: PortReference) -> Result<(), ExecutionError> {
    debug!(
      port = ?port
    );
    let tx = self.get_tx(&tx_id)?;
    let graph = tx.schematic();
    let port_name = graph.get_port_name(&port);

    let span = trace_span!("output", port = port_name, component = port.component_index());
    let _guard = span.enter();

    tx.stats
      .mark(format!("output:{}:{}:ready", port.component_index(), port.port_index()));

    if let Some(wrapper) = tx.take_component_output(&port) {
      debug!(?wrapper);

      let connections = graph.get_port(&port).connections();
      for index in connections {
        let connection = &graph.connections()[*index];
        let downport = connection.to();
        let name = graph.get_port_name(downport);

        trace!(connections = connection.to_string().as_str(), "delivering packet",);

        let payload = if let (Some(default), MessageTransport::Failure(err)) = (connection.data(), &wrapper.payload) {
          let err = err.message();
          make_default_transport(default, err)
        } else {
          wrapper.payload.clone()
        };

        tx.accept_inputs(downport, vec![TransportWrapper::new(name, payload)])
          .await?;
      }
    } else {
      panic!("got port_data message with no payload to act on");
    }

    Ok(())
  }

  pub(super) async fn handle_port_data(&mut self, tx_id: Uuid, port: PortReference) -> Result<(), ExecutionError> {
    debug!(
      tx_id = ?tx_id,
      port = ?port
    );

    match port.direction() {
      PortDirection::Out => self.handle_output_data(tx_id, port).await,
      PortDirection::In => self.handle_input_data(tx_id, port).await,
    }
  }

  #[instrument(skip_all, name = "call_complete")]
  pub(super) async fn handle_call_complete(&self, tx_id: Uuid, data: CallComplete) -> Result<(), ExecutionError> {
    debug!(component = &data.index,);
    let tx = self.get_tx(&tx_id)?;
    let instance = tx.instance(data.index);

    if let Some(err) = data.err {
      // If the call contains an error, then the component panicked.
      // We need to shortcircuit the error downward...
      tx.handle_short_circuit(data.index, err).await?;
      // ...and clean up the call.
      instance.handle_call_complete()?;
    }

    Ok(())
  }

  #[cfg(test)]
  pub(super) fn debug_print_transactions(&self) -> Vec<serde_json::Value> {
    let mut lines = Vec::new();
    for (uuid, tx) in self.transactions.iter() {
      lines.push(serde_json::json!({
        "schematic": tx.schematic_name(),
        "tx": uuid.to_string(),
        "state": tx.debug_status()
      }));
    }
    lines
  }
}

#[derive(Debug, Default)]
struct TransactionMap(HashMap<Uuid, Transaction>);

impl TransactionMap {
  pub(crate) fn init_tx(&mut self, uuid: Uuid, transaction: Transaction) {
    self.0.insert(uuid, transaction);
  }

  fn get(&self, uuid: &Uuid) -> Option<&Transaction> {
    self.0.get(uuid)
  }

  fn remove(&mut self, uuid: &Uuid) -> Option<Transaction> {
    self.0.remove(uuid)
  }

  // Used in debug JSON log.
  #[cfg(test)]
  fn iter(&self) -> impl Iterator<Item = (&Uuid, &Transaction)> {
    self.0.iter()
  }
}
