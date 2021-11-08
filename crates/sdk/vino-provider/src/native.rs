/// Module for native provider errors.
pub mod error;
use async_trait::async_trait;
pub use error::Error;
/// Module for native ports.
pub mod port_sender;
/// Module for making transport streams simpler to use.
pub mod provider_output;

/// The JobResult for native components.
pub type JobResult = Result<(), NativeComponentError>;

#[async_trait]
/// Trait used by auto-generated provider components. You shouldn't need to implement this if you are using Vino's code generator.
pub trait NativeComponent {
  /// The provider state passed to every component's execution.
  type Context: Send + Sync;
  /// The wrapper method that is called to execute the component's job.
  async fn execute(
    &self,
    context: Self::Context,
    data: TransportMap,
  ) -> Result<TransportStream, Box<NativeComponentError>>;
}

pub use vino_entity as entity;
use vino_transport::{TransportMap, TransportStream};

use self::prelude::NativeComponentError;

/// A list of imports that are common to native providers.
pub mod prelude {
  use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
  /// Type alias for [UnboundedSender<PacketWrapper>];.
  pub type PacketSender = UnboundedSender<PacketWrapper>;
  /// Type alias for [UnboundedReceiver<PacketWrapper>];.
  pub type PacketReceiver = UnboundedReceiver<PacketWrapper>;
  pub use async_trait::async_trait;
  pub use vino_entity::{Entity, Error as EntityError};
  pub use vino_packet::v1::Payload;
  pub use vino_packet::PacketWrapper;
  pub use vino_transport::error::TransportError;
  pub use vino_transport::{
    BoxedTransportStream, MessageTransport, TransportMap, TransportStream, TransportWrapper,
  };
  pub use vino_types::*;

  pub use super::error::{Error as ProviderError, NativeComponentError};
  pub use super::port_sender::{PortChannel, PortSender};
  pub use super::provider_output::*;
  pub use super::{Dispatch, JobResult, NativeComponent};
  pub use crate::raw::RawPacket;
}

#[doc(hidden)]
#[async_trait]
pub trait Dispatch {
  type Context: Send + Sync;
  async fn dispatch(
    op: &str,
    context: Self::Context,
    data: TransportMap,
  ) -> Result<TransportStream, Box<NativeComponentError>>;
}
