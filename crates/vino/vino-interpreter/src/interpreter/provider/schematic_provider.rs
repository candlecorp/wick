use std::sync::Arc;

use futures::future::BoxFuture;
use parking_lot::Mutex;
use serde_json::Value;
use tracing_futures::Instrument;
use vino_transport::{Invocation, TransportStream};
use vino_types::{MapWrapper, ProviderSignature};

use crate::interpreter::program::ProgramState;
use crate::{BoxError, InterpreterDispatchChannel, Provider, Providers, SchematicExecutor};

pub(crate) const SELF_NAMESPACE: &str = "self";

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub(crate) enum Error {
  #[error("Schematic {0} not found on this network")]
  SchematicNotFound(String),
}

#[derive(Debug)]
pub(crate) struct SchematicProvider {
  signature: ProviderSignature,
  schematics: Arc<Vec<SchematicExecutor>>,
  providers: Arc<Providers>,
  self_provider: Mutex<Option<Arc<Self>>>,
}

impl SchematicProvider {
  pub(crate) fn new(
    providers: Arc<Providers>,
    state: &ProgramState,
    dispatcher: &InterpreterDispatchChannel,
  ) -> Arc<Self> {
    let schematics: Arc<Vec<SchematicExecutor>> = Arc::new(
      state
        .network
        .schematics()
        .iter()
        .map(|s| SchematicExecutor::new(s.clone(), dispatcher.clone()))
        .collect(),
    );
    let signature = state.providers.get(SELF_NAMESPACE).unwrap().clone();
    let this = Arc::new(Self {
      signature,
      schematics,
      self_provider: Mutex::new(None),
      providers,
    });
    let mut lock = this.self_provider.lock();
    lock.replace(this.clone());
    drop(lock);
    this
  }
}

impl Provider for SchematicProvider {
  fn handle(&self, invocation: Invocation, _config: Option<Value>) -> BoxFuture<Result<TransportStream, BoxError>> {
    trace!(target = ?invocation.target, namespace = SELF_NAMESPACE);

    let operation = invocation.target.name().to_owned();
    let fut = self
      .schematics
      .iter()
      .find(|s| s.name() == operation)
      .map(|s| {
        let lock = self.self_provider.lock();
        let self_provider = lock.clone().unwrap();
        s.invoke(invocation, self.providers.clone(), self_provider)
      })
      .ok_or_else(|| Error::SchematicNotFound(operation.clone()));

    Box::pin(async move {
      let span = trace_span!("self", name = operation.as_str());
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
