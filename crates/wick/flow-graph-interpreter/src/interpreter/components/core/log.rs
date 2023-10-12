use flow_component::{ComponentError, Context, Operation, RenderConfiguration};
use serde_json::Value;
use tokio_stream::StreamExt;
use wick_interface_types::{operation, OperationSignature};
use wick_packet::{Invocation, PacketExt, PacketStream, RuntimeConfig};

use crate::BoxFuture;
#[derive()]
pub(crate) struct Op {
  signature: OperationSignature,
}

impl std::fmt::Debug for Op {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct(Op::ID).field("signature", &self.signature).finish()
  }
}

impl Op {
  pub(crate) fn new() -> Self {
    Self {
      signature: operation!(Op::ID=> {inputs:{"input"=>"object"}, outputs: {"output"=>"object"},}),
    }
  }
}

impl crate::graph::NodeDecorator for Op {
  fn decorate(node: &mut crate::graph::types::Node) -> Result<(), String> {
    node.add_input("input");
    node.add_output("output");
    Ok(())
  }
}

impl Operation for Op {
  const ID: &'static str = "log";
  type Config = ();

  fn handle(
    &self,
    invocation: Invocation,
    _context: Context<Self::Config>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let stream = invocation.into_stream();

    let mapped = stream.map(|mut next| {
      match &mut next {
        Ok(packet) => {
          if packet.has_data() {
            match packet.decode_value() {
              Ok(v) => match v {
                Value::String(v) => println!("{}", v),
                _ => println!("{}", v),
              },
              Err(e) => println!("Error decoding packet to log: {}", e),
            }
          }
          packet.set_port("output");
        }
        Err(e) => println!("Error: {}", e),
      };
      next
    });

    Box::pin(async move { Ok(PacketStream::new(mapped)) })
  }

  fn get_signature(&self, _config: Option<&Self::Config>) -> &OperationSignature {
    &self.signature
  }

  fn input_names(&self, _config: &Self::Config) -> Vec<String> {
    self.signature.inputs.iter().map(|n| n.name.clone()).collect()
  }
}

impl RenderConfiguration for Op {
  type Config = ();
  type ConfigSource = RuntimeConfig;

  fn decode_config(_data: Option<Self::ConfigSource>) -> Result<Self::Config, ComponentError> {
    Ok(())
  }
}
