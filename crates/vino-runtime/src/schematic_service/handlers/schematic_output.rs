use crate::dev::prelude::*;

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<(), SchematicError>")]
pub struct SchematicOutput {
  pub port: String,
  pub tx_id: String,
  pub payload: MessageTransport,
}

impl Handler<SchematicOutput> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: SchematicOutput, _ctx: &mut Context<Self>) -> Self::Result {
    trace!("SC:{}:PORT:[{}]:READY", self.name, msg.port);

    let tx = actix_try!(self
      .tx_external
      .get(&msg.tx_id)
      .ok_or_else(|| SchematicError::TransactionNotFound(msg.tx_id.clone())));

    let output_msg = InvocationTransport {
      payload: msg.payload,
      port: msg.port,
    };

    ok_or_log!(tx.send(output_msg));

    ActorResult::reply(Ok(()))
  }
}
