use std::collections::HashSet;

use crate::dev::prelude::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct ComponentModel {
  inner: ComponentSignature,
}

impl ComponentModel {
  #[allow(unused)]
  pub(crate) fn name(&self) -> &str {
    &self.inner.name
  }
  pub(crate) fn name_owned(&self) -> String {
    self.inner.name.clone()
  }
  pub(crate) fn inputs(&self) -> &TypeMap {
    &self.inner.inputs
  }
  pub(crate) fn outputs(&self) -> &TypeMap {
    &self.inner.outputs
  }
  pub(crate) fn get_input<T: AsRef<str>>(&self, field: T) -> Option<&TypeSignature> {
    self.inner.inputs.get(field.as_ref())
  }
  #[allow(unused)]
  pub(crate) fn get_input_names(&self) -> Vec<String> {
    self.inner.inputs.names()
  }
  pub(crate) fn get_output<T: AsRef<str>>(&self, field: T) -> Option<&TypeSignature> {
    self.inner.outputs.get(field.as_ref())
  }
  pub(crate) fn get_output_names(&self) -> Vec<String> {
    self.inner.outputs.names()
  }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct RawPorts {
  pub(crate) inputs: HashSet<ConnectionTargetDefinition>,
  pub(crate) outputs: HashSet<ConnectionTargetDefinition>,
}

impl From<&ComponentModel> for ComponentSignature {
  fn from(v: &ComponentModel) -> Self {
    v.inner.clone()
  }
}

impl From<ComponentModel> for ComponentSignature {
  fn from(v: ComponentModel) -> Self {
    v.inner
  }
}

impl From<&ComponentSignature> for ComponentModel {
  fn from(v: &ComponentSignature) -> Self {
    Self { inner: v.clone() }
  }
}

impl From<ComponentSignature> for ComponentModel {
  fn from(v: ComponentSignature) -> Self {
    Self { inner: v }
  }
}

pub(crate) trait WithSignature<T> {
  fn get_signature(&self, name: Option<String>) -> T;
}

impl ComponentModel {
  pub(crate) fn get_signature(&self) -> ComponentSignature {
    self.inner.clone()
  }
}
