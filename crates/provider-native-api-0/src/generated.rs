/**********************************************
***** This file is generated, do not edit *****
***********************************************/

use vino_provider::native::prelude::*;

use crate::generated;

#[derive(Debug)]
pub(crate) struct Dispatcher {}
#[async_trait]
impl Dispatch for Dispatcher {
  type Context = crate::Context;
  async fn dispatch(
    op: &str,
    context: Self::Context,
    data: TransportMap,
  ) -> Result<TransportStream, Box<NativeComponentError>> {
    use generated::*;
    let result = match op {
      "add" => add::Component::default().execute(context, data).await,
      "concatenate" => {
        concatenate::Component::default()
          .execute(context, data)
          .await
      }
      "error" => error::Component::default().execute(context, data).await,
      "gate" => gate::Component::default().execute(context, data).await,
      "log" => log::Component::default().execute(context, data).await,
      "negate" => negate::Component::default().execute(context, data).await,
      "panic" => panic::Component::default().execute(context, data).await,
      "short-circuit" => {
        short_circuit::Component::default()
          .execute(context, data)
          .await
      }
      "string-to-bytes" => {
        string_to_bytes::Component::default()
          .execute(context, data)
          .await
      }
      "uuid" => uuid::Component::default().execute(context, data).await,
      _ => Err(Box::new(NativeComponentError::new(format!(
        "Component not found on this provider: {}",
        op
      )))),
    }?;
    Ok(result)
  }
}

pub(crate) fn get_all_components() -> Vec<ComponentSignature> {
  vec![
    generated::add::signature(),
    generated::concatenate::signature(),
    generated::error::signature(),
    generated::gate::signature(),
    generated::log::signature(),
    generated::negate::signature(),
    generated::panic::signature(),
    generated::short_circuit::signature(),
    generated::string_to_bytes::signature(),
    generated::uuid::signature(),
  ]
}

pub(crate) mod add {
  #![allow(unused, unreachable_pub)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  pub(crate) use vino_provider::native::prelude::*;

