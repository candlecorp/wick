use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use actix::clock::timeout;
use parking_lot::Mutex;
use tokio::sync::mpsc::{
  unbounded_channel,
  UnboundedReceiver,
  UnboundedSender,
};

use super::Transaction;
use crate::dev::prelude::*;
use crate::schematic_service::handlers::component_payload::ComponentPayload;
use crate::schematic_service::handlers::schematic_output::SchematicOutput;
use crate::schematic_service::handlers::transaction_update::TransactionUpdate;
#[derive(Debug)]
pub(crate) struct TransactionExecutor {
  model: Arc<Mutex<SchematicModel>>,
  timeout: Duration,
}

impl TransactionExecutor {
  pub(crate) fn new(model: Arc<Mutex<SchematicModel>>, timeout: Duration) -> Self {
    Self { model, timeout }
  }
  #[allow(clippy::too_many_lines)]
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

    let expire = self.timeout;

    // let self_tx = outbound_tx.clone();
    tokio::spawn(async move {
      let mut self_msgs = VecDeque::new();
      let log_prefix = transaction.log_prefix();
      let mut iter = 0;
      trace!("{}:START", log_prefix);
      'root: loop {
        let iter_prefix = format!("{}[{}]", log_prefix, iter);
        iter += 1;

        let mut sender_messages = transaction.check_senders();
        trace!("{}:{} SENDER MESSAGES", iter_prefix, sender_messages.len());

        self_msgs.append(&mut sender_messages);

        trace!(
          "{}:NEXTWAIT:(internal:{},timeout:{:?})",
          iter_prefix,
          self_msgs.len(),
          expire
        );
        let msg = if let Some(msg) = self_msgs.pop_front() {
          msg
        } else {
          match timeout(expire, outbound_rx.recv()).await {
            Ok(Some(msg)) => msg,
            Ok(None) => TransactionUpdate::Drained,
            Err(e) => TransactionUpdate::Timeout(e),
          }
        };

        match msg {
          TransactionUpdate::Drained => {
            trace!("{}:DRAINED", iter_prefix);
            self_msgs.push_back(TransactionUpdate::Done(tx_id.clone()));
          }
          TransactionUpdate::Timeout(e) => {
            trace!("{}:TIMEOUT:{}", iter_prefix, e);
            trace!("Port statuses: \n{:#?}", transaction.ports);
            warn!("Transaction {} timeout out waiting for next message. This can happen if a component does not close its ports when finished sending.", tx_id);
            self_msgs.push_back(TransactionUpdate::Done(tx_id.clone()));
          }
          TransactionUpdate::Execute(payload) => {
            trace!("{}:EXECUTE:[{}]", iter_prefix, payload.instance);
            let _ = log_ie!(inbound_tx.send(TransactionUpdate::Execute(payload)), 9001);
          }
          TransactionUpdate::Transition(target) => {
            trace!("{}:TRANSITION:[{}]", iter_prefix, target.get_instance());
            let map = transaction.ports.take_inputs(&target)?;
            self_msgs.push_back(TransactionUpdate::Execute(ComponentPayload {
              tx_id: tx_id.clone(),
              instance: target.get_instance_owned(),
              payload_map: map,
            }));
          }
          TransactionUpdate::Result(output) => {
            trace!("{}:RESULT:{}", iter_prefix, output.payload.port,);
            let port = output.payload.port.clone();
            let _ = log_ie!(inbound_tx.send(TransactionUpdate::Result(output)), 9002);
            let target = ConnectionTargetDefinition::new(SCHEMATIC_OUTPUT, &port);
            if !transaction.has_active_upstream(&target) {
              trace!("{}:CLOSING:{}", iter_prefix, target);
              transaction.ports.close(&target);
            }
            // if transaction.ports.is_closed(&target) {
            //   ok_or_continue!(log_ie!(
            //     inbound_tx.send(TransactionUpdate::Result(SchematicOutput {
            //       payload: TransportWrapper {
            //         payload: MessageTransport::done(),
            //         port
            //       },
            //       tx_id: tx_id.clone()
            //     })),
            //     9003
            //   ));
            // }
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
            let _ = log_ie!(inbound_tx.send(TransactionUpdate::Done(tx_id)), 9004);
            break;
          }
          TransactionUpdate::Update(input) => {
            trace!("{}:UPDATE:{}", iter_prefix, input.connection,);
            trace!("{}:MSG[{}]", iter_prefix, input.payload,);
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
      debug!("{}:STOPPING", log_prefix);
      Ok!(())
    });

    (inbound_rx, outbound_tx)
  }
}
