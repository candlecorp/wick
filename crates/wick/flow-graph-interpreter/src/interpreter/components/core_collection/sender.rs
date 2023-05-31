use flow_component::{ComponentError, Context, Operation};
use serde_json::Value;
use wick_interface_types::{operation, OperationSignature};
use wick_packet::{packet_stream, Invocation, PacketStream};

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
      signature: operation!(Op::ID=> {inputs:{}, outputs: {"output"=>"object"},}),
    }
  }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub(crate) struct SenderData {
  output: Value,
}

impl Operation for Op {
  const ID: &'static str = "sender";
  type Config = SenderData;
  fn handle(
    &self,
    _invocation: Invocation,
    context: Context<Self::Config>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    let config = context.config;
    let task = async move { Ok(packet_stream!(("output", &config.output))) };
    Box::pin(task)
  }

  fn get_signature(&self, _config: Option<&Self::Config>) -> &OperationSignature {
    &self.signature
  }

  fn input_names(&self, _config: &Self::Config) -> Vec<String> {
    self.signature.inputs.iter().map(|n| n.name.clone()).collect()
  }

  fn decode_config(data: Option<wick_packet::GenericConfig>) -> Result<Self::Config, ComponentError> {
    let config = data.ok_or_else(|| {
      ComponentError::message("Merge component requires configuration, please specify configuration.")
    })?;
    Ok(Self::Config {
      output: config.get_into("output").map_err(ComponentError::new)?,
    })
  }
}
