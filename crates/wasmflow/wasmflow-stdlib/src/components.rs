/**********************************************
***** This file is generated, do not edit *****
***********************************************/
#![allow(
  unused_qualifications,
  unused_imports,
  missing_copy_implementations,
  unused_qualifications
)]

use wasmflow_sdk::v1::stateful::BatchedJobExecutor;

#[cfg(all(target_arch = "wasm32"))]
type CallResult = wasmflow_sdk::v1::BoxedFuture<Result<Vec<u8>, wasmflow_sdk::v1::BoxedError>>;

#[cfg(all(target_arch = "wasm32"))]
#[allow(unsafe_code)]
#[no_mangle]
pub(crate) extern "C" fn wapc_init() {
  wasmflow_sdk::v1::wasm::runtime::register_dispatcher(Box::new(ComponentDispatcher::default()));
}

pub mod core {

  pub mod error; // core::error
  pub mod log; // core::log
  pub mod panic; // core::panic
}
pub mod math {

  pub mod add; // math::add
  pub mod subtract; // math::subtract
}
pub mod rand {

  pub mod bytes; // rand::bytes
  pub mod string; // rand::string
  pub mod uuid; // rand::uuid
}
pub mod string {

  pub mod concat; // string::concat
}
pub mod __batch__;

#[allow(unused)]
static ALL_COMPONENTS: &[&str] = &[
  "core::error",
  "core::log",
  "core::panic",
  "math::add",
  "math::subtract",
  "rand::bytes",
  "rand::string",
  "rand::uuid",
  "string::concat",
];

