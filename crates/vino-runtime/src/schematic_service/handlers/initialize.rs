use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;

use crate::dev::prelude::*;
use crate::models::validator::SchematicValidator;
use crate::schematic_service::State;
use crate::transaction::executor::TransactionExecutor;
use crate::VINO_V0_NAMESPACE;

#[derive(Message, Debug)]
#[rtype(result = "Result<(), SchematicError>")]
pub(crate) struct Initialize {
  pub(crate) schematic: SchematicDefinition,
  pub(crate) seed: String,
  pub(crate) timeout: Duration,
  pub(crate) providers: HashMap<String, ProviderChannel>,
  pub(crate) model: Arc<RwLock<SchematicModel>>,
}

impl Handler<Initialize> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
    trace!("SC[{}]:INIT", msg.schematic.get_name());

    self.name = msg.schematic.name.clone();
    let allowed_providers = vec![
      msg.schematic.providers.clone(),
      vec![VINO_V0_NAMESPACE.to_owned(), SELF_NAMESPACE.to_owned()],
    ]
    .concat();
    trace!(
      "SC[{}]:AVAIL_PROVIDERS[{}]:ALLOWED_PROVIDERS[{}]",
      self.name,
      msg.providers.iter().map(|(k, _)| k).join(","),
      allowed_providers.join(",")
    );
    let mut exposed_providers = HashMap::new();
    for provider in allowed_providers {
      match msg.providers.get(&provider) {
        Some(channel) => {
          exposed_providers.insert(provider, channel.clone());
        }
        None => return ActorResult::reply(Err(SchematicError::ProviderNotFound(provider))),
      }
    }
    self.providers = exposed_providers;
    let mut model = msg.model.write();

    actix_try!(SchematicValidator::validate_early_errors(&model));

    let models: Vec<_> = self
      .providers
      .iter()
      .map(|(ns, pr)| (ns.clone(), pr.model.clone()))
      .collect();

    actix_try!(model.commit_providers(models));

    let state = State {
      transactions: TransactionExecutor::new(msg.model.clone(), msg.timeout),
      model: msg.model.clone(),
    };
    self.state = Some(state);
    actix_try!(SchematicValidator::validate_early_errors(&model));

    ActorResult::reply(Ok(()))
  }
}
