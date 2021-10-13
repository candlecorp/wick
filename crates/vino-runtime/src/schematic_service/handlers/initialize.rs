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
  pub(crate) timeout: Duration,
  pub(crate) providers: HashMap<String, ProviderChannel>,
  pub(crate) model: Arc<RwLock<SchematicModel>>,
}

impl Handler<Initialize> for SchematicService {
  type Result = ActorResult<Self, Result<(), SchematicError>>;

  fn handle(&mut self, msg: Initialize, _ctx: &mut Self::Context) -> Self::Result {
    let mut model = msg.model.write();
    self.name = model.get_name();
    trace!("SC[{}]:INIT", self.name);

    let allowed_providers = vec![
      model.get_allowed_providers().clone(),
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
    trace!("s0");
    for provider in allowed_providers {
      match msg.providers.get(&provider) {
        Some(channel) => {
          exposed_providers.insert(provider, channel.clone());
        }
        None => return ActorResult::reply(Err(SchematicError::ProviderNotFound(provider))),
      }
    }
    trace!("s1");
    self.providers = exposed_providers;
    trace!("s2");
    trace!("sa");
    actix_try!(SchematicValidator::validate_early_errors(&model));
    trace!("sb");

    let models: Vec<_> = self
      .providers
      .iter()
      .map(|(ns, pr)| (ns.clone(), pr.model.clone()))
      .collect();
    trace!("sc");

    actix_try!(model.commit_providers(models));
    trace!("sd");

    let state = State {
      transactions: TransactionExecutor::new(msg.model.clone(), msg.timeout),
      model: msg.model.clone(),
    };
    self.state = Some(state);
    actix_try!(SchematicValidator::validate_early_errors(&model));
    trace!("se");

    ActorResult::reply(Ok(()))
  }
}
