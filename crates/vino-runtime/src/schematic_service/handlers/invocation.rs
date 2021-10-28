use std::sync::Arc;

use futures::Future;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;
use vino_manifest::parse::CORE_ID;

use crate::dev::prelude::*;
use crate::schematic_service::input_message::InputMessage;
use crate::CORE_PORT_SEED;

type Result<T> = std::result::Result<T, SchematicError>;

pub(crate) fn handle_schematic(
  schematic: &Arc<SchematicService>,
  name: &str,
  invocation: &InvocationMessage,
) -> Result<impl Future<Output = Result<InvocationResponse>>> {
  let tx_id = invocation.get_tx_id().to_owned();
  let log_prefix = format!("SC[{}]:{}", name, tx_id);
  trace!("{}:INVOKE", log_prefix);

  let (mut outbound, inbound) = schematic.start_tx(tx_id.clone());
  let (tx, rx) = unbounded_channel::<TransportWrapper>();

  let inner = log_prefix;
  let inner_schematic = schematic.clone();
  tokio::spawn(async move {
    while let Some(msg) = outbound.recv().await {
      match msg {
        TransactionUpdate::Done(tx_id) => {
          let output_msg = TransportWrapper {
            payload: MessageTransport::done(),
            port: "<system>".to_owned(),
          };
          if tx.send(output_msg).is_err() {
            warn!("TX:{} {}", tx_id, SchematicError::SchematicClosedEarly);
          }

          trace!("{}:DONE", inner);
          break;
        }
        TransactionUpdate::Result(a) => {
          if let Err(e) = tx.send(a.payload) {
            error!("Error sending result {}", e);
          }
        }
        TransactionUpdate::Execute(a) => {
          if let Err(e) = inner_schematic.component_payload(a).await {
            error!("Error sending execute command {}", e);
          }
        }
        rest => {
          warn!("Unhandled state: {:?}", rest);
        }
      }
    }
    drop(outbound);
    trace!("{}:STOPPING", inner);
  });

  match make_input_packets(name, &schematic.get_model(), &tx_id, invocation) {
    Ok(messages) => {
      for message in messages {
        inbound.send(TransactionUpdate::Update(message.handle_default()))?;
      }
    }
    Err(e) => {
      inbound.send(TransactionUpdate::Error(e.to_string()))?;
      return Err(e);
    }
  };
  let rx = UnboundedReceiverStream::new(rx);

  Ok(async move { Ok(InvocationResponse::stream(tx_id.clone(), rx)) })
}

fn make_input_packets(
  name: &str,
  model: &SharedModel,
  tx_id: &str,
  invocation: &InvocationMessage,
) -> Result<Vec<InputMessage>> {
  let map = invocation.get_payload();
  let model = model.read();
  let connections = model.get_downstream_connections(SCHEMATIC_INPUT);

  let mut messages: Vec<InputMessage> = vec![];
  for conn in connections {
    let transport = map.get(conn.from.get_port()).ok_or_else(|| {
      SchematicError::FailedPreRequestCondition(format!(
        "Port {} not found in transport payload",
        conn.from
      ))
    })?;
    debug!(
      "SC[{}]:INPUT[{}]:PAYLOAD:{:?}",
      name,
      conn.from.get_port(),
      transport
    );
    messages.push(InputMessage {
      connection: conn.clone(),
      tx_id: tx_id.to_owned(),
      payload: transport.clone(),
    });
    messages.push(InputMessage {
      connection: conn.clone(),
      tx_id: tx_id.to_owned(),
      payload: MessageTransport::done(),
    });
  }
  let connections = model.get_downstream_connections(CORE_ID);
  for conn in connections {
    let msg = match conn.from.get_port() {
      CORE_PORT_SEED => MessageTransport::success(&invocation.get_init_data().seed),
      x => MessageTransport::error(format!("{} port {} does not exist.", CORE_ID, x)),
    };
    messages.push(InputMessage {
      connection: conn.clone(),
      tx_id: tx_id.to_owned(),
      payload: msg,
    });
    messages.push(InputMessage {
      connection: conn.clone(),
      tx_id: tx_id.to_owned(),
      payload: MessageTransport::done(),
    });
  }

  Ok(messages)
}
