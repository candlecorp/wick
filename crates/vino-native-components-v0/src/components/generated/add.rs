use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use async_trait::async_trait;
use serde::{
  Deserialize,
  Serialize,
};
use vino_codec::messagepack::deserialize;
use vino_provider::error::ProviderError;
use vino_provider::{
  Context as ProviderContext,
  VinoProviderComponent,
};
use vino_rpc::port::{
  Port,
  PortStream,
  Sender,
};

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub(crate) struct Inputs {
  pub(crate) left: u64,
  pub(crate) right: u64,
}

pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
  vec![("left", "u64"), ("right", "u64")]
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub(crate) struct InputEncoded {
  #[serde(rename = "left")]
  pub(crate) left: Vec<u8>,
  #[serde(rename = "right")]
  pub(crate) right: Vec<u8>,
}

pub(crate) fn deserialize_inputs(
  map: &HashMap<String, Vec<u8>>,
) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
  Ok(Inputs {
    left: deserialize(map.get("left").unwrap())?,
    right: deserialize(map.get("right").unwrap())?,
  })
}

#[derive(Default)]
pub(crate) struct Outputs {
  pub(crate) output: OutputSender,
}

pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
  vec![("output", "u64")]
}

pub(crate) struct OutputSender {
  port: Arc<Mutex<Port>>,
}
impl Default for OutputSender {
  fn default() -> Self {
    Self {
      port: Arc::new(Mutex::new(Port::new("output".into()))),
    }
  }
}
impl Sender for OutputSender {
  type PayloadType = u64;

  fn get_port(&self) -> Arc<Mutex<Port>> {
    self.port.clone()
  }
}

pub(crate) fn get_outputs() -> (Outputs, PortStream) {
  let outputs = Outputs::default();
  let ports = vec![outputs.output.port.clone()];
  let stream = PortStream::new(ports);
  (outputs, stream)
}

pub(crate) struct Component {}
impl Default for Component {
  fn default() -> Self {
    Self {}
  }
}

#[async_trait]
impl VinoProviderComponent for Component {
  type Context = crate::State;

  fn get_name(&self) -> String {
    format!("vino::{}", "add")
  }
  fn get_input_ports(&self) -> Vec<(&'static str, &'static str)> {
    inputs_list()
  }
  fn get_output_ports(&self) -> Vec<(&'static str, &'static str)> {
    outputs_list()
  }
  async fn job_wrapper(
    &self,
    context: ProviderContext<Self::Context>,
    data: HashMap<String, Vec<u8>>,
  ) -> Result<PortStream, Box<dyn std::error::Error + Send + Sync>> {
    let inputs = deserialize_inputs(&data).map_err(ProviderError::InputDeserializationError)?;
    let (outputs, stream) = get_outputs();
    let result = super::super::add::job(inputs, outputs, context).await;
    match result {
      Ok(_) => Ok(stream),
      Err(e) => Err(ProviderError::JobError(format!("Job failed: {}", e.to_string())).into()),
    }
  }
}
