use flow_component::{Component, ComponentError, Context, Operation, RuntimeCallback};
use serde_json::Value;
use wick_interface_types::{ComponentSignature, TypeDefinition};
use wick_packet::{Invocation, PacketStream, StreamMap};

use crate::constants::*;
use crate::graph::types::Network;
use crate::interpreter::components::dyn_component_id;
use crate::BoxFuture;

mod merge;
mod pluck;
mod sender;

#[derive(Debug)]
pub(crate) struct CoreCollection {
  signature: ComponentSignature,
  pluck: pluck::Op,
  sender: sender::Op,
  merge: merge::Op,
}

impl CoreCollection {
  pub(crate) fn new(graph: &Network) -> Self {
    let mut this = Self {
      signature: ComponentSignature::new(NS_CORE).version("0.0.0"),
      pluck: pluck::Op::new(),
      sender: sender::Op::new(),
      merge: merge::Op::new(),
    };
    this.signature = this.signature.add_operation(this.pluck.signature(None).clone());
    this.signature = this.signature.add_operation(this.sender.signature(None).clone());

    // scour program for dynamic components
    for schematic in graph.schematics() {
      for operation in schematic.nodes() {
        trace!("operation: {:?}", operation.cref());
        // only handle core:: components
        if operation.cref().component_id() != NS_CORE {
          continue;
        }
        // set up dynamic merge components
        if operation.cref().name() == merge::Op::ID {
          assert!(
            operation.has_data(),
            "Dynamic merge component ({}, instance {}) must be configured with its expected inputs.",
            merge::Op::ID,
            operation.id()
          );
          let config = match merge::Op::decode_config(operation.data().clone()) {
            Ok(c) => c,
            Err(e) => panic!("Configuration for dynamic merge component invalid: {}", e),
          };
          let id = dyn_component_id(merge::Op::ID, schematic.name(), operation.id());
          debug!(%id,"adding dynamic component");
          let (op_sig, output_sig) = merge::Op::gen_signature(id, config);

          this.signature.types.push(TypeDefinition::Struct(output_sig));

          this.signature.operations.push(op_sig);
        }
      }
    }

    trace!(?this.signature, "core signature");

    this
  }
}

macro_rules! core_op {
  ($type:ty, $name:expr, $stream:ident, $data:ident) => {{
    let config = <$type>::decode_config($data)?;
    $name
      .handle(
        StreamMap::from_stream($stream, $name.input_names(&config)),
        Context::new(config),
      )
      .await
  }};
}

impl Component for CoreCollection {
  fn handle(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    data: Option<Value>,
    _callback: std::sync::Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    trace!(target = %invocation.target, namespace = NS_CORE);
    debug!("data: {:?}", data);
    let task = async move {
      match invocation.target.operation_id() {
        sender::Op::ID => core_op! {sender::Op, self.sender, stream, data},
        pluck::Op::ID => core_op! {pluck::Op, self.pluck, stream, data},
        merge::Op::ID => core_op! {merge::Op, self.merge, stream, data},

        // TODO re-evaluate merge component
        // CORE_ID_MERGE => merge::MergeComponent::default().handle(invocation.payload, data).await,
        _ => {
          panic!("Core operation {} not handled.", invocation.target.operation_id());
        }
      }
    };
    Box::pin(task)
  }

  fn list(&self) -> &ComponentSignature {
    &self.signature
  }
}
