use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use actix::ActorTryFutureExt;
use parking_lot::RwLock;
use vino_lattice::lattice::Lattice;

use crate::dev::prelude::validator::NetworkValidator;
use crate::dev::prelude::*;
use crate::network_service::State;
use crate::providers::native_provider_service::NativeProviderService;
use crate::providers::{
  initialize_grpc_provider,
  initialize_lattice_provider,
  initialize_network_provider,
  initialize_wasm_provider,
  NetworkOptions,
};
use crate::VINO_V0_NAMESPACE;
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
  pub(crate) rng_seed: u64,
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

    let kp = actix_try_or_err!(keypair_from_seed(&seed), InternalError::E5002);
    let schematics = msg.network.schematics.clone();
    let address_map = start_schematic_services(&schematics);

    let model = actix_try!(NetworkModel::try_from(msg.network.clone()));
    let model = Arc::new(RwLock::new(model));
    self.definition = msg.network;
    self.lattice = msg.lattice;
    self.schematics = address_map.clone();
    self.uid = nuid.clone();
    let inner_model = model.clone();
    self.state = Some(State { kp, model });

    let task = initialize_providers(
      global_providers,
      msg.rng_seed,
      nuid.clone(),
      seed.clone(),
      self.lattice.clone(),
      allow_latest,
      allowed_insecure,
      msg.timeout,
    )
    .into_actor(self)
    .and_then(|channels, network, _ctx| {
      network.providers = channels
        .into_iter()
        .map(|prv_channel| (prv_channel.namespace.clone(), prv_channel))
        .collect();
      start_self_network(nuid).into_actor(network)
    })
    .and_then(
      move |(provider_addr, self_channel), network: &mut NetworkService, _ctx| {
        network
          .providers
          .insert(SELF_NAMESPACE.to_owned(), self_channel);
        let providers = network.providers.clone();
        initialize_schematics(
          inner_model,
          schematics,
          address_map,
          seed.clone(),
          timeout,
          providers,
          provider_addr,
        )
        .into_actor(network)
      },
    )
    .and_then(|addr, network: &mut NetworkService, _ctx| {
      start_self_provider(addr).into_actor(network)
    })
    .and_then(|self_model, network: &mut NetworkService, _ctx| {
      let state = network.state.as_mut().unwrap();
      let mut self_channel = network.providers.get_mut(SELF_NAMESPACE).unwrap();
      self_channel.model = Some(self_model);

      let result: Result<_> = update_providers(&state.model, &network.providers)
        .and_then(|_| state.model.write().finalize().map_err(|e| e.into()))
        .and_then(|_| NetworkValidator::validate(&state.model.write()).map_err(|e| e.into()));

      async { result }.into_actor(network)
    });

    ActorResult::reply_async(task)
  }
}

fn update_providers(
  model: &Arc<RwLock<NetworkModel>>,
  providers: &HashMap<String, ProviderChannel>,
) -> Result<()> {
  let mut map = HashMap::new();
  for (ns, channel) in providers {
    let ns = ns.clone();
    map.insert(ns, channel.model.clone());
  }
  let result = model.write().update_providers(map)?;
  Ok(result)
}

async fn start_self_network(
  nuid: String,
) -> Result<(Addr<NativeProviderService>, ProviderChannel)> {
  trace!("NETWORK:PROVIDER:SELF:START");
  let provider_addr = start_network_provider(nuid).await?;
  let self_channel = ProviderChannel {
    namespace: SELF_NAMESPACE.to_owned(),
    recipient: provider_addr.clone().recipient(),
    model: None,
  };
  trace!("NETWORK:PROVIDER:SELF:STARTED");
  Ok((provider_addr, self_channel))
}

async fn initialize_schematics(
  model: Arc<RwLock<NetworkModel>>,
  schematics: Vec<SchematicDefinition>,
  addresses: HashMap<String, Addr<SchematicService>>,
  seed: String,
  timeout: Duration,
  providers: HashMap<String, ProviderChannel>,
  provider_addr: Addr<NativeProviderService>,
) -> Result<Addr<NativeProviderService>> {
  for def in schematics {
    let addr = addresses.get(&def.name).unwrap();
    let name = def.name.clone();
    trace!("NETWORK:SCHEMATIC[{}]", name);
    let schematic_model = model.read().get_schematic(&name).unwrap().clone();
    addr
      .send(crate::schematic_service::handlers::initialize::Initialize {
        seed: seed.clone(),
        schematic: def,
        timeout,
        providers: providers.clone(),
        model: schematic_model,
      })
      .await
      .map_err(|_| InternalError::E5001)??;
    trace!("NETWORK:SCHEMATIC[{}]:INITIALIZED", name);
  }

  Ok(provider_addr)
}

async fn start_self_provider(addr: Addr<NativeProviderService>) -> Result<ProviderModel> {
  trace!("NETWORK:PROVIDER:SELF:QUERY");
  let model = create_network_provider_model(addr.clone()).await?;
  trace!("NETWORK:PROVIDER:SELF:SUCCESS");

  Ok(model.into())
}

fn start_schematic_services(
  schematics: &[SchematicDefinition],
) -> HashMap<String, Addr<SchematicService>> {
  trace!("NETWORK:SCHEMATICS:STARTING");
  let result = map(schematics, |def| {
    // let arbiter = Arbiter::new();
    let arbiter = Arbiter::with_tokio_rt(|| tokio::runtime::Runtime::new().unwrap());
    let addr =
      SchematicService::start_in_arbiter(&arbiter.handle(), |_| SchematicService::default());
    (def.name.clone(), addr)
  });
  trace!("NETWORK:SCHEMATICS:STARTED");
  result
}

#[allow(clippy::too_many_arguments)]
async fn initialize_providers(
  providers: Vec<ProviderDefinition>,
  rng_seed: u64,
  nuid: String,
  seed: String,
  lattice: Option<Arc<Lattice>>,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
  timeout: Duration,
) -> Result<Vec<ProviderChannel>> {
  let channel = initialize_native_provider(VINO_V0_NAMESPACE, rng_seed).await?;
  let mut channels = vec![channel];

  for provider in providers {
    let namespace = provider.namespace.clone();

    let result = match provider.kind {
      ProviderKind::Network => {
        let opts = NetworkOptions {
          rng_seed,
          allow_latest,
          insecure: &allowed_insecure,
          lattice: &lattice,
          timeout,
        };
        initialize_network_provider(provider, &namespace, opts).await},
      ProviderKind::Native => unreachable!(), // Should not be handled via this route
      ProviderKind::GrpcUrl => initialize_grpc_provider(provider, &seed, &namespace).await,
      ProviderKind::Wapc => {
        initialize_wasm_provider(provider, &namespace, allow_latest, &allowed_insecure, nuid.clone()).await
      }
      ProviderKind::Lattice => match &lattice {
        Some(lattice) => initialize_lattice_provider(provider, &namespace, lattice.clone()).await,
        None => Err(ProviderError::Lattice(
          "Attempted to initialize a lattice provider without an active lattice connection. Did you enable the lattice on the command line or in a manifest?"
            .to_owned(),
        )),
      },
    };
    let channel = result?;
    channels.push(channel);
  }

  trace!("NETWORK:PROVIDERS:REGISTERED[{}]", channels.len());
  Ok(channels)
}
