/**********************************************
***** This file is generated, do not edit *****
***********************************************/

use vino_provider::{
  ComponentSignature,
  VinoProviderComponent,
};

use crate::generated;

pub(crate) fn get_component(
  name: &str,
) -> Option<Box<dyn VinoProviderComponent<Context = crate::State> + Sync + Send>> {
  match name {
    "add" => Some(Box::new(generated::add::Component::default())),
    "concatenate" => Some(Box::new(generated::concatenate::Component::default())),
    "error" => Some(Box::new(generated::error::Component::default())),
    "log" => Some(Box::new(generated::log::Component::default())),
    "short-circuit" => Some(Box::new(generated::short_circuit::Component::default())),
    "string-to-bytes" => Some(Box::new(generated::string_to_bytes::Component::default())),
    "uuid" => Some(Box::new(generated::uuid::Component::default())),
    _ => None,
  }
}

pub(crate) fn get_all_components() -> Vec<ComponentSignature> {
  vec![
    ComponentSignature {
      name: "add".to_owned(),
      inputs: generated::add::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::add::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "concatenate".to_owned(),
      inputs: generated::concatenate::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::concatenate::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "error".to_owned(),
      inputs: generated::error::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::error::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "log".to_owned(),
      inputs: generated::log::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::log::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "short-circuit".to_owned(),
      inputs: generated::short_circuit::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::short_circuit::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "string-to-bytes".to_owned(),
      inputs: generated::string_to_bytes::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::string_to_bytes::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
    ComponentSignature {
      name: "uuid".to_owned(),
      inputs: generated::uuid::inputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
      outputs: generated::uuid::outputs_list()
        .into_iter()
        .map(From::from)
        .collect(),
    },
  ]
}

pub(crate) mod add {

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
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  pub(crate) use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
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
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::add::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod concatenate {

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
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  pub(crate) use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    pub(crate) left: String,
    pub(crate) right: String,
  }

  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("left", "string"), ("right", "string")]
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
    vec![("output", "string")]
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
      format!("vino::{}", "concatenate")
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
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::concatenate::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod error {

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
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  pub(crate) use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    pub(crate) input: String,
  }

  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("input", "string")]
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

  pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("output", "string")]
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
      format!("vino::{}", "error")
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
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::error::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod log {

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
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  pub(crate) use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    pub(crate) input: String,
  }

  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("input", "string")]
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

  pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("output", "string")]
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
      format!("vino::{}", "log")
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
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::log::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod short_circuit {

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
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  pub(crate) use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    pub(crate) input: String,
  }

  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("input", "string")]
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

  pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("output", "string")]
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
      format!("vino::{}", "short-circuit")
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
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::short_circuit::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod string_to_bytes {

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
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  pub(crate) use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    pub(crate) input: String,
  }

  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("input", "string")]
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

  pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("output", "bytes")]
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
    type PayloadType = Vec<u8>;

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
      format!("vino::{}", "string-to-bytes")
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
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::string_to_bytes::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
pub(crate) mod uuid {

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
  use vino_provider::error::ProviderComponentError;
  use vino_provider::{
    Context as ProviderContext,
    VinoProviderComponent,
  };
  pub(crate) use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {}

  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![]
  }

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct InputEncoded {}

  pub(crate) fn deserialize_inputs(
    map: &HashMap<String, Vec<u8>>,
  ) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Inputs {})
  }

  #[derive(Default)]
  pub(crate) struct Outputs {
    pub(crate) output: OutputSender,
  }

  pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("output", "string")]
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
      format!("vino::{}", "uuid")
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
    ) -> Result<PortStream, Box<ProviderComponentError>> {
      let inputs = deserialize_inputs(&data).map_err(|e| {
        ProviderComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::uuid::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(ProviderComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }
}
