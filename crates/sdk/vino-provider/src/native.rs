use std::sync::{
  Arc,
  Mutex,
};

/// Module for native provider errors
pub mod error;
use async_trait::async_trait;
pub use error::Error;
/// Module for native ports
pub mod port_sender;

/// The type of a provider's context.
pub type Context<T> = Arc<Mutex<T>>;
/// The JobResult for native components
pub type JobResult = Result<(), NativeComponentError>;

#[async_trait]
/// Trait used by auto-generated provider components. You shouldn't need to implement this if you are using Vino's code generator.
pub trait NativeComponent {
  /// The provider state passed to every component's execution.
  type State;
  /// The wrapper method that is called to execute the component's job.
  async fn execute(
    &self,
    context: Arc<Mutex<Self::State>>,
    data: TransportMap,
  ) -> Result<MessageTransportStream, Box<NativeComponentError>>;
}

pub use vino_entity as entity;
use vino_transport::{
  MessageTransportStream,
  TransportMap,
};

use self::prelude::NativeComponentError;

/// A list of imports that are common to native providers.
pub mod prelude {
  use tokio::sync::mpsc::{
    UnboundedReceiver,
    UnboundedSender,
  };
  /// Type alias for [UnboundedSender<PacketWrapper>];
  pub type PacketSender = UnboundedSender<PacketWrapper>;
  /// Type alias for [UnboundedReceiver<PacketWrapper>];
  pub type PacketReceiver = UnboundedReceiver<PacketWrapper>;
  pub use async_trait::async_trait;
  pub use vino_component::PacketWrapper;
  pub use vino_entity::{
    Entity,
    Error as EntityError,
  };
  pub use vino_transport::error::TransportError;
  pub use vino_transport::message_transport::stream::MessageTransportStream;
  pub use vino_transport::message_transport::{
    MessageTransport,
    TransportMap,
    TransportWrapper,
  };
  pub use vino_types::signatures::*;

  pub use super::error::{
    Error as ProviderError,
    NativeComponentError,
  };
  pub use super::port_sender::{
    PortChannel,
    PortSender,
  };
  pub use super::{
    Context,
    JobResult,
    NativeComponent,
  };
}
