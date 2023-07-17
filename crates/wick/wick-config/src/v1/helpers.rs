use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct LocationReference(pub(super) String);

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

#[allow(clippy::from_over_into)]
impl Into<String> for super::ComponentReference {
  fn into(self) -> String {
    self.id
  }
}

pub(super) fn serialize_component_expression<S>(
  value: &super::ComponentOperationExpression,
  s: S,
) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let name = match &value.component {
    super::ComponentDefinition::ComponentReference(r) => &r.id,
    _ => return value.serialize(s),
  };

  if value.with.is_none() {
    s.serialize_str(&format!("{}::{}", name, value.name))
  } else {
    let mut m = if value.timeout.is_some() {
      s.serialize_map(Some(4))?
    } else {
      s.serialize_map(Some(3))?
    };
    m.serialize_entry("name", &value.name)?;
    m.serialize_entry("component", &name)?;
    m.serialize_entry("with", &value.with)?;
    if let Some(timeout) = &value.timeout {
      m.serialize_entry("timeout", timeout)?;
    }
    m.end()
  }
}
