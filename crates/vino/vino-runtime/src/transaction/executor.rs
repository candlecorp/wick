use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::time::timeout;

use super::Transaction;
use crate::dev::prelude::*;
use crate::transaction::ComponentPayload;

#[derive(Clone, Debug)]
pub struct SchematicOutput {
  pub tx_id: String,
  pub payload: TransportWrapper,
}

#[derive(Debug)]
pub(crate) struct TransactionExecutor {
  model: Arc<RwLock<SchematicModel>>,
  timeout: Duration,
}

impl TransactionExecutor {
  pub(crate) fn new(model: SharedModel, timeout: Duration) -> Self {
    Self { model, timeout }
  }

  #[allow(clippy::too_many_lines)]
  pub(crate) fn new_transaction<T: AsRef<str>>(
    &mut self,
    tx_id: T,
  ) -> (UnboundedReceiver<TransactionUpdate>, UnboundedSender<TransactionUpdate>) {
    let mut transaction = Transaction::new(&tx_id, &self.model);

    let (inbound_tx, inbound_rx) = unbounded_channel::<TransactionUpdate>();
    let (outbound_tx, mut outbound_rx) = unbounded_channel::<TransactionUpdate>();

    let expire = self.timeout;

    let tx_id = tx_id.as_ref().to_owned();

    tokio::spawn(async move {
      let mut self_msgs = VecDeque::new();
      let log_prefix = transaction.log_prefix();
      let mut iter = 0;
      trace!("{}:START", log_prefix);
      let start_time = Instant::now();
      let mut last_time = Instant::now();
      'root: loop {
        let iter_prefix = format!("{}[{}]", log_prefix, iter);
        iter += 1;

        trace!(
          "{}:NEXTWAIT:(queue:{},timeout:{:?})",
          iter_prefix,
          self_msgs.len(),
          expire
        );
        let msg = if let Some(msg) = self_msgs.pop_front() {
          msg
        } else {
          let actual = Instant::now();
          match timeout(expire, outbound_rx.recv()).await {
            Ok(Some(msg)) => msg,
            Ok(None) => TransactionUpdate::Drained,
            Err(_) => TransactionUpdate::Timeout(actual.elapsed()),
          }
        };
        trace!(
          "{}:NEXT[last +{} μs][total +{} μs]",
          iter_prefix,
          last_time.elapsed().as_micros(),
          start_time.elapsed().as_micros()
        );
        last_time = Instant::now();

        match msg {
          TransactionUpdate::NoOp => {
            trace!("{}:NOOP", iter_prefix);
          }
          TransactionUpdate::Drained => {
            trace!("{}:DRAINED", iter_prefix);
            self_msgs.push_back(TransactionUpdate::Done(tx_id.clone()));
          }
          TransactionUpdate::Timeout(d) => {
            trace!("{}:TIMEOUT:{}ms", iter_prefix, d.as_millis());
            warn!("Transaction {} timeout waiting for next message. This can happen if a component does not close its ports when finished sending.", tx_id);
            self_msgs.push_back(TransactionUpdate::Done(tx_id.clone()));
          }
          TransactionUpdate::Error(e) => {
            trace!("{}:ERROR:{}", iter_prefix, e);
            warn!("Transaction {} error: {}", tx_id, e);
            self_msgs.push_back(TransactionUpdate::Done(tx_id.clone()));
          }
          TransactionUpdate::Execute(payload) => {
            trace!("{}:EXECUTE[{}]", iter_prefix, payload.instance);
            let _ = map_err!(
              inbound_tx.send(TransactionUpdate::Execute(payload)),
              InternalError::E9001
            );
          }
          TransactionUpdate::Transition(target) => {
            trace!("{}:TRANSITION:TO[{}]", iter_prefix, target.get_instance());
            let map = transaction.ports.take_inputs(&target)?;
            self_msgs.push_back(TransactionUpdate::Execute(ComponentPayload {
              tx_id: tx_id.clone(),
              instance: target.get_instance_owned(),
              payload_map: map,
            }));
          }
          TransactionUpdate::Result(output) => {
            trace!("{}:RESULT:PORT[{}]", iter_prefix, output.payload.port,);
            let _ = map_err!(inbound_tx.send(TransactionUpdate::Result(output)), InternalError::E9002);

            if !transaction.is_done() {
              trace!("{}:WAIT", iter_prefix);
              // continue the base loop
              continue 'root;
            }

            trace!("{}:PORTS_CLOSED", iter_prefix);
            // If all connections to the schematic outputs are closed, finish up.
            self_msgs.push_back(TransactionUpdate::Done(tx_id.clone()));
          }
          TransactionUpdate::Done(tx_id) => {
            trace!("{}:DONE", iter_prefix);
            outbound_rx.close();
            let _ = map_err!(inbound_tx.send(TransactionUpdate::Done(tx_id)), InternalError::E9004);
            break;
          }
          TransactionUpdate::Update(input) => {
            trace!("{}:UPDATE:{}", iter_prefix, input.connection);
            trace!("{}:MSG[{}]", iter_prefix, input.payload);
            transaction.ports.receive(&input.connection, input.payload);
            let target = &input.connection.to;
            let port = target.get_port();

            if target.matches_instance(SCHEMATIC_OUTPUT) {
              if let Some(payload) = transaction.ports.take_from_port(target) {
                self_msgs.push_back(TransactionUpdate::Result(SchematicOutput {
                  tx_id: tx_id.clone(),
                  payload: TransportWrapper {
                    port: port.to_owned(),
                    payload,
                  },
                }));
              }
            } else if transaction.ports.is_target_ready(target) {
              trace!("{}:TARGET_READY:[{}]", iter_prefix, target.get_instance());
              self_msgs.push_back(TransactionUpdate::Transition(target.clone()));
            }
          }
        };
      }
      debug!("{}STOPPING", log_prefix);
      Ok::<_, TransactionError>(())
    });

    (inbound_rx, outbound_tx)
  }
}
