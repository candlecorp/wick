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

  #[error("Invalid port '{port}' on operation '{id}' ('{component}::{operation}')")]
  InvalidPort {
    port: String,
    id: String,
    component: String,
    operation: String,
  },

  #[error("Input port '{port}' on operation '{id}' ('{component}::{operation}') not connected to anything")]
  MissingConnection {
    port: String,
    id: String,
    component: String,
    operation: String,
  },

  #[error(
    "Signature for operation '{id}' ('{component}::{operation}') describes port '{port}' but '{port}' not found in graph"
  )]
  MissingPort {
    port: String,
    id: String,
    component: String,
    operation: String,
  },

  #[error(
    "Input '{port}' on operation '{id}' ('{component}::{operation}') is connected but does not exist on operation."
  )]
  UnknownInput {
    port: String,
    id: String,
    component: String,
    operation: String,
  },

  #[error(
    "Output '{port}' on operation '{id}' ('{component}::{operation}') is connected but does not exist on operation."
  )]
  UnknownOutput {
    port: String,
    id: String,
    component: String,
    operation: String,
  },

  #[error("Unused output port '{port}' on operation '{id}' ('{component}::{operation}')")]
  UnusedOutput {
    port: String,
    id: String,
    component: String,
    operation: String,
  },
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct OperationInvalid {
  errors: Vec<ValidationError>,
  schematic: String,
}

impl OperationInvalid {
  pub fn new(schematic: String, errors: Vec<ValidationError>) -> Self {
    Self { schematic, errors }
  }
}

impl std::fmt::Display for OperationInvalid {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Flow '{}' could not be validated: {}",
      self.schematic,
      self.errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", ")
    )
  }
}
