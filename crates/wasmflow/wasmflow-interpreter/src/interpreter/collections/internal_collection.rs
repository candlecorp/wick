use futures::future::BoxFuture;
use serde_json::Value;
use wasmflow_schematic_graph::{SCHEMATIC_INPUT, SCHEMATIC_OUTPUT};
use wasmflow_sdk::v1::transport::TransportStream;
use wasmflow_sdk::v1::types::{CollectionFeatures, CollectionSignature, ComponentSignature, TypeSignature};
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
    let signature = CollectionSignature::new(NS_INTERNAL)
      .format(1)
      .version("0.0.0")
      .features(CollectionFeatures::v0(false, false))
      .add_component(
        ComponentSignature::new(INTERNAL_ID_INHERENT)
          .add_input("seed", TypeSignature::U64)
          .add_input("timestamp", TypeSignature::U64)
          .add_output("seed", TypeSignature::U64)
          .add_output("timestamp", TypeSignature::U64),
      );

    Self { signature }
  }
}

impl Collection for InternalCollection {
  fn handle(&self, invocation: Invocation, data: Option<Value>) -> BoxFuture<Result<TransportStream, BoxError>> {
    trace!(target = %invocation.target, id=%invocation.id,namespace = NS_INTERNAL);
    let op = invocation.target.name().to_owned();

    let is_oneshot = op == SCHEMATIC_INPUT || op == INTERNAL_ID_INHERENT;
    let task = async move {
      if op == SCHEMATIC_OUTPUT {
        panic!("Output component should not be executed");
      } else if is_oneshot {
        oneshot::OneShotComponent::default()
          .handle(invocation.payload, data)
          .await
      } else {
        panic!("Internal component {} not handled.", op);
      }
    };
    Box::pin(task)
  }

  fn list(&self) -> &CollectionSignature {
    &self.signature
  }
}
