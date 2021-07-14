/**********************************************
***** This file is generated, do not edit *****
***********************************************/

pub mod add_item {

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
  pub use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    pub document_id: String,
    pub collection_id: String,
    pub document: String,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![
      ("document_id", "string"),
      ("collection_id", "string"),
      ("document", "string"),
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

  #[derive(Default, Debug)]
  pub struct Outputs {
    pub document_id: DocumentIdSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("document_id", "string")]
  }

  #[derive(Debug)]
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

  pub fn get_outputs() -> (Outputs, PortStream) {
    let outputs = Outputs::default();
    let ports = vec![outputs.document_id.port.clone()];
    let stream = PortStream::new(ports);
    (outputs, stream)
  }
}
pub mod get_item {

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
  pub use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    pub collection_id: String,
    pub document_id: String,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("collection_id", "string"), ("document_id", "string")]
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

  #[derive(Default, Debug)]
  pub struct Outputs {
    pub document: DocumentSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("document", "string")]
  }

  #[derive(Debug)]
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

  pub fn get_outputs() -> (Outputs, PortStream) {
    let outputs = Outputs::default();
    let ports = vec![outputs.document.port.clone()];
    let stream = PortStream::new(ports);
    (outputs, stream)
  }
}
pub mod list_items {

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
  pub use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    pub collection_id: String,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("collection_id", "string")]
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

  #[derive(Default, Debug)]
  pub struct Outputs {
    pub document_ids: DocumentIdsSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("document_ids", "[string]")]
  }

  #[derive(Debug)]
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

  pub fn get_outputs() -> (Outputs, PortStream) {
    let outputs = Outputs::default();
    let ports = vec![outputs.document_ids.port.clone()];
    let stream = PortStream::new(ports);
    (outputs, stream)
  }
}
pub mod rm_item {

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
  pub use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    pub collection_id: String,
    pub document_id: String,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("collection_id", "string"), ("document_id", "string")]
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

  #[derive(Default, Debug)]
  pub struct Outputs {}

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![]
  }

  pub fn get_outputs() -> (Outputs, PortStream) {
    let outputs = Outputs::default();
    let ports = vec![];
    let stream = PortStream::new(ports);
    (outputs, stream)
  }
}
