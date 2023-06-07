use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct LocationReference(
  #[serde(deserialize_with = "serde_with_expand_env::with_expand_envs")] pub(super) String,
);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub(crate) struct Glob(pub(super) String);

#[allow(clippy::from_over_into)]
impl Into<String> for super::TypeSignature {
  fn into(self) -> String {
    let ty: wick_interface_types::Type = self.try_into().unwrap();
    ty.to_string()
  }
}

#[allow(clippy::from_over_into)]
impl Into<String> for super::ConnectionDefinition {
  fn into(self) -> String {
    let ty: flow_expression_parser::ast::ConnectionExpression = self.try_into().unwrap();
    ty.to_string()
  }
}
