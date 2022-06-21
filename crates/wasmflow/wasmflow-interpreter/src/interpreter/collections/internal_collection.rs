use futures::future::BoxFuture;
use serde_json::Value;
use wasmflow_schematic_graph::{SCHEMATIC_INPUT, SCHEMATIC_OUTPUT};
use wasmflow_sdk::v1::transport::TransportStream;
use wasmflow_sdk::v1::types::CollectionSignature;
use wasmflow_sdk::v1::Invocation;

use crate::constants::*;
use crate::{BoxError, Collection, Component};

pub(crate) mod oneshot;

#[derive(Debug)]
pub(crate) struct InternalCollection {
  signature: CollectionSignature,
}

impl Default for InternalCollection {
  fn default() -> Self {
    Self {
      signature: serde_json::from_value(serde_json::json!({
        "name": NS_INTERNAL,
        "format": 1,
        "version": "0.0.0",
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

impl Collection for InternalCollection {
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

  fn list(&self) -> &CollectionSignature {
    &self.signature
  }
}
