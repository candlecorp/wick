use std::sync::Arc;

use futures::future::BoxFuture;
use parking_lot::Mutex;
use serde_json::Value;
use tracing_futures::Instrument;
use seeded_random::{Random, Seed};
use wasmflow_transport::TransportStream;
use wasmflow_interface::ProviderSignature;
use wasmflow_invocation::Invocation;

use crate::constants::*;
use crate::interpreter::program::ProgramState;
use crate::{BoxError, HandlerMap, InterpreterDispatchChannel, Provider, SchematicExecutor};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub(crate) enum Error {
  #[error("Schematic {0} not found on this network")]
  SchematicNotFound(String),
}

#[derive(Debug)]
pub(crate) struct SchematicProvider {
  signature: ProviderSignature,
  schematics: Arc<Vec<SchematicExecutor>>,
  providers: Arc<HandlerMap>,
  self_provider: Mutex<Option<Arc<Self>>>,
  rng: Random,
}

impl SchematicProvider {
  pub(crate) fn new(
    providers: Arc<HandlerMap>,
    state: &ProgramState,
    dispatcher: &InterpreterDispatchChannel,
    seed: Seed,
  ) -> Arc<Self> {
    let schematics: Arc<Vec<SchematicExecutor>> = Arc::new(
      state
        .network
        .schematics()
        .iter()
        .map(|s| SchematicExecutor::new(s.clone(), dispatcher.clone()))
        .collect(),
    );
    let signature = state.providers.get(NS_SELF).unwrap().clone();
    let provider = Arc::new(Self {
      signature,
      schematics,
      self_provider: Mutex::new(None),
      providers,
      rng: Random::from_seed(seed),
    });
    provider.update_self_provider();
    provider
  }

  fn update_self_provider(self: &Arc<Self>) {
    let mut lock = self.self_provider.lock();
    lock.replace(self.clone());
    drop(lock);
  }

  fn clone_self_provider(&self) -> Arc<Self> {
    let lock = self.self_provider.lock();
    lock.clone().unwrap()
  }
}

impl Provider for SchematicProvider {
  fn handle(&self, invocation: Invocation, _config: Option<Value>) -> BoxFuture<Result<TransportStream, BoxError>> {
    trace!(target = %invocation.target, id=%invocation.id,namespace = NS_SELF);

    let operation = invocation.target.name().to_owned();
    let fut = self
      .schematics
      .iter()
      .find(|s| s.name() == operation)
      .map(|s| {
        s.invoke(
          invocation,
          self.rng.seed(),
          self.providers.clone(),
          self.clone_self_provider(),
        )
      })
      .ok_or_else(|| Error::SchematicNotFound(operation.clone()));

    Box::pin(async move {
      let span = trace_span!("ns_self", name = %operation);
      match fut {
        Ok(fut) => fut.instrument(span).await.map_err(|e| e.into()),
        Err(e) => Err(e.into()),
      }
    })
  }

  fn list(&self) -> &ProviderSignature {
    &self.signature
  }
}
