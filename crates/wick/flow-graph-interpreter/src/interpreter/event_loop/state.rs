use std::collections::HashMap;

use flow_graph::{PortDirection, PortReference};
use uuid::Uuid;
use wick_packet::PacketPayload;

use super::EventLoop;
use crate::interpreter::channel::{CallComplete, InterpreterDispatchChannel};
use crate::interpreter::executor::error::ExecutionError;
use crate::interpreter::executor::transaction::{Transaction, TxState};
use crate::InterpreterOptions;

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

  fn get_tx(&self, uuid: &Uuid) -> Option<&Transaction> {
    self.transactions.get(uuid)
  }

  pub fn transactions(&self) -> &TransactionMap {
    &self.transactions
  }

  pub(super) fn run_cleanup(&mut self) -> Result<(), ExecutionError> {
    let mut cleanup = Vec::new();
    for (tx_id, tx) in self.transactions.iter() {
      let last_update = tx.last_access().elapsed().unwrap();

      if last_update > EventLoop::SLOW_TX_TIMEOUT {
        let active_instances = tx.active_instances().iter().map(|i| i.id()).collect::<Vec<_>>();
        warn!(%tx_id, ?active_instances, "slow tx: no packet received in a long time");
      }
      if last_update > EventLoop::STALLED_TX_TIMEOUT {
        match tx.check_stalled() {
          Ok(TxState::Finished) => {
            // transaction has completed its output and isn't generating more data, clean it up.
            cleanup.push(*tx_id);
          }
          Ok(TxState::OutputPending) => {
            error!(%tx_id, "tx reached timeout while still waiting for output data");
            cleanup.push(*tx_id);
          }
          Ok(TxState::CompleteWithTasksPending) => {
            error!(%tx_id, "tx reached timeout while still waiting for tasks to complete");
            cleanup.push(*tx_id);
          }
          Err(error) => {
            error!(%error, %tx_id, "stalled transaction generated error determining hung state");
          }
        }
      }
    }
    for tx_id in cleanup {
      debug!(%tx_id, "transaction hung");
      self.transactions.remove(&tx_id);
    }
    Ok(())
  }

  fn get_mut(&mut self, tx_id: &Uuid) -> Option<&mut Transaction> {
    self.transactions.get_mut(tx_id)
  }

  pub(super) async fn handle_transaction_start(
    &mut self,
    mut transaction: Transaction,
    options: &InterpreterOptions,
  ) -> Result<(), ExecutionError> {
    match transaction.start(options).await {
      Ok(_) => {
        self.transactions.init_tx(transaction.id(), transaction);
        trace!("transaction started");
        Ok(())
      }
      Err(e) => Err(e),
    }
  }

  #[allow(clippy::unused_async)]
  pub(super) async fn handle_transaction_done(&mut self, tx_id: Uuid) -> Result<(), ExecutionError> {
    if let Some(tx) = self.get_mut(&tx_id) {
      let statistics = tx.finish()?;
      trace!(?statistics);
    }
    Ok(())
  }

  #[allow(clippy::unused_async)]
  async fn handle_input_data(&mut self, tx_id: Uuid, port: PortReference) -> Result<(), ExecutionError> {
    let tx = match self.get_tx(&tx_id) {
      Some(tx) => tx,
      None => {
        // This is a warning, not an error, because it's possible the transaction completes OK, it's just that a
        // component is misbehaving.
        warn!(
          port = %port, %tx_id, "still receiving upstream data for missing tx, this may be due to a component panic or premature close"
        );
        return Ok(());
      }
    };

    let graph = tx.schematic();
    let port_name = graph.get_port_name(&port);
    let instance = tx.instance(port.node_index());

    debug!(
      operation = %instance,
      port = port_name,
      "handling port input"
    );

    tx.stats
      .mark(format!("input:{}:{}:ready", port.node_index(), port.port_index()));

    let is_schematic_output = port.node_index() == graph.output().index();

    if is_schematic_output {
      tx.handle_schematic_output()?;
    } else if let Some(packet) = tx.take_instance_input(&port) {
      tx.push_packets(port.node_index(), vec![packet]).await?;
    }
    Ok(())
  }

  #[allow(clippy::unused_async)]
  async fn handle_output_data(&mut self, tx_id: Uuid, port: PortReference) -> Result<(), ExecutionError> {
    let tx = match self.get_tx(&tx_id) {
      Some(tx) => tx,
      None => {
        // This is a warning, not an error, because it's possible the transaction completes OK, it's just that a
        // component is misbehaving.
        warn!(
          port = %port, %tx_id, "still receiving downstream data for missing tx, this may be due to a component panic or premature close"
        );
        return Ok(());
      }
    };

    let graph = tx.schematic();
    let port_name = graph.get_port_name(&port);

    let instance = tx.instance(port.node_index());

    debug!(
      operation = %instance,
      port = port_name,
      "handling port output"
    );

    tx.stats
      .mark(format!("output:{}:{}:ready", port.node_index(), port.port_index()));

    if let Some(message) = tx.take_instance_output(&port) {
      trace!(?message, "delivering component output",);
      let connections = graph.get_port(&port).connections();
      for index in connections {
        let connection = &graph.connections()[*index];
        let downport = *connection.to();
        let name = graph.get_port_name(&downport);

        let channel = self.channel.clone();
        let downstream_instance = tx.instance(downport.node_index()).clone();
        let message = message.clone().set_port(name);
        trace!(%connection, ?message, "delivering packet",);
        downstream_instance.buffer_in(&downport, message);
        channel.dispatch_data(tx_id, downport);
      }
    } else {
      panic!("got port_data message with no payload to act on, port: {:?}", port);
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

  #[allow(clippy::unused_async)]
  pub(super) async fn handle_call_complete(&self, tx_id: Uuid, data: CallComplete) -> Result<(), ExecutionError> {
    let tx = match self.get_tx(&tx_id) {
      Some(tx) => tx,
      None => {
        // This is a warning, not an error, because it's possible the transaction completes OK, it's just that a
        // component is misbehaving.
        warn!(
          ?data,
          %tx_id, "tried to cleanup call for missing tx, this may be due to a component panic or premature close"
        );
        return Ok(());
      }
    };
    let instance = tx.instance(data.index);
    debug!(operation = instance.id(), entity = %instance.entity(), "call complete");

    if let Some(PacketPayload::Err(err)) = data.err {
      warn!(?err, "op:error");
      // If the call contains an error, then the component panicked.
      // We need to propagate the error downward...
      tx.handle_op_err(data.index, &err)?;
      // ...and clean up the call.
      // instance.handle_stream_complete(CompletionStatus::Error)?;
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

  fn get_mut(&mut self, uuid: &Uuid) -> Option<&mut Transaction> {
    self.0.get_mut(uuid).map(|tx| {
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
