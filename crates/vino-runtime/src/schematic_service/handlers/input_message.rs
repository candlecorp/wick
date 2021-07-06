use serde::{
  Deserialize,
  Serialize,
};

use crate::dev::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Message, PartialEq)]
#[rtype(result = "Result<(), SchematicError>")]
pub struct InputMessage {
  pub tx_id: String,
  pub origin: PortReference,
  pub target: PortReference,
  pub payload: MessageTransport,
}

impl Handler<InputMessage> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: InputMessage, _ctx: &mut Context<Self>) -> Self::Result {
    trace!("Payload received: {:?}", msg);

    let handler = actix_try!(self.tx_internal.get(&msg.tx_id).ok_or(InternalError(6003)));

    debug!("Received payload from {} to {}", msg.origin, msg.target);

    actix_try!(handler.send(msg));

    ActorResult::reply(Ok(()))
  }
}
