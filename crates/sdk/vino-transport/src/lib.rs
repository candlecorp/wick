pub mod error;
pub mod message_transport;

pub type Result<T> = std::result::Result<T, error::TransportError>;
pub type Error = error::TransportError;

pub use message_transport::MessageTransport;
