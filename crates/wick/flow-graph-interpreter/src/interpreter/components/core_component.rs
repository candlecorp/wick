use flow_component::{Component, ComponentError, Context, Operation, RenderConfiguration, RuntimeCallback};
use wick_interface_types::{ComponentSignature, TypeDefinition};
use wick_packet::{InherentData, Invocation, PacketStream, RuntimeConfig};

use crate::graph::types::Network;
use crate::interpreter::components::dyn_component_id;
use crate::{BoxFuture, HandlerMap};

mod collect;
mod merge;
mod pluck;
mod sender;
mod switch;

pub(crate) static DYNAMIC_OPERATIONS: &[&str] = &[collect::Op::ID, merge::Op::ID, switch::Op::ID];

#[derive(Debug)]
pub(crate) struct CoreComponent {
  signature: ComponentSignature,
  pluck: pluck::Op,
  sender: sender::Op,
  merge: merge::Op,
  switch: switch::Op,
  collect: collect::Op,
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
  Collect,
}

impl std::fmt::Display for DynamicOperation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DynamicOperation::Merge => f.write_str("merge"),
      DynamicOperation::Switch => f.write_str("switch"),
      DynamicOperation::Collect => f.write_str("collect"),
    }
  }
}

impl CoreComponent {
  pub(crate) const ID: &str = "core";

  pub(crate) fn new(graph: &Network, handlers: &HandlerMap) -> Result<Self, OpInitError> {
    let mut this = Self {
      signature: ComponentSignature::new(Self::ID).version("0.0.0"),
      pluck: pluck::Op::new(),
      sender: sender::Op::new(),
      merge: merge::Op::new(),
      switch: switch::Op::new(),
      collect: collect::Op::new(),
    };

    this.signature = this.signature.add_operation(this.pluck.get_signature(None).clone());
    this.signature = this.signature.add_operation(this.sender.get_signature(None).clone());

    // scour program for dynamic components
    for schematic in graph.schematics() {
      for operation in schematic.nodes() {
        // only handle core:: components
        if operation.cref().component_id() != Self::ID {
          continue;
        }

        let op = match operation.cref().name() {
          merge::Op::ID => DynamicOperation::Merge,
          switch::Op::ID => DynamicOperation::Switch,
          collect::Op::ID => DynamicOperation::Collect,
          _ => continue,
        };

        let config = operation
          .data()
          .config
          .clone()
          .render(&InherentData::unsafe_default()) // this is a first pass render to extract details so using unsafe_default should be OK.
          .map_err(|e| OpInitError::new(ComponentError::new(e), op))?;

        let result = match op {
          DynamicOperation::Collect => match collect::Op::decode_config(config) {
            Ok(config) => {
              let id = dyn_component_id(collect::Op::ID, schematic.name(), operation.id());
              debug!(%id,%op,"adding type signature for dynamic component");
              let op_sig = collect::Op::gen_signature(&id, config);

              this.signature.operations.push(op_sig);
              Ok(())
            }
            Err(e) => Err(OpInitError::new(e, op)),
          },
          DynamicOperation::Merge => match merge::Op::decode_config(config) {
            Ok(config) => {
              let id = dyn_component_id(merge::Op::ID, schematic.name(), operation.id());
              debug!(%id,%op,"adding type signature for dynamic component");
              let (op_sig, output_sig) = merge::Op::gen_signature(id, config);

              this.signature.types.push(TypeDefinition::Struct(output_sig));
              this.signature.operations.push(op_sig);
              Ok(())
            }
            Err(e) => Err(OpInitError::new(e, op)),
          },
          DynamicOperation::Switch => match switch::Op::decode_config(config) {
            Ok(config) => {
              let id = dyn_component_id(switch::Op::ID, schematic.name(), operation.id());
              debug!(%id,%op,"adding type signature for dynamic component");
              let op_sig = this.switch.gen_signature(id, schematic, graph, handlers, config);

              this.signature.operations.push(op_sig);
              Ok(())
            }
            Err(e) => Err(OpInitError::new(e, op)),
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

impl Component for CoreComponent {
  fn handle(
    &self,
    invocation: Invocation,
    data: Option<RuntimeConfig>,
    callback: std::sync::Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    invocation.trace(|| trace!(target = %invocation.target, namespace = Self::ID));

    let task = async move {
      match invocation.target.operation_id() {
        sender::Op::ID => core_op! {sender::Op, invocation, self.sender, callback, data},
        pluck::Op::ID => core_op! {pluck::Op, invocation, self.pluck, callback, data},
        merge::Op::ID => core_op! {merge::Op, invocation, self.merge, callback, data},
        switch::Op::ID => core_op! {switch::Op, invocation, self.switch, callback, data},
        collect::Op::ID => core_op! {collect::Op, invocation, self.collect, callback, data},
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
