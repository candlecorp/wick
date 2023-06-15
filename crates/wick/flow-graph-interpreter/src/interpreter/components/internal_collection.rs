use flow_component::{Component, ComponentError, RuntimeCallback};
use flow_graph::{SCHEMATIC_INPUT, SCHEMATIC_OUTPUT};
use wick_interface_types::ComponentSignature;
use wick_packet::{GenericConfig, Invocation, PacketStream};

use crate::constants::*;
use crate::BoxFuture;

// pub(crate) mod oneshot;

#[derive(Debug)]
pub(crate) struct InternalCollection {
  signature: ComponentSignature,
}

impl Default for InternalCollection {
  fn default() -> Self {
    let signature = ComponentSignature::new(NS_INTERNAL).version("0.0.0");

    Self { signature }
  }
}

impl Component for InternalCollection {
  fn handle(
    &self,
    invocation: Invocation,
    _config: Option<GenericConfig>,
    _callback: std::sync::Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    invocation.trace(|| trace!(target = %invocation.target, id=%invocation.id,namespace = NS_INTERNAL));
    let op = invocation.target.operation_id().to_owned();

    let is_oneshot = op == SCHEMATIC_INPUT;
    let task = async move {
      if op == SCHEMATIC_OUTPUT {
        panic!("Output component should not be executed");
      } else if is_oneshot {
        Ok(invocation.packets)
      } else {
        panic!("Internal component {} not handled.", op);
      }
    };
    Box::pin(task)
  }

  fn signature(&self) -> &ComponentSignature {
    &self.signature
  }
}
