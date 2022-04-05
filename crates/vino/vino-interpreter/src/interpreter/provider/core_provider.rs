use futures::future::BoxFuture;
use serde_json::Value;
use vino_transport::{Invocation, TransportStream};
use vino_types::ProviderSignature;

use crate::constants::*;
use crate::{BoxError, Component, Provider};

mod sender;

#[derive(Debug)]
pub(crate) struct CoreProvider {
  signature: ProviderSignature,
}

impl Default for CoreProvider {
  fn default() -> Self {
    Self {
      signature: serde_json::from_value(serde_json::json!({
        "name":NS_CORE,
        "components" : {
          CORE_ID_SENDER:{
            "name": CORE_ID_SENDER,
            "inputs": {},
            "outputs": {
              "output": {"type":"raw"}
            }
          }
        }
      }))
      .unwrap(),
    }
  }
}

impl Provider for CoreProvider {
  fn handle(&self, invocation: Invocation, data: Option<Value>) -> BoxFuture<Result<TransportStream, BoxError>> {
    trace!(target = %invocation.target, id=%invocation.id, namespace = NS_CORE);

    let task = async move {
      let result = if invocation.target.name() == CORE_ID_SENDER {
        sender::SenderComponent::default()
          .handle(invocation.payload, data)
          .await
      } else {
        panic!("Core component {} not handled.", invocation.target.name());
      };
      result
    };
    Box::pin(task)
  }

  fn list(&self) -> &ProviderSignature {
    &self.signature
  }
}
