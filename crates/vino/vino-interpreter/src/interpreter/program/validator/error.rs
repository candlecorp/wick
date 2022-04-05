#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
  #[error("Unused sender: {0}")]
  UnusedSender(String),
  #[error("Network contains circular references: {:?}", .0)]
  NetworkUnresolvable(Vec<String>),
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
  #[error("Input port '{port}' on component '{namespace}::{component}' not connected to anything")]
  MissingConnection {
    port: String,
    namespace: String,
    component: String,
  },
  #[error("Unused output port '{port}' on component '{namespace}::{component}'")]
  UnusedOutput {
    port: String,
    namespace: String,
    component: String,
  },
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct SchematicInvalid {
  errors: Vec<ValidationError>,
  schematic: String,
}

impl SchematicInvalid {
  pub fn new(schematic: String, errors: Vec<ValidationError>) -> Self {
    Self { schematic, errors }
  }
}

impl std::fmt::Display for SchematicInvalid {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Schematic '{}' could not be validated: {}",
      self.schematic,
      self.errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", ")
    )
  }
}
