use futures::future::BoxFuture;
use serde_json::Value;
use wasmflow_sdk::v1::transport::TransportStream;
use wasmflow_sdk::v1::types::{
  CollectionSignature,
  ComponentSignature,
  FieldMap,
  StructSignature,
  TypeDefinition,
  TypeSignature,
};
use wasmflow_sdk::v1::Invocation;

use crate::constants::*;
use crate::graph::types::Network;
use crate::interpreter::collections::dyn_component_id;
use crate::{BoxError, Collection, Component};

mod merge;
mod sender;

#[derive(Debug)]
pub(crate) struct CoreCollection {
  signature: CollectionSignature,
}

impl CoreCollection {
  pub(crate) fn new(graph: &Network) -> Self {
    let mut signature: CollectionSignature = serde_json::from_value(serde_json::json!({
      "name":NS_CORE,
      "format": 1u8,
      "version": "0.0.0",
      "components" : {
        CORE_ID_SENDER:{
          "name": CORE_ID_SENDER,
          "inputs": {},
          "outputs": {
            "output": {"type":"value"}
          }
        }
      }
    }))
    .unwrap();

    for schematic in graph.schematics() {
      for component in schematic.components() {
        // only handle core:: components
        if component.cref().namespace() != NS_CORE {
          continue;
        }
        // set up dynamic merge components
        if component.cref().name() == CORE_ID_MERGE {
          assert!(
            component.has_data(),
            "Dynamic merge component ({}, instance {}) must be configured with its expected inputs.",
            CORE_ID_MERGE,
            component.id()
          );

          let result = serde_json::from_value::<ComponentSignature>(component.data().clone().unwrap());
          if let Err(e) = result {
            panic!("Configuration for dynamic merge component invalid: {}", e);
          }
          let id = dyn_component_id(CORE_ID_MERGE, schematic.name(), component.id());
          let mut component_signature = result.unwrap();
          let output_type = FieldMap::new();
          let mut output_signature = StructSignature::new(&id, output_type);
          for (name, type_sig) in component_signature.inputs.inner() {
            output_signature.fields.insert(name, type_sig.clone());
          }
          signature.types.insert(&id, TypeDefinition::Struct(output_signature));

          component_signature
            .outputs
            .insert("output", TypeSignature::Ref { reference: id.clone() });
          debug!(%id,"adding dynamic component");
          signature.components.insert(id, component_signature);
        }
      }
    }

    trace!(?signature, "core signature");

    Self { signature }
  }
}

impl Collection for CoreCollection {
  fn handle(&self, invocation: Invocation, data: Option<Value>) -> BoxFuture<Result<TransportStream, BoxError>> {
    trace!(target = %invocation.target, id=%invocation.id, namespace = NS_CORE);

    let task = async move {
      match invocation.target.name() {
        CORE_ID_SENDER => {
          sender::SenderComponent::default()
            .handle(invocation.payload, data)
            .await
        }
        CORE_ID_MERGE => merge::MergeComponent::default().handle(invocation.payload, data).await,
        _ => {
          panic!("Core component {} not handled.", invocation.target.name());
        }
      }
    };
    Box::pin(task)
  }

  fn list(&self) -> &CollectionSignature {
    &self.signature
  }
}
