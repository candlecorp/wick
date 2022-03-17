use futures::future::BoxFuture;
use serde_json::Value;
use vino_transport::{Invocation, TransportStream};
use vino_types::ProviderSignature;

use crate::{BoxError, Component, Provider};

pub(crate) const CORE_PROVIDER_NS: &str = "core";
pub(crate) const SENDER_ID: &str = "sender";

mod sender;

#[derive(Debug)]
pub(crate) struct CoreProvider {
  signature: ProviderSignature,
}

impl Default for CoreProvider {
  fn default() -> Self {
    Self {
      signature: serde_json::from_value(serde_json::json!({
        "name":"core" ,
        "components" : {
          "sender":{
            "name": "sender",
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
    trace!(target = ?invocation.target, namespace = CORE_PROVIDER_NS);

    let task = async move {
      let result = if invocation.target.name() == SENDER_ID {
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
