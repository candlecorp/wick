use std::collections::HashMap;

use crate::dev::prelude::provider_model::{
  create_network_provider_model,
  start_network_provider,
};
use crate::dev::prelude::*;
use crate::network_service::State;
use crate::schematic_service::handlers::update_provider::UpdateProvider;
type Result<T> = std::result::Result<T, NetworkError>;

#[derive(Message, Debug)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) network_id: String,
  pub(crate) seed: String,
  pub(crate) network: NetworkDefinition,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) allow_latest: bool,
}

impl Handler<Initialize> for NetworkService {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: Initialize, _ctx: &mut Context<Self>) -> Self::Result {
    trace!("Network {} initializing", msg.network_id);
    self.started = true;

    self.id = msg.network_id.clone();
    let network_id = msg.network_id;
    self.definition = msg.network;
    let allowed_insecure = msg.allowed_insecure;

    let seed = msg.seed;
    let kp = actix_try!(keypair_from_seed(&seed));
    self.state = Some(State { kp });
    let allow_latest = msg.allow_latest;
    let schematics = self.definition.schematics.clone();
    self.schematics = start_schematic_services(&schematics);
    let address_map = self.schematics.clone();

    let task = async move {
      let provider_addr = start_network_provider(SELF_NAMESPACE, network_id).await?;
      let channel = ProviderChannel {
        namespace: SELF_NAMESPACE.to_owned(),
        recipient: provider_addr.clone().recipient(),
      };
      let init_msgs = schematics.into_iter().map(|def| {
        let addr = address_map.get(&def.name).unwrap();
        addr.send(crate::schematic_service::handlers::initialize::Initialize {
          seed: seed.clone(),
          network_provider_channel: Some(channel.clone()),
          schematic: def,
          allow_latest,
          allowed_insecure: allowed_insecure.clone(),
        })
      });

      let results = join_or_err(init_msgs, 5001).await?;

      let errors: Vec<SchematicError> = filter_map(results, |e| e.err());
      if errors.is_empty() {
        debug!("Schematics initialized");
        Ok(provider_addr.clone())
      } else {
        Err(NetworkError::InitializationError(errors))
      }
    }
    .into_actor(self)
    .then(|result, network, _ctx| {
      let schematics = network.schematics.clone();
      async move {
        let addr = result?;
        for _ in 1..5 {
          let result = create_network_provider_model("self", addr.clone()).await;
          if result.is_err() {
            continue;
          }
          let model = result.unwrap();
          let result = join_or_err(
            schematics.values().map(|addr| {
              addr.send(UpdateProvider {
                model: model.clone(),
              })
            }),
            5020,
          )
          .await;
          if result.is_ok() {
            return Ok(());
          }
        }
        Err(NetworkError::MaxTriesReached)
      }
      .into_actor(network)
    });

    ActorResult::reply_async(task)
  }
}

fn start_schematic_services(
  schematics: &[SchematicDefinition],
) -> HashMap<String, Addr<SchematicService>> {
  trace!("Starting schematic arbiters");
  map(schematics, |def| {
    let arbiter = Arbiter::new();
    let addr =
      SchematicService::start_in_arbiter(&arbiter.handle(), |_| SchematicService::default());
    (def.name.clone(), addr)
  })
}
