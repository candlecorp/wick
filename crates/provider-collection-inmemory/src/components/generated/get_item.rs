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
  pub(crate) collection_id: String,
  pub(crate) document_id: String,
}

pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
  vec![("collection_id", "string"), ("document_id", "string")]
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub(crate) struct InputEncoded {
  #[serde(rename = "collection_id")]
  pub(crate) collection_id: Vec<u8>,
  #[serde(rename = "document_id")]
  pub(crate) document_id: Vec<u8>,
}

pub(crate) fn deserialize_inputs(
  map: &HashMap<String, Vec<u8>>,
) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
  Ok(Inputs {
    collection_id: deserialize(map.get("collection_id").unwrap())?,
    document_id: deserialize(map.get("document_id").unwrap())?,
  })
}

#[derive(Default)]
pub(crate) struct Outputs {
  pub(crate) document: DocumentSender,
}

pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
  vec![("document", "string")]
}

pub(crate) struct DocumentSender {
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

pub(crate) fn get_outputs() -> (Outputs, PortStream) {
  let outputs = Outputs::default();
  let ports = vec![outputs.document.port.clone()];
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
    format!("vino::{}", "get-item")
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
    let result = super::super::get_item::job(inputs, outputs, context).await;
    match result {
      Ok(_) => Ok(stream),
      Err(e) => Err(ProviderError::JobError(format!("Job failed: {}", e.to_string())).into()),
    }
  }
}
