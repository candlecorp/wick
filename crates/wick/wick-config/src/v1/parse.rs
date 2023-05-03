use std::collections::HashMap;
use std::str::FromStr;

use flow_expression_parser::ast;
use serde::Deserialize;
use serde_json::Value;

use crate::error::ManifestError;
use crate::{v1, Error};

type Result<T> = std::result::Result<T, Error>;

/// The reserved identifier representing an as-of-yet-undetermined default value.
const DEFAULT_ID: &str = "<>";

pub(crate) fn parse_target(s: &str) -> Result<(String, Option<&str>)> {
  Ok(flow_expression_parser::parse::v1::parse_target(s)?)
}

pub(crate) fn parse_connection_target(s: &str) -> Result<v1::ConnectionTargetDefinition> {
  let (t_ref, t_port) = parse_target(s)?;
  Ok(v1::ConnectionTargetDefinition {
    instance: t_ref,
    port: t_port.unwrap_or(DEFAULT_ID).to_owned(),
    data: Default::default(),
  })
}

pub(crate) fn parse_connection(s: &str) -> Result<v1::ConnectionDefinition> {
  let (from, to) = flow_expression_parser::parse::v1::parse_connection_expression(s)?.into_parts();
  let (from, to) = (from.into_parts(), to.into_parts());
  Ok(v1::ConnectionDefinition {
    from: v1::ConnectionTargetDefinition {
      instance: from.0.to_string(),
      port: from.1,
      data: from.2,
    },
    to: v1::ConnectionTargetDefinition {
      instance: to.0.to_string(),
      port: to.1,
      data: to.2,
    },
  })
}

impl TryFrom<(String, String, Option<HashMap<String, Value>>)> for v1::ConnectionTargetDefinition {
  type Error = Error;

  fn try_from(value: (String, String, Option<HashMap<String, Value>>)) -> Result<Self> {
    Ok(Self {
      instance: value.0,
      port: value.1,
      data: value.2,
    })
  }
}

impl FromStr for crate::v1::ConnectionDefinition {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    parse_connection(s)
  }
}

impl FromStr for crate::v1::ConnectionTargetDefinition {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    parse_connection_target(s)
  }
}

pub(crate) fn vec_connection<'de, D>(deserializer: D) -> std::result::Result<Vec<crate::v1::FlowExpression>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct Visitor;
  impl<'de> serde::de::Visitor<'de> for Visitor {
    type Value = Vec<crate::v1::FlowExpression>;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "a list of connections")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(
      self,
      mut seq: A,
    ) -> std::result::Result<Vec<crate::v1::FlowExpression>, A::Error> {
      let mut v = vec![];
      while let Some(thing) = seq.next_element::<serde_value::Value>()? {
        let result = match thing {
          serde_value::Value::String(s) => ast::FlowExpression::from_str(&s)
            .map_err(|e| serde::de::Error::custom(e.to_string()))?
            .try_into()
            .map_err(|e: ManifestError| serde::de::Error::custom(e.to_string()))?,
          serde_value::Value::Map(map) => {
            crate::v1::FlowExpression::deserialize(serde_value::ValueDeserializer::new(serde_value::Value::Map(map)))?
          }
          _ => {
            return Err(serde::de::Error::invalid_type(
              serde::de::Unexpected::Other("other"),
              &self,
            ))
          }
        };
        v.push(result);
      }
      Ok(v)
    }
  }

  deserializer.deserialize_seq(Visitor)
}

pub(crate) fn connection_target_shortform<'de, D>(
  deserializer: D,
) -> std::result::Result<crate::v1::ConnectionTargetDefinition, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct ConnectionTargetVisitor;

  impl<'de> serde::de::Visitor<'de> for ConnectionTargetVisitor {
    type Value = crate::v1::ConnectionTargetDefinition;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a connection target definition")
    }

    fn visit_str<E>(self, s: &str) -> std::result::Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      crate::v1::ConnectionTargetDefinition::from_str(s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }

    fn visit_map<A>(self, map: A) -> std::result::Result<Self::Value, A::Error>
    where
      A: serde::de::MapAccess<'de>,
    {
      crate::v1::ConnectionTargetDefinition::deserialize(serde::de::value::MapAccessDeserializer::new(map))
    }
  }

  deserializer.deserialize_any(ConnectionTargetVisitor)
}

pub(crate) fn component_operation_syntax<'de, D>(
  deserializer: D,
) -> std::result::Result<crate::v1::ComponentOperationExpression, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct ComponentOperationExpressionVisitor;

  impl<'de> serde::de::Visitor<'de> for ComponentOperationExpressionVisitor {
    type Value = crate::v1::ComponentOperationExpression;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a connection target definition")
    }

    fn visit_str<E>(self, s: &str) -> std::result::Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      crate::v1::ComponentOperationExpression::from_str(s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }

    fn visit_map<A>(self, map: A) -> std::result::Result<Self::Value, A::Error>
    where
      A: serde::de::MapAccess<'de>,
    {
      crate::v1::ComponentOperationExpression::deserialize(serde::de::value::MapAccessDeserializer::new(map))
    }
  }

  deserializer.deserialize_any(ComponentOperationExpressionVisitor)
}

impl FromStr for v1::ComponentOperationExpression {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    let mut parts = s.split("::");

    let id = parts
      .next()
      .ok_or_else(|| crate::Error::InvalidOperationExpression(s.to_owned()))?
      .to_owned();
    let operation = parts
      .next()
      .ok_or_else(|| crate::Error::InvalidOperationExpression(s.to_owned()))?
      .to_owned();

    Ok(Self {
      name: operation,
      component: crate::v1::ComponentDefinition::ComponentReference(crate::v1::ComponentReference { id }),
    })
  }
}

impl Default for v1::ComponentDefinition {
  fn default() -> Self {
    Self::ComponentReference(crate::v1::ComponentReference {
      id: "<anonymous>".to_owned(),
    })
  }
}

pub(crate) fn component_shortform<'de, D>(
  deserializer: D,
) -> std::result::Result<crate::v1::ComponentDefinition, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct Visitor;
  impl<'de> serde::de::Visitor<'de> for Visitor {
    type Value = crate::v1::ComponentDefinition;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(
        f,
        "a component definition structure or path pointing to a WebAssembly module"
      )
    }

    fn visit_str<E>(self, s: &str) -> std::result::Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Ok(crate::v1::ComponentDefinition::ComponentReference(
        crate::v1::ComponentReference { id: s.to_owned() },
      ))
    }

    fn visit_map<A>(self, map: A) -> std::result::Result<Self::Value, A::Error>
    where
      A: serde::de::MapAccess<'de>,
    {
      crate::v1::ComponentDefinition::deserialize(serde::de::value::MapAccessDeserializer::new(map))
    }
  }

  deserializer.deserialize_map(Visitor)
}
