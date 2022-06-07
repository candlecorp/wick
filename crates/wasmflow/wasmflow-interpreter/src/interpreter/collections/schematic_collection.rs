use std::sync::Arc;

use futures::future::BoxFuture;
use parking_lot::Mutex;
use seeded_random::{Random, Seed};
use serde_json::Value;
use tracing_futures::Instrument;
use wasmflow_interface::CollectionSignature;
use wasmflow_invocation::Invocation;
use wasmflow_transport::TransportStream;

use crate::constants::*;
use crate::interpreter::program::ProgramState;
use crate::{BoxError, Collection, HandlerMap, InterpreterDispatchChannel, SchematicExecutor};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub(crate) enum Error {
  #[error("Schematic {0} not found on this network")]
  SchematicNotFound(String),
}

#[derive(Debug)]
pub(crate) struct SchematicCollection {
  signature: CollectionSignature,
  schematics: Arc<Vec<SchematicExecutor>>,
  collections: Arc<HandlerMap>,
  self_collection: Mutex<Option<Arc<Self>>>,
  rng: Random,
}

impl SchematicCollection {
  pub(crate) fn new(
    collections: Arc<HandlerMap>,
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
    let signature = state.collections.get(NS_SELF).unwrap().clone();
    let collection = Arc::new(Self {
      signature,
      schematics,
      self_collection: Mutex::new(None),
      collections,
      rng: Random::from_seed(seed),
    });
    collection.update_self_collection();
    collection
  }

  fn update_self_collection(self: &Arc<Self>) {
    let mut lock = self.self_collection.lock();
    lock.replace(self.clone());
    drop(lock);
  }

  fn clone_self_collection(&self) -> Arc<Self> {
    let lock = self.self_collection.lock();
    lock.clone().unwrap()
  }
}

impl Collection for SchematicCollection {
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
          self.collections.clone(),
          self.clone_self_collection(),
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

  fn list(&self) -> &CollectionSignature {
    &self.signature
  }
}
