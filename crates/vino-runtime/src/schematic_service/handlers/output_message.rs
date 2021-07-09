use crate::dev::prelude::*;
use crate::schematic_service::handlers::input_message::InputMessage;

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<(),SchematicError>")]
pub(crate) struct OutputMessage {
  pub(crate) port: ConnectionTargetDefinition,
  pub(crate) tx_id: String,
  pub(crate) payload: MessageTransport,
}

impl Handler<OutputMessage> for SchematicService {
  type Result = ResponseActFuture<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: OutputMessage, ctx: &mut Context<Self>) -> Self::Result {
    trace!("Output ready on {}", msg.port);
    let defs = self.get_port_connections(&msg.port);
    let tx_id = msg.tx_id;
    let data = msg.payload;
    let addr = ctx.address();

    let task = async move {
      let to_message = |connection: Connection| InputMessage {
        tx_id: tx_id.clone(),
        connection,
        payload: data.clone(),
      };

      join_or_err(
        defs.into_iter().map(to_message).map(|ips| addr.send(ips)),
        6001,
      )
      .await?;

      Ok(())
    };

    Box::pin(task.into_actor(self))
  }
}
