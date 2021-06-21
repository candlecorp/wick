pub mod codec;
pub mod error;

pub type Result<T> = std::result::Result<T, error::TransportError>;
pub type Error = error::TransportError;

pub use codec::{
  deserialize,
  serialize,
};
