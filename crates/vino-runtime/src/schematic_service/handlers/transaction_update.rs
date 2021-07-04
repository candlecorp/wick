use super::reference_data::ReferenceData;
use super::schematic_output::SchematicOutput;
use crate::dev::prelude::*;
use crate::schematic_service::handlers::component_output::ComponentOutput;

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<(), SchematicError>")]
pub(crate) enum TransactionUpdate {
  Transition(ReferenceData),
  Result(SchematicOutput),
  Done(String),
}

impl Handler<TransactionUpdate> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: TransactionUpdate, ctx: &mut Context<Self>) -> Self::Result {
    trace!("Transaction update: {:?}", msg);
    let addr = ctx.address();
    match msg {
      TransactionUpdate::Transition(msg) => ActorResult::reply_async(
        async move { addr.send(msg).await.map_err(|_| InternalError(6011))? }.into_actor(self),
      ),
      TransactionUpdate::Result(msg) => ActorResult::reply_async(
        async move { addr.send(msg).await.map_err(|_| InternalError(6012))? }.into_actor(self),
      ),
      TransactionUpdate::Done(tx_id) => {
        let tx = actix_try!(self
          .tx_external
          .get(&tx_id)
          .ok_or_else(|| SchematicError::TransactionNotFound(tx_id.clone())));

        debug!("Sending output on transmitter");
        let output_msg = ComponentOutput {
          invocation_id: tx_id.clone(),
          payload: Packet::V0(packet::v0::Payload::Close),
          port: "<system>".to_owned(),
        };
        match tx.send(output_msg) {
          Ok(_) => debug!("Sent output to receiver for tx {}", tx_id),
          Err(e) => warn!("{}", SchematicError::SchematicClosedEarly(e.to_string())),
        }

        ActorResult::reply(Ok(()))
      }
    }
  }
}
