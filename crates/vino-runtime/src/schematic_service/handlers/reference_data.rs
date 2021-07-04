use std::collections::HashMap;

use nkeys::KeyPair;

use crate::dev::prelude::*;
use crate::schematic_service::handlers::short_circuit::ShortCircuit;

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<(), SchematicError>")]
pub(crate) struct ReferenceData {
  pub(crate) tx_id: String,
  pub(crate) reference: String,
  pub(crate) payload_map: HashMap<String, MessageTransport>,
}

impl Handler<ReferenceData> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: ReferenceData, ctx: &mut Context<Self>) -> Self::Result {
    trace!("Reference '{}' is ready to continue", msg.reference);
    let seed = self.get_state().seed.clone();
    let reference = msg.reference.clone();
    let tx_id = msg.tx_id;

    let kp = actix_try!(KeyPair::from_seed(&seed));
    let def = self.get_component_model(&msg.reference).unwrap();

    let mut invoke_payload = HashMap::new();
    for (name, payload) in msg.payload_map {
      match payload {
        MessageTransport::MessagePack(bytes) => {
          invoke_payload.insert(name, bytes);
        }
        payload => {
          let addr = ctx.address();
          return ActorResult::reply_async(
            async move {
              addr
                .send(ShortCircuit {
                  payload,
                  reference,
                  tx_id,
                })
                .await
                .map_err(|_| InternalError(6010))?
            }
            .into_actor(self),
          );
        }
      }
    }

    let invocation = Invocation::next(
      &tx_id,
      &kp,
      Entity::Schematic("<state>".to_owned()),
      Entity::Component(ComponentEntity {
        name: def.name,
        reference: msg.reference.clone(),
      }),
      MessageTransport::MultiBytes(invoke_payload),
    );
    let handler = actix_try!(self
      .get_recipient(&msg.reference)
      .ok_or_else(|| SchematicError::ReferenceNotFound(reference.clone())));

    let addr = ctx.address();
    let name = self.get_name();

    self.invocation_map.insert(
      invocation.id.clone(),
      (tx_id, name.clone(), invocation.target.clone()),
    );

    let task = async move {
      let target = invocation.target.url();

      let response = handler
        .send(invocation)
        .await
        .map_err(|_| InternalError(6009))?;

      match response {
        InvocationResponse::Stream { tx_id, mut rx } => {
          trace!(
            "spawning task to handle output for {}:{}|{}",
            tx_id,
            name,
            target
          );
          tokio::spawn(async move {
            while let Some(next) = rx.next().await {
              match addr.send(next).await {
                Ok(_) => {
                  debug!(
                    "Sent ready output to network for {}:{}:{}",
                    tx_id, name, target
                  );
                }
                Err(e) => {
                  error!("Error sending ready output: {}", e);
                }
              };
            }
            trace!("Task finished");
          });
          Ok(())
        }
        InvocationResponse::Error { tx_id, msg } => {
          warn!(
            "Tx '{}': schematic '{}' short-circuiting from '{}': {}",
            tx_id, name, reference, msg
          );
          addr
            .send(ShortCircuit {
              tx_id: tx_id.clone(),
              reference: reference.clone(),
              payload: MessageTransport::Error(msg),
            })
            .await
            .map_err(|_| InternalError(6007))?
        }
      }
    };

    ActorResult::reply_async(task.into_actor(self))
  }
}
