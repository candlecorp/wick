use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use vino_lattice::lattice::Lattice;
use vino_lattice::nats::NatsOptions;

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
  pub(crate) lattice_config: Option<NatsOptions>,
  pub(crate) timeout: Duration,
}

impl Handler<Initialize> for NetworkService {
  type Result = ActorResult<Self, Result<()>>;

  fn handle(&mut self, msg: Initialize, _ctx: &mut Context<Self>) -> Self::Result {
    trace!("NETWORK:INIT:{}", msg.network_id);
    self.started = true;

    let network_id = msg.network_id;
    let global_providers = msg.network.providers.clone();
    let allowed_insecure = msg.allowed_insecure;
    let seed = msg.seed;
    let allow_latest = msg.allow_latest;
    let timeout = msg.timeout;
    let lattice_config = msg.lattice_config;

    let kp = actix_try!(keypair_from_seed(&seed), 5002);
    let schematics = msg.network.schematics.clone();
    let address_map = start_schematic_services(&schematics);

    self.schematics = address_map.clone();
    self.state = Some(State { kp });
    self.id = network_id.clone();
    self.definition = msg.network;

    let task = async move {
      let lattice = match lattice_config {
        Some(config) => Some(Arc::new(Lattice::connect(config).await?)),
        None => None,
      };
      let provider_addr = start_network_provider(SELF_NAMESPACE, network_id).await?;
      let channel = ProviderChannel {
        namespace: SELF_NAMESPACE.to_owned(),
        recipient: provider_addr.clone().recipient(),
      };
      let init_msgs = schematics.into_iter().map(|def| {
        let addr = address_map.get(&def.name).unwrap();
        addr.send(crate::schematic_service::handlers::initialize::Initialize {
          seed: seed.clone(),
          lattice: lattice.clone(),
          network_provider_channel: Some(channel.clone()),
          schematic: def,
          allow_latest,
          allowed_insecure: allowed_insecure.clone(),
          global_providers: global_providers.clone(),
          timeout,
        })
      });

      let mut results = Vec::new();
      for msg in init_msgs {
        results.push(msg.await.map_err(|_| InternalError(5001))?);
      }

      let errors: Vec<SchematicError> = filter_map(results, |e| e.err());
      if errors.is_empty() {
        debug!("Schematics initialized");
        Ok((provider_addr.clone(), lattice))
      } else {
        Err(NetworkError::InitializationError(errors))
      }
    }
    .into_actor(self)
    .then(|result, network, _ctx| {
      let schematics = network.schematics.clone();
      if let Err(e) = result {
        error!("Initialization error: {}", e);
        panic!();
      }
      let (addr, lattice) = result.unwrap();
      network.lattice = lattice;
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
