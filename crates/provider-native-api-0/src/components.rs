/**********************************************
***** This file is generated, do not edit *****
***********************************************/

pub use vino_provider::prelude::*;

pub mod add;
pub mod concatenate;
pub mod error;
pub mod log;
pub mod panic;
pub mod random_bytes;
pub mod random_string;
pub mod uuid;

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
    let result = match op {
      "add" => {
        generated::add::Component::default()
          .execute(context, data)
          .await
      }
      "concatenate" => {
        generated::concatenate::Component::default()
          .execute(context, data)
          .await
      }
      "error" => {
        generated::error::Component::default()
          .execute(context, data)
          .await
      }
      "log" => {
        generated::log::Component::default()
          .execute(context, data)
          .await
      }
      "panic" => {
        generated::panic::Component::default()
          .execute(context, data)
          .await
      }
      "random-bytes" => {
        generated::random_bytes::Component::default()
          .execute(context, data)
          .await
      }
      "random-string" => {
        generated::random_string::Component::default()
          .execute(context, data)
          .await
      }
      "uuid" => {
        generated::uuid::Component::default()
          .execute(context, data)
          .await
      }
      _ => Err(Box::new(NativeComponentError::new(format!(
        "Component not found on this provider: {}",
        op
      )))),
    }?;
    Ok(result)
  }
}

pub fn get_signature() -> ProviderSignature {
  let mut components = std::collections::HashMap::new();

  components.insert("add".to_owned(), generated::add::signature());
  components.insert(
    "concatenate".to_owned(),
    generated::concatenate::signature(),
  );
  components.insert("error".to_owned(), generated::error::signature());
  components.insert("log".to_owned(), generated::log::signature());
  components.insert("panic".to_owned(), generated::panic::signature());
  components.insert(
    "random-bytes".to_owned(),
    generated::random_bytes::signature(),
  );
  components.insert(
    "random-string".to_owned(),
    generated::random_string::signature(),
  );
  components.insert("uuid".to_owned(), generated::uuid::signature());

  ProviderSignature {
    name: Some("vino-native-api-0".to_owned()),
    types: std::collections::HashMap::from([]).into(),
    components: components.into(),
  }
}

pub mod types {
  // no additional types
}

pub mod generated {
  pub mod add {
    #![allow(unused, unreachable_pub)]
    use std::collections::HashMap;

    use async_trait::async_trait;

