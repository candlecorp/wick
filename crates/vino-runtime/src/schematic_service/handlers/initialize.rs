use std::sync::Arc;
use std::time::Duration;

use actix::ActorTryFutureExt;
use parking_lot::RwLock;
use vino_lattice::lattice::Lattice;

use crate::dev::prelude::*;
use crate::models::validator::Validator;
use crate::providers::{
  initialize_grpc_provider,
  initialize_lattice_provider,
  initialize_network_provider,
  initialize_wasm_provider,
  NetworkOptions,
};
use crate::schematic_service::State;
use crate::transaction::executor::TransactionExecutor;

#[derive(Message, Debug)]
#[rtype(result = "Result<(), SchematicError>")]
pub(crate) struct Initialize {
  pub(crate) schematic: SchematicDefinition,
  pub(crate) network_provider_channel: Option<ProviderChannel>,
  pub(crate) seed: String,
  pub(crate) lattice: Option<Arc<Lattice>>,
  pub(crate) allow_latest: bool,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) global_providers: Vec<ProviderDefinition>,
  pub(crate) timeout: Duration,
}

impl Handler<Initialize> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
    trace!("SC[{}]:INIT", msg.schematic.get_name());
    let seed = msg.seed;
    let allow_latest = msg.allow_latest;
    self.name = msg.schematic.name.clone();
    let providers = concat(vec![msg.global_providers, msg.schematic.providers.clone()]);
    let model = actix_try!(SchematicModel::try_from(msg.schematic), 6021);
    actix_try!(Validator::validate_early_errors(&model), 6022);
    let allowed_insecure = msg.allowed_insecure;
    let network_provider_channel = msg.network_provider_channel;
    let model = Arc::new(RwLock::new(model));

    let task = initialize_providers(
      providers,
      seed,
      msg.lattice,
      allow_latest,
      allowed_insecure,
      msg.timeout,
    )
    .into_actor(self)
    .map_ok(move |(mut channels, providers), schematic, _ctx| {
      if let Some(network_provider_channel) = network_provider_channel {
        channels.push(network_provider_channel);
      }
      schematic.providers = channels
        .into_iter()
        .map(|prv_channel| (prv_channel.namespace.clone(), prv_channel))
        .collect();
      let mut model = schematic.get_model().write();
      model.commit_providers(providers);
      Ok::<_, SchematicError>(model.partial_initialization()?)
    })
    .map(|result, _schematic, _| {
      if result.is_ok() {
        _schematic.validate_model()
      } else {
        result?
      }
    });

    let state = State {
      transactions: TransactionExecutor::new(model.clone(), msg.timeout),
      model,
    };
    self.state = Some(state);

    ActorResult::reply_async(task)
  }
}

async fn initialize_providers(
  providers: Vec<ProviderDefinition>,
  seed: String,
  lattice: Option<Arc<Lattice>>,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
  timeout: Duration,
) -> Result<(Vec<ProviderChannel>, Vec<ProviderModel>), SchematicError> {
  let (channel, provider_model) = initialize_native_provider("vino::v0").await?;
  let mut channels = vec![channel];
  let mut models = vec![provider_model];

  let num_providers = providers.len();
  for provider in providers {
    let namespace = provider.namespace.clone();

    let result = match provider.kind {
      ProviderKind::Network => {
        let opts = NetworkOptions {
          seed: &seed,
          allow_latest,
          insecure: &allowed_insecure,
          lattice: &lattice,
          timeout,
        };
        initialize_network_provider(provider, &namespace, opts).await},
      ProviderKind::Native => unreachable!(), // Should not be handled via this route
      ProviderKind::GrpcUrl => initialize_grpc_provider(provider, &seed, &namespace).await,
      ProviderKind::Wapc => {
        initialize_wasm_provider(provider, &namespace, allow_latest, &allowed_insecure).await
      }
      ProviderKind::Lattice => match &lattice {
        Some(lattice) => initialize_lattice_provider(provider, &namespace, lattice.clone()).await,
        None => Err(ProviderError::Lattice(
          "Attempted to initialize a lattice provider without an active lattice connection. Did you enable the lattice on the command line or in a manifest?"
            .to_owned(),
        )),
      },
    };
    let (channel, provider_model) = result?;
    channels.push(channel);
    models.push(provider_model);
  }
  trace!("SC:PROVIDERS:REGISTERED[{}]", num_providers);
  Ok((channels, models))
}
