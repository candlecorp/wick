#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
  #[error("Unused sender: {0}")]
  UnusedSender(String),

  #[error("Network contains circular references: {:?}", .0)]
  NetworkUnresolvable(Vec<String>),

  #[error("Missing component with id '{0}'")]
  MissingComponent(String),

  #[error("Could not find component referenced by id '{0}'")]
  ComponentIdNotFound(String),

  #[error("Missing operation '{name}' on component '{component}'")]
  MissingOperation { component: String, name: String },

  #[error("Invalid port '{port}' on operation '{component}::{operation}'")]
  InvalidPort {
    port: String,
    component: String,
    operation: String,
  },

  #[error("Input port '{port}' on operation '{component}::{operation}' not connected to anything")]
  MissingConnection {
    port: String,
    component: String,
    operation: String,
  },

  #[error("Unused output port '{port}' on component '{component}::{operation}'")]
  UnusedOutput {
    port: String,
    component: String,
    operation: String,
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
