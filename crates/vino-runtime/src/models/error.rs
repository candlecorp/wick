use std::fmt::Display;

use itertools::join;
use thiserror::Error;

use crate::dev::prelude::*;

#[derive(Error, Debug, PartialEq)]
pub enum ValidationErrorKind {
  #[error("Schematic has no outputs")]
  NoOutputs,
  #[error("Schematic has no inputs")]
  NoInputs,
  #[error("Model has an error: {0}")]
  ModelError(String),
  #[error("Can not find definition for instance: {0}")]
  InstanceNotFound(String),
  #[error("Can't find a model for '{0}', is it spelled correctly?")]
  MissingComponentModel(String),
  #[error("Dangling reference: '{0}'")]
  DanglingReference(String),
  #[error("Component definition '{0}' not fully qualified")]
  NotFullyQualified(String),
  #[error("Invalid output port '{}' on {}. Valid output ports are [{}]", .0.get_port().unwrap_or("<No port>"), .1, join(.2, ", "))]
  InvalidOutputPort(
    ConnectionTargetDefinition,
    ConnectionDefinition,
    Vec<PortSignature>,
  ),
  #[error("Invalid input port '{}' on {}. Valid input ports are [{}]", .0.get_port().unwrap_or("<No port>"), .1, join(.2, ", "))]
  InvalidInputPort(
    ConnectionTargetDefinition,
    ConnectionDefinition,
    Vec<PortSignature>,
  ),
  #[error("Invalid connection: {0}")]
  InvalidConnection(ConnectionDefinition),
}

#[derive(Error, Debug, PartialEq)]
#[must_use]
pub struct ValidationError {
  name: String,
  errors: Vec<ValidationErrorKind>,
}

impl ValidationError {
  pub fn new(name: &str, errors: Vec<ValidationErrorKind>) -> Self {
    Self {
      name: name.to_owned(),
      errors,
    }
  }
}

impl Display for ValidationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "Schematic {} has validation errors: {}",
      self.name,
      join(&self.errors, "\n  ")
    ))
  }
}

impl From<SchematicModelError> for ValidationErrorKind {
  fn from(e: SchematicModelError) -> Self {
    ValidationErrorKind::ModelError(e.to_string())
  }
}

#[derive(Error, Debug)]
pub enum SchematicModelError {
  #[error("Schematic model not able to finish initialization")]
  IncompleteInitialization,
  #[error("Schematic model not initialized")]
  ModelNotInitialized,
  #[error("The reference '{0}' has an incomplete component model. Component may have failed to load or be in a partial state.")]
  MissingComponentModel(String),
  #[error(transparent)]
  DefaultsError(#[from] serde_json::error::Error),
}