    pub use vino_provider::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "add".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default)]
    pub struct Component {}

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

    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
      Ok(Inputs {
        left: payload.consume("left")?,
        right: payload.consume("right")?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "left")]
      pub left: u64,
      #[serde(rename = "right")]
      pub right: u64,
    }

    #[cfg(any(feature = "native", feature = "wasm"))]
    impl From<Inputs> for TransportMap {
      fn from(inputs: Inputs) -> TransportMap {
        let mut map = TransportMap::new();
        map.insert("left".to_owned(), MessageTransport::success(&inputs.left));

        map.insert("right".to_owned(), MessageTransport::success(&inputs.right));

        map
      }
    }

    #[must_use]
    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn inputs_list() -> HashMap<String, TypeSignature> {
      let mut map = HashMap::new();
      map.insert("left".to_owned(), TypeSignature::U64);
      map.insert("right".to_owned(), TypeSignature::U64);
      map
    }
    #[derive(Debug, Default)]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    #[must_use]
    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn outputs_list() -> HashMap<String, TypeSignature> {
      let mut map = HashMap::new();
      map.insert("output".to_owned(), TypeSignature::U64);
      map
    }

    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct OutputPortSender {
      port: PortChannel,
    }

    #[cfg(feature = "provider")]
    impl Default for OutputPortSender {
      fn default() -> Self {
        Self {
          port: PortChannel::new("output"),
        }
      }
    }

    #[cfg(feature = "provider")]
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
    #[cfg(feature = "provider")]
    pub fn get_outputs() -> (OutputPorts, TransportStream) {
      let mut outputs = OutputPorts::default();
      let mut ports = vec![&mut outputs.output.port];
      let stream = PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl Outputs {
      pub async fn output(&mut self) -> Result<PortOutput<u64>, ProviderError> {
        let packets = self.packets.take("output").await;
        Ok(PortOutput::new("output".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl Outputs {}

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl From<ProviderOutput> for Outputs {
      fn from(packets: ProviderOutput) -> Self {
        Self { packets }
      }
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl From<BoxedTransportStream> for Outputs {
      fn from(stream: BoxedTransportStream) -> Self {
        Self {
          packets: ProviderOutput::new(stream),
        }
      }
    }
  }
  pub mod concatenate {
    #![allow(unused, unreachable_pub)]
    use std::collections::HashMap;

    use async_trait::async_trait;

    pub use vino_provider::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "concatenate".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default)]
    pub struct Component {}

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

    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
      Ok(Inputs {
        left: payload.consume("left")?,
        right: payload.consume("right")?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "left")]
      pub left: String,
      #[serde(rename = "right")]
      pub right: String,
    }

    #[cfg(any(feature = "native", feature = "wasm"))]
    impl From<Inputs> for TransportMap {
      fn from(inputs: Inputs) -> TransportMap {
        let mut map = TransportMap::new();
        map.insert("left".to_owned(), MessageTransport::success(&inputs.left));

        map.insert("right".to_owned(), MessageTransport::success(&inputs.right));

        map
      }
    }

    #[must_use]
    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn inputs_list() -> HashMap<String, TypeSignature> {
      let mut map = HashMap::new();
      map.insert("left".to_owned(), TypeSignature::String);
      map.insert("right".to_owned(), TypeSignature::String);
      map
    }
    #[derive(Debug, Default)]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    #[must_use]
    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn outputs_list() -> HashMap<String, TypeSignature> {
      let mut map = HashMap::new();
      map.insert("output".to_owned(), TypeSignature::String);
      map
    }

    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct OutputPortSender {
      port: PortChannel,
    }

    #[cfg(feature = "provider")]
    impl Default for OutputPortSender {
      fn default() -> Self {
        Self {
          port: PortChannel::new("output"),
        }
      }
    }

    #[cfg(feature = "provider")]
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
    #[cfg(feature = "provider")]
    pub fn get_outputs() -> (OutputPorts, TransportStream) {
      let mut outputs = OutputPorts::default();
      let mut ports = vec![&mut outputs.output.port];
      let stream = PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl Outputs {
      pub async fn output(&mut self) -> Result<PortOutput<String>, ProviderError> {
        let packets = self.packets.take("output").await;
        Ok(PortOutput::new("output".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl Outputs {}

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl From<ProviderOutput> for Outputs {
      fn from(packets: ProviderOutput) -> Self {
        Self { packets }
      }
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl From<BoxedTransportStream> for Outputs {
      fn from(stream: BoxedTransportStream) -> Self {
        Self {
          packets: ProviderOutput::new(stream),
        }
      }
    }
  }
  pub mod error {
    #![allow(unused, unreachable_pub)]
    use std::collections::HashMap;

    use async_trait::async_trait;

    pub use vino_provider::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "error".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default)]
    pub struct Component {}

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

    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
      Ok(Inputs {
        input: payload.consume("input")?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "input")]
      pub input: String,
    }

    #[cfg(any(feature = "native", feature = "wasm"))]
    impl From<Inputs> for TransportMap {
      fn from(inputs: Inputs) -> TransportMap {
        let mut map = TransportMap::new();
        map.insert("input".to_owned(), MessageTransport::success(&inputs.input));

        map
      }
    }

    #[must_use]
    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn inputs_list() -> HashMap<String, TypeSignature> {
      let mut map = HashMap::new();
      map.insert("input".to_owned(), TypeSignature::String);
      map
    }
    #[derive(Debug, Default)]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    #[must_use]
    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn outputs_list() -> HashMap<String, TypeSignature> {
      let mut map = HashMap::new();
      map.insert("output".to_owned(), TypeSignature::String);
      map
    }

    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct OutputPortSender {
      port: PortChannel,
    }

    #[cfg(feature = "provider")]
    impl Default for OutputPortSender {
      fn default() -> Self {
        Self {
          port: PortChannel::new("output"),
        }
      }
    }

    #[cfg(feature = "provider")]
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
    #[cfg(feature = "provider")]
    pub fn get_outputs() -> (OutputPorts, TransportStream) {
      let mut outputs = OutputPorts::default();
      let mut ports = vec![&mut outputs.output.port];
      let stream = PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl Outputs {
      pub async fn output(&mut self) -> Result<PortOutput<String>, ProviderError> {
        let packets = self.packets.take("output").await;
        Ok(PortOutput::new("output".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl Outputs {}

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl From<ProviderOutput> for Outputs {
      fn from(packets: ProviderOutput) -> Self {
        Self { packets }
      }
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl From<BoxedTransportStream> for Outputs {
      fn from(stream: BoxedTransportStream) -> Self {
        Self {
          packets: ProviderOutput::new(stream),
        }
      }
    }
  }
  pub mod log {
    #![allow(unused, unreachable_pub)]
    use std::collections::HashMap;

    use async_trait::async_trait;

    pub use vino_provider::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "log".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default)]
    pub struct Component {}

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

    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
      Ok(Inputs {
        input: payload.consume("input")?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "input")]
      pub input: String,
    }

    #[cfg(any(feature = "native", feature = "wasm"))]
    impl From<Inputs> for TransportMap {
      fn from(inputs: Inputs) -> TransportMap {
        let mut map = TransportMap::new();
        map.insert("input".to_owned(), MessageTransport::success(&inputs.input));

        map
      }
    }

    #[must_use]
    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn inputs_list() -> HashMap<String, TypeSignature> {
      let mut map = HashMap::new();
      map.insert("input".to_owned(), TypeSignature::String);
      map
    }
    #[derive(Debug, Default)]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    #[must_use]
    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn outputs_list() -> HashMap<String, TypeSignature> {
      let mut map = HashMap::new();
      map.insert("output".to_owned(), TypeSignature::String);
      map
    }

    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct OutputPortSender {
      port: PortChannel,
    }

    #[cfg(feature = "provider")]
    impl Default for OutputPortSender {
      fn default() -> Self {
        Self {
          port: PortChannel::new("output"),
        }
      }
    }

    #[cfg(feature = "provider")]
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
    #[cfg(feature = "provider")]
    pub fn get_outputs() -> (OutputPorts, TransportStream) {
      let mut outputs = OutputPorts::default();
      let mut ports = vec![&mut outputs.output.port];
      let stream = PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl Outputs {
      pub async fn output(&mut self) -> Result<PortOutput<String>, ProviderError> {
        let packets = self.packets.take("output").await;
        Ok(PortOutput::new("output".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl Outputs {}

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl From<ProviderOutput> for Outputs {
      fn from(packets: ProviderOutput) -> Self {
        Self { packets }
      }
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl From<BoxedTransportStream> for Outputs {
      fn from(stream: BoxedTransportStream) -> Self {
        Self {
          packets: ProviderOutput::new(stream),
        }
      }
    }
  }
  pub mod panic {
    #![allow(unused, unreachable_pub)]
    use std::collections::HashMap;

    use async_trait::async_trait;

    pub use vino_provider::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "panic".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default)]
    pub struct Component {}

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

    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
      Ok(Inputs {
        input: payload.consume("input")?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "input")]
      pub input: String,
    }

    #[cfg(any(feature = "native", feature = "wasm"))]
    impl From<Inputs> for TransportMap {
      fn from(inputs: Inputs) -> TransportMap {
        let mut map = TransportMap::new();
        map.insert("input".to_owned(), MessageTransport::success(&inputs.input));

        map
      }
    }

    #[must_use]
    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn inputs_list() -> HashMap<String, TypeSignature> {
      let mut map = HashMap::new();
      map.insert("input".to_owned(), TypeSignature::String);
      map
    }
    #[derive(Debug, Default)]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    #[must_use]
    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn outputs_list() -> HashMap<String, TypeSignature> {
      let mut map = HashMap::new();
      map.insert("output".to_owned(), TypeSignature::String);
      map
    }

    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct OutputPortSender {
      port: PortChannel,
    }

    #[cfg(feature = "provider")]
    impl Default for OutputPortSender {
      fn default() -> Self {
        Self {
          port: PortChannel::new("output"),
        }
      }
    }

    #[cfg(feature = "provider")]
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
    #[cfg(feature = "provider")]
    pub fn get_outputs() -> (OutputPorts, TransportStream) {
      let mut outputs = OutputPorts::default();
      let mut ports = vec![&mut outputs.output.port];
      let stream = PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl Outputs {
      pub async fn output(&mut self) -> Result<PortOutput<String>, ProviderError> {
        let packets = self.packets.take("output").await;
        Ok(PortOutput::new("output".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl Outputs {}

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl From<ProviderOutput> for Outputs {
      fn from(packets: ProviderOutput) -> Self {
        Self { packets }
      }
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl From<BoxedTransportStream> for Outputs {
      fn from(stream: BoxedTransportStream) -> Self {
        Self {
          packets: ProviderOutput::new(stream),
        }
      }
    }
  }
  pub mod random_bytes {
    #![allow(unused, unreachable_pub)]
    use std::collections::HashMap;

    use async_trait::async_trait;

    pub use vino_provider::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "random-bytes".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default)]
    pub struct Component {}

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
        let result = tokio::spawn(crate::components::random_bytes::job(
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

    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
      Ok(Inputs {
        seed: payload.consume("seed")?,
        length: payload.consume("length")?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "seed")]
      pub seed: u64,
      #[serde(rename = "length")]
      pub length: u32,
    }

    #[cfg(any(feature = "native", feature = "wasm"))]
    impl From<Inputs> for TransportMap {
      fn from(inputs: Inputs) -> TransportMap {
        let mut map = TransportMap::new();
        map.insert("seed".to_owned(), MessageTransport::success(&inputs.seed));

        map.insert(
          "length".to_owned(),
          MessageTransport::success(&inputs.length),
        );

        map
      }
    }

    #[must_use]
    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn inputs_list() -> HashMap<String, TypeSignature> {
      let mut map = HashMap::new();
      map.insert("seed".to_owned(), TypeSignature::U64);
      map.insert("length".to_owned(), TypeSignature::U32);
      map
    }
    #[derive(Debug, Default)]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    #[must_use]
    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn outputs_list() -> HashMap<String, TypeSignature> {
      let mut map = HashMap::new();
      map.insert("output".to_owned(), TypeSignature::Bytes);
      map
    }

    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct OutputPortSender {
      port: PortChannel,
    }

    #[cfg(feature = "provider")]
    impl Default for OutputPortSender {
      fn default() -> Self {
        Self {
          port: PortChannel::new("output"),
        }
      }
    }

    #[cfg(feature = "provider")]
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
    #[cfg(feature = "provider")]
    pub fn get_outputs() -> (OutputPorts, TransportStream) {
      let mut outputs = OutputPorts::default();
      let mut ports = vec![&mut outputs.output.port];
      let stream = PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl Outputs {
      pub async fn output(&mut self) -> Result<PortOutput<Vec<u8>>, ProviderError> {
        let packets = self.packets.take("output").await;
        Ok(PortOutput::new("output".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl Outputs {}

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl From<ProviderOutput> for Outputs {
      fn from(packets: ProviderOutput) -> Self {
        Self { packets }
      }
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl From<BoxedTransportStream> for Outputs {
      fn from(stream: BoxedTransportStream) -> Self {
        Self {
          packets: ProviderOutput::new(stream),
        }
      }
    }
  }
  pub mod random_string {
    #![allow(unused, unreachable_pub)]
    use std::collections::HashMap;

    use async_trait::async_trait;

    pub use vino_provider::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "random-string".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default)]
    pub struct Component {}

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
        let result = tokio::spawn(crate::components::random_string::job(
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

    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
      Ok(Inputs {
        seed: payload.consume("seed")?,
        length: payload.consume("length")?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "seed")]
      pub seed: u64,
      #[serde(rename = "length")]
      pub length: u32,
    }

    #[cfg(any(feature = "native", feature = "wasm"))]
    impl From<Inputs> for TransportMap {
      fn from(inputs: Inputs) -> TransportMap {
        let mut map = TransportMap::new();
        map.insert("seed".to_owned(), MessageTransport::success(&inputs.seed));

        map.insert(
          "length".to_owned(),
          MessageTransport::success(&inputs.length),
        );

        map
      }
    }

    #[must_use]
    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn inputs_list() -> HashMap<String, TypeSignature> {
      let mut map = HashMap::new();
      map.insert("seed".to_owned(), TypeSignature::U64);
      map.insert("length".to_owned(), TypeSignature::U32);
      map
    }
    #[derive(Debug, Default)]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    #[must_use]
    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn outputs_list() -> HashMap<String, TypeSignature> {
      let mut map = HashMap::new();
      map.insert("output".to_owned(), TypeSignature::String);
      map
    }

    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct OutputPortSender {
      port: PortChannel,
    }

    #[cfg(feature = "provider")]
    impl Default for OutputPortSender {
      fn default() -> Self {
        Self {
          port: PortChannel::new("output"),
        }
      }
    }

    #[cfg(feature = "provider")]
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
    #[cfg(feature = "provider")]
    pub fn get_outputs() -> (OutputPorts, TransportStream) {
      let mut outputs = OutputPorts::default();
      let mut ports = vec![&mut outputs.output.port];
      let stream = PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl Outputs {
      pub async fn output(&mut self) -> Result<PortOutput<String>, ProviderError> {
        let packets = self.packets.take("output").await;
        Ok(PortOutput::new("output".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl Outputs {}

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl From<ProviderOutput> for Outputs {
      fn from(packets: ProviderOutput) -> Self {
        Self { packets }
      }
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl From<BoxedTransportStream> for Outputs {
      fn from(stream: BoxedTransportStream) -> Self {
        Self {
          packets: ProviderOutput::new(stream),
        }
      }
    }
  }
  pub mod uuid {
    #![allow(unused, unreachable_pub)]
    use std::collections::HashMap;

    use async_trait::async_trait;

    pub use vino_provider::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "uuid".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default)]
    pub struct Component {}

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

    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
      Ok(Inputs {
        seed: payload.consume("seed")?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "seed")]
      pub seed: u64,
    }

    #[cfg(any(feature = "native", feature = "wasm"))]
    impl From<Inputs> for TransportMap {
      fn from(inputs: Inputs) -> TransportMap {
        let mut map = TransportMap::new();
        map.insert("seed".to_owned(), MessageTransport::success(&inputs.seed));

        map
      }
    }

    #[must_use]
    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn inputs_list() -> HashMap<String, TypeSignature> {
      let mut map = HashMap::new();
      map.insert("seed".to_owned(), TypeSignature::U64);
      map
    }
    #[derive(Debug, Default)]
    #[cfg(feature = "provider")]
    pub struct OutputPorts {
      pub output: OutputPortSender,
    }

    #[must_use]
    #[cfg(any(feature = "native", feature = "wasm"))]
    pub fn outputs_list() -> HashMap<String, TypeSignature> {
      let mut map = HashMap::new();
      map.insert("output".to_owned(), TypeSignature::String);
      map
    }

    #[derive(Debug)]
    #[cfg(feature = "provider")]
    pub struct OutputPortSender {
      port: PortChannel,
    }

    #[cfg(feature = "provider")]
    impl Default for OutputPortSender {
      fn default() -> Self {
        Self {
          port: PortChannel::new("output"),
        }
      }
    }

    #[cfg(feature = "provider")]
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
    #[cfg(feature = "provider")]
    pub fn get_outputs() -> (OutputPorts, TransportStream) {
      let mut outputs = OutputPorts::default();
      let mut ports = vec![&mut outputs.output.port];
      let stream = PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ProviderOutput,
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl Outputs {
      pub async fn output(&mut self) -> Result<PortOutput<String>, ProviderError> {
        let packets = self.packets.take("output").await;
        Ok(PortOutput::new("output".to_owned(), packets))
      }
    }

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl Outputs {}

    #[cfg(all(feature = "wasm", feature = "guest"))]
    impl From<ProviderOutput> for Outputs {
      fn from(packets: ProviderOutput) -> Self {
        Self { packets }
      }
    }

    #[cfg(all(feature = "native", feature = "guest"))]
    impl From<BoxedTransportStream> for Outputs {
      fn from(stream: BoxedTransportStream) -> Self {
        Self {
          packets: ProviderOutput::new(stream),
        }
      }
    }
  }
}
