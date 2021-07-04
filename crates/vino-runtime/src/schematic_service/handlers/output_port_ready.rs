use crate::dev::prelude::*;
use crate::schematic_service::handlers::payload_received::PayloadReceived;

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<(),SchematicError>")]
pub(crate) struct OutputPortReady {
  pub(crate) port: PortReference,
  pub(crate) tx_id: String,
  pub(crate) payload: MessageTransport,
}

impl Handler<OutputPortReady> for SchematicService {
  type Result = ResponseActFuture<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: OutputPortReady, ctx: &mut Context<Self>) -> Self::Result {
    trace!("Output ready on {}", msg.port);
    let defs = self.get_port_connections(&msg.port);
    let reference = msg.port.reference;
    let port = msg.port.name;
    let tx_id = msg.tx_id;
    let data = msg.payload;
    let addr = ctx.address();
    let task = async move {
      let origin = PortReference {
        name: port.clone(),
        reference: reference.clone(),
      };
      let to_message = |conn: Connection| PayloadReceived {
        tx_id: tx_id.clone(),
        origin: origin.clone(),
        target: PortReference {
          name: conn.to.name.clone(),
          reference: conn.to.reference,
        },
        payload: data.clone(),
      };
      join_or_err(
        defs.into_iter().map(to_message).map(|ips| addr.send(ips)),
        6001,
      )
      .await?;

      Ok(())
    }
    .into_actor(self);

    Box::pin(task)
  }
}
