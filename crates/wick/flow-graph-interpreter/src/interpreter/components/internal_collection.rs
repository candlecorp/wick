use flow_component::{Component, ComponentError, RuntimeCallback};
use flow_graph::{SCHEMATIC_INPUT, SCHEMATIC_OUTPUT};
use wick_interface_types::{ComponentSignature, OperationSignature, TypeSignature};
use wick_packet::{Invocation, PacketStream};

use crate::constants::*;
use crate::BoxFuture;

// pub(crate) mod oneshot;

#[derive(Debug)]
pub(crate) struct InternalCollection {
  signature: ComponentSignature,
}

impl Default for InternalCollection {
  fn default() -> Self {
    let signature = ComponentSignature::new(NS_INTERNAL).version("0.0.0").add_operation(
      OperationSignature::new(INTERNAL_ID_INHERENT)
        .add_input("seed", TypeSignature::U64)
        .add_input("timestamp", TypeSignature::U64)
        .add_output("seed", TypeSignature::U64)
        .add_output("timestamp", TypeSignature::U64),
    );

    Self { signature }
  }
}

impl Component for InternalCollection {
  fn handle(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    _config: Option<wick_packet::OperationConfig>,
    _callback: std::sync::Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    invocation.trace(|| trace!(target = %invocation.target, id=%invocation.id,namespace = NS_INTERNAL));
    let op = invocation.target.operation_id().to_owned();

    let is_oneshot = op == SCHEMATIC_INPUT || op == INTERNAL_ID_INHERENT;
    let task = async move {
      if op == SCHEMATIC_OUTPUT {
        panic!("Output component should not be executed");
      } else if is_oneshot {
        Ok(stream)
      } else {
        panic!("Internal component {} not handled.", op);
      }
    };
    Box::pin(task)
  }

  fn list(&self) -> &ComponentSignature {
    &self.signature
  }
}
