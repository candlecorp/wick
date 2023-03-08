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

pub mod __batch__;
pub mod decr; // decr
pub mod delete; // delete
pub mod exists; // exists
pub mod incr; // incr
pub mod key_get; // key-get
pub mod key_set; // key-set
pub mod list_add; // list-add
pub mod list_range; // list-range
pub mod list_remove; // list-remove
pub mod set_add; // set-add
pub mod set_contains; // set-contains
pub mod set_get; // set-get
pub mod set_remove; // set-remove
pub mod set_scan; // set-scan

#[allow(unused)]
static ALL_COMPONENTS: &[&str] = &[
  "decr",
  "delete",
  "exists",
  "incr",
  "key-get",
  "key-set",
  "list-add",
  "list-range",
  "list-remove",
  "set-add",
  "set-contains",
  "set-get",
  "set-remove",
  "set-scan",
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
        "decr" => {
          crate::components::generated::decr::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "delete" => {
          crate::components::generated::delete::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "exists" => {
          crate::components::generated::exists::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "incr" => {
          crate::components::generated::incr::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "key-get" => {
          crate::components::generated::key_get::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "key-set" => {
          crate::components::generated::key_set::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "list-add" => {
          crate::components::generated::list_add::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "list-range" => {
          crate::components::generated::list_range::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "list-remove" => {
          crate::components::generated::list_remove::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "set-add" => {
          crate::components::generated::set_add::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "set-contains" => {
          crate::components::generated::set_contains::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "set-get" => {
          crate::components::generated::set_get::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "set-remove" => {
          crate::components::generated::set_remove::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_buffer(payload)?, context)
            .await
        }
        "set-scan" => {
          crate::components::generated::set_scan::Component::default()
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
        "decr" => {
          crate::components::generated::decr::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "delete" => {
          crate::components::generated::delete::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "exists" => {
          crate::components::generated::exists::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "incr" => {
          crate::components::generated::incr::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "key-get" => {
          crate::components::generated::key_get::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "key-set" => {
          crate::components::generated::key_set::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "list-add" => {
          crate::components::generated::list_add::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "list-range" => {
          crate::components::generated::list_range::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "list-remove" => {
          crate::components::generated::list_remove::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "set-add" => {
          crate::components::generated::set_add::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "set-contains" => {
          crate::components::generated::set_contains::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "set-get" => {
          crate::components::generated::set_get::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "set-remove" => {
          crate::components::generated::set_remove::Component::default()
            .execute(wasmflow_sdk::v1::payload::from_invocation(invocation)?, context)
            .await
        }
        "set-scan" => {
          crate::components::generated::set_scan::Component::default()
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
  let mut components: std::collections::HashMap<String, wasmflow_sdk::v1::types::OperationSignature> =
    std::collections::HashMap::new();

  components.insert("decr".to_owned(), wick_interface_types_keyvalue::decr::signature());

  components.insert("delete".to_owned(), wick_interface_types_keyvalue::delete::signature());

  components.insert("exists".to_owned(), wick_interface_types_keyvalue::exists::signature());

  components.insert("incr".to_owned(), wick_interface_types_keyvalue::incr::signature());

  components.insert(
    "key-get".to_owned(),
    wick_interface_types_keyvalue::key_get::signature(),
  );

  components.insert(
    "key-set".to_owned(),
    wick_interface_types_keyvalue::key_set::signature(),
  );

  components.insert(
    "list-add".to_owned(),
    wick_interface_types_keyvalue::list_add::signature(),
  );

  components.insert(
    "list-range".to_owned(),
    wick_interface_types_keyvalue::list_range::signature(),
  );

  components.insert(
    "list-remove".to_owned(),
    wick_interface_types_keyvalue::list_remove::signature(),
  );

  components.insert(
    "set-add".to_owned(),
    wick_interface_types_keyvalue::set_add::signature(),
  );

  components.insert(
    "set-contains".to_owned(),
    wick_interface_types_keyvalue::set_contains::signature(),
  );

  components.insert(
    "set-get".to_owned(),
    wick_interface_types_keyvalue::set_get::signature(),
  );

  components.insert(
    "set-remove".to_owned(),
    wick_interface_types_keyvalue::set_remove::signature(),
  );

  components.insert(
    "set-scan".to_owned(),
    wick_interface_types_keyvalue::set_scan::signature(),
  );

  wasmflow_sdk::v1::types::CollectionSignature {
    name: Some("wasmflow-keyvalue-redis".to_owned()),
    features: wasmflow_sdk::v1::types::CollectionFeatures {
      streaming: false,
      stateful: true,
      version: wasmflow_sdk::v1::types::CollectionVersion::V0,
    },
    format: 1,
    version: "1".to_owned(),
    types: std::collections::HashMap::from([]).into(),
    operations: components.into(),
    wellknown: Vec::new(),
    config: wasmflow_sdk::v1::types::TypeMap::new(),
  }
}

pub mod types {
  // no additional types
}
pub mod generated {

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
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);
      type Context = crate::Context;

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
        context: Self::Context,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::stateful::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, context, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
    pub enum ComponentInputs {
      Decr(wick_interface_types_keyvalue::decr::Inputs),
      Delete(wick_interface_types_keyvalue::delete::Inputs),
      Exists(wick_interface_types_keyvalue::exists::Inputs),
      Incr(wick_interface_types_keyvalue::incr::Inputs),
      KeyGet(wick_interface_types_keyvalue::key_get::Inputs),
      KeySet(wick_interface_types_keyvalue::key_set::Inputs),
      ListAdd(wick_interface_types_keyvalue::list_add::Inputs),
      ListRange(wick_interface_types_keyvalue::list_range::Inputs),
      ListRemove(wick_interface_types_keyvalue::list_remove::Inputs),
      SetAdd(wick_interface_types_keyvalue::set_add::Inputs),
      SetContains(wick_interface_types_keyvalue::set_contains::Inputs),
      SetGet(wick_interface_types_keyvalue::set_get::Inputs),
      SetRemove(wick_interface_types_keyvalue::set_remove::Inputs),
      SetScan(wick_interface_types_keyvalue::set_scan::Inputs),
    }

    #[cfg(all(feature = "guest"))]
    #[allow(missing_debug_implementations)]
    pub enum ComponentOutputs {
      Decr(wick_interface_types_keyvalue::decr::Outputs),
      Delete(wick_interface_types_keyvalue::delete::Outputs),
      Exists(wick_interface_types_keyvalue::exists::Outputs),
      Incr(wick_interface_types_keyvalue::incr::Outputs),
      KeyGet(wick_interface_types_keyvalue::key_get::Outputs),
      KeySet(wick_interface_types_keyvalue::key_set::Outputs),
      ListAdd(wick_interface_types_keyvalue::list_add::Outputs),
      ListRange(wick_interface_types_keyvalue::list_range::Outputs),
      ListRemove(wick_interface_types_keyvalue::list_remove::Outputs),
      SetAdd(wick_interface_types_keyvalue::set_add::Outputs),
      SetContains(wick_interface_types_keyvalue::set_contains::Outputs),
      SetGet(wick_interface_types_keyvalue::set_get::Outputs),
      SetRemove(wick_interface_types_keyvalue::set_remove::Outputs),
      SetScan(wick_interface_types_keyvalue::set_scan::Outputs),
    }

    #[derive(Debug, serde::Deserialize)]
    pub enum Config {
      Decr(wick_interface_types_keyvalue::decr::Config),
      Delete(wick_interface_types_keyvalue::delete::Config),
      Exists(wick_interface_types_keyvalue::exists::Config),
      Incr(wick_interface_types_keyvalue::incr::Config),
      KeyGet(wick_interface_types_keyvalue::key_get::Config),
      KeySet(wick_interface_types_keyvalue::key_set::Config),
      ListAdd(wick_interface_types_keyvalue::list_add::Config),
      ListRange(wick_interface_types_keyvalue::list_range::Config),
      ListRemove(wick_interface_types_keyvalue::list_remove::Config),
      SetAdd(wick_interface_types_keyvalue::set_add::Config),
      SetContains(wick_interface_types_keyvalue::set_contains::Config),
      SetGet(wick_interface_types_keyvalue::set_get::Config),
      SetRemove(wick_interface_types_keyvalue::set_remove::Config),
      SetScan(wick_interface_types_keyvalue::set_scan::Config),
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

  // wellknown interface: ../../interfaces/wick-interface-types-keyvalue/interface.json

  // start component decr
  pub mod decr {
    // The user-facing implementation job impl.
    // The generated definition of inputs, outputs, config, et al.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};
    use wick_interface_types_keyvalue::decr as definition;

    // The generated integration code between the definition and the implementation.
    use super::decr as integration;
    use crate::components::decr as implementation;

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
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);
      type Context = crate::Context;

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
        context: Self::Context,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::stateful::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, context, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component decr
  // start component delete
  pub mod delete {
    // The user-facing implementation job impl.
    // The generated definition of inputs, outputs, config, et al.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};
    use wick_interface_types_keyvalue::delete as definition;

    // The generated integration code between the definition and the implementation.
    use super::delete as integration;
    use crate::components::delete as implementation;

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
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);
      type Context = crate::Context;

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
        context: Self::Context,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::stateful::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, context, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component delete
  // start component exists
  pub mod exists {
    // The user-facing implementation job impl.
    // The generated definition of inputs, outputs, config, et al.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};
    use wick_interface_types_keyvalue::exists as definition;

    // The generated integration code between the definition and the implementation.
    use super::exists as integration;
    use crate::components::exists as implementation;

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
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);
      type Context = crate::Context;

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
        context: Self::Context,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::stateful::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, context, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component exists
  // start component incr
  pub mod incr {
    // The user-facing implementation job impl.
    // The generated definition of inputs, outputs, config, et al.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};
    use wick_interface_types_keyvalue::incr as definition;

    // The generated integration code between the definition and the implementation.
    use super::incr as integration;
    use crate::components::incr as implementation;

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
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);
      type Context = crate::Context;

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
        context: Self::Context,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::stateful::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, context, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component incr
  // start component key-get
  pub mod key_get {
    // The user-facing implementation job impl.
    // The generated definition of inputs, outputs, config, et al.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};
    use wick_interface_types_keyvalue::key_get as definition;

    // The generated integration code between the definition and the implementation.
    use super::key_get as integration;
    use crate::components::key_get as implementation;

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
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);
      type Context = crate::Context;

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
        context: Self::Context,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::stateful::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, context, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component key-get
  // start component key-set
  pub mod key_set {
    // The user-facing implementation job impl.
    // The generated definition of inputs, outputs, config, et al.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};
    use wick_interface_types_keyvalue::key_set as definition;

    // The generated integration code between the definition and the implementation.
    use super::key_set as integration;
    use crate::components::key_set as implementation;

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
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);
      type Context = crate::Context;

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
        context: Self::Context,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::stateful::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, context, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component key-set
  // start component list-add
  pub mod list_add {
    // The user-facing implementation job impl.
    // The generated definition of inputs, outputs, config, et al.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};
    use wick_interface_types_keyvalue::list_add as definition;

    // The generated integration code between the definition and the implementation.
    use super::list_add as integration;
    use crate::components::list_add as implementation;

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
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);
      type Context = crate::Context;

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
        context: Self::Context,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::stateful::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, context, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component list-add
  // start component list-range
  pub mod list_range {
    // The user-facing implementation job impl.
    // The generated definition of inputs, outputs, config, et al.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};
    use wick_interface_types_keyvalue::list_range as definition;

    // The generated integration code between the definition and the implementation.
    use super::list_range as integration;
    use crate::components::list_range as implementation;

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
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);
      type Context = crate::Context;

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
        context: Self::Context,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::stateful::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, context, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component list-range
  // start component list-remove
  pub mod list_remove {
    // The user-facing implementation job impl.
    // The generated definition of inputs, outputs, config, et al.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};
    use wick_interface_types_keyvalue::list_remove as definition;

    // The generated integration code between the definition and the implementation.
    use super::list_remove as integration;
    use crate::components::list_remove as implementation;

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
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);
      type Context = crate::Context;

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
        context: Self::Context,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::stateful::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, context, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component list-remove
  // start component set-add
  pub mod set_add {
    // The user-facing implementation job impl.
    // The generated definition of inputs, outputs, config, et al.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};
    use wick_interface_types_keyvalue::set_add as definition;

    // The generated integration code between the definition and the implementation.
    use super::set_add as integration;
    use crate::components::set_add as implementation;

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
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);
      type Context = crate::Context;

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
        context: Self::Context,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::stateful::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, context, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component set-add
  // start component set-contains
  pub mod set_contains {
    // The user-facing implementation job impl.
    // The generated definition of inputs, outputs, config, et al.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};
    use wick_interface_types_keyvalue::set_contains as definition;

    // The generated integration code between the definition and the implementation.
    use super::set_contains as integration;
    use crate::components::set_contains as implementation;

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
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);
      type Context = crate::Context;

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
        context: Self::Context,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::stateful::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, context, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component set-contains
  // start component set-get
  pub mod set_get {
    // The user-facing implementation job impl.
    // The generated definition of inputs, outputs, config, et al.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};
    use wick_interface_types_keyvalue::set_get as definition;

    // The generated integration code between the definition and the implementation.
    use super::set_get as integration;
    use crate::components::set_get as implementation;

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
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);
      type Context = crate::Context;

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
        context: Self::Context,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::stateful::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, context, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component set-get
  // start component set-remove
  pub mod set_remove {
    // The user-facing implementation job impl.
    // The generated definition of inputs, outputs, config, et al.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};
    use wick_interface_types_keyvalue::set_remove as definition;

    // The generated integration code between the definition and the implementation.
    use super::set_remove as integration;
    use crate::components::set_remove as implementation;

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
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);
      type Context = crate::Context;

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
        context: Self::Context,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::stateful::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, context, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component set-remove
  // start component set-scan
  pub mod set_scan {
    // The user-facing implementation job impl.
    // The generated definition of inputs, outputs, config, et al.
    pub use wasmflow_sdk::v1::packet::v1::Packet;
    pub use wasmflow_sdk::v1::{console_log, ComponentOutput, Writable};
    use wick_interface_types_keyvalue::set_scan as definition;

    // The generated integration code between the definition and the implementation.
    use super::set_scan as integration;
    use crate::components::set_scan as implementation;

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
      type Config = Config;
      type Return = (wasmflow_sdk::v1::PacketStream, u32);
      type Context = crate::Context;

      fn execute(
        &self,
        payload: wasmflow_sdk::v1::IncomingPayload<Self::Payload, Self::Config>,
        context: Self::Context,
      ) -> wasmflow_sdk::v1::BoxedFuture<Result<Self::Return, wasmflow_sdk::v1::BoxedError>> {
        Box::pin(async move {
          use wasmflow_sdk::v1::stateful::BatchedComponent;
          let id = payload.id();
          let (outputs, stream) = definition::get_outputs(id);
          let (payload, config) = payload.into_parts();
          let inputs = definition::convert_inputs(payload)?;

          Component::job(inputs, outputs, context, config).await?;
          Ok((stream, id))
        })
      }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct Config {}
  }
  // end component set-scan
}
