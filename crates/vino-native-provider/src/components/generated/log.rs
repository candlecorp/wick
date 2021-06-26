use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use serde::{
  Deserialize,
  Serialize,
};
use vino_codec::messagepack::deserialize;
use vino_rpc::port::{
  Port,
  Receiver,
  Sender,
};

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub(crate) struct Inputs {
  pub(crate) input: String,
}

pub(crate) fn inputs_list() -> Vec<(String, String)> {
  vec![("input".to_string(), "string".to_string())]
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub(crate) struct InputEncoded {
  #[serde(rename = "input")]
  pub(crate) input: Vec<u8>,
}

pub(crate) fn deserialize_inputs(
  map: &HashMap<String, Vec<u8>>,
) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
  Ok(Inputs {
    input: deserialize(map.get("input").unwrap())?,
  })
}

#[derive(Default)]
pub(crate) struct Outputs {
  pub(crate) output: OutputSender,
}

pub(crate) fn outputs_list() -> Vec<(String, String)> {
  vec![("output".to_string(), "string".to_string())]
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
  type PayloadType = String;

  fn get_port(&self) -> Arc<Mutex<Port>> {
    self.port.clone()
  }
}

pub(crate) fn get_outputs() -> (Outputs, Receiver) {
  let outputs = Outputs::default();
  let ports = vec![outputs.output.port.clone()];
  let receiver = Receiver::new(ports);
  (outputs, receiver)
}
