use futures::Future;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;
use vino_manifest::parse::CORE_ID;

use crate::dev::prelude::*;
use crate::dispatch::inv_error;
use crate::schematic_service::input_message::InputMessage;
use crate::CORE_PORT_SEED;

type Result<T> = std::result::Result<T, SchematicError>;

impl Handler<InvocationMessage> for SchematicService {
  type Result = ActorResult<Self, InvocationResponse>;

  fn handle(&mut self, msg: InvocationMessage, ctx: &mut Context<Self>) -> Self::Result {
    let tx_id = msg.get_tx_id().to_owned();
    let target = msg.get_target();

    let result = match target {
      Entity::Schematic(name) => handle_schematic(self, ctx.address(), name, &msg),
      Entity::Component(_, name) => handle_schematic(self, ctx.address(), name, &msg),
      Entity::Reference(reference) => get_component_definition(self.get_model(), reference)
        .and_then(|def| handle_schematic(self, ctx.address(), &def.id(), &msg)),
      _ => Err(SchematicError::FailedPreRequestCondition(
        "Schematic invoked with entity it doesn't handle".into(),
      )),
    };

    match result {
      Ok(task) => {
        ActorResult::reply_async(task.into_actor(self).map(move |result, _, _| {
          result.map_or_else(|e| inv_error(&tx_id, &e.to_string()), |r| r)
        }))
      }
      Err(e) => ActorResult::reply(inv_error(&tx_id, &e.to_string())),
    }
  }
}

fn handle_schematic(
  schematic: &mut SchematicService,
  addr: Addr<SchematicService>,
  name: &str,
  invocation: &InvocationMessage,
) -> Result<impl Future<Output = Result<InvocationResponse>>> {
  let tx_id = invocation.get_tx_id().to_owned();
  let log_prefix = format!("SC[{}]:{}", name, tx_id);
  trace!("{}:INVOKE", log_prefix);

  let (mut outbound, inbound) = schematic.start(tx_id.clone());
  schematic.executor.insert(tx_id.clone(), inbound.clone());
  let (tx, rx) = unbounded_channel::<TransportWrapper>();

  let inner = log_prefix.clone();
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
          if let Err(e) = addr.send(a).await {
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
    Ok!(())
  });

  match make_input_packets(name, schematic.get_model(), &tx_id, invocation) {
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
