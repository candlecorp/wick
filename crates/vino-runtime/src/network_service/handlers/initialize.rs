use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use actix::ActorTryFutureExt;
use vino_lattice::lattice::Lattice;

use crate::dev::prelude::*;
use crate::network_service::State;
use crate::schematic_service::handlers::update_provider::UpdateProvider;
type Result<T> = std::result::Result<T, NetworkError>;

#[derive(Message, Debug)]
#[rtype(result = "Result<()>")]
pub(crate) struct Initialize {
  pub(crate) network_uid: String,
  pub(crate) seed: String,
  pub(crate) network: NetworkDefinition,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) allow_latest: bool,
  pub(crate) lattice: Option<Arc<Lattice>>,
  pub(crate) timeout: Duration,
}

impl Handler<Initialize> for NetworkService {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: Initialize, _ctx: &mut Context<Self>) -> Self::Result {
    trace!("NETWORK:INIT:{}", msg.network_uid);
    self.started = true;

    let nuid = msg.network_uid;
    let global_providers = msg.network.providers.clone();
    let allowed_insecure = msg.allowed_insecure;
    let seed = msg.seed;
    let allow_latest = msg.allow_latest;
    let timeout = msg.timeout;
    let lattice = msg.lattice;

    let kp = actix_try!(keypair_from_seed(&seed), 5002);
    let schematics = msg.network.schematics.clone();
    let address_map = start_schematic_services(&schematics);

    self.schematics = address_map.clone();
    self.state = Some(State { kp });
    self.uid = nuid.clone();
    self.definition = msg.network;

    let task = async move {
      let provider_addr = start_network_provider(nuid).await?;
      let channel = ProviderChannel {
        namespace: SELF_NAMESPACE.to_owned(),
        recipient: provider_addr.clone().recipient(),
      };
      for def in schematics {
        let addr = address_map.get(&def.name).unwrap();
        let name = def.name.clone();
        trace!("NETWORK:SCHEMATIC[{}]", name);

        addr
          .send(crate::schematic_service::handlers::initialize::Initialize {
            seed: seed.clone(),
            lattice: lattice.clone(),
            network_provider_channel: Some(channel.clone()),
            schematic: def,
            allow_latest,
            allowed_insecure: allowed_insecure.clone(),
            global_providers: global_providers.clone(),
            timeout,
          })
          .await
          .map_err(|_| InternalError(5001))??;

        trace!("NETWORK:SCHEMATIC[{}]:INITIALIZED", name);
      }

      debug!("Schematics initialized");
      Ok((provider_addr.clone(), lattice))
    }
    .into_actor(self)
    .and_then(|(addr, lattice), network, _ctx| {
      let schematics = network.schematics.clone();
      network.lattice = lattice;
      trace!("NETWORK:PROVIDER:SELF:START");
      async move {
        // TODO Make cross-schematic resolution smarter.
        for _ in 1..5 {
          let result = create_network_provider_model("self", addr.clone()).await;
          if result.is_err() {
            continue;
          }
          let model = result.unwrap();
          let mut results = Vec::new();
          for addr in schematics.values() {
            results.push(
              addr
                .send(UpdateProvider {
                  model: model.clone(),
                })
                .await
                .map_err(|_| InternalError(5001))?,
            );
          }
          if results.iter().any(|r| r.is_err()) {
            continue;
          }
          return Ok(());
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
  trace!("NETWORK:Starting schematic services");
  map(schematics, |def| {
    let arbiter = Arbiter::new();
    let addr =
      SchematicService::start_in_arbiter(&arbiter.handle(), |_| SchematicService::default());
    (def.name.clone(), addr)
  })
}