#[derive(Default, Copy, Clone)]
#[allow(missing_debug_implementations)]
pub struct ComponentDispatcher {}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::too_many_lines)]
impl wasmflow_sdk::v1::stateful::WasmDispatcher for ComponentDispatcher {
  type Context = crate::Context;
  fn dispatch(&self, op: &'static str, payload: &'static [u8], context: Self::Context) -> CallResult {
    Box::pin(async move {
      let (mut stream, id) = match op {
        "core::error" => {
          crate::components::generated::core::error::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "core::log" => {
          crate::components::generated::core::log::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "core::panic" => {
          crate::components::generated::core::panic::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "math::add" => {
          crate::components::generated::math::add::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "math::subtract" => {
          crate::components::generated::math::subtract::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "rand::bytes" => {
          crate::components::generated::rand::bytes::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "rand::string" => {
          crate::components::generated::rand::string::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "rand::uuid" => {
          crate::components::generated::rand::uuid::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "string::concat" => {
          crate::components::generated::string::concat::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        _ => Err(wasmflow_sdk::v1::error::Error::ComponentNotFound(op.to_owned(), ALL_COMPONENTS.join(", ")).into()),
      }?;
      while let Some(next) = wasmflow_sdk::v1::StreamExt::next(&mut stream).await {
        wasmflow_sdk::v1::wasm::port_send(&next.port, id, next.payload)?;
      }

      Ok(Vec::new())
    })
  }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_lines)]
impl wasmflow_sdk::v1::stateful::NativeDispatcher for ComponentDispatcher {
  type Context = crate::Context;
  fn dispatch(
    &self,
    invocation: wasmflow_sdk::v1::Invocation,
    context: Self::Context,
  ) -> wasmflow_sdk::v1::BoxedFuture<Result<wasmflow_sdk::v1::PacketStream, wasmflow_sdk::v1::BoxedError>> {
    Box::pin(async move {
      let (stream, _id) = match invocation.target.name() {
        "core::error" => {
          crate::components::generated::core::error::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "core::log" => {
          crate::components::generated::core::log::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "core::panic" => {
          crate::components::generated::core::panic::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "math::add" => {
          crate::components::generated::math::add::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "math::subtract" => {
          crate::components::generated::math::subtract::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "rand::bytes" => {
          crate::components::generated::rand::bytes::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "rand::string" => {
          crate::components::generated::rand::string::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "rand::uuid" => {
          crate::components::generated::rand::uuid::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "string::concat" => {
          crate::components::generated::string::concat::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "__batch__" => {
          crate::components::generated::__batch__::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        op => Err(format!("Component not found on this collection: {}", op).into()),
      }?;
      Ok(stream)
    })
  }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_signature() -> wasmflow_sdk::v1::types::CollectionSignature {
  let mut components: std::collections::HashMap<String, wasmflow_sdk::v1::types::ComponentSignature> =
    std::collections::HashMap::new();

  components.insert("core::error".to_owned(), generated::core::error::signature());
  components.insert("core::log".to_owned(), generated::core::log::signature());
  components.insert("core::panic".to_owned(), generated::core::panic::signature());
  components.insert("math::add".to_owned(), generated::math::add::signature());
  components.insert("math::subtract".to_owned(), generated::math::subtract::signature());
  components.insert("rand::bytes".to_owned(), generated::rand::bytes::signature());
  components.insert("rand::string".to_owned(), generated::rand::string::signature());
  components.insert("rand::uuid".to_owned(), generated::rand::uuid::signature());
  components.insert("string::concat".to_owned(), generated::string::concat::signature());

  wasmflow_sdk::v1::types::CollectionSignature {
    name: Some("wasmflow-stdlib".to_owned()),
    features: wasmflow_sdk::v1::types::CollectionFeatures {
      streaming: false,
      stateful: true,
      version: wasmflow_sdk::v1::types::CollectionVersion::V0,
    },
    format: 1,
    version: "0.1.0".to_owned(),
    types: std::collections::HashMap::from([]).into(),
    components: components.into(),
    wellknown: Vec::new(),
    config: wasmflow_sdk::v1::types::TypeMap::new(),
  }
}

pub mod types {
  // no additional types
}
pub mod generated {

  // start namespace core
  pub mod core {

    // start component core::error
    pub mod error {
      // The user-facing implementation for State and job impl.
      pub use wasmflow_sdk::v1::packet::v1::Packet;
      pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

      // The generated definition of inputs, outputs, config, et al.
      use super::error as definition;
      // The generated integration code between the definition and the implementation.
      use super::error as integration;
      use crate::components::core::error as implementation;

      #[derive(Default, Clone, Copy)]
      #[allow(missing_debug_implementations)]
      pub struct Component {}

      impl wasmflow_sdk::v1::Component for Component {
        type Inputs = definition::Inputs;
        type Outputs = definition::OutputPorts;
        type Config = integration::Config;
      }

      impl wasmflow_sdk::v1::stateful::BatchedJobExecutor for Component {
        #[cfg(not(target_arch = "wasm32"))]
        type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
        #[cfg(target_arch = "wasm32")]
        type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
        type State = implementation::State;
        type Config = Config;
        type Return = (wasmflow_sdk::v1::PacketStream, u32);
        type Context = crate::Context;

        fn execute(
          &self,
          payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config, Self::State>,
          context: Self::Context,
        ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
          Box::pin(async move {
            use wasmflow_sdk::v1::stateful::BatchedComponent;
            let id = payload.id();
            let (outputs, mut stream) = definition::get_outputs(id);
            let (payload, config, state) = payload.into_parts();
            let inputs = definition::convert_inputs(payload)?;

            let new_state = Component::job(inputs, outputs, context, state, config).await?;
            stream.push(wasmflow_sdk::v1::packet::PacketWrapper::state(
              Packet::success(&new_state).into(),
            ));
            Ok((stream, id))
          })
        }
      }

      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn signature() -> wasmflow_sdk::v1::types::ComponentSignature {
        wasmflow_sdk::v1::types::ComponentSignature {
          name: "core::error".to_owned(),
          inputs: inputs_list().into(),
          outputs: outputs_list().into(),
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      pub fn convert_inputs(
        mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          input: payload
            .remove("input")
            .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("input".to_owned()))?
            .deserialize()?,
        })
      }

      #[cfg(target_arch = "wasm32")]
      pub fn convert_inputs(
        payload: wasmflow_sdk::v1::wasm::EncodedMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          input: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("input")?)?,
        })
      }

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "input")]
        pub input: String,
      }

      impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
        fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
          let mut map = std::collections::HashMap::default();
          map.insert(
            "input".to_owned(),
            wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.input).into(),
          );
          wasmflow_sdk::v1::packet::PacketMap::new(map)
        }
      }

      #[must_use]
      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("input".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
        map
      }

      // A list of ports and their type signatures.
      #[must_use]
      #[cfg(feature = "host")]
      pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("output".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
        map
      }

      // A list of output ports and their associated stream sender implementations.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPorts {
        pub output: OutputPortSender,
      }

      impl OutputPorts {
        fn new(id: u32) -> Self {
          Self {
            output: OutputPortSender::new(id),
          }
        }
      }

      // Definition and implementation of each port's sender.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPortSender {
        port: wasmflow_sdk::v1::PortChannel,
        id: u32,
      }

      #[cfg(feature = "host")]
      impl OutputPortSender {
        fn new(id: u32) -> Self {
          Self {
            id,
            port: wasmflow_sdk::v1::PortChannel::new("output"),
          }
        }
      }

      #[cfg(all(feature = "host"))]
      impl wasmflow_sdk::v1::Writable for OutputPortSender {
        type PayloadType = String;

        fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
          if self.port.is_closed() {
            Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
          } else {
            Ok(&self.port)
          }
        }

        fn get_port_name(&self) -> &str {
          &self.port.name
        }

        fn get_id(&self) -> u32 {
          self.id
        }
      }

      #[cfg(all(feature = "host"))]
      pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
        let mut outputs = OutputPorts::new(id);
        let mut ports = vec![&mut outputs.output.port];
        let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
        (outputs, stream)
      }

      #[allow(missing_debug_implementations)]
      pub struct Outputs {
        packets: ComponentOutput,
      }

      impl Outputs {
        pub async fn output(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<String>, wasmflow_sdk::v1::error::Error> {
          let packets = self.packets.drain_port("output").await?;
          Ok(wasmflow_sdk::v1::PortOutput::new("output".to_owned(), packets))
        }
      }

      impl From<ComponentOutput> for Outputs {
        fn from(packets: ComponentOutput) -> Self {
          Self { packets }
        }
      }

      impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
          Self {
            packets: ComponentOutput::new(stream),
          }
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
          Self {
            packets: ComponentOutput::new_from_ts(stream),
          }
        }
      }

      #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Config {}
    }
    // end component core::error
    // start component core::log
    pub mod log {
      // The user-facing implementation for State and job impl.
      pub use wasmflow_sdk::v1::packet::v1::Packet;
      pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

      // The generated definition of inputs, outputs, config, et al.
      use super::log as definition;
      // The generated integration code between the definition and the implementation.
      use super::log as integration;
      use crate::components::core::log as implementation;

      #[derive(Default, Clone, Copy)]
      #[allow(missing_debug_implementations)]
      pub struct Component {}

      impl wasmflow_sdk::v1::Component for Component {
        type Inputs = definition::Inputs;
        type Outputs = definition::OutputPorts;
        type Config = integration::Config;
      }

      impl wasmflow_sdk::v1::stateful::BatchedJobExecutor for Component {
        #[cfg(not(target_arch = "wasm32"))]
        type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
        #[cfg(target_arch = "wasm32")]
        type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
        type State = implementation::State;
        type Config = Config;
        type Return = (wasmflow_sdk::v1::PacketStream, u32);
        type Context = crate::Context;

        fn execute(
          &self,
          payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config, Self::State>,
          context: Self::Context,
        ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
          Box::pin(async move {
            use wasmflow_sdk::v1::stateful::BatchedComponent;
            let id = payload.id();
            let (outputs, mut stream) = definition::get_outputs(id);
            let (payload, config, state) = payload.into_parts();
            let inputs = definition::convert_inputs(payload)?;

            let new_state = Component::job(inputs, outputs, context, state, config).await?;
            stream.push(wasmflow_sdk::v1::packet::PacketWrapper::state(
              Packet::success(&new_state).into(),
            ));
            Ok((stream, id))
          })
        }
      }

      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn signature() -> wasmflow_sdk::v1::types::ComponentSignature {
        wasmflow_sdk::v1::types::ComponentSignature {
          name: "core::log".to_owned(),
          inputs: inputs_list().into(),
          outputs: outputs_list().into(),
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      pub fn convert_inputs(
        mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          input: payload
            .remove("input")
            .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("input".to_owned()))?
            .deserialize()?,
        })
      }

      #[cfg(target_arch = "wasm32")]
      pub fn convert_inputs(
        payload: wasmflow_sdk::v1::wasm::EncodedMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          input: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("input")?)?,
        })
      }

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "input")]
        pub input: String,
      }

      impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
        fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
          let mut map = std::collections::HashMap::default();
          map.insert(
            "input".to_owned(),
            wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.input).into(),
          );
          wasmflow_sdk::v1::packet::PacketMap::new(map)
        }
      }

      #[must_use]
      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("input".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
        map
      }

      // A list of ports and their type signatures.
      #[must_use]
      #[cfg(feature = "host")]
      pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("output".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
        map
      }

      // A list of output ports and their associated stream sender implementations.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPorts {
        pub output: OutputPortSender,
      }

      impl OutputPorts {
        fn new(id: u32) -> Self {
          Self {
            output: OutputPortSender::new(id),
          }
        }
      }

      // Definition and implementation of each port's sender.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPortSender {
        port: wasmflow_sdk::v1::PortChannel,
        id: u32,
      }

      #[cfg(feature = "host")]
      impl OutputPortSender {
        fn new(id: u32) -> Self {
          Self {
            id,
            port: wasmflow_sdk::v1::PortChannel::new("output"),
          }
        }
      }

      #[cfg(all(feature = "host"))]
      impl wasmflow_sdk::v1::Writable for OutputPortSender {
        type PayloadType = String;

        fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
          if self.port.is_closed() {
            Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
          } else {
            Ok(&self.port)
          }
        }

        fn get_port_name(&self) -> &str {
          &self.port.name
        }

        fn get_id(&self) -> u32 {
          self.id
        }
      }

      #[cfg(all(feature = "host"))]
      pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
        let mut outputs = OutputPorts::new(id);
        let mut ports = vec![&mut outputs.output.port];
        let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
        (outputs, stream)
      }

      #[allow(missing_debug_implementations)]
      pub struct Outputs {
        packets: ComponentOutput,
      }

      impl Outputs {
        pub async fn output(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<String>, wasmflow_sdk::v1::error::Error> {
          let packets = self.packets.drain_port("output").await?;
          Ok(wasmflow_sdk::v1::PortOutput::new("output".to_owned(), packets))
        }
      }

      impl From<ComponentOutput> for Outputs {
        fn from(packets: ComponentOutput) -> Self {
          Self { packets }
        }
      }

      impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
          Self {
            packets: ComponentOutput::new(stream),
          }
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
          Self {
            packets: ComponentOutput::new_from_ts(stream),
          }
        }
      }

      #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Config {}
    }
    // end component core::log
    // start component core::panic
    pub mod panic {
      // The user-facing implementation for State and job impl.
      pub use wasmflow_sdk::v1::packet::v1::Packet;
      pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

      // The generated definition of inputs, outputs, config, et al.
      use super::panic as definition;
      // The generated integration code between the definition and the implementation.
      use super::panic as integration;
      use crate::components::core::panic as implementation;

      #[derive(Default, Clone, Copy)]
      #[allow(missing_debug_implementations)]
      pub struct Component {}

      impl wasmflow_sdk::v1::Component for Component {
        type Inputs = definition::Inputs;
        type Outputs = definition::OutputPorts;
        type Config = integration::Config;
      }

      impl wasmflow_sdk::v1::stateful::BatchedJobExecutor for Component {
        #[cfg(not(target_arch = "wasm32"))]
        type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
        #[cfg(target_arch = "wasm32")]
        type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
        type State = implementation::State;
        type Config = Config;
        type Return = (wasmflow_sdk::v1::PacketStream, u32);
        type Context = crate::Context;

        fn execute(
          &self,
          payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config, Self::State>,
          context: Self::Context,
        ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
          Box::pin(async move {
            use wasmflow_sdk::v1::stateful::BatchedComponent;
            let id = payload.id();
            let (outputs, mut stream) = definition::get_outputs(id);
            let (payload, config, state) = payload.into_parts();
            let inputs = definition::convert_inputs(payload)?;

            let new_state = Component::job(inputs, outputs, context, state, config).await?;
            stream.push(wasmflow_sdk::v1::packet::PacketWrapper::state(
              Packet::success(&new_state).into(),
            ));
            Ok((stream, id))
          })
        }
      }

      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn signature() -> wasmflow_sdk::v1::types::ComponentSignature {
        wasmflow_sdk::v1::types::ComponentSignature {
          name: "core::panic".to_owned(),
          inputs: inputs_list().into(),
          outputs: outputs_list().into(),
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      pub fn convert_inputs(
        mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          input: payload
            .remove("input")
            .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("input".to_owned()))?
            .deserialize()?,
        })
      }

      #[cfg(target_arch = "wasm32")]
      pub fn convert_inputs(
        payload: wasmflow_sdk::v1::wasm::EncodedMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          input: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("input")?)?,
        })
      }

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "input")]
        pub input: String,
      }

      impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
        fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
          let mut map = std::collections::HashMap::default();
          map.insert(
            "input".to_owned(),
            wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.input).into(),
          );
          wasmflow_sdk::v1::packet::PacketMap::new(map)
        }
      }

      #[must_use]
      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("input".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
        map
      }

      // A list of ports and their type signatures.
      #[must_use]
      #[cfg(feature = "host")]
      pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("output".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
        map
      }

      // A list of output ports and their associated stream sender implementations.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPorts {
        pub output: OutputPortSender,
      }

      impl OutputPorts {
        fn new(id: u32) -> Self {
          Self {
            output: OutputPortSender::new(id),
          }
        }
      }

      // Definition and implementation of each port's sender.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPortSender {
        port: wasmflow_sdk::v1::PortChannel,
        id: u32,
      }

      #[cfg(feature = "host")]
      impl OutputPortSender {
        fn new(id: u32) -> Self {
          Self {
            id,
            port: wasmflow_sdk::v1::PortChannel::new("output"),
          }
        }
      }

      #[cfg(all(feature = "host"))]
      impl wasmflow_sdk::v1::Writable for OutputPortSender {
        type PayloadType = String;

        fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
          if self.port.is_closed() {
            Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
          } else {
            Ok(&self.port)
          }
        }

        fn get_port_name(&self) -> &str {
          &self.port.name
        }

        fn get_id(&self) -> u32 {
          self.id
        }
      }

      #[cfg(all(feature = "host"))]
      pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
        let mut outputs = OutputPorts::new(id);
        let mut ports = vec![&mut outputs.output.port];
        let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
        (outputs, stream)
      }

      #[allow(missing_debug_implementations)]
      pub struct Outputs {
        packets: ComponentOutput,
      }

      impl Outputs {
        pub async fn output(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<String>, wasmflow_sdk::v1::error::Error> {
          let packets = self.packets.drain_port("output").await?;
          Ok(wasmflow_sdk::v1::PortOutput::new("output".to_owned(), packets))
        }
      }

      impl From<ComponentOutput> for Outputs {
        fn from(packets: ComponentOutput) -> Self {
          Self { packets }
        }
      }

      impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
          Self {
            packets: ComponentOutput::new(stream),
          }
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
          Self {
            packets: ComponentOutput::new_from_ts(stream),
          }
        }
      }

      #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Config {}
    }
    // end component core::panic
  }
  // end namespace core

  // start namespace math
  pub mod math {

    // start component math::add
    pub mod add {
      // The user-facing implementation for State and job impl.
      pub use wasmflow_sdk::v1::packet::v1::Packet;
      pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

      // The generated definition of inputs, outputs, config, et al.
      use super::add as definition;
      // The generated integration code between the definition and the implementation.
      use super::add as integration;
      use crate::components::math::add as implementation;

      #[derive(Default, Clone, Copy)]
      #[allow(missing_debug_implementations)]
      pub struct Component {}

      impl wasmflow_sdk::v1::Component for Component {
        type Inputs = definition::Inputs;
        type Outputs = definition::OutputPorts;
        type Config = integration::Config;
      }

      impl wasmflow_sdk::v1::stateful::BatchedJobExecutor for Component {
        #[cfg(not(target_arch = "wasm32"))]
        type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
        #[cfg(target_arch = "wasm32")]
        type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
        type State = implementation::State;
        type Config = Config;
        type Return = (wasmflow_sdk::v1::PacketStream, u32);
        type Context = crate::Context;

        fn execute(
          &self,
          payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config, Self::State>,
          context: Self::Context,
        ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
          Box::pin(async move {
            use wasmflow_sdk::v1::stateful::BatchedComponent;
            let id = payload.id();
            let (outputs, mut stream) = definition::get_outputs(id);
            let (payload, config, state) = payload.into_parts();
            let inputs = definition::convert_inputs(payload)?;

            let new_state = Component::job(inputs, outputs, context, state, config).await?;
            stream.push(wasmflow_sdk::v1::packet::PacketWrapper::state(
              Packet::success(&new_state).into(),
            ));
            Ok((stream, id))
          })
        }
      }

      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn signature() -> wasmflow_sdk::v1::types::ComponentSignature {
        wasmflow_sdk::v1::types::ComponentSignature {
          name: "math::add".to_owned(),
          inputs: inputs_list().into(),
          outputs: outputs_list().into(),
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      pub fn convert_inputs(
        mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          left: payload
            .remove("left")
            .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("left".to_owned()))?
            .deserialize()?,
          right: payload
            .remove("right")
            .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("right".to_owned()))?
            .deserialize()?,
        })
      }

      #[cfg(target_arch = "wasm32")]
      pub fn convert_inputs(
        payload: wasmflow_sdk::v1::wasm::EncodedMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          left: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("left")?)?,
          right: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("right")?)?,
        })
      }

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "left")]
        pub left: u64,
        #[serde(rename = "right")]
        pub right: u64,
      }

      impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
        fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
          let mut map = std::collections::HashMap::default();
          map.insert(
            "left".to_owned(),
            wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.left).into(),
          );
          map.insert(
            "right".to_owned(),
            wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.right).into(),
          );
          wasmflow_sdk::v1::packet::PacketMap::new(map)
        }
      }

      #[must_use]
      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("left".to_owned(), wasmflow_sdk::v1::types::TypeSignature::U64);
        map.insert("right".to_owned(), wasmflow_sdk::v1::types::TypeSignature::U64);
        map
      }

      // A list of ports and their type signatures.
      #[must_use]
      #[cfg(feature = "host")]
      pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("output".to_owned(), wasmflow_sdk::v1::types::TypeSignature::U64);
        map
      }

      // A list of output ports and their associated stream sender implementations.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPorts {
        pub output: OutputPortSender,
      }

      impl OutputPorts {
        fn new(id: u32) -> Self {
          Self {
            output: OutputPortSender::new(id),
          }
        }
      }

      // Definition and implementation of each port's sender.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPortSender {
        port: wasmflow_sdk::v1::PortChannel,
        id: u32,
      }

      #[cfg(feature = "host")]
      impl OutputPortSender {
        fn new(id: u32) -> Self {
          Self {
            id,
            port: wasmflow_sdk::v1::PortChannel::new("output"),
          }
        }
      }

      #[cfg(all(feature = "host"))]
      impl wasmflow_sdk::v1::Writable for OutputPortSender {
        type PayloadType = u64;

        fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
          if self.port.is_closed() {
            Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
          } else {
            Ok(&self.port)
          }
        }

        fn get_port_name(&self) -> &str {
          &self.port.name
        }

        fn get_id(&self) -> u32 {
          self.id
        }
      }

      #[cfg(all(feature = "host"))]
      pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
        let mut outputs = OutputPorts::new(id);
        let mut ports = vec![&mut outputs.output.port];
        let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
        (outputs, stream)
      }

      #[allow(missing_debug_implementations)]
      pub struct Outputs {
        packets: ComponentOutput,
      }

      impl Outputs {
        pub async fn output(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<u64>, wasmflow_sdk::v1::error::Error> {
          let packets = self.packets.drain_port("output").await?;
          Ok(wasmflow_sdk::v1::PortOutput::new("output".to_owned(), packets))
        }
      }

      impl From<ComponentOutput> for Outputs {
        fn from(packets: ComponentOutput) -> Self {
          Self { packets }
        }
      }

      impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
          Self {
            packets: ComponentOutput::new(stream),
          }
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
          Self {
            packets: ComponentOutput::new_from_ts(stream),
          }
        }
      }

      #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Config {}
    }
    // end component math::add
    // start component math::subtract
    pub mod subtract {
      // The user-facing implementation for State and job impl.
      pub use wasmflow_sdk::v1::packet::v1::Packet;
      pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

      // The generated definition of inputs, outputs, config, et al.
      use super::subtract as definition;
      // The generated integration code between the definition and the implementation.
      use super::subtract as integration;
      use crate::components::math::subtract as implementation;

      #[derive(Default, Clone, Copy)]
      #[allow(missing_debug_implementations)]
      pub struct Component {}

      impl wasmflow_sdk::v1::Component for Component {
        type Inputs = definition::Inputs;
        type Outputs = definition::OutputPorts;
        type Config = integration::Config;
      }

      impl wasmflow_sdk::v1::stateful::BatchedJobExecutor for Component {
        #[cfg(not(target_arch = "wasm32"))]
        type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
        #[cfg(target_arch = "wasm32")]
        type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
        type State = implementation::State;
        type Config = Config;
        type Return = (wasmflow_sdk::v1::PacketStream, u32);
        type Context = crate::Context;

        fn execute(
          &self,
          payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config, Self::State>,
          context: Self::Context,
        ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
          Box::pin(async move {
            use wasmflow_sdk::v1::stateful::BatchedComponent;
            let id = payload.id();
            let (outputs, mut stream) = definition::get_outputs(id);
            let (payload, config, state) = payload.into_parts();
            let inputs = definition::convert_inputs(payload)?;

            let new_state = Component::job(inputs, outputs, context, state, config).await?;
            stream.push(wasmflow_sdk::v1::packet::PacketWrapper::state(
              Packet::success(&new_state).into(),
            ));
            Ok((stream, id))
          })
        }
      }

      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn signature() -> wasmflow_sdk::v1::types::ComponentSignature {
        wasmflow_sdk::v1::types::ComponentSignature {
          name: "math::subtract".to_owned(),
          inputs: inputs_list().into(),
          outputs: outputs_list().into(),
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      pub fn convert_inputs(
        mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          left: payload
            .remove("left")
            .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("left".to_owned()))?
            .deserialize()?,
          right: payload
            .remove("right")
            .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("right".to_owned()))?
            .deserialize()?,
        })
      }

      #[cfg(target_arch = "wasm32")]
      pub fn convert_inputs(
        payload: wasmflow_sdk::v1::wasm::EncodedMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          left: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("left")?)?,
          right: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("right")?)?,
        })
      }

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "left")]
        pub left: u64,
        #[serde(rename = "right")]
        pub right: u64,
      }

      impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
        fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
          let mut map = std::collections::HashMap::default();
          map.insert(
            "left".to_owned(),
            wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.left).into(),
          );
          map.insert(
            "right".to_owned(),
            wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.right).into(),
          );
          wasmflow_sdk::v1::packet::PacketMap::new(map)
        }
      }

      #[must_use]
      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("left".to_owned(), wasmflow_sdk::v1::types::TypeSignature::U64);
        map.insert("right".to_owned(), wasmflow_sdk::v1::types::TypeSignature::U64);
        map
      }

      // A list of ports and their type signatures.
      #[must_use]
      #[cfg(feature = "host")]
      pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("output".to_owned(), wasmflow_sdk::v1::types::TypeSignature::U64);
        map
      }

      // A list of output ports and their associated stream sender implementations.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPorts {
        pub output: OutputPortSender,
      }

      impl OutputPorts {
        fn new(id: u32) -> Self {
          Self {
            output: OutputPortSender::new(id),
          }
        }
      }

      // Definition and implementation of each port's sender.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPortSender {
        port: wasmflow_sdk::v1::PortChannel,
        id: u32,
      }

      #[cfg(feature = "host")]
      impl OutputPortSender {
        fn new(id: u32) -> Self {
          Self {
            id,
            port: wasmflow_sdk::v1::PortChannel::new("output"),
          }
        }
      }

      #[cfg(all(feature = "host"))]
      impl wasmflow_sdk::v1::Writable for OutputPortSender {
        type PayloadType = u64;

        fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
          if self.port.is_closed() {
            Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
          } else {
            Ok(&self.port)
          }
        }

        fn get_port_name(&self) -> &str {
          &self.port.name
        }

        fn get_id(&self) -> u32 {
          self.id
        }
      }

      #[cfg(all(feature = "host"))]
      pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
        let mut outputs = OutputPorts::new(id);
        let mut ports = vec![&mut outputs.output.port];
        let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
        (outputs, stream)
      }

      #[allow(missing_debug_implementations)]
      pub struct Outputs {
        packets: ComponentOutput,
      }

      impl Outputs {
        pub async fn output(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<u64>, wasmflow_sdk::v1::error::Error> {
          let packets = self.packets.drain_port("output").await?;
          Ok(wasmflow_sdk::v1::PortOutput::new("output".to_owned(), packets))
        }
      }

      impl From<ComponentOutput> for Outputs {
        fn from(packets: ComponentOutput) -> Self {
          Self { packets }
        }
      }

      impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
          Self {
            packets: ComponentOutput::new(stream),
          }
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
          Self {
            packets: ComponentOutput::new_from_ts(stream),
          }
        }
      }

      #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Config {}
    }
    // end component math::subtract
  }
  // end namespace math

  // start namespace rand
  pub mod rand {

    // start component rand::bytes
    pub mod bytes {
      // The user-facing implementation for State and job impl.
      pub use wasmflow_sdk::v1::packet::v1::Packet;
      pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

      // The generated definition of inputs, outputs, config, et al.
      use super::bytes as definition;
      // The generated integration code between the definition and the implementation.
      use super::bytes as integration;
      use crate::components::rand::bytes as implementation;

      #[derive(Default, Clone, Copy)]
      #[allow(missing_debug_implementations)]
      pub struct Component {}

      impl wasmflow_sdk::v1::Component for Component {
        type Inputs = definition::Inputs;
        type Outputs = definition::OutputPorts;
        type Config = integration::Config;
      }

      impl wasmflow_sdk::v1::stateful::BatchedJobExecutor for Component {
        #[cfg(not(target_arch = "wasm32"))]
        type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
        #[cfg(target_arch = "wasm32")]
        type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
        type State = implementation::State;
        type Config = Config;
        type Return = (wasmflow_sdk::v1::PacketStream, u32);
        type Context = crate::Context;

        fn execute(
          &self,
          payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config, Self::State>,
          context: Self::Context,
        ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
          Box::pin(async move {
            use wasmflow_sdk::v1::stateful::BatchedComponent;
            let id = payload.id();
            let (outputs, mut stream) = definition::get_outputs(id);
            let (payload, config, state) = payload.into_parts();
            let inputs = definition::convert_inputs(payload)?;

            let new_state = Component::job(inputs, outputs, context, state, config).await?;
            stream.push(wasmflow_sdk::v1::packet::PacketWrapper::state(
              Packet::success(&new_state).into(),
            ));
            Ok((stream, id))
          })
        }
      }

      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn signature() -> wasmflow_sdk::v1::types::ComponentSignature {
        wasmflow_sdk::v1::types::ComponentSignature {
          name: "rand::bytes".to_owned(),
          inputs: inputs_list().into(),
          outputs: outputs_list().into(),
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      pub fn convert_inputs(
        mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          seed: payload
            .remove("seed")
            .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("seed".to_owned()))?
            .deserialize()?,
          length: payload
            .remove("length")
            .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("length".to_owned()))?
            .deserialize()?,
        })
      }

      #[cfg(target_arch = "wasm32")]
      pub fn convert_inputs(
        payload: wasmflow_sdk::v1::wasm::EncodedMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          seed: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("seed")?)?,
          length: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("length")?)?,
        })
      }

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "seed")]
        pub seed: u64,
        #[serde(rename = "length")]
        pub length: u32,
      }

      impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
        fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
          let mut map = std::collections::HashMap::default();
          map.insert(
            "seed".to_owned(),
            wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.seed).into(),
          );
          map.insert(
            "length".to_owned(),
            wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.length).into(),
          );
          wasmflow_sdk::v1::packet::PacketMap::new(map)
        }
      }

      #[must_use]
      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("seed".to_owned(), wasmflow_sdk::v1::types::TypeSignature::U64);
        map.insert("length".to_owned(), wasmflow_sdk::v1::types::TypeSignature::U32);
        map
      }

      // A list of ports and their type signatures.
      #[must_use]
      #[cfg(feature = "host")]
      pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("output".to_owned(), wasmflow_sdk::v1::types::TypeSignature::Bytes);
        map
      }

      // A list of output ports and their associated stream sender implementations.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPorts {
        pub output: OutputPortSender,
      }

      impl OutputPorts {
        fn new(id: u32) -> Self {
          Self {
            output: OutputPortSender::new(id),
          }
        }
      }

      // Definition and implementation of each port's sender.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPortSender {
        port: wasmflow_sdk::v1::PortChannel,
        id: u32,
      }

      #[cfg(feature = "host")]
      impl OutputPortSender {
        fn new(id: u32) -> Self {
          Self {
            id,
            port: wasmflow_sdk::v1::PortChannel::new("output"),
          }
        }
      }

      #[cfg(all(feature = "host"))]
      impl wasmflow_sdk::v1::Writable for OutputPortSender {
        type PayloadType = Vec<u8>;

        fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
          if self.port.is_closed() {
            Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
          } else {
            Ok(&self.port)
          }
        }

        fn get_port_name(&self) -> &str {
          &self.port.name
        }

        fn get_id(&self) -> u32 {
          self.id
        }
      }

      #[cfg(all(feature = "host"))]
      pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
        let mut outputs = OutputPorts::new(id);
        let mut ports = vec![&mut outputs.output.port];
        let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
        (outputs, stream)
      }

      #[allow(missing_debug_implementations)]
      pub struct Outputs {
        packets: ComponentOutput,
      }

      impl Outputs {
        pub async fn output(
          &mut self,
        ) -> Result<wasmflow_sdk::v1::PortOutput<Vec<u8>>, wasmflow_sdk::v1::error::Error> {
          let packets = self.packets.drain_port("output").await?;
          Ok(wasmflow_sdk::v1::PortOutput::new("output".to_owned(), packets))
        }
      }

      impl From<ComponentOutput> for Outputs {
        fn from(packets: ComponentOutput) -> Self {
          Self { packets }
        }
      }

      impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
          Self {
            packets: ComponentOutput::new(stream),
          }
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
          Self {
            packets: ComponentOutput::new_from_ts(stream),
          }
        }
      }

      #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Config {}
    }
    // end component rand::bytes
    // start component rand::string
    pub mod string {
      // The user-facing implementation for State and job impl.
      pub use wasmflow_sdk::v1::packet::v1::Packet;
      pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

      // The generated definition of inputs, outputs, config, et al.
      use super::string as definition;
      // The generated integration code between the definition and the implementation.
      use super::string as integration;
      use crate::components::rand::string as implementation;

      #[derive(Default, Clone, Copy)]
      #[allow(missing_debug_implementations)]
      pub struct Component {}

      impl wasmflow_sdk::v1::Component for Component {
        type Inputs = definition::Inputs;
        type Outputs = definition::OutputPorts;
        type Config = integration::Config;
      }

      impl wasmflow_sdk::v1::stateful::BatchedJobExecutor for Component {
        #[cfg(not(target_arch = "wasm32"))]
        type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
        #[cfg(target_arch = "wasm32")]
        type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
        type State = implementation::State;
        type Config = Config;
        type Return = (wasmflow_sdk::v1::PacketStream, u32);
        type Context = crate::Context;

        fn execute(
          &self,
          payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config, Self::State>,
          context: Self::Context,
        ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
          Box::pin(async move {
            use wasmflow_sdk::v1::stateful::BatchedComponent;
            let id = payload.id();
            let (outputs, mut stream) = definition::get_outputs(id);
            let (payload, config, state) = payload.into_parts();
            let inputs = definition::convert_inputs(payload)?;

            let new_state = Component::job(inputs, outputs, context, state, config).await?;
            stream.push(wasmflow_sdk::v1::packet::PacketWrapper::state(
              Packet::success(&new_state).into(),
            ));
            Ok((stream, id))
          })
        }
      }

      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn signature() -> wasmflow_sdk::v1::types::ComponentSignature {
        wasmflow_sdk::v1::types::ComponentSignature {
          name: "rand::string".to_owned(),
          inputs: inputs_list().into(),
          outputs: outputs_list().into(),
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      pub fn convert_inputs(
        mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          seed: payload
            .remove("seed")
            .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("seed".to_owned()))?
            .deserialize()?,
          length: payload
            .remove("length")
            .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("length".to_owned()))?
            .deserialize()?,
        })
      }

      #[cfg(target_arch = "wasm32")]
      pub fn convert_inputs(
        payload: wasmflow_sdk::v1::wasm::EncodedMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          seed: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("seed")?)?,
          length: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("length")?)?,
        })
      }

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "seed")]
        pub seed: u64,
        #[serde(rename = "length")]
        pub length: u32,
      }

      impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
        fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
          let mut map = std::collections::HashMap::default();
          map.insert(
            "seed".to_owned(),
            wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.seed).into(),
          );
          map.insert(
            "length".to_owned(),
            wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.length).into(),
          );
          wasmflow_sdk::v1::packet::PacketMap::new(map)
        }
      }

      #[must_use]
      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("seed".to_owned(), wasmflow_sdk::v1::types::TypeSignature::U64);
        map.insert("length".to_owned(), wasmflow_sdk::v1::types::TypeSignature::U32);
        map
      }

      // A list of ports and their type signatures.
      #[must_use]
      #[cfg(feature = "host")]
      pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("output".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
        map
      }

      // A list of output ports and their associated stream sender implementations.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPorts {
        pub output: OutputPortSender,
      }

      impl OutputPorts {
        fn new(id: u32) -> Self {
          Self {
            output: OutputPortSender::new(id),
          }
        }
      }

      // Definition and implementation of each port's sender.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPortSender {
        port: wasmflow_sdk::v1::PortChannel,
        id: u32,
      }

      #[cfg(feature = "host")]
      impl OutputPortSender {
        fn new(id: u32) -> Self {
          Self {
            id,
            port: wasmflow_sdk::v1::PortChannel::new("output"),
          }
        }
      }

      #[cfg(all(feature = "host"))]
      impl wasmflow_sdk::v1::Writable for OutputPortSender {
        type PayloadType = String;

        fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
          if self.port.is_closed() {
            Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
          } else {
            Ok(&self.port)
          }
        }

        fn get_port_name(&self) -> &str {
          &self.port.name
        }

        fn get_id(&self) -> u32 {
          self.id
        }
      }

      #[cfg(all(feature = "host"))]
      pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
        let mut outputs = OutputPorts::new(id);
        let mut ports = vec![&mut outputs.output.port];
        let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
        (outputs, stream)
      }

      #[allow(missing_debug_implementations)]
      pub struct Outputs {
        packets: ComponentOutput,
      }

      impl Outputs {
        pub async fn output(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<String>, wasmflow_sdk::v1::error::Error> {
          let packets = self.packets.drain_port("output").await?;
          Ok(wasmflow_sdk::v1::PortOutput::new("output".to_owned(), packets))
        }
      }

      impl From<ComponentOutput> for Outputs {
        fn from(packets: ComponentOutput) -> Self {
          Self { packets }
        }
      }

      impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
          Self {
            packets: ComponentOutput::new(stream),
          }
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
          Self {
            packets: ComponentOutput::new_from_ts(stream),
          }
        }
      }

      #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Config {}
    }
    // end component rand::string
    // start component rand::uuid
    pub mod uuid {
      // The user-facing implementation for State and job impl.
      pub use wasmflow_sdk::v1::packet::v1::Packet;
      pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

      // The generated definition of inputs, outputs, config, et al.
      use super::uuid as definition;
      // The generated integration code between the definition and the implementation.
      use super::uuid as integration;
      use crate::components::rand::uuid as implementation;

      #[derive(Default, Clone, Copy)]
      #[allow(missing_debug_implementations)]
      pub struct Component {}

      impl wasmflow_sdk::v1::Component for Component {
        type Inputs = definition::Inputs;
        type Outputs = definition::OutputPorts;
        type Config = integration::Config;
      }

      impl wasmflow_sdk::v1::stateful::BatchedJobExecutor for Component {
        #[cfg(not(target_arch = "wasm32"))]
        type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
        #[cfg(target_arch = "wasm32")]
        type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
        type State = implementation::State;
        type Config = Config;
        type Return = (wasmflow_sdk::v1::PacketStream, u32);
        type Context = crate::Context;

        fn execute(
          &self,
          payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config, Self::State>,
          context: Self::Context,
        ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
          Box::pin(async move {
            use wasmflow_sdk::v1::stateful::BatchedComponent;
            let id = payload.id();
            let (outputs, mut stream) = definition::get_outputs(id);
            let (payload, config, state) = payload.into_parts();
            let inputs = definition::convert_inputs(payload)?;

            let new_state = Component::job(inputs, outputs, context, state, config).await?;
            stream.push(wasmflow_sdk::v1::packet::PacketWrapper::state(
              Packet::success(&new_state).into(),
            ));
            Ok((stream, id))
          })
        }
      }

      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn signature() -> wasmflow_sdk::v1::types::ComponentSignature {
        wasmflow_sdk::v1::types::ComponentSignature {
          name: "rand::uuid".to_owned(),
          inputs: inputs_list().into(),
          outputs: outputs_list().into(),
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      pub fn convert_inputs(
        mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          seed: payload
            .remove("seed")
            .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("seed".to_owned()))?
            .deserialize()?,
        })
      }

      #[cfg(target_arch = "wasm32")]
      pub fn convert_inputs(
        payload: wasmflow_sdk::v1::wasm::EncodedMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          seed: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("seed")?)?,
        })
      }

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "seed")]
        pub seed: u64,
      }

      impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
        fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
          let mut map = std::collections::HashMap::default();
          map.insert(
            "seed".to_owned(),
            wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.seed).into(),
          );
          wasmflow_sdk::v1::packet::PacketMap::new(map)
        }
      }

      #[must_use]
      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("seed".to_owned(), wasmflow_sdk::v1::types::TypeSignature::U64);
        map
      }

      // A list of ports and their type signatures.
      #[must_use]
      #[cfg(feature = "host")]
      pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("output".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
        map
      }

      // A list of output ports and their associated stream sender implementations.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPorts {
        pub output: OutputPortSender,
      }

      impl OutputPorts {
        fn new(id: u32) -> Self {
          Self {
            output: OutputPortSender::new(id),
          }
        }
      }

      // Definition and implementation of each port's sender.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPortSender {
        port: wasmflow_sdk::v1::PortChannel,
        id: u32,
      }

      #[cfg(feature = "host")]
      impl OutputPortSender {
        fn new(id: u32) -> Self {
          Self {
            id,
            port: wasmflow_sdk::v1::PortChannel::new("output"),
          }
        }
      }

      #[cfg(all(feature = "host"))]
      impl wasmflow_sdk::v1::Writable for OutputPortSender {
        type PayloadType = String;

        fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
          if self.port.is_closed() {
            Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
          } else {
            Ok(&self.port)
          }
        }

        fn get_port_name(&self) -> &str {
          &self.port.name
        }

        fn get_id(&self) -> u32 {
          self.id
        }
      }

      #[cfg(all(feature = "host"))]
      pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
        let mut outputs = OutputPorts::new(id);
        let mut ports = vec![&mut outputs.output.port];
        let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
        (outputs, stream)
      }

      #[allow(missing_debug_implementations)]
      pub struct Outputs {
        packets: ComponentOutput,
      }

      impl Outputs {
        pub async fn output(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<String>, wasmflow_sdk::v1::error::Error> {
          let packets = self.packets.drain_port("output").await?;
          Ok(wasmflow_sdk::v1::PortOutput::new("output".to_owned(), packets))
        }
      }

      impl From<ComponentOutput> for Outputs {
        fn from(packets: ComponentOutput) -> Self {
          Self { packets }
        }
      }

      impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
          Self {
            packets: ComponentOutput::new(stream),
          }
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
          Self {
            packets: ComponentOutput::new_from_ts(stream),
          }
        }
      }

      #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Config {}
    }
    // end component rand::uuid
  }
  // end namespace rand

  // start namespace string
  pub mod string {

    // start component string::concat
    pub mod concat {
      // The user-facing implementation for State and job impl.
      pub use wasmflow_sdk::v1::packet::v1::Packet;
      pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

      // The generated definition of inputs, outputs, config, et al.
      use super::concat as definition;
      // The generated integration code between the definition and the implementation.
      use super::concat as integration;
      use crate::components::string::concat as implementation;

      #[derive(Default, Clone, Copy)]
      #[allow(missing_debug_implementations)]
      pub struct Component {}

      impl wasmflow_sdk::v1::Component for Component {
        type Inputs = definition::Inputs;
        type Outputs = definition::OutputPorts;
        type Config = integration::Config;
      }

      impl wasmflow_sdk::v1::stateful::BatchedJobExecutor for Component {
        #[cfg(not(target_arch = "wasm32"))]
        type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
        #[cfg(target_arch = "wasm32")]
        type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
        type State = implementation::State;
        type Config = Config;
        type Return = (wasmflow_sdk::v1::PacketStream, u32);
        type Context = crate::Context;

        fn execute(
          &self,
          payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config, Self::State>,
          context: Self::Context,
        ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
          Box::pin(async move {
            use wasmflow_sdk::v1::stateful::BatchedComponent;
            let id = payload.id();
            let (outputs, mut stream) = definition::get_outputs(id);
            let (payload, config, state) = payload.into_parts();
            let inputs = definition::convert_inputs(payload)?;

            let new_state = Component::job(inputs, outputs, context, state, config).await?;
            stream.push(wasmflow_sdk::v1::packet::PacketWrapper::state(
              Packet::success(&new_state).into(),
            ));
            Ok((stream, id))
          })
        }
      }

      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn signature() -> wasmflow_sdk::v1::types::ComponentSignature {
        wasmflow_sdk::v1::types::ComponentSignature {
          name: "string::concat".to_owned(),
          inputs: inputs_list().into(),
          outputs: outputs_list().into(),
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      pub fn convert_inputs(
        mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          left: payload
            .remove("left")
            .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("left".to_owned()))?
            .deserialize()?,
          right: payload
            .remove("right")
            .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("right".to_owned()))?
            .deserialize()?,
        })
      }

      #[cfg(target_arch = "wasm32")]
      pub fn convert_inputs(
        payload: wasmflow_sdk::v1::wasm::EncodedMap,
      ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(definition::Inputs {
          left: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("left")?)?,
          right: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("right")?)?,
        })
      }

      #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Inputs {
        #[serde(rename = "left")]
        pub left: String,
        #[serde(rename = "right")]
        pub right: String,
      }

      impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
        fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
          let mut map = std::collections::HashMap::default();
          map.insert(
            "left".to_owned(),
            wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.left).into(),
          );
          map.insert(
            "right".to_owned(),
            wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.right).into(),
          );
          wasmflow_sdk::v1::packet::PacketMap::new(map)
        }
      }

      #[must_use]
      #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
      pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("left".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
        map.insert("right".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
        map
      }

      // A list of ports and their type signatures.
      #[must_use]
      #[cfg(feature = "host")]
      pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
        let mut map = std::collections::HashMap::new();
        map.insert("output".to_owned(), wasmflow_sdk::v1::types::TypeSignature::String);
        map
      }

      // A list of output ports and their associated stream sender implementations.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPorts {
        pub output: OutputPortSender,
      }

      impl OutputPorts {
        fn new(id: u32) -> Self {
          Self {
            output: OutputPortSender::new(id),
          }
        }
      }

      // Definition and implementation of each port's sender.
      #[derive(Debug)]
      #[cfg(feature = "host")]
      pub struct OutputPortSender {
        port: wasmflow_sdk::v1::PortChannel,
        id: u32,
      }

      #[cfg(feature = "host")]
      impl OutputPortSender {
        fn new(id: u32) -> Self {
          Self {
            id,
            port: wasmflow_sdk::v1::PortChannel::new("output"),
          }
        }
      }

      #[cfg(all(feature = "host"))]
      impl wasmflow_sdk::v1::Writable for OutputPortSender {
        type PayloadType = String;

        fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
          if self.port.is_closed() {
            Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
          } else {
            Ok(&self.port)
          }
        }

        fn get_port_name(&self) -> &str {
          &self.port.name
        }

        fn get_id(&self) -> u32 {
          self.id
        }
      }

      #[cfg(all(feature = "host"))]
      pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
        let mut outputs = OutputPorts::new(id);
        let mut ports = vec![&mut outputs.output.port];
        let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
        (outputs, stream)
      }

      #[allow(missing_debug_implementations)]
      pub struct Outputs {
        packets: ComponentOutput,
      }

      impl Outputs {
        pub async fn output(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<String>, wasmflow_sdk::v1::error::Error> {
          let packets = self.packets.drain_port("output").await?;
          Ok(wasmflow_sdk::v1::PortOutput::new("output".to_owned(), packets))
        }
      }

      impl From<ComponentOutput> for Outputs {
        fn from(packets: ComponentOutput) -> Self {
          Self { packets }
        }
      }

      impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
          Self {
            packets: ComponentOutput::new(stream),
          }
        }
      }

      #[cfg(not(target_arch = "wasm32"))]
      impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
        fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
          Self {
            packets: ComponentOutput::new_from_ts(stream),
          }
        }
      }

      #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
      pub struct Config {}
    }
    // end component string::concat
  }
  // end namespace string

  pub mod __batch__ {
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};

    use super::{__batch__ as integration, __batch__ as definition};
    use crate::components::__batch__ as implementation;

    impl wasmflow_sdk::v1::stateful::BatchedJobExecutor for Component {
      #[cfg(not(target_arch = "wasm32"))]
      type Payload = wasmflow_sdk::v1::packet::v1::PacketMap;
      #[cfg(target_arch = "wasm32")]
      type Payload = wasmflow_sdk::v1::wasm::EncodedMap;
      type State = implementation::State;
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);
      type Context = crate::Context;

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config, Self::State>,
        context: Self::Context,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::stateful::BatchedComponent;
          let id = payload.id();
          let (outputs, mut stream) = definition::get_outputs(id);
          let (payload, config, state) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          let new_state = Component::job(inputs, outputs, context, state, config).await?;
          stream.push(wasmflow_sdk::v1::packet::PacketWrapper::state(
            Packet::success(&new_state).into(),
          ));
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub enum ComponentInputs {
      CoreError(super::core::error::Inputs),
      CoreLog(super::core::log::Inputs),
      CorePanic(super::core::panic::Inputs),
      MathAdd(super::math::add::Inputs),
      MathSubtract(super::math::subtract::Inputs),
      RandBytes(super::rand::bytes::Inputs),
      RandString(super::rand::string::Inputs),
      RandUuid(super::rand::uuid::Inputs),
      StringConcat(super::string::concat::Inputs),
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub enum ComponentOutputs {
      CoreError(super::core::error::Outputs),
      CoreLog(super::core::log::Outputs),
      CorePanic(super::core::panic::Outputs),
      MathAdd(super::math::add::Outputs),
      MathSubtract(super::math::subtract::Outputs),
      RandBytes(super::rand::bytes::Outputs),
      RandString(super::rand::string::Outputs),
      RandUuid(super::rand::uuid::Outputs),
      StringConcat(super::string::concat::Outputs),
    }

    #[derive(Debug, serde::Deserialize)]
    pub enum Config {
      CoreError(super::core::error::Config),
      CoreLog(super::core::log::Config),
      CorePanic(super::core::panic::Config),
      MathAdd(super::math::add::Config),
      MathSubtract(super::math::subtract::Config),
      RandBytes(super::rand::bytes::Config),
      RandString(super::rand::string::Config),
      RandUuid(super::rand::uuid::Config),
      StringConcat(super::string::concat::Config),
    }

    #[derive(Default, Clone, Copy)]
    #[allow(missing_debug_implementations)]
    pub struct Component {}

    impl wasmflow_sdk::v1::Component for Component {
      type Inputs = definition::Inputs;
      type Outputs = definition::OutputPorts;
      type Config = integration::Config;
    }
    // A list of ports and their type signatures.
    #[must_use]
    #[cfg(feature = "host")]
    pub fn outputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert("result".to_owned(), wasmflow_sdk::v1::types::TypeSignature::Bool);
      map
    }

    // A list of output ports and their associated stream sender implementations.
    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct OutputPorts {
      pub result: ResultPortSender,
    }

    impl OutputPorts {
      fn new(id: u32) -> Self {
        Self {
          result: ResultPortSender::new(id),
        }
      }
    }

    // Definition and implementation of each port's sender.
    #[derive(Debug)]
    #[cfg(feature = "host")]
    pub struct ResultPortSender {
      port: wasmflow_sdk::v1::PortChannel,
      id: u32,
    }

    #[cfg(feature = "host")]
    impl ResultPortSender {
      fn new(id: u32) -> Self {
        Self {
          id,
          port: wasmflow_sdk::v1::PortChannel::new("result"),
        }
      }
    }

    #[cfg(all(feature = "host"))]
    impl wasmflow_sdk::v1::Writable for ResultPortSender {
      type PayloadType = bool;

      fn get_port(&self) -> Result<&wasmflow_sdk::v1::PortChannel, wasmflow_sdk::v1::BoxedError> {
        if self.port.is_closed() {
          Err(Box::new(wasmflow_sdk::v1::error::Error::SendError("@key".to_owned())))
        } else {
          Ok(&self.port)
        }
      }

      fn get_port_name(&self) -> &str {
        &self.port.name
      }

      fn get_id(&self) -> u32 {
        self.id
      }
    }

    #[cfg(all(feature = "host"))]
    pub fn get_outputs(id: u32) -> (OutputPorts, wasmflow_sdk::v1::PacketStream) {
      let mut outputs = OutputPorts::new(id);
      let mut ports = vec![&mut outputs.result.port];
      let stream = wasmflow_sdk::v1::PortChannel::merge_all(&mut ports);
      (outputs, stream)
    }

    #[allow(missing_debug_implementations)]
    pub struct Outputs {
      packets: ComponentOutput,
    }

    impl Outputs {
      pub async fn result(&mut self) -> Result<wasmflow_sdk::v1::PortOutput<bool>, wasmflow_sdk::v1::error::Error> {
        let packets = self.packets.drain_port("result").await?;
        Ok(wasmflow_sdk::v1::PortOutput::new("result".to_owned(), packets))
      }
    }

    impl From<ComponentOutput> for Outputs {
      fn from(packets: ComponentOutput) -> Self {
        Self { packets }
      }
    }

    impl From<wasmflow_sdk::v1::PacketStream> for Outputs {
      fn from(stream: wasmflow_sdk::v1::PacketStream) -> Self {
        Self {
          packets: ComponentOutput::new(stream),
        }
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl From<wasmflow_sdk::v1::transport::TransportStream> for Outputs {
      fn from(stream: wasmflow_sdk::v1::transport::TransportStream) -> Self {
        Self {
          packets: ComponentOutput::new_from_ts(stream),
        }
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn convert_inputs(
      mut payload: wasmflow_sdk::v1::packet::v1::PacketMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        inputs: payload
          .remove("inputs")
          .ok_or_else(|| wasmflow_sdk::v1::error::Error::MissingInput("inputs".to_owned()))?
          .deserialize()?,
      })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn convert_inputs(
      payload: wasmflow_sdk::v1::wasm::EncodedMap,
    ) -> Result<definition::Inputs, Box<dyn std::error::Error + Send + Sync>> {
      Ok(definition::Inputs {
        inputs: wasmflow_sdk::v1::codec::messagepack::deserialize(payload.get("inputs")?)?,
      })
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Inputs {
      #[serde(rename = "inputs")]
      pub inputs: Vec<ComponentInputs>,
    }

    impl From<Inputs> for wasmflow_sdk::v1::packet::PacketMap {
      fn from(inputs: Inputs) -> wasmflow_sdk::v1::packet::PacketMap {
        let mut map = std::collections::HashMap::default();
        map.insert(
          "inputs".to_owned(),
          wasmflow_sdk::v1::packet::v1::Packet::success(&inputs.inputs).into(),
        );
        wasmflow_sdk::v1::packet::PacketMap::new(map)
      }
    }

    #[must_use]
    #[cfg(all(feature = "host", not(target_arch = "wasm32")))]
    pub fn inputs_list() -> std::collections::HashMap<String, wasmflow_sdk::v1::types::TypeSignature> {
      let mut map = std::collections::HashMap::new();
      map.insert(
        "inputs".to_owned(),
        wasmflow_sdk::v1::types::TypeSignature::List {
          element: Box::new(wasmflow_sdk::v1::types::TypeSignature::Internal(
            wasmflow_sdk::v1::types::InternalType::ComponentInput,
          )),
        },
      );
      map
    }
  }
}
