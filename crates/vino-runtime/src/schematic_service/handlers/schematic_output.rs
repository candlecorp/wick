use crate::dev::prelude::*;

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<(), SchematicError>")]
pub struct SchematicOutput {
  pub tx_id: String,
  pub payload: TransportWrapper,
}

impl Handler<SchematicOutput> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: SchematicOutput, _ctx: &mut Context<Self>) -> Self::Result {
    trace!("SC:{}:PORT:[{}]:READY", self.name, msg.payload.port);

    let tx = actix_try!(
      self
        .tx_external
        .get(&msg.tx_id)
        .ok_or_else(|| SchematicError::TransactionNotFound(msg.tx_id.clone())),
      6031
    );

    let output_msg = msg.payload;

    match tx.send(output_msg) {
      Ok(_) => ActorResult::reply(Ok(())),
      _ => ActorResult::reply(Err(InternalError(6004).into())),
    }
  }
}
