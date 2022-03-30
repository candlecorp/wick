use futures::future::BoxFuture;
use serde_json::Value;
use vino_interpreter::{BoxError, Component, OneShotComponent, Provider};
use vino_rpc::ComponentSignature;
use vino_transport::{Invocation, TransportStream};
use vino_types::{MapWrapper, ProviderSignature};

pub(crate) static RUNTIME_NAMESPACE: &str = "runtime_core";

#[derive(Debug)]
pub(crate) struct RuntimeCoreProvider {
  #[allow(unused)]
  signature: ProviderSignature,
}

impl RuntimeCoreProvider {
  pub(crate) fn new() -> Self {
    let mut signature = ProviderSignature::new(RUNTIME_NAMESPACE);
    let mut component_sig = ComponentSignature::new("inherent");
    component_sig.inputs.insert("seed", vino_types::TypeSignature::U64);
    component_sig.inputs.insert("timestamp", vino_types::TypeSignature::U64);
    signature.components.insert("inherent", component_sig);
    Self { signature }
  }
}

impl Provider for RuntimeCoreProvider {
  fn handle(&self, invocation: Invocation, config: Option<Value>) -> BoxFuture<Result<TransportStream, BoxError>> {
    let operation = invocation.target.name().to_owned();
    trace!(target=?invocation.target, namespace = RUNTIME_NAMESPACE);

    let is_passthrough = operation == "inherent";
    let task = async move {
      let result = if is_passthrough {
        OneShotComponent::default().handle(invocation.payload, config).await
      } else {
        panic!("Internal component {} not handled.", operation);
      };
      result
    };
    Box::pin(task)
  }

  fn list(&self) -> &ProviderSignature {
    &self.signature
  }
}
