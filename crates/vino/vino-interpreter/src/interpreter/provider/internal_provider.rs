use futures::future::BoxFuture;
use serde_json::Value;
use vino_schematic_graph::{SCHEMATIC_INPUT, SCHEMATIC_OUTPUT};
use vino_transport::{Invocation, TransportStream};
use vino_types::ProviderSignature;

use crate::constants::*;
use crate::{BoxError, Component, Provider};

pub(crate) mod oneshot;

#[derive(Debug)]
pub(crate) struct InternalProvider {
  signature: ProviderSignature,
}

impl Default for InternalProvider {
  fn default() -> Self {
    Self {
      signature: serde_json::from_value(serde_json::json!({
        "name": NS_INTERNAL,
        "components": {
          INTERNAL_ID_INHERENT : {
            "name":INTERNAL_ID_INHERENT,
            "inputs": {
              "seed": {
                "type":"u64",
              },
              "timestamp": {
                "type":"u64",
              },
            },
            "outputs": {
              "seed": {
                "type":"u64",
              },
              "timestamp": {
                "type":"u64",
              },
            }

          }
        }
      }))
      .unwrap(),
    }
  }
}

impl Provider for InternalProvider {
  fn handle(&self, invocation: Invocation, data: Option<Value>) -> BoxFuture<Result<TransportStream, BoxError>> {
    trace!(target = %invocation.target, id=%invocation.id,namespace = NS_INTERNAL);
    let op = invocation.target.name().to_owned();

    let is_oneshot = op == SCHEMATIC_INPUT || op == INTERNAL_ID_INHERENT;
    let task = async move {
      let result = if op == SCHEMATIC_OUTPUT {
        panic!("Output component should not be executed");
      } else if is_oneshot {
        oneshot::OneShotComponent::default()
          .handle(invocation.payload, data)
          .await
      } else {
        panic!("Internal component {} not handled.", op);
      };
      result
    };
    Box::pin(task)
  }

  fn list(&self) -> &ProviderSignature {
    &self.signature
  }
}
