use std::sync::Arc;

use flow_component::{Component, ComponentError, RuntimeCallback};
use parking_lot::Mutex;
use tracing_futures::Instrument;
use wick_interface_types::ComponentSignature;
use wick_packet::{Invocation, PacketStream, RuntimeConfig};

use crate::constants::*;
use crate::graph::OperationConfig;
use crate::interpreter::channel::InterpreterDispatchChannel;
use crate::interpreter::executor::SchematicExecutor;
use crate::interpreter::program::ProgramState;
use crate::{BoxFuture, HandlerMap};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub(crate) enum Error {
  #[error("Schematic {0} not found on this network")]
  SchematicNotFound(String),
}

#[derive(Debug)]
pub(crate) struct SchematicComponent {
  signature: ComponentSignature,
  schematics: Arc<Vec<SchematicExecutor>>,
  components: Arc<HandlerMap>,
  self_collection: Mutex<Option<Arc<Self>>>,
  config: Option<RuntimeConfig>,
}

impl SchematicComponent {
  pub(crate) fn new(
    components: Arc<HandlerMap>,
    state: &ProgramState,
    config: Option<RuntimeConfig>,
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
    let signature = state.components.get(NS_SELF).unwrap().clone();
    let collection = Arc::new(Self {
      signature,
      schematics,
      self_collection: Mutex::new(None),
      components,
      config,
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

impl Component for SchematicComponent {
  fn handle(
    &self,
    invocation: Invocation,
    config: Option<RuntimeConfig>,
    callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    invocation.trace(|| trace!(target = %invocation.target, namespace = NS_SELF));

    let mut op_config = OperationConfig::new_value(config);
    op_config.set_root(self.config.clone());

    let operation = invocation.target.operation_id().to_owned();
    let fut = self
      .schematics
      .iter()
      .find(|s| s.name() == operation)
      .map(|s| {
        s.invoke(
          invocation,
          self.components.clone(),
          self.clone_self_collection(),
          op_config,
          callback,
        )
      })
      .ok_or_else(|| Error::SchematicNotFound(operation.clone()));

    Box::pin(async move {
      let span = trace_span!("ns_self", name = %operation);
      match fut {
        Ok(fut) => fut.instrument(span).await.map_err(ComponentError::new),
        Err(e) => Err(ComponentError::new(e)),
      }
    })
  }

  fn signature(&self) -> &ComponentSignature {
    &self.signature
  }
}
