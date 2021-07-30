use crate::dev::prelude::*;

impl Handler<Invocation> for NetworkService {
  type Result = ActorResult<Self, InvocationResponse>;

  fn handle(&mut self, msg: Invocation, _ctx: &mut Context<Self>) -> Self::Result {
    let tx_id = msg.tx_id.clone();
    let target = msg.target.clone();
    actix_ensure_ok!(self
      .ensure_is_started()
      .map_err(|e| inv_error(&tx_id, &e.to_string())));

    let schematic_name = match target {
      Entity::Schematic(name) => name,
      Entity::Component(name) => name,
      _ => return ActorResult::reply(inv_error(&tx_id, "Sent invalid entity")),
    };

    trace!("NETWORK:INVOKE:{}", schematic_name);

    let schematic = actix_ensure_ok!(self
      .get_schematic_addr(&schematic_name)
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
