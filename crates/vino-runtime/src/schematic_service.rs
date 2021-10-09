pub(crate) mod default;
pub(crate) mod error;
pub(crate) mod handlers;

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use tokio::sync::mpsc::{
  UnboundedReceiver,
  UnboundedSender,
};
pub(crate) mod input_message;

use error::SchematicError;

use crate::dev::prelude::*;
use crate::transaction::executor::TransactionExecutor;

type Result<T> = std::result::Result<T, SchematicError>;

#[derive(Debug)]
pub(crate) struct SchematicService {
  name: String,
  providers: HashMap<String, ProviderChannel>,
  state: Option<State>,
  executor: HashMap<String, UnboundedSender<TransactionUpdate>>,
  rng_seed: u64,
}

#[derive(Debug)]
struct State {
  model: Arc<RwLock<SchematicModel>>,
  transactions: TransactionExecutor,
}

impl Supervised for SchematicService {}

impl Default for SchematicService {
  fn default() -> Self {
    SchematicService {
      name: "".to_owned(),
      providers: HashMap::new(),
      state: None,
      executor: HashMap::new(),
      rng_seed: new_seed(),
    }
  }
}

impl Actor for SchematicService {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    trace!("SC:Service starting");
  }

  fn stopped(&mut self, _ctx: &mut Self::Context) {}
}

impl SchematicService {
  fn get_state(&self) -> &State {
    if self.state.is_none() {
      panic!("Internal Error: schematic uninitialized");
    }
    let state = self.state.as_ref().unwrap();
    state
  }

  fn get_state_mut(&mut self) -> &mut State {
    if self.state.is_none() {
      panic!("Internal Error: schematic uninitialized");
    }
    let state = self.state.as_mut().unwrap();
    state
  }

  fn start(
    &mut self,
    tx_id: String,
  ) -> (
    UnboundedReceiver<TransactionUpdate>,
    UnboundedSender<TransactionUpdate>,
  ) {
    let state = self.get_state_mut();
    state.transactions.new_transaction(tx_id)
  }

  fn get_model(&self) -> &SharedModel {
    &self.get_state().model
  }

  fn get_provider(&self, instance: &str) -> Result<Arc<BoxedInvocationHandler>> {
    let component = get_component_definition(self.get_model(), instance)?;
    let model = self.get_model().read();
    let err = SchematicError::InstanceNotFound(instance.to_owned());
    if !model.has_component(&component) {
      warn!(
        "SC[{}]: {} does not have a valid model.",
        self.name, instance
      );
      return Err(err);
    }
    trace!(
      "SC[{}]:INSTANCE[{}]->[{}]",
      self.name,
      instance,
      component.id()
    );
    let channel = self.providers.get(&component.namespace).ok_or(err)?;
    Ok(channel.recipient.clone())
  }
}

#[derive(Debug, Clone)]
pub(crate) struct ProviderInitResponse {
  pub(crate) model: ProviderModel,
  pub(crate) channel: ProviderChannel,
}
