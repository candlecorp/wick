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
    let log_prefix = format!("OutputMessage:{}:", msg.port);

    let defs = if msg.port.matches_port(crate::COMPONENT_ERROR) {
      error!("{} Component-wide error received", log_prefix);
      self.get_downstream_connections(msg.port.get_instance())
    } else {
      trace!("{} Output ready", log_prefix);
      self.get_port_connections(&msg.port)
    };

    let addr = ctx.address();

    let task = async move {
      join_or_err(
        defs
          .into_iter()
          .map(|connection| InputMessage {
            tx_id: msg.tx_id.clone(),
            connection,
            payload: msg.payload.clone(),
          })
          .map(|ips| addr.send(ips)),
        6001,
      )
      .await?;

      Ok(())
    };

    Box::pin(task.into_actor(self))
  }
}
