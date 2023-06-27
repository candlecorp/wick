use flow_component::{Component, ComponentError, Context, Operation, RenderConfiguration, RuntimeCallback};
use wick_interface_types::{ComponentSignature, TypeDefinition};
use wick_packet::{Invocation, PacketStream, RuntimeConfig};

use crate::constants::*;
use crate::graph::types::Network;
use crate::interpreter::components::dyn_component_id;
use crate::{BoxFuture, HandlerMap};

mod merge;
mod pluck;
mod sender;
mod switch;

#[derive(Debug)]
pub(crate) struct CoreCollection {
  signature: ComponentSignature,
  pluck: pluck::Op,
  sender: sender::Op,
  merge: merge::Op,
  switch: switch::Op,
}

#[derive(Debug, thiserror::Error)]
pub struct OpInitError {
  error: ComponentError,
  kind: DynamicOperation,
}

impl OpInitError {
  fn new(error: ComponentError, kind: DynamicOperation) -> Self {
    Self { error, kind }
  }
}

impl std::fmt::Display for OpInitError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("dynamic ")?;
    self.kind.fmt(f)?;
    f.write_str(" component failed to initialize: ")?;
    self.error.fmt(f)
  }
}

#[derive(Debug, Clone, Copy)]
enum DynamicOperation {
  Merge,
  Switch,
}

impl std::fmt::Display for DynamicOperation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DynamicOperation::Merge => f.write_str("merge"),
      DynamicOperation::Switch => f.write_str("switch"),
    }
  }
}

impl CoreCollection {
  pub(crate) fn new(graph: &Network, handlers: &HandlerMap) -> Result<Self, OpInitError> {
    let mut this = Self {
      signature: ComponentSignature::new(NS_CORE).version("0.0.0"),
      pluck: pluck::Op::new(),
      sender: sender::Op::new(),
      merge: merge::Op::new(),
      switch: switch::Op::new(),
    };

    this.signature = this.signature.add_operation(this.pluck.get_signature(None).clone());
    this.signature = this.signature.add_operation(this.sender.get_signature(None).clone());

    // scour program for dynamic components
    for schematic in graph.schematics() {
      for operation in schematic.nodes() {
        // only handle core:: components
        if operation.cref().component_id() != NS_CORE {
          continue;
        }

        let op = match operation.cref().name() {
          merge::Op::ID => DynamicOperation::Merge,
          switch::Op::ID => DynamicOperation::Switch,
          _ => continue,
        };

        let config = operation
          .data()
          .config
          .clone()
          .render()
          .map_err(|e| OpInitError::new(ComponentError::new(e), op))?;

        let result = match op {
          DynamicOperation::Merge => match merge::Op::decode_config(config) {
            Ok(config) => {
              let id = dyn_component_id(merge::Op::ID, schematic.name(), operation.id());
              debug!(%id,"adding dynamic type signature for merge component");
              let (op_sig, output_sig) = merge::Op::gen_signature(id, config);

              this.signature.types.push(TypeDefinition::Struct(output_sig));
              this.signature.operations.push(op_sig);
              Ok(())
            }
            Err(e) => Err(OpInitError::new(e, DynamicOperation::Merge)),
          },
          DynamicOperation::Switch => match switch::Op::decode_config(config) {
            Ok(config) => {
              let op_sig = this.switch.gen_signature(schematic, graph, handlers, config);

              this.signature.operations.push(op_sig);
              Ok(())
            }
            Err(e) => Err(OpInitError::new(e, DynamicOperation::Switch)),
          },
        };
        if let Err(error) = result {
          error!(%error, "Failed to add dynamic signature");
          return Err(error);
        }
      }
    }

    trace!(?this.signature, "core signature");

    Ok(this)
  }
}

macro_rules! core_op {
  ($type:ty, $inv:expr, $name:expr, $callback:expr, $data:ident) => {{
    let config = <$type>::decode_config($data)?;
    let ctx = Context::new(config, &$inv.inherent, $callback);
    $name.handle($inv, ctx).await
  }};
}

impl Component for CoreCollection {
  fn handle(
    &self,
    invocation: Invocation,
    data: Option<RuntimeConfig>,
    callback: std::sync::Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    invocation.trace(|| trace!(target = %invocation.target, namespace = NS_CORE));

    let task = async move {
      match invocation.target.operation_id() {
        sender::Op::ID => core_op! {sender::Op, invocation, self.sender, callback, data},
        pluck::Op::ID => core_op! {pluck::Op, invocation, self.pluck, callback, data},
        merge::Op::ID => core_op! {merge::Op, invocation, self.merge, callback, data},
        switch::Op::ID => core_op! {switch::Op, invocation, self.switch, callback, data},
        _ => {
          panic!("Core operation {} not handled.", invocation.target.operation_id());
        }
      }
    };
    Box::pin(task)
  }

  fn signature(&self) -> &ComponentSignature {
    &self.signature
  }
}
