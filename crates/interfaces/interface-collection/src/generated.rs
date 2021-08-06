/**********************************************
***** This file is generated, do not edit *****
***********************************************/

pub mod add_item {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  #[must_use]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "add-item".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      document_id: payload.consume("document_id")?,
      collection_id: payload.consume("collection_id")?,
      document: payload.consume("document")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    #[serde(rename = "document_id")]
    pub document_id: String,
    #[serde(rename = "collection_id")]
    pub collection_id: String,
    #[serde(rename = "document")]
    pub document: String,
  }

  static INPUTS_LIST: &[(&str, &str)] = &[
    ("document_id", "string"),
    ("collection_id", "string"),
    ("document", "string"),
  ];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct Outputs {
    pub document_id: DocumentIdPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("document_id", "string")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct DocumentIdPortSender {
    port: PortChannel,
  }

  impl Default for DocumentIdPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("document_id"),
      }
    }
  }
  impl PortSender for DocumentIdPortSender {
    type PayloadType = String;

    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (Outputs, TransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.document_id.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }
}
pub mod get_item {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  #[must_use]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "get-item".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      collection_id: payload.consume("collection_id")?,
      document_id: payload.consume("document_id")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    #[serde(rename = "collection_id")]
    pub collection_id: String,
    #[serde(rename = "document_id")]
    pub document_id: String,
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("collection_id", "string"), ("document_id", "string")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct Outputs {
    pub document: DocumentPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("document", "string")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct DocumentPortSender {
    port: PortChannel,
  }

  impl Default for DocumentPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("document"),
      }
    }
  }
  impl PortSender for DocumentPortSender {
    type PayloadType = String;

    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (Outputs, TransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.document.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }
}
pub mod list_items {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  #[must_use]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "list-items".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      collection_id: payload.consume("collection_id")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    #[serde(rename = "collection_id")]
    pub collection_id: String,
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("collection_id", "string")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct Outputs {
    pub document_ids: DocumentIdsPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("document_ids", "[string]")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct DocumentIdsPortSender {
    port: PortChannel,
  }

  impl Default for DocumentIdsPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("document_ids"),
      }
    }
  }
  impl PortSender for DocumentIdsPortSender {
    type PayloadType = Vec<String>;

    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (Outputs, TransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.document_ids.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }
}
pub mod rm_item {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  #[must_use]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "rm-item".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      collection_id: payload.consume("collection_id")?,
      document_id: payload.consume("document_id")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    #[serde(rename = "collection_id")]
    pub collection_id: String,
    #[serde(rename = "document_id")]
    pub document_id: String,
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("collection_id", "string"), ("document_id", "string")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct Outputs {}

  static OUTPUTS_LIST: &[(&str, &str)] = &[];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[must_use]
  pub fn get_outputs() -> (Outputs, TransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }
}
