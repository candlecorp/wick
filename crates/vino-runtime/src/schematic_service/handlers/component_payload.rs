use crate::dev::prelude::*;
use crate::schematic_service::handlers::output_message::OutputMessage;
use crate::schematic_service::handlers::short_circuit::ShortCircuit;

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<(), SchematicError>")]
pub struct ComponentPayload {
  pub tx_id: String,
  pub instance: String,
  pub payload_map: TransportMap,
}

impl Handler<ComponentPayload> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: ComponentPayload, ctx: &mut Context<Self>) -> Self::Result {
    trace!("SC[{}]:INSTANCE[{}]:READY", self.name, msg.instance);
    let instance = msg.instance.clone();
    let tx_id = msg.tx_id;

    let def = actix_try!(get_component_definition(self.get_model(), &instance));

    if msg.payload_map.has_error() {
      let err_payload = msg.payload_map.take_error().unwrap();
      let addr = ctx.address();
      let msg = ShortCircuit::new(tx_id, instance, err_payload);
      return ActorResult::reply_async(
        async move { map_err!(addr.send(msg).await, InternalError::E6012)? }.into_actor(self),
      );
    }

    let invocation = InvocationMessage::from(Invocation::next(
      &tx_id,
      Entity::schematic(&self.name),
      Entity::component(def.namespace, def.name),
      msg.payload_map,
    ));

    let handler = actix_try!(self.get_provider(&msg.instance));

    let addr = ctx.address();
    let sc_name = self.name.clone();

    let task = async move {
      let target = invocation.get_target_url();

      let response = map_err!(
        tokio::spawn(async move { handler.invoke(invocation).await }).await,
        InternalError::E6009
      )??;

      match response {
        InvocationResponse::Stream { tx_id, mut rx } => {
          let log_prefix = format!("SC[{}]:OUTPUT:{}:{}:", sc_name, tx_id, target);
          trace!("{}:STREAM_HANDLER:START", log_prefix,);
          tokio::spawn(async move {
            while let Some(packet) = rx.next().await {
              let logmsg = format!("ref: {}, port: {}", instance, packet.port);
              let port = ConnectionTargetDefinition::new(instance.clone(), packet.port);
              let msg = OutputMessage {
                port,
                tx_id: tx_id.clone(),
                payload: packet.payload,
              };
              if addr.send(msg).await.is_err() {
                error!(
                  "{} Error sending output {} {}",
                  log_prefix,
                  logmsg,
                  InternalError::E6013
                );
              }
            }
            trace!("{}:STREAM_HANDLER:COMPLETE", log_prefix);
          });
          Ok(())
        }
        InvocationResponse::Error { tx_id, msg } => {
          warn!("Tx '{}' short-circuiting '{}': {}", tx_id, instance, msg);
          let msg = ShortCircuit::new(tx_id, instance, MessageTransport::error(msg));
          map_err!(addr.send(msg).await, InternalError::E6007)?
        }
      }
    };

    ActorResult::reply_async(task.into_actor(self))
  }
}