  pub(crate) fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "add".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type Context = crate::Context;
    async fn execute(
      &self,
      context: Self::Context,
      data: TransportMap,
    ) -> Result<TransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
      let (outputs, stream) = get_outputs();
      let result = tokio::spawn(crate::components::add::job(inputs, outputs, context))
        .await
        .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
      }
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      left: payload.consume("left")?,
      right: payload.consume("right")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "left")]
    pub left: u64,
    #[serde(rename = "right")]
    pub right: u64,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("left".to_owned(), MessageTransport::success(&inputs.left));

      map.insert("right".to_owned(), MessageTransport::success(&inputs.right));

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("left", "u64"), ("right", "u64")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub output: OutputPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("output", "u64")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct OutputPortSender {
    port: PortChannel,
  }

  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("output"),
      }
    }
  }
  impl PortSender for OutputPortSender {
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
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| ComponentError::new("No packets for port 'output' found"))?;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub(crate) mod concatenate {
  #![allow(unused, unreachable_pub)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  pub(crate) use vino_provider::native::prelude::*;

  pub(crate) fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "concatenate".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type Context = crate::Context;
    async fn execute(
      &self,
      context: Self::Context,
      data: TransportMap,
    ) -> Result<TransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
      let (outputs, stream) = get_outputs();
      let result = tokio::spawn(crate::components::concatenate::job(
        inputs, outputs, context,
      ))
      .await
      .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
      }
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      left: payload.consume("left")?,
      right: payload.consume("right")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "left")]
    pub left: String,
    #[serde(rename = "right")]
    pub right: String,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("left".to_owned(), MessageTransport::success(&inputs.left));

      map.insert("right".to_owned(), MessageTransport::success(&inputs.right));

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("left", "string"), ("right", "string")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub output: OutputPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("output", "string")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct OutputPortSender {
    port: PortChannel,
  }

  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("output"),
      }
    }
  }
  impl PortSender for OutputPortSender {
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
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| ComponentError::new("No packets for port 'output' found"))?;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub(crate) mod error {
  #![allow(unused, unreachable_pub)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  pub(crate) use vino_provider::native::prelude::*;

  pub(crate) fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "error".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type Context = crate::Context;
    async fn execute(
      &self,
      context: Self::Context,
      data: TransportMap,
    ) -> Result<TransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
      let (outputs, stream) = get_outputs();
      let result = tokio::spawn(crate::components::error::job(inputs, outputs, context))
        .await
        .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
      }
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      input: payload.consume("input")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("input".to_owned(), MessageTransport::success(&inputs.input));

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("input", "string")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub output: OutputPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("output", "string")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct OutputPortSender {
    port: PortChannel,
  }

  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("output"),
      }
    }
  }
  impl PortSender for OutputPortSender {
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
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| ComponentError::new("No packets for port 'output' found"))?;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub(crate) mod gate {
  #![allow(unused, unreachable_pub)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  pub(crate) use vino_provider::native::prelude::*;

  pub(crate) fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "gate".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type Context = crate::Context;
    async fn execute(
      &self,
      context: Self::Context,
      data: TransportMap,
    ) -> Result<TransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
      let (outputs, stream) = get_outputs();
      let result = tokio::spawn(crate::components::gate::job(inputs, outputs, context))
        .await
        .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
      }
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      condition: payload.consume("condition")?,
      value: payload.consume_raw("value")?.into(),
      exception: payload.consume("exception")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "condition")]
    pub condition: bool,
    #[serde(rename = "value")]
    pub value: RawPacket,
    #[serde(rename = "exception")]
    pub exception: String,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert(
        "condition".to_owned(),
        MessageTransport::success(&inputs.condition),
      );

      map.insert("value".to_owned(), inputs.value.into());

      map.insert(
        "exception".to_owned(),
        MessageTransport::success(&inputs.exception),
      );

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[
    ("condition", "bool"),
    ("value", "raw"),
    ("exception", "string"),
  ];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub output: OutputPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("output", "raw")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct OutputPortSender {
    port: PortChannel,
  }

  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("output"),
      }
    }
  }
  impl PortSender for OutputPortSender {
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
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| ComponentError::new("No packets for port 'output' found"))?;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub(crate) mod log {
  #![allow(unused, unreachable_pub)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  pub(crate) use vino_provider::native::prelude::*;

  pub(crate) fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "log".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type Context = crate::Context;
    async fn execute(
      &self,
      context: Self::Context,
      data: TransportMap,
    ) -> Result<TransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
      let (outputs, stream) = get_outputs();
      let result = tokio::spawn(crate::components::log::job(inputs, outputs, context))
        .await
        .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
      }
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      input: payload.consume("input")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("input".to_owned(), MessageTransport::success(&inputs.input));

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("input", "string")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub output: OutputPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("output", "string")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct OutputPortSender {
    port: PortChannel,
  }

  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("output"),
      }
    }
  }
  impl PortSender for OutputPortSender {
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
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| ComponentError::new("No packets for port 'output' found"))?;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub(crate) mod negate {
  #![allow(unused, unreachable_pub)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  pub(crate) use vino_provider::native::prelude::*;

  pub(crate) fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "negate".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type Context = crate::Context;
    async fn execute(
      &self,
      context: Self::Context,
      data: TransportMap,
    ) -> Result<TransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
      let (outputs, stream) = get_outputs();
      let result = tokio::spawn(crate::components::negate::job(inputs, outputs, context))
        .await
        .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
      }
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      input: payload.consume("input")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "input")]
    pub input: bool,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("input".to_owned(), MessageTransport::success(&inputs.input));

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("input", "bool")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub output: OutputPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("output", "bool")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct OutputPortSender {
    port: PortChannel,
  }

  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("output"),
      }
    }
  }
  impl PortSender for OutputPortSender {
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
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| ComponentError::new("No packets for port 'output' found"))?;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub(crate) mod panic {
  #![allow(unused, unreachable_pub)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  pub(crate) use vino_provider::native::prelude::*;

  pub(crate) fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "panic".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type Context = crate::Context;
    async fn execute(
      &self,
      context: Self::Context,
      data: TransportMap,
    ) -> Result<TransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
      let (outputs, stream) = get_outputs();
      let result = tokio::spawn(crate::components::panic::job(inputs, outputs, context))
        .await
        .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
      }
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      input: payload.consume("input")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("input".to_owned(), MessageTransport::success(&inputs.input));

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("input", "string")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub output: OutputPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("output", "string")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct OutputPortSender {
    port: PortChannel,
  }

  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("output"),
      }
    }
  }
  impl PortSender for OutputPortSender {
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
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| ComponentError::new("No packets for port 'output' found"))?;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub(crate) mod short_circuit {
  #![allow(unused, unreachable_pub)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  pub(crate) use vino_provider::native::prelude::*;

  pub(crate) fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "short-circuit".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type Context = crate::Context;
    async fn execute(
      &self,
      context: Self::Context,
      data: TransportMap,
    ) -> Result<TransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
      let (outputs, stream) = get_outputs();
      let result = tokio::spawn(crate::components::short_circuit::job(
        inputs, outputs, context,
      ))
      .await
      .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
      }
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      input: payload.consume("input")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("input".to_owned(), MessageTransport::success(&inputs.input));

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("input", "string")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub output: OutputPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("output", "string")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct OutputPortSender {
    port: PortChannel,
  }

  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("output"),
      }
    }
  }
  impl PortSender for OutputPortSender {
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
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| ComponentError::new("No packets for port 'output' found"))?;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub(crate) mod string_to_bytes {
  #![allow(unused, unreachable_pub)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  pub(crate) use vino_provider::native::prelude::*;

  pub(crate) fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "string-to-bytes".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type Context = crate::Context;
    async fn execute(
      &self,
      context: Self::Context,
      data: TransportMap,
    ) -> Result<TransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
      let (outputs, stream) = get_outputs();
      let result = tokio::spawn(crate::components::string_to_bytes::job(
        inputs, outputs, context,
      ))
      .await
      .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
      }
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      input: payload.consume("input")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "input")]
    pub input: String,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert("input".to_owned(), MessageTransport::success(&inputs.input));

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("input", "string")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub output: OutputPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("output", "bytes")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct OutputPortSender {
    port: PortChannel,
  }

  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("output"),
      }
    }
  }
  impl PortSender for OutputPortSender {
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
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| ComponentError::new("No packets for port 'output' found"))?;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub(crate) mod uuid {
  #![allow(unused, unreachable_pub)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use serde::{
    Deserialize,
    Serialize,
  };
  pub(crate) use vino_provider::native::prelude::*;

  pub(crate) fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "uuid".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  #[derive(Default)]
  pub(crate) struct Component {}

  #[async_trait]
  impl NativeComponent for Component {
    type Context = crate::Context;
    async fn execute(
      &self,
      context: Self::Context,
      data: TransportMap,
    ) -> Result<TransportStream, Box<NativeComponentError>> {
      let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
      let (outputs, stream) = get_outputs();
      let result = tokio::spawn(crate::components::uuid::job(inputs, outputs, context))
        .await
        .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
      match result {
        Ok(_) => Ok(stream),
        Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
      }
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {})
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {}

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub output: OutputPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("output", "string")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct OutputPortSender {
    port: PortChannel,
  }

  impl Default for OutputPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("output"),
      }
    }
  }
  impl PortSender for OutputPortSender {
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
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.output.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn output(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("output")
        .ok_or_else(|| ComponentError::new("No packets for port 'output' found"))?;
      Ok(PortOutput::new("output".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
