use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use vino_lattice::lattice::Lattice;

use crate::dev::prelude::*;
use crate::models::validator::Validator;
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
    trace!("SC:{}:INIT", msg.schematic.get_name());
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
      seed.clone(),
      msg.lattice,
      allow_latest,
      allowed_insecure,
    )
    .into_actor(self)
    .map(move |result, this, _ctx| {
      match result {
        Ok((mut channels, providers)) => {
          if let Some(network_provider_channel) = network_provider_channel {
            channels.push(network_provider_channel);
          }
          this.recipients = channels
            .into_iter()
            .map(|c| (c.namespace.clone(), c))
            .collect();
          let mut model = this.get_model().write();
          model.commit_providers(providers);
          model.partial_initialization()?;
        }
        Err(e) => {
          error!("Error starting providers: {}", e);
        }
      }
      Ok!(())
    });
    let task = task.map(|_, this, _| this.validate_model());

    let state = State {
      kp: KeyPair::from_seed(&seed).unwrap(),
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
) -> Result<(Vec<ProviderChannel>, Vec<ProviderModel>), SchematicError> {
  let (channel, provider_model) = initialize_native_provider("vino::v0").await?;
  let mut channels = vec![channel];
  let mut models = vec![provider_model];

  for provider in providers {
    let (channel, provider_model) =
      initialize_provider(provider, &seed, &lattice, allow_latest, &allowed_insecure).await?;
    channels.push(channel);
    models.push(provider_model);
  }
  Ok((channels, models))
}
