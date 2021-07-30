use crate::dev::prelude::*;
use crate::schematic_service::input_message::InputMessage;

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<(),SchematicError>")]
pub(crate) struct OutputMessage {
  pub(crate) port: ConnectionTargetDefinition,
  pub(crate) tx_id: String,
  pub(crate) payload: MessageTransport,
}

impl Handler<OutputMessage> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: OutputMessage, _ctx: &mut Context<Self>) -> Self::Result {
    let log_prefix = format!("SC:{}:OUTPUT:{}:", self.name, msg.port);

    let defs = if msg.port.matches_port(crate::COMPONENT_ERROR) {
      error!("{}Component-wide error received", log_prefix);
      self.get_downstream_connections(actix_try!(msg.port.get_instance()))
    } else {
      trace!("{}Output ready", log_prefix);
      self.get_port_connections(&msg.port)
    };

    for connection in defs {
      let msg = InputMessage {
        tx_id: msg.tx_id.clone(),
        connection,
        payload: msg.payload.clone(),
      };
      actix_try!(self
        .update_transaction(msg)
        .map_err(|_| InternalError(6001)));
    }

    let task = async move { Ok(()) }.into_actor(self);
    ActorResult::reply_async(task)
  }
}
