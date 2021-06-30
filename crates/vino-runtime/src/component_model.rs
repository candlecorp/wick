use crate::components::{
  Inputs,
  Outputs,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentModel {
  /// The fully qualified name, including the namespace.
  pub id: String,
  /// The name of the component
  pub name: String,
  pub inputs: Inputs,
  pub outputs: Outputs,
}

pub fn format_id(namespace: &str, name: &str) -> String {
  format!("{}::{}", namespace, name)
}
