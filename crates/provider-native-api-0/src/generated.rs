/**********************************************
***** This file is generated, do not edit *****
***********************************************/

use vino_provider::native::prelude::*;

use crate::generated;

pub(crate) fn get_component(
  name: &str,
) -> Option<Box<dyn NativeComponent<State = crate::State> + Sync + Send>> {
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
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  pub(crate) use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| {
        NativeComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::add::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }

  pub(crate) fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      left: payload.consume("left")?,
      right: payload.consume("right")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    #[serde(rename = "left")]
    pub(crate) left: u64,
    #[serde(rename = "right")]
    pub(crate) right: u64,
  }

  #[must_use]
  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("left", "u64"), ("right", "u64")]
  }

  #[derive(Debug, Default)]
  pub(crate) struct Outputs {
    pub(crate) output: OutputPortSender,
  }

  #[must_use]
  pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("output", "u64")]
  }

  #[derive(Debug)]
  pub(crate) struct OutputPortSender {
    port: Port,
  }
  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: Port::new("output".into()),
      }
    }
  }
  impl PortSender for OutputPortSender {
    type PayloadType = u64;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub(crate) fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortStream::create(&mut ports);
    (outputs, stream)
  }
}
pub(crate) mod concatenate {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  pub(crate) use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| {
        NativeComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::concatenate::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }

  pub(crate) fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      left: payload.consume("left")?,
      right: payload.consume("right")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    #[serde(rename = "left")]
    pub(crate) left: String,
    #[serde(rename = "right")]
    pub(crate) right: String,
  }

  #[must_use]
  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("left", "string"), ("right", "string")]
  }

  #[derive(Debug, Default)]
  pub(crate) struct Outputs {
    pub(crate) output: OutputPortSender,
  }

  #[must_use]
  pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("output", "string")]
  }

  #[derive(Debug)]
  pub(crate) struct OutputPortSender {
    port: Port,
  }
  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: Port::new("output".into()),
      }
    }
  }
  impl PortSender for OutputPortSender {
    type PayloadType = String;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub(crate) fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortStream::create(&mut ports);
    (outputs, stream)
  }
}
pub(crate) mod error {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  pub(crate) use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| {
        NativeComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::error::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }

  pub(crate) fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      input: payload.consume("input")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    #[serde(rename = "input")]
    pub(crate) input: String,
  }

  #[must_use]
  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("input", "string")]
  }

  #[derive(Debug, Default)]
  pub(crate) struct Outputs {
    pub(crate) output: OutputPortSender,
  }

  #[must_use]
  pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("output", "string")]
  }

  #[derive(Debug)]
  pub(crate) struct OutputPortSender {
    port: Port,
  }
  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: Port::new("output".into()),
      }
    }
  }
  impl PortSender for OutputPortSender {
    type PayloadType = String;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub(crate) fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortStream::create(&mut ports);
    (outputs, stream)
  }
}
pub(crate) mod log {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  pub(crate) use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| {
        NativeComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::log::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }

  pub(crate) fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      input: payload.consume("input")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    #[serde(rename = "input")]
    pub(crate) input: String,
  }

  #[must_use]
  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("input", "string")]
  }

  #[derive(Debug, Default)]
  pub(crate) struct Outputs {
    pub(crate) output: OutputPortSender,
  }

  #[must_use]
  pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("output", "string")]
  }

  #[derive(Debug)]
  pub(crate) struct OutputPortSender {
    port: Port,
  }
  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: Port::new("output".into()),
      }
    }
  }
  impl PortSender for OutputPortSender {
    type PayloadType = String;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub(crate) fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortStream::create(&mut ports);
    (outputs, stream)
  }
}
pub(crate) mod short_circuit {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  pub(crate) use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| {
        NativeComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::short_circuit::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }

  pub(crate) fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      input: payload.consume("input")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    #[serde(rename = "input")]
    pub(crate) input: String,
  }

  #[must_use]
  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("input", "string")]
  }

  #[derive(Debug, Default)]
  pub(crate) struct Outputs {
    pub(crate) output: OutputPortSender,
  }

  #[must_use]
  pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("output", "string")]
  }

  #[derive(Debug)]
  pub(crate) struct OutputPortSender {
    port: Port,
  }
  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: Port::new("output".into()),
      }
    }
  }
  impl PortSender for OutputPortSender {
    type PayloadType = String;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub(crate) fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortStream::create(&mut ports);
    (outputs, stream)
  }
}
pub(crate) mod string_to_bytes {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  pub(crate) use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| {
        NativeComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::string_to_bytes::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }

  pub(crate) fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      input: payload.consume("input")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {
    #[serde(rename = "input")]
    pub(crate) input: String,
  }

  #[must_use]
  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("input", "string")]
  }

  #[derive(Debug, Default)]
  pub(crate) struct Outputs {
    pub(crate) output: OutputPortSender,
  }

  #[must_use]
  pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("output", "bytes")]
  }

  #[derive(Debug)]
  pub(crate) struct OutputPortSender {
    port: Port,
  }
  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: Port::new("output".into()),
      }
    }
  }
  impl PortSender for OutputPortSender {
    type PayloadType = Vec<u8>;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub(crate) fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortStream::create(&mut ports);
    (outputs, stream)
  }
}
pub(crate) mod uuid {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  pub(crate) use vino_provider::native::prelude::*;

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type State = crate::State;
    async fn execute(
      &self,
      context: Context<Self::State>,
      data: TransportMap,
    ) -> Result<MessageTransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| {
        NativeComponentError::new(format!("Input deserialization error: {}", e.to_string()))
      })?;
      let (outputs, stream) = get_outputs();
      let result = crate::components::uuid::job(inputs, outputs, context).await;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(format!(
          "Job failed: {}",
          e.to_string()
        )))),
      }
    }
  }

  pub(crate) fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {})
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub(crate) struct Inputs {}

  #[must_use]
  pub(crate) fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![]
  }

  #[derive(Debug, Default)]
  pub(crate) struct Outputs {
    pub(crate) output: OutputPortSender,
  }

  #[must_use]
  pub(crate) fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("output", "string")]
  }

  #[derive(Debug)]
  pub(crate) struct OutputPortSender {
    port: Port,
  }
  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: Port::new("output".into()),
      }
    }
  }
  impl PortSender for OutputPortSender {
    type PayloadType = String;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub(crate) fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortStream::create(&mut ports);
    (outputs, stream)
  }
}
