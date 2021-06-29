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
pub struct Inputs {
  pub collection_id: String,
}

pub fn inputs_list() -> Vec<(String, String)> {
  vec![("collection_id".to_string(), "string".to_string())]
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct InputEncoded {
  #[serde(rename = "collection_id")]
  pub collection_id: Vec<u8>,
}

pub fn deserialize_inputs(
  map: &HashMap<String, Vec<u8>>,
) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
  Ok(Inputs {
    collection_id: deserialize(map.get("collection_id").unwrap())?,
  })
}

#[derive(Default)]
pub struct Outputs {
  pub document_ids: DocumentIdsSender,
}

pub fn outputs_list() -> Vec<(String, String)> {
  vec![("document_ids".to_string(), "[string]".to_string())]
}

pub struct DocumentIdsSender {
  port: Arc<Mutex<Port>>,
}
impl Default for DocumentIdsSender {
  fn default() -> Self {
    Self {
      port: Arc::new(Mutex::new(Port::new("document_ids".into()))),
    }
  }
}
impl Sender for DocumentIdsSender {
  type PayloadType = Vec<String>;

  fn get_port(&self) -> Arc<Mutex<Port>> {
    self.port.clone()
  }
}

pub fn get_outputs() -> (Outputs, Receiver) {
  let outputs = Outputs::default();
  let ports = vec![outputs.document_ids.port.clone()];
  let receiver = Receiver::new(ports);
  (outputs, receiver)
}
