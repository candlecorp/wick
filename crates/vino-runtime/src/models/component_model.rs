use crate::dev::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ComponentModel {
  /// The name of the component
  pub(crate) name: String,
  pub(crate) namespace: String,
  pub(crate) inputs: Vec<PortSignature>,
  pub(crate) outputs: Vec<PortSignature>,
}
