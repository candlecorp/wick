use super::component_payload::ComponentPayload;
use super::schematic_output::SchematicOutput;
use crate::dev::prelude::*;
use crate::schematic_service::input_message::InputMessage;

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<(), SchematicError>")]
pub enum TransactionUpdate {
  Transition(ComponentPayload),
  Result(SchematicOutput),
  Done(String),
  Update(InputMessage),
}

impl std::fmt::Display for TransactionUpdate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let name = match self {
      TransactionUpdate::Transition(_) => "transition",
      TransactionUpdate::Result(_) => "result",
      TransactionUpdate::Done(_) => "done",
      TransactionUpdate::Update(_) => "update",
    };
    f.write_str(name)
  }
}

impl Handler<TransactionUpdate> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: TransactionUpdate, ctx: &mut Context<Self>) -> Self::Result {
    let addr = ctx.address();
    match msg {
      TransactionUpdate::Transition(msg) => ActorResult::reply_async(
        async move { log_ie!(addr.send(msg).await, 6011)? }.into_actor(self),
      ),
      TransactionUpdate::Result(msg) => {
        trace!("TX:{}: received result", msg.tx_id);
        ActorResult::reply_async(
          async move { log_ie!(addr.send(msg).await, 6012)? }.into_actor(self),
        )
      }
      TransactionUpdate::Done(tx_id) => {
        trace!("TX:{}: finished", tx_id);
        let tx = actix_try!(self
          .tx_external
          .get(&tx_id)
          .ok_or_else(|| SchematicError::TransactionNotFound(tx_id.clone())));

        let output_msg = InvocationTransport {
          payload: MessageTransport::close(),
          port: "<system>".to_owned(),
        };
        if tx.send(output_msg).is_err() {
          warn!("TX:{} {}", tx_id, SchematicError::SchematicClosedEarly);
        }

        ActorResult::reply(Ok(()))
      }
      _ => unreachable!(),
    }
  }
}
