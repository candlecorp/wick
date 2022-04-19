use futures::future::BoxFuture;
use serde_json::Value;
use vino_transport::{Invocation, TransportStream};
use vino_types::{ComponentSignature, MapWrapper, ProviderSignature};

use crate::constants::*;
use crate::graph::types::Network;
use crate::interpreter::provider::dyn_component_id;
use crate::{BoxError, Component, Provider};

mod merge;
mod sender;

#[derive(Debug)]
pub(crate) struct CoreProvider {
  signature: ProviderSignature,
}

impl CoreProvider {
  pub(crate) fn new(graph: &Network) -> Self {
    let mut signature: ProviderSignature = serde_json::from_value(serde_json::json!({
      "name":NS_CORE,
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
          let output_type = vino_types::TypeMap::new();
          let mut output_signature = vino_types::StructSignature::new(&id, output_type);
          for (name, type_sig) in component_signature.inputs.inner() {
            output_signature.fields.insert(name, type_sig.clone());
          }
          signature.types.insert(&id, output_signature);

          component_signature
            .outputs
            .insert("output", vino_types::TypeSignature::Ref { reference: id.clone() });
          debug!(%id,"adding dynamic component");
          signature.components.insert(id, component_signature);
        }
      }
    }

    trace!(?signature, "core signature");

    Self { signature }
  }
}

impl Provider for CoreProvider {
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

  fn list(&self) -> &ProviderSignature {
    &self.signature
  }
}
