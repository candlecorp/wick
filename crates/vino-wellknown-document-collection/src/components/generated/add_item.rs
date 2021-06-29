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
  pub document_id: String,
  pub collection_id: String,
  pub document: String,
}

pub fn inputs_list() -> Vec<(String, String)> {
  vec![
    ("document_id".to_string(), "string".to_string()),
    ("collection_id".to_string(), "string".to_string()),
    ("document".to_string(), "string".to_string()),
  ]
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct InputEncoded {
  #[serde(rename = "document_id")]
  pub document_id: Vec<u8>,
  #[serde(rename = "collection_id")]
  pub collection_id: Vec<u8>,
  #[serde(rename = "document")]
  pub document: Vec<u8>,
}

pub fn deserialize_inputs(
  map: &HashMap<String, Vec<u8>>,
) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
  Ok(Inputs {
    document_id: deserialize(map.get("document_id").unwrap())?,
    collection_id: deserialize(map.get("collection_id").unwrap())?,
    document: deserialize(map.get("document").unwrap())?,
  })
}

#[derive(Default)]
pub struct Outputs {
  pub document_id: DocumentIdSender,
}

pub fn outputs_list() -> Vec<(String, String)> {
  vec![("document_id".to_string(), "string".to_string())]
}

pub struct DocumentIdSender {
  port: Arc<Mutex<Port>>,
}
impl Default for DocumentIdSender {
  fn default() -> Self {
    Self {
      port: Arc::new(Mutex::new(Port::new("document_id".into()))),
    }
  }
}
impl Sender for DocumentIdSender {
  type PayloadType = String;

  fn get_port(&self) -> Arc<Mutex<Port>> {
    self.port.clone()
  }
}

pub fn get_outputs() -> (Outputs, Receiver) {
  let outputs = Outputs::default();
  let ports = vec![outputs.document_id.port.clone()];
  let receiver = Receiver::new(ports);
  (outputs, receiver)
}
