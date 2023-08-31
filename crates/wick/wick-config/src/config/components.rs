mod grpcurl;
mod http_client;
mod manifest;
mod native;
mod reference;
mod sql;
mod types;
mod wasm;
mod websocket_client;

use std::borrow::Cow;

pub use grpcurl::*;
pub use http_client::*;
pub use manifest::*;
pub use native::*;
pub use reference::*;
pub use sql::*;
pub use types::*;
pub use wasm::*;
pub use websocket_client::*;
use wick_interface_types::{Field, OperationSignatures};

pub trait OperationConfig {
  /// The name of the operation.
  fn name(&self) -> &str;

  /// The inputs to the operation.
  fn inputs(&self) -> Cow<Vec<Field>>;

  /// The outpus to the operation.
  fn outputs(&self) -> Cow<Vec<Field>>;
}

pub trait ComponentConfig: OperationSignatures {
  type Operation: OperationConfig;

  /// Get the operations defined by this configuration.
  fn operations(&self) -> &[Self::Operation];

  /// Get the operations defined by this configuration.
  fn operations_mut(&mut self) -> &mut Vec<Self::Operation>;

  /// Get an operation definition by name.
  fn get_operation(&self, name: &str) -> Option<&Self::Operation> {
    self.operations().iter().find(|o| o.name() == name)
  }
}
