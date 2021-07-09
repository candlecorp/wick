use serde::{
  Deserialize,
  Serialize,
};

use crate::dev::prelude::*;
use crate::schematic_service::default::make_default_transport;

#[derive(Debug, Clone, Serialize, Deserialize, Message, PartialEq)]
#[rtype(result = "Result<(), SchematicError>")]
pub struct InputMessage {
  pub tx_id: String,
  pub connection: ConnectionDefinition,
  pub payload: MessageTransport,
}

impl Handler<InputMessage> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: InputMessage, _ctx: &mut Context<Self>) -> Self::Result {
    debug!("Received payload for connection {}", msg.connection);

    let transaction_handler =
      actix_try!(self.tx_internal.get(&msg.tx_id).ok_or(InternalError(6003)));

    let payload = match &msg.connection.default {
      Some(default) => match msg.payload {
        MessageTransport::Exception(msg) => make_default_transport(default, &msg),
        MessageTransport::Error(msg) => make_default_transport(default, &msg),
        MessageTransport::Invalid => make_default_transport(default, "Internal Error: 7801"),
        MessageTransport::MultiBytes(_) => make_default_transport(default, "Internal Error: 7802"),
        MessageTransport::OutputMap(_) => make_default_transport(default, "Internal Error: 7803"),
        MessageTransport::Test(_) => make_default_transport(default, "Internal Error: 7804"),
        MessageTransport::Signal(_) => make_default_transport(default, "Internal Error: 7805"),
        rest => rest,
      },
      None => msg.payload,
    };

    let new_msg = InputMessage { payload, ..msg };

    actix_try!(transaction_handler.send(new_msg));

    ActorResult::reply(Ok(()))
  }
}
