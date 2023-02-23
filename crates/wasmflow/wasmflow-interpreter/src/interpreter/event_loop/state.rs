use std::collections::HashMap;

use uuid::Uuid;
use wasmflow_packet_stream::Packet;
use wasmflow_schematic_graph::{PortDirection, PortReference};

use super::EventLoop;
use crate::interpreter::channel::{CallComplete, InterpreterDispatchChannel};
use crate::interpreter::executor::error::ExecutionError;
use crate::interpreter::executor::transaction::operation::CompletionStatus;
use crate::interpreter::executor::transaction::{accept_input, Transaction};

#[derive(Debug)]
pub struct State {
  transactions: TransactionMap,
  channel: InterpreterDispatchChannel,
}

impl State {
  pub(super) fn new(channel: InterpreterDispatchChannel) -> Self {
    Self {
      transactions: TransactionMap::default(),
      channel,
    }
  }

  fn get_tx(&self, uuid: &Uuid) -> Result<&Transaction, ExecutionError> {
    self.transactions.get(uuid).ok_or(ExecutionError::MissingTx(*uuid))
  }

  pub fn transactions(&self) -> &TransactionMap {
    &self.transactions
  }

  pub(super) async fn check_hung(&mut self, panic_on_hung: bool) -> Result<(), ExecutionError> {
    let mut cleanup = Vec::new();
    for (tx_id, tx) in self.transactions.iter() {
      let last_update = tx.last_access();
      if last_update.elapsed().unwrap() > EventLoop::HUNG_TX_TIMEOUT {
        warn!(%tx_id, elapsed=?last_update.elapsed().unwrap(),"hung transaction");
        if panic_on_hung {
          let err = ExecutionError::HungTransaction(*tx_id);
          tx.emit_output_message(vec![Packet::component_error(err.to_string())])
            .await?;
          return Err(err);
        }

        match tx.check_hung().await {
          Ok(true) => {
            cleanup.push(*tx_id);
          }
          Ok(false) => {
            // not hung, continue as normal.
          }
          Err(error) => {
            error!(%error, %tx_id, "stalled transaction generated error determining hung state");
          }
        }
      }
    }
    for tx_id in cleanup {
      debug!(%tx_id, "transaction hung");
      self.cleanup(&tx_id);
    }
    Ok(())
  }

  fn cleanup(&mut self, tx_id: &Uuid) -> Option<Transaction> {
    trace!(%tx_id, "cleaning up transaction");
    self.transactions.remove(tx_id)
  }

  pub(super) async fn handle_transaction_start(&mut self, mut transaction: Transaction) -> Result<(), ExecutionError> {
    let result = match transaction.start().await {
      Ok(_) => {
        self.transactions.init_tx(transaction.id(), transaction);
        Ok(())
      }
      Err(e) => {
        error!(tx_error = %e);
        Err(e)
      }
    };
    trace!("transaction started");
    result
  }

  #[allow(clippy::unused_async)]
  pub(super) async fn handle_transaction_done(&mut self, tx_id: Uuid) -> Result<(), ExecutionError> {
    let tx = self.cleanup(&tx_id).ok_or(ExecutionError::MissingTx(tx_id))?;
    trace!("handling transaction done");

    let statistics = tx.finish()?;
    trace!(?statistics);
    Ok(())
  }

  #[allow(clippy::unused_async)]
  pub(super) async fn handle_port_status_change(
    &mut self,
    _tx_id: Uuid,
    port: PortReference,
  ) -> Result<(), ExecutionError> {
    debug!(
      port = %port, "handling port status change"
    );

    Ok(())
  }

  async fn handle_input_data(&mut self, tx_id: Uuid, port: PortReference) -> Result<(), ExecutionError> {
    debug!(
      port = %port, "handling port input"
    );
    let tx = self.get_tx(&tx_id)?;
    let graph = tx.schematic();
    let port_name = graph.get_port_name(&port);

    tx.stats
      .mark(format!("input:{}:{}:ready", port.node_index(), port.port_index()));

    let instance = tx.instance(port.node_index());

    let span = trace_span!("input", port = port_name, component = instance.id());
    let _guard = span.enter();

    let is_schematic_output = port.node_index() == graph.output().index();

    if instance.done() {
      warn!(instance = instance.id(), "component finished but still receiving input");
    } else if is_schematic_output {
      tx.handle_schematic_output().await?;
    } else {
      let packets = tx.take_packets(instance)?;
      if !packets.is_empty() {
        tx.push_packets(port.node_index(), packets)?;
      }
    }

    Ok(())
  }

  async fn handle_output_data(&mut self, tx_id: Uuid, port: PortReference) -> Result<(), ExecutionError> {
    debug!(
      port = %port, "handling port output"
    );
    let tx = self.get_tx(&tx_id)?;
    let graph = tx.schematic();
    let port_name = graph.get_port_name(&port);

    let instance = tx.instance(port.node_index());

    let span = trace_span!("output", port = port_name, component = instance.id());
    let _guard = span.enter();

    tx.stats
      .mark(format!("output:{}:{}:ready", port.node_index(), port.port_index()));

    if let Some(message) = tx.take_component_output(&port) {
      let connections = graph.get_port(&port).connections();
      for index in connections {
        let connection = &graph.connections()[*index];
        let downport = *connection.to();
        let name = graph.get_port_name(&downport);

        trace!(%connection, "delivering packet",);
        let channel = self.channel.clone();
        let downstream_instance = tx.instance(downport.node_index()).clone();
        let message = message.clone().set_port(name);
        tokio::spawn(async move {
          accept_input(tx_id, downport, &downstream_instance, &channel, message).await;
        });
      }
    } else {
      panic!("got port_data message with no payload to act on");
    }

    trace!(count = instance.num_pending(), "number still pending");

    Ok(())
  }

  pub(super) async fn handle_port_data(&mut self, tx_id: Uuid, port: PortReference) -> Result<(), ExecutionError> {
    match port.direction() {
      PortDirection::Out => self.handle_output_data(tx_id, port).await,
      PortDirection::In => self.handle_input_data(tx_id, port).await,
    }
  }

  pub(super) async fn handle_call_complete(&self, tx_id: Uuid, data: CallComplete) -> Result<(), ExecutionError> {
    let tx = self.get_tx(&tx_id)?;
    let instance = tx.instance(data.index);
    debug!(component = instance.id(), entity = %instance.entity(), "call complete");

    if let Some(err) = data.err {
      warn!(?err, "op:error");
      // If the call contains an error, then the component panicked.
      // We need to propagate the error downward...
      tx.handle_op_err(data.index, err).await?;
      // ...and clean up the call.
      instance.handle_call_complete(CompletionStatus::Error)?;
    }

    Ok(())
  }
}

#[derive(Debug, Default)]
#[must_use]
pub struct TransactionMap(HashMap<Uuid, Transaction>);

impl TransactionMap {
  pub(crate) fn init_tx(&mut self, uuid: Uuid, transaction: Transaction) {
    self.0.insert(uuid, transaction);
  }

  fn get(&self, uuid: &Uuid) -> Option<&Transaction> {
    self.0.get(uuid).map(|tx| {
      tx.update_last_access();
      tx
    })
  }

  fn remove(&mut self, uuid: &Uuid) -> Option<Transaction> {
    self.0.remove(uuid)
  }

  fn iter(&self) -> impl Iterator<Item = (&Uuid, &Transaction)> {
    self.0.iter()
  }
}
