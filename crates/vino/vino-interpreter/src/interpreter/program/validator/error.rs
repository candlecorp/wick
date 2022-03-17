#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
  #[error("Missing provider namespace '{0}'")]
  MissingProvider(String),
  #[error("Missing component '{name}' on namespace '{namespace}'")]
  MissingComponent { namespace: String, name: String },
  #[error("Invalid port '{port}' on component '{namespace}::{component}'")]
  InvalidPort {
    port: String,
    namespace: String,
    component: String,
  },
  #[error("Port '{port}' missing on component '{namespace}::{component}'")]
  MissingPort {
    port: String,
    namespace: String,
    component: String,
  },
}
