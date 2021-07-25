use std::collections::HashMap;

use crate::dev::prelude::*;
use crate::schematic_service::handlers::output_message::OutputMessage;
use crate::schematic_service::handlers::short_circuit::ShortCircuit;

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<(), SchematicError>")]
pub(crate) struct ComponentPayload {
  pub(crate) tx_id: String,
  pub(crate) instance: String,
  pub(crate) payload_map: HashMap<String, MessageTransport>,
}

impl Handler<ComponentPayload> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: ComponentPayload, ctx: &mut Context<Self>) -> Self::Result {
    trace!("Reference '{}' is ready to continue", msg.instance);
    let kp = &self.get_state().kp;
    let instance = msg.instance.clone();
    let tx_id = msg.tx_id;

    let def = actix_try!(self.get_component_definition(&instance));

    let mut invoke_payload = HashMap::new();
    for (name, payload) in msg.payload_map {
      match payload {
        MessageTransport::MessagePack(bytes) => {
          invoke_payload.insert(name, bytes);
        }
        payload => {
          let addr = ctx.address();
          let msg = ShortCircuit::new(tx_id, instance, payload);
          return ActorResult::reply_async(
            async move { log_ie!(addr.send(msg).await, 6010,)? }.into_actor(self),
          );
        }
      }
    }

    let invocation = Invocation::next(
      &tx_id,
      kp,
      Entity::system("SchematicService", "Component Invocation"),
      Entity::Component(def.name),
      MessageTransport::MultiBytes(invoke_payload),
    );
    let handler = actix_try!(self.get_recipient(&msg.instance));

    let addr = ctx.address();

    let task = async move {
      let target = invocation.target.url();

      let response = log_ie!(handler.send(invocation).await, 6009)?;

      match response {
        InvocationResponse::Stream { tx_id, mut rx } => {
          let log_prefix = format!("Output:{}:{}:", tx_id, target);
          trace!("{} handler spawned", log_prefix,);
          tokio::spawn(async move {
            while let Some(packet) = rx.next().await {
              let logmsg = format!("ref: {}, port: {}", instance, packet.port);
              let port = ConnectionTargetDefinition::new(instance.clone(), packet.port);
              let msg = OutputMessage {
                port,
                tx_id: tx_id.clone(),
                payload: packet.payload.into(),
              };
              if addr.send(msg).await.is_err() {
                error!(
                  "{} Error sending output {} {}",
                  log_prefix,
                  logmsg,
                  InternalError(6013)
                );
              }
            }
            trace!("Task finished");
          });
          Ok(())
        }
        InvocationResponse::Error { tx_id, msg } => {
          warn!("Tx '{}' short-circuiting '{}': {}", tx_id, instance, msg);
          let msg = ShortCircuit::new(tx_id, instance, MessageTransport::Error(msg));
          log_ie!(addr.send(msg).await, 6007)?
        }
      }
    };

    ActorResult::reply_async(task.into_actor(self))
  }
}
