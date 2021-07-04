use serde::{
  Deserialize,
  Serialize,
};

use crate::dev::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Message, PartialEq)]
#[rtype(result = "Result<(), SchematicError>")]
pub struct PayloadReceived {
  pub tx_id: String,
  pub origin: PortReference,
  pub target: PortReference,
  pub payload: MessageTransport,
}

impl Handler<PayloadReceived> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: PayloadReceived, _ctx: &mut Context<Self>) -> Self::Result {
    trace!("Payload received: {:?}", msg);
    let port = msg.target.clone();
    let tx_id = msg.tx_id.clone();
    trace!("Receiving on port {}", port);

    let transaction_handler = actix_try!(self.tx_internal.get(&tx_id).ok_or(InternalError(6003)));
    debug!("Sent output to transaction handler for {:?}", msg);
    actix_try!(transaction_handler.send(msg));

    ActorResult::reply(Ok(()))
  }
}
