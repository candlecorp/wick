use thiserror::Error;

#[derive(Error, Debug)]
pub enum MeshProviderError {
  #[error("Component error : {0}")]
  ComponentError(String),
  #[error(transparent)]
  Mesh(#[from] wasmflow_mesh::Error),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error("JSON Serialization/Deserialization error : {0}")]
  JsonError(String),
  #[error("Could not extract claims from component. Is it a signed WebAssembly module?")]
  ClaimsExtraction,
  #[error("Error sending output to stream.")]
  SendError,
  #[error("Internal Error : {0}")]
  Other(String),
  #[error("Internal Error : {0}")]
  InternalError(u32),
  #[error("Could not create KeyPair from invalid seed")]
  KeyPairFailed,
  #[error("Component '{0}' not found. Valid components are: {}", .1.join(", "))]
  ComponentNotFound(String, Vec<String>),
}

// impl From<serde_json::error::Error> for MeshProviderError {
//   fn from(e: serde_json::error::Error) -> Self {
//     MeshProviderError::JsonError(e.to_string())
//   }
// }
