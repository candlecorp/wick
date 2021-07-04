use crate::dev::prelude::*;
use crate::schematic_service::handlers::output_port_ready::OutputPortReady;

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<(), SchematicError>")]
pub struct ComponentOutput {
  pub port: String,
  pub invocation_id: String,
  pub payload: Packet,
}

/// Maps output by invocation ID to its transaction and reference
impl Handler<ComponentOutput> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: ComponentOutput, ctx: &mut Context<Self>) -> Self::Result {
    let metadata = self.invocation_map.get(&msg.invocation_id).cloned();
    let (tx_id, schematic_name, entity) = metadata.unwrap();
    trace!(
      "Got output for tx '{}' on schematic '{}' and port {}",
      tx_id,
      schematic_name,
      entity
    );

    let receiver = ctx.address();
    let payload = msg.payload;
    let port = msg.port;

    ActorResult::reply_async(
      async move {
        let port = PortReference {
          name: port,
          reference: entity.into_reference()?,
        };
        trace!("Sending output ready to schematic");
        receiver
          .send(OutputPortReady {
            port,
            tx_id,
            payload: payload.into(),
          })
          .await
          .map_err(|_| InternalError(6013))??;
        trace!("Sent output ready to schematic");
        Ok(())
      }
      .into_actor(self),
    )
  }
}
