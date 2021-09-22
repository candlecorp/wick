use crate::dev::prelude::*;
use crate::schematic_service::input_message::InputMessage;

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<(),SchematicError>")]
pub(crate) struct OutputMessage {
  pub(crate) port: ConnectionTargetDefinition,
  pub(crate) tx_id: String,
  pub(crate) payload: MessageTransport,
}

impl OutputMessage {
  pub(crate) fn new<T: AsRef<str>>(
    tx_id: T,
    port: ConnectionTargetDefinition,
    payload: MessageTransport,
  ) -> Self {
    Self {
      tx_id: tx_id.as_ref().to_owned(),
      port,
      payload,
    }
  }
}

impl Handler<OutputMessage> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: OutputMessage, _ctx: &mut Context<Self>) -> Self::Result {
    let log_prefix = format!("SC[{}]:OUTPUT:{}:", self.name, msg.port);

    let defs = if msg.port.matches_port(vino_transport::COMPONENT_ERROR) {
      error!("{}Component-wide error received", log_prefix);
      get_downstream_connections(self.get_model(), msg.port.get_instance())
    } else {
      trace!("{}Output ready", log_prefix);
      get_port_connections(self.get_model(), &msg.port)
    };

    for connection in defs {
      let upstream_port = connection.from.to_string();
      let next = InputMessage {
        tx_id: msg.tx_id.clone(),
        connection,
        payload: msg.payload.clone(),
      };

      let send_result = self
        .executor
        .get(&msg.tx_id)
        .map(|e| e.send(TransactionUpdate::Update(next.handle_default())))
        .ok_or(InternalError::E6003);

      if let Err(e) = send_result {
        debug!("{}ERROR:6001 {:?}", log_prefix, e);
        warn!(
          "Error sending message in transaction {}. This is likely a bug in the upstream (i.e. {})",
          msg.tx_id, upstream_port
        );
      }
    }

    ActorResult::reply(Ok(()))
  }
}
