use flow_component::{Component, ComponentError, Context, Operation, RuntimeCallback};
use serde_json::Value;
use wick_interface_types::{ComponentSignature, TypeDefinition};
use wick_packet::{Invocation, PacketStream};

use crate::constants::*;
use crate::graph::types::Network;
use crate::interpreter::components::dyn_component_id;
use crate::BoxFuture;

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

impl CoreCollection {
  pub(crate) fn new(graph: &Network) -> Self {
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
      warn!("schematic: {}", schematic.name());
      for operation in schematic.nodes() {
        warn!("operation: {}", operation.name);
        // only handle core:: components
        if operation.cref().component_id() != NS_CORE {
          continue;
        }

        match operation.cref().name() {
          merge::Op::ID => {
            let config = match merge::Op::decode_config(operation.data().clone()) {
              Ok(c) => c,
              Err(e) => {
                error!("Configuration for dynamic merge component invalid: {}", e);
                panic!()
              }
            };
            let id = dyn_component_id(merge::Op::ID, schematic.name(), operation.id());
            debug!(%id,"adding dynamic type signature for merge component");
            let (op_sig, output_sig) = merge::Op::gen_signature(id, config);

            this.signature.types.push(TypeDefinition::Struct(output_sig));
            this.signature.operations.push(op_sig);
          }
          switch::Op::ID => {
            let config = match switch::Op::decode_config(operation.data().clone()) {
              Ok(c) => c,
              Err(e) => {
                error!("Configuration for dynamic switch component invalid: {}", e);
                panic!();
              }
            };
            let op_sig = this.switch.gen_signature(graph, config);

            this.signature.operations.push(op_sig);
          }
          _ => {}
        }
      }
    }

    trace!(?this.signature, "core signature");

    this
  }
}

macro_rules! core_op {
  ($type:ty, $name:expr, $stream:ident, $callback:expr, $data:ident) => {{
    let config = <$type>::decode_config($data)?;
    $name.handle($stream, Context::new(config, $callback)).await
  }};
}

impl Component for CoreCollection {
  fn handle(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    data: Option<Value>,
    callback: std::sync::Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    trace!(target = %invocation.target, namespace = NS_CORE);
    debug!("data: {:?}", data);
    let task = async move {
      match invocation.target.operation_id() {
        sender::Op::ID => core_op! {sender::Op, self.sender, stream, callback, data},
        pluck::Op::ID => core_op! {pluck::Op, self.pluck, stream, callback, data},
        merge::Op::ID => core_op! {merge::Op, self.merge, stream, callback, data},
        switch::Op::ID => core_op! {switch::Op, self.switch, stream, callback, data},

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
