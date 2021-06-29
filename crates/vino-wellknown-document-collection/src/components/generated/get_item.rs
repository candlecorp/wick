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
  pub document_id: String,
}

pub fn inputs_list() -> Vec<(String, String)> {
  vec![
    ("collection_id".to_string(), "string".to_string()),
    ("document_id".to_string(), "string".to_string()),
  ]
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct InputEncoded {
  #[serde(rename = "collection_id")]
  pub collection_id: Vec<u8>,
  #[serde(rename = "document_id")]
  pub document_id: Vec<u8>,
}

pub fn deserialize_inputs(
  map: &HashMap<String, Vec<u8>>,
) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
  Ok(Inputs {
    collection_id: deserialize(map.get("collection_id").unwrap())?,
    document_id: deserialize(map.get("document_id").unwrap())?,
  })
}

#[derive(Default)]
pub struct Outputs {
  pub document: DocumentSender,
}

pub fn outputs_list() -> Vec<(String, String)> {
  vec![("document".to_string(), "string".to_string())]
}

pub struct DocumentSender {
  port: Arc<Mutex<Port>>,
}
impl Default for DocumentSender {
  fn default() -> Self {
    Self {
      port: Arc::new(Mutex::new(Port::new("document".into()))),
    }
  }
}
impl Sender for DocumentSender {
  type PayloadType = String;

  fn get_port(&self) -> Arc<Mutex<Port>> {
    self.port.clone()
  }
}

pub fn get_outputs() -> (Outputs, Receiver) {
  let outputs = Outputs::default();
  let ports = vec![outputs.document.port.clone()];
  let receiver = Receiver::new(ports);
  (outputs, receiver)
}
