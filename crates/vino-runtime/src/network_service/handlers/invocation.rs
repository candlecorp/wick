use crate::dev::prelude::*;

impl Handler<InvocationMessage> for NetworkService {
  type Result = ActorResult<Self, InvocationResponse>;

  fn handle(&mut self, msg: InvocationMessage, _ctx: &mut Context<Self>) -> Self::Result {
    let tx_id = msg.get_tx_id().to_owned();
    actix_ensure_ok!(self
      .ensure_is_started()
      .map_err(|e| inv_error(&tx_id, &e.to_string())));

    let schematic_name = match msg.get_target() {
      Entity::Schematic(name) => name,
      Entity::Component(_, name) => name,
      _ => return ActorResult::reply(inv_error(&tx_id, "Sent invalid entity")),
    };

    trace!("NETWORK[{}]:INVOKE:{}", self.uid, schematic_name);

    let schematic = actix_ensure_ok!(self
      .get_schematic_addr(schematic_name)
      .map_err(|e| inv_error(&tx_id, &e.to_string())));

    let task = async move {
      match schematic.send(msg).await {
        Ok(response) => response,
        Err(e) => {
          InvocationResponse::error(tx_id, format!("Internal error invoking schematic: {}", e))
        }
      }
    }
    .into_actor(self);

    ActorResult::reply_async(task)
  }
}
