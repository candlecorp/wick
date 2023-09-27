use flow_component::{Component, ComponentError, LocalScope};
use flow_graph::{SCHEMATIC_INPUT, SCHEMATIC_OUTPUT};
use wick_interface_types::ComponentSignature;
use wick_packet::{Invocation, PacketStream, RuntimeConfig};

use crate::BoxFuture;

// pub(crate) mod oneshot;

#[derive(Debug)]
pub(crate) struct InternalComponent {
  signature: ComponentSignature,
}

impl Default for InternalComponent {
  fn default() -> Self {
    let signature = ComponentSignature::new_named(Self::ID).set_version("0.0.0");

    Self { signature }
  }
}

impl InternalComponent {
  pub(crate) const ID: &'static str = flow_graph::NS_SCHEMATIC;
}

impl Component for InternalComponent {
  fn handle(
    &self,
    invocation: Invocation,
    _config: Option<RuntimeConfig>,
    _callback: LocalScope,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    invocation.trace(|| debug!(target = %invocation.target(), id=%invocation.id(),namespace = Self::ID));
    let op = invocation.target().operation_id().to_owned();

    let is_oneshot = op == SCHEMATIC_INPUT;
    let task = async move {
      if op == SCHEMATIC_OUTPUT {
        panic!("Output component should not be executed");
      } else if is_oneshot {
        Ok(invocation.into_stream())
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
