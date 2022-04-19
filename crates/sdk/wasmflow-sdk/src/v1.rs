pub mod packet {
  // pub use wasmflow_packet::*;
  pub use wasmflow_packet::{PacketMap, PacketWrapper};
  pub mod v1 {
    pub use wasmflow_packet::v1::{Packet, PacketMap};
  }
}

pub mod sdk {
  #[cfg(target_arch = "wasm32")]
  pub type BoxedFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'static>>;
  #[cfg(not(target_arch = "wasm32"))]
  pub type BoxedFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'static>>;
  pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;

  pub use futures::{Stream, StreamExt};
  pub use wasmflow_boundary::IncomingPayload;
  pub use wasmflow_collection_link::ProviderLink;
  pub use wasmflow_entity::{Entity, SystemEntity};
  #[cfg(not(target_arch = "wasm32"))]
  pub use wasmflow_invocation::Invocation;
  pub use wasmflow_output::{PortOutput, ProviderOutput};
  pub use wasmflow_traits::{Component, IntoInputs, PortChannel, Writable};

  pub mod payload {
    #[cfg(not(target_arch = "wasm32"))]
    pub use wasmflow_boundary::native::v1::from_invocation;
    #[cfg(target_arch = "wasm32")]
    pub use wasmflow_boundary::wasm::from_buffer;
  }

  pub mod stateful {
    #[cfg(not(target_arch = "wasm32"))]
    pub use wasmflow_component::guest::stateful::native::Dispatcher as NativeDispatcher;
    #[cfg(target_arch = "wasm32")]
    pub use wasmflow_component::guest::stateful::wasm::Dispatcher as WasmDispatcher;
    pub use wasmflow_component::guest::stateful::BatchedJobExecutor;
    pub use wasmflow_traits::stateful::BatchedComponent;
  }

  pub mod ephemeral {
    #[cfg(not(target_arch = "wasm32"))]
    pub use wasmflow_component::guest::ephemeral::native::Dispatcher as NativeDispatcher;
    #[cfg(target_arch = "wasm32")]
    pub use wasmflow_component::guest::ephemeral::wasm::Dispatcher as WasmDispatcher;
    pub use wasmflow_component::guest::ephemeral::BatchedJobExecutor;
    pub use wasmflow_traits::ephemeral::BatchedComponent;
  }

  #[cfg(target_arch = "wasm32")]
  pub mod wasm {
    pub use wasmflow_boundary::wasm::EncodedMap;
    pub mod runtime {
      pub use wasmflow_component::guest::wasm::runtime::register_dispatcher;
    }
    pub use wasmflow_component::guest::wasm::runtime::{port_send, port_send_close};
  }

  #[cfg(not(target_arch = "wasm32"))]
  pub mod native {}
}

pub mod error {
  pub use crate::sdk::BoxedError;

  #[derive(Debug)]
  pub enum Error {
    /// An input the component expects was not found.
    MissingInput(String),

    /// An error from an upstream module.
    Upstream(Box<dyn std::error::Error + Send + Sync>),

    /// Error sending packet to output port.
    SendError(String),

    /// The requested component was not found in this module.
    ComponentNotFound(String, String),

    /// An error resulting from deserializing or serializing a payload.
    CodecError(String),
    /// culling line
    /// An error returned from the WaPC host, the system running the WebAssembly module.
    HostError(String),

    /// Async runtime failure.
    Async,

    /// Dispatcher not set before guest call
    DispatcherNotSet,
  }

  #[derive(Debug)]
  /// Error originating from a component task.
  pub struct ComponentError(String);

  impl ComponentError {
    /// Constructor for a [ComponentError].
    pub fn new<T: std::fmt::Display>(message: T) -> Self {
      Self(message.to_string())
    }
  }

  impl std::error::Error for ComponentError {}

  impl std::fmt::Display for ComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{}", self.0)
    }
  }

  impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
        Error::ComponentNotFound(v, valid) => write!(f, "Component '{}' not found. Valid components are: {}", v, valid),
        Error::Upstream(v) => write!(f, "{}", v),
        Error::MissingInput(v) => write!(f, "Missing input for port '{}'", v),
        Error::SendError(port) => write!(f, "Error sending packet to output port '{}'", port),
        Error::CodecError(v) => write!(f, "{}", v),
        Error::HostError(v) => write!(f, "Error executing host call: {}", v),
        Error::Async => write!(f, "Async runtime error"),
        Error::DispatcherNotSet => write!(f, "Dispatcher not set before host call"),
      }
    }
  }

  impl std::error::Error for Error {}

  impl From<wasmflow_packet::error::Error> for Error {
    fn from(e: wasmflow_packet::error::Error) -> Self {
      Error::Upstream(Box::new(e))
    }
  }

  impl From<wasmflow_output::error::Error> for Error {
    fn from(e: wasmflow_output::error::Error) -> Self {
      Error::Upstream(Box::new(e))
    }
  }

  impl From<wasmflow_codec::Error> for Error {
    fn from(e: wasmflow_codec::Error) -> Self {
      Error::CodecError(e.to_string())
    }
  }

  impl From<BoxedError> for Error {
    fn from(e: BoxedError) -> Self {
      Error::Upstream(e)
    }
  }
}

pub mod codec {
  pub use wasmflow_codec::{json, messagepack};
}

pub mod provider {
  pub mod error {}
}

pub mod types {
  pub use vino_transport::TransportStream;
  pub use wasmflow_interface::*;
  pub use wasmflow_streams::PacketStream;
}

#[macro_use]
#[allow(unreachable_pub)]
pub use wasmflow_component::console_log;
