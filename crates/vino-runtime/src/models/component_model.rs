use std::collections::HashSet;

use crate::dev::prelude::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct ComponentModel {
  /// The name of the component.
  pub(crate) name: String,
  pub(crate) inputs: Vec<PortSignature>,
  pub(crate) outputs: Vec<PortSignature>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct RawPorts {
  pub(crate) inputs: HashSet<ConnectionTargetDefinition>,
  pub(crate) outputs: HashSet<ConnectionTargetDefinition>,
}

impl From<&ComponentModel> for ComponentSignature {
  fn from(v: &ComponentModel) -> Self {
    ComponentSignature {
      name: v.name.clone(),
      inputs: v.inputs.clone(),
      outputs: v.outputs.clone(),
    }
  }
}
