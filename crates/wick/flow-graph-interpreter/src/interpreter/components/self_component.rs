use std::sync::Arc;

use flow_component::{Component, ComponentError, RuntimeCallback};
use wick_interface_types::ComponentSignature;
use wick_packet::{Invocation, PacketStream, RuntimeConfig};

use crate::interpreter::channel::InterpreterDispatchChannel;
use crate::interpreter::executor::SchematicExecutor;
use crate::interpreter::program::ProgramState;
use crate::{BoxFuture, HandlerMap};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub(crate) enum Error {
  #[error("Operation '{0}' not found on this component")]
  SchematicNotFound(String),
}

#[derive(Debug)]
pub(crate) struct InnerSelf {
  signature: ComponentSignature,
  schematics: Arc<Vec<SchematicExecutor>>,
  components: Arc<HandlerMap>,
}

impl InnerSelf {
  pub(crate) fn new(
    components: Arc<HandlerMap>,
    state: &ProgramState,
    dispatcher: &InterpreterDispatchChannel,
    root_config: Option<&RuntimeConfig>,
  ) -> Self {
    let schematics: Arc<Vec<SchematicExecutor>> = Arc::new(
      state
        .network
        .schematics()
        .iter()
        .map(|s| SchematicExecutor::new(s.clone(), dispatcher.clone(), root_config.cloned()))
        .collect(),
    );
    let signature = state.components.get(SelfComponent::ID).unwrap().clone();
    Self {
      signature,
      schematics,
      components,
    }
  }
}

#[derive(Debug, Clone)]
pub(crate) struct SelfComponent {
  inner: Arc<InnerSelf>,
}

impl SelfComponent {
  pub(crate) const ID: &str = "self";

  pub(crate) fn new(
    components: Arc<HandlerMap>,
    state: &ProgramState,
    dispatcher: &InterpreterDispatchChannel,
    root_config: Option<&RuntimeConfig>,
  ) -> Self {
    let inner_self = InnerSelf::new(components, state, dispatcher, root_config);
    Self {
      inner: Arc::new(inner_self),
    }
  }
}

impl Component for SelfComponent {
  fn handle(
    &self,
    invocation: Invocation,
    config: Option<RuntimeConfig>,
    callback: Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    invocation.trace(|| debug!(target = %invocation.target, namespace = Self::ID));

    let operation = invocation.target.operation_id().to_owned();
    let fut = self
      .inner
      .schematics
      .iter()
      .find(|s| s.name() == operation)
      .map(|s| {
        s.invoke(
          invocation,
          self.inner.components.clone(),
          self.clone(),
          config,
          callback,
        )
      })
      .ok_or_else(|| Error::SchematicNotFound(operation.clone()));

    Box::pin(async move {
      match fut {
        Ok(fut) => fut.await.map_err(ComponentError::new),
        Err(e) => Err(ComponentError::new(e)),
      }
    })
  }

  fn signature(&self) -> &ComponentSignature {
    &self.inner.signature
  }
}
