use std::collections::HashMap;

use futures::Future;
use tokio::sync::mpsc::unbounded_channel;

use crate::dev::prelude::*;
use crate::dispatch::inv_error;
use crate::schematic_service::handlers::transaction_update::TransactionUpdate;
use crate::schematic_service::input_message::InputMessage;

type Result<T> = std::result::Result<T, SchematicError>;

impl Handler<Invocation> for SchematicService {
  type Result = ActorResult<Self, InvocationResponse>;

  fn handle(&mut self, msg: Invocation, ctx: &mut Context<Self>) -> Self::Result {
    let tx_id = msg.tx_id.clone();
    let target = msg.target.clone();
    let result = match target {
      Entity::Schematic(name) => handle_schematic(self, ctx.address(), &name, msg),
      Entity::Component(name) => handle_schematic(self, ctx.address(), &name, msg),
      Entity::Reference(reference) => self
        .get_component_definition(&reference)
        .and_then(|def| handle_schematic(self, ctx.address(), &def.id, msg)),
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
  invocation: Invocation,
) -> Result<impl Future<Output = Result<InvocationResponse>>> {
  trace!("Requesting schematic '{}'", name);
  let tx_id = invocation.tx_id.clone();

  let (mut outbound, inbound) = schematic.start(tx_id.clone());
  schematic.tx_internal.insert(tx_id.clone(), inbound.clone());

  tokio::spawn(async move {
    while let Some(msg) = outbound.recv().await {
      if let TransactionUpdate::Done(tx_id) = &msg {
        trace!("Schematic request finishing on transaction {}", tx_id);
        outbound.close();
      }
      ok_or_log!(addr.send(msg).await);
    }
    Ok!(())
  });

  let (tx, rx) = unbounded_channel::<OutputPacket>();
  schematic.tx_external.insert(tx_id.clone(), tx);

  let payload = invocation.msg.into_multibytes().map_err(|_| {
    SchematicError::FailedPreRequestCondition("Schematic sent invalid payload".into())
  })?;

  let model = schematic.get_state().model.lock();
  let connections = model.get_downstream_connections(SCHEMATIC_INPUT);
  let input_messages = make_input_packets(connections, &tx_id, &payload)?;
  let defaults = model.get_defaults();
  let defaults_messages = make_default_packets(defaults, &tx_id)?;
  drop(model);
  let messages = concat(vec![input_messages, defaults_messages]);
  for message in messages {
    inbound.send(TransactionUpdate::Update(message.handle_default()))?;
  }

  Ok(async move { Ok(InvocationResponse::stream(tx_id, rx)) })
}

fn make_input_packets<'a>(
  connections: impl Iterator<Item = &'a ConnectionDefinition>,
  tx_id: &str,
  bytemap: &HashMap<String, Vec<u8>>,
) -> Result<Vec<InputMessage>> {
  let mut messages: Vec<InputMessage> = vec![];
  for conn in connections {
    let bytes = bytemap.get(conn.from.get_port()).ok_or_else(|| {
      SchematicError::FailedPreRequestCondition(format!("Port {} not found in input", conn.from))
    })?;
    messages.push(InputMessage {
      connection: conn.clone(),
      tx_id: tx_id.to_owned(),
      payload: MessageTransport::MessagePack(bytes.clone()),
    });
  }

  Ok(messages)
}

fn make_default_packets<'a>(
  connections: impl Iterator<Item = &'a ConnectionDefinition>,
  tx_id: &str,
) -> Result<Vec<InputMessage>> {
  let mut messages: Vec<InputMessage> = vec![];
  for conn in connections {
    let json = conn.process_default("")?;
    let bytes = mp_serialize(&json)?;
    messages.push(InputMessage {
      connection: conn.clone(),
      tx_id: tx_id.to_owned(),
      payload: MessageTransport::MessagePack(bytes.clone()),
    });
  }

  Ok(messages)
}
