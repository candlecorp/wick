/// The crate's error module
pub mod error;

/// Module that exposes [wasmflow_packet] APIs.
pub mod packet {
  pub use wasmflow_packet::{Packet, PacketMap, PacketWrapper};
  /// v1 Packets
  pub mod v1 {
    pub use wasmflow_packet::v1::{Failure, Packet, PacketMap, Serialized, Signal};
  }
}

/// Utility type for a [std::pin::Pin<Box<Future>>]
pub type BoxedFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'static>>;

/// Utility type for a [Box<Error + Send + Sync>]
pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;

pub use futures::{Stream, StreamExt};
pub use wasmflow_boundary::IncomingPayload;
pub use wasmflow_collection_link::CollectionLink;
pub use wasmflow_entity::{Entity, SystemEntity};
#[cfg(not(target_arch = "wasm32"))]
pub use wasmflow_invocation::{InherentData, Invocation};
pub use wasmflow_output::{ComponentOutput, PortOutput};
pub use wasmflow_streams::PacketStream;
pub use wasmflow_traits::{Component, IntoInputs, PortChannel, Writable};

/// Module that exposes [wasmflow_transport] APIs.
pub mod transport {
  pub use wasmflow_transport::{
    Failure,
    JsonError,
    MessageSignal,
    MessageTransport,
    Serialized,
    TransportJson,
    TransportMap,
    TransportStream,
    TransportWrapper,
  };
}

/// Module that exposes runtime interfaces.
pub mod runtime {
  pub use wasmflow_component::{HostCommand, LogLevel, OutputSignal};
}

/// Module that exposes handlers to deal with payloads from different sources.
pub mod payload {
  #[cfg(not(target_arch = "wasm32"))]
  pub use wasmflow_boundary::native::v1::from_invocation;
  #[cfg(target_arch = "wasm32")]
  pub use wasmflow_boundary::wasm::from_buffer;
}

/// Module for stateful components.
pub mod stateful {
  #[cfg(not(target_arch = "wasm32"))]
  pub use wasmflow_component::guest::stateful::native::Dispatcher as NativeDispatcher;
  #[cfg(target_arch = "wasm32")]
  pub use wasmflow_component::guest::stateful::wasm::Dispatcher as WasmDispatcher;
  pub use wasmflow_component::guest::stateful::BatchedJobExecutor;
  pub use wasmflow_traits::stateful::BatchedComponent;
}

/// Module for ephemeral components.
pub mod ephemeral {
  #[cfg(not(target_arch = "wasm32"))]
  pub use wasmflow_component::guest::ephemeral::native::Dispatcher as NativeDispatcher;
  #[cfg(target_arch = "wasm32")]
  pub use wasmflow_component::guest::ephemeral::wasm::Dispatcher as WasmDispatcher;
  pub use wasmflow_component::guest::ephemeral::BatchedJobExecutor;
  pub use wasmflow_traits::ephemeral::BatchedComponent;
}

#[cfg(target_arch = "wasm32")]
/// Module for components targeting wasm32.
pub mod wasm {
  pub use wasmflow_boundary::wasm::EncodedMap;
  /// Module for the wasm32 runtime.
  pub mod runtime {
    pub use wasmflow_component::guest::wasm::runtime::register_dispatcher;
  }
  pub use wasmflow_component::guest::wasm::runtime::{port_send, port_send_close};
}

/// Module that exposes [wasmflow_codec] APIs.
pub mod codec {
  pub use wasmflow_codec::{json, messagepack, raw};
}

/// Module that exposes [wasmflow_interface] APIs.
pub mod types {
  pub use wasmflow_interface::*;
}

#[macro_use]
pub use wasmflow_component::console_log;
