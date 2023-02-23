use futures::future::BoxFuture;
use serde_json::Value;
use wasmflow_interface::{
  CollectionFeatures,
  CollectionSignature,
  FieldMap,
  OperationSignature,
  StructSignature,
  TypeDefinition,
  TypeSignature,
};
use wasmflow_packet_stream::{Invocation, PacketStream, StreamMap};

use crate::constants::*;
use crate::graph::types::Network;
use crate::interpreter::collections::dyn_component_id;
use crate::{BoxError, Collection, Operation};

// mod merge;
mod sender;

#[derive(Debug)]
pub(crate) struct CoreCollection {
  signature: CollectionSignature,
}

impl CoreCollection {
  pub(crate) fn new(graph: &Network) -> Self {
    let mut signature = CollectionSignature::new(NS_CORE)
      .format(1)
      .version("0.0.0")
      .features(CollectionFeatures::v0(false, false))
      .add_component(OperationSignature::new(CORE_ID_SENDER).add_output("output", TypeSignature::Value));

    for schematic in graph.schematics() {
      for component in schematic.nodes() {
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

          let result = serde_json::from_value::<OperationSignature>(component.data().clone().unwrap());
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
          signature.operations.insert(id, component_signature);
        }
      }
    }

    trace!(?signature, "core signature");

    Self { signature }
  }
}

impl Collection for CoreCollection {
  fn handle(
    &self,
    invocation: Invocation,
    _stream: PacketStream,
    data: Option<Value>,
  ) -> BoxFuture<Result<PacketStream, BoxError>> {
    trace!(target = %invocation.target, namespace = NS_CORE);

    let task = async move {
      match invocation.target.name() {
        CORE_ID_SENDER => {
          let map = StreamMap::default();
          sender::SenderOperation::default().handle(map, data).await
        }
        // TODO re-evaluate merge component
        // CORE_ID_MERGE => merge::MergeComponent::default().handle(invocation.payload, data).await,
        _ => {
          panic!("Core operation {} not handled.", invocation.target.name());
        }
      }
    };
    Box::pin(task)
  }

  fn list(&self) -> &CollectionSignature {
    &self.signature
  }
}
