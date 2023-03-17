use std::collections::HashMap;
use std::str::FromStr;

use serde::Deserialize;

use crate::app_config::BoundResource;
use crate::{v1, BoundComponent, Error};

type Result<T> = std::result::Result<T, Error>;

/// The reserved identifier representing an as-of-yet-undetermined default value.
const DEFAULT_ID: &str = "<>";

pub(crate) fn parse_target(s: &str) -> Result<(Option<&str>, Option<&str>)> {
  Ok(flow_expression_parser::parse::v1::parse_target(s)?)
}

pub(crate) fn parse_connection_target(s: &str) -> Result<v1::ConnectionTargetDefinition> {
  let (t_ref, t_port) = parse_target(s)?;
  Ok(v1::ConnectionTargetDefinition {
    instance: t_ref.unwrap_or(DEFAULT_ID).to_owned(),
    port: t_port.unwrap_or(DEFAULT_ID).to_owned(),
    data: None,
  })
}

pub(crate) fn parse_connection(s: &str) -> Result<v1::ConnectionDefinition> {
  let (from, to, data) = flow_expression_parser::parse::v1::parse_connection(s)?;
  Ok(v1::ConnectionDefinition {
    from: from.try_into()?,
    to: to.try_into()?,
    default: data,
  })
}

impl TryFrom<(String, String, Option<serde_json::Value>)> for v1::ConnectionTargetDefinition {
  type Error = Error;

  fn try_from(value: (String, String, Option<serde_json::Value>)) -> Result<Self> {
    Ok(Self {
      instance: value.0,
      port: value.1,
      data: value.2,
    })
  }
}

impl FromStr for crate::v1::InstanceDefinition {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    Ok(Self {
      id: s.to_owned(),
      config: None,
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

pub(crate) fn map_component_def<'de, D>(
  deserializer: D,
) -> std::result::Result<HashMap<String, crate::v1::InstanceDefinition>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct InstanceDefinitionVisitor;
  impl<'de> serde::de::Visitor<'de> for InstanceDefinitionVisitor {
    type Value = HashMap<String, crate::v1::InstanceDefinition>;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "a map of instances to their components")
    }

    fn visit_map<M>(self, mut access: M) -> std::result::Result<Self::Value, M::Error>
    where
      M: serde::de::MapAccess<'de>,
    {
      let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));

      while let Some((key, value)) = access.next_entry::<String, serde_value::Value>()? {
        let result = match value {
          serde_value::Value::String(s) => {
            crate::v1::InstanceDefinition::from_str(&s).map_err(|e| serde::de::Error::custom(e.to_string()))?
          }
          serde_value::Value::Map(map) => crate::v1::InstanceDefinition::deserialize(
            serde_value::ValueDeserializer::new(serde_value::Value::Map(map)),
          )?,
          _ => {
            return Err(serde::de::Error::invalid_type(
              serde::de::Unexpected::Other("other"),
              &self,
            ))
          }
        };

        map.insert(key, result);
      }

      Ok(map)
    }
  }

  deserializer.deserialize_map(InstanceDefinitionVisitor)
}

pub(crate) fn vec_connection<'de, D>(
  deserializer: D,
) -> std::result::Result<Vec<crate::v1::ConnectionDefinition>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct ConnectionDefVisitor;
  impl<'de> serde::de::Visitor<'de> for ConnectionDefVisitor {
    type Value = Vec<crate::v1::ConnectionDefinition>;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "a list of connections")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(
      self,
      mut seq: A,
    ) -> std::result::Result<Vec<crate::v1::ConnectionDefinition>, A::Error> {
      let mut v = vec![];
      while let Some(thing) = seq.next_element::<serde_value::Value>()? {
        let result = match thing {
          serde_value::Value::String(s) => {
            crate::v1::ConnectionDefinition::from_str(&s).map_err(|e| serde::de::Error::custom(e.to_string()))?
          }
          serde_value::Value::Map(map) => crate::v1::ConnectionDefinition::deserialize(
            serde_value::ValueDeserializer::new(serde_value::Value::Map(map)),
          )?,
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

  deserializer.deserialize_seq(ConnectionDefVisitor)
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

impl FromStr for crate::v1::ComponentOperationExpression {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    let mut parts = s.split("::");

    let operation = parts
      .next()
      .ok_or_else(|| crate::Error::InvalidOperationExpression(s.to_owned()))?
      .to_owned();
    let id = parts
      .next()
      .ok_or_else(|| crate::Error::InvalidOperationExpression(s.to_owned()))?
      .to_owned();

    Ok(Self {
      operation,
      component: crate::v1::ComponentDefinition::ComponentReference(crate::v1::ComponentReference { id }),
    })
  }
}

impl Default for crate::v1::ComponentDefinition {
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

impl From<v1::ComponentBinding> for BoundComponent {
  fn from(value: v1::ComponentBinding) -> Self {
    Self {
      id: value.name,
      kind: value.component.into(),
    }
  }
}

impl From<v1::ResourceBinding> for BoundResource {
  fn from(value: v1::ResourceBinding) -> Self {
    Self {
      id: value.name,
      kind: value.resource.into(),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use anyhow::Result;
  use flow_expression_parser::parse;
  use pretty_assertions::assert_eq;
  use serde_json::Value;

  use super::*;
  #[test_logger::test]
  fn test_reserved() -> Result<()> {
    let parsed = parse_target("input.foo")?;
    assert_eq!(parsed, (Some("input"), Some("foo")));
    Ok(())
  }

  #[test_logger::test]
  fn test_basic() -> Result<()> {
    let parsed = parse_target("ref.foo")?;
    assert_eq!(parsed, (Some("ref"), Some("foo")));
    Ok(())
  }

  #[test_logger::test]
  fn test_default_with_port() -> Result<()> {
    let parsed = parse_target("<>.foo")?;
    assert_eq!(parsed, (Some(DEFAULT_ID), Some("foo")));
    Ok(())
  }

  #[test_logger::test]
  fn test_default() -> Result<()> {
    let parsed = parse_target("<>")?;
    assert_eq!(parsed, (Some(DEFAULT_ID), None));
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_basic() -> Result<()> {
    let parsed = parse_connection("ref1.in -> ref2.out")?;
    assert_eq!(
      parsed,
      v1::ConnectionDefinition {
        from: v1::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "in".to_owned(),
          data: None,
        },
        to: v1::ConnectionTargetDefinition {
          instance: "ref2".to_owned(),
          port: "out".to_owned(),
          data: None,
        },
        default: None
      }
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_bare_num_default() -> Result<()> {
    let parsed = parse_connection("5 -> ref2.out")?;
    let num = 5;
    assert_eq!(
      parsed,
      v1::ConnectionDefinition {
        from: v1::ConnectionTargetDefinition {
          instance: parse::SENDER_ID.to_owned(),
          port: parse::SENDER_PORT.to_owned(),
          data: Some(num.into()),
        },
        to: v1::ConnectionTargetDefinition {
          instance: "ref2".to_owned(),
          port: "out".to_owned(),
          data: None,
        },
        default: None
      }
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_default_input_named_port() -> Result<()> {
    let parsed = parse_connection("<>.in->ref2.out")?;
    assert_eq!(
      parsed,
      v1::ConnectionDefinition {
        from: v1::ConnectionTargetDefinition {
          instance: parse::SCHEMATIC_INPUT.to_owned(),
          port: "in".to_owned(),
          data: None,
        },
        to: v1::ConnectionTargetDefinition {
          instance: "ref2".to_owned(),
          port: "out".to_owned(),
          data: None,
        },
        default: None
      }
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_default_output_named_port() -> Result<()> {
    let parsed = parse_connection("ref1.in-><>.out")?;
    assert_eq!(
      parsed,
      v1::ConnectionDefinition {
        from: v1::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "in".to_owned(),
          data: None,
        },
        to: v1::ConnectionTargetDefinition {
          instance: parse::SCHEMATIC_OUTPUT.to_owned(),
          port: "out".to_owned(),
          data: None,
        },
        default: None
      }
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_default_output() -> Result<()> {
    let parsed = parse_connection("ref1.port-><>")?;
    assert_eq!(
      parsed,
      v1::ConnectionDefinition {
        from: v1::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "port".to_owned(),
          data: None,
        },
        to: v1::ConnectionTargetDefinition {
          instance: parse::SCHEMATIC_OUTPUT.to_owned(),
          port: "port".to_owned(),
          data: None,
        },
        default: None
      }
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_default_input() -> Result<()> {
    let parsed = parse_connection("<> -> ref1.port")?;
    assert_eq!(
      parsed,
      v1::ConnectionDefinition {
        from: v1::ConnectionTargetDefinition {
          instance: parse::SCHEMATIC_INPUT.to_owned(),
          port: "port".to_owned(),
          data: None,
        },
        to: v1::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "port".to_owned(),
          data: None,
        },
        default: None
      }
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_with_default_data() -> Result<()> {
    let parsed = parse_connection(r#""default"->ref1.port"#)?;
    assert_eq!(
      parsed,
      v1::ConnectionDefinition {
        from: v1::ConnectionTargetDefinition {
          instance: parse::SENDER_ID.to_owned(),
          port: parse::SENDER_PORT.to_owned(),
          data: Some(Value::from_str(r#""default""#)?),
        },
        to: v1::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "port".to_owned(),
          data: None,
        },
        default: None
      }
    );
    Ok(())
  }

  #[test_logger::test]
  fn regression_1() -> Result<()> {
    let parsed = parse_connection(r#""1234512345" -> <>.output"#)?;
    assert_eq!(
      parsed,
      v1::ConnectionDefinition {
        from: v1::ConnectionTargetDefinition {
          instance: parse::SENDER_ID.to_owned(),
          port: parse::SENDER_PORT.to_owned(),
          data: Some(Value::from_str(r#""1234512345""#)?),
        },
        to: v1::ConnectionTargetDefinition {
          instance: parse::SCHEMATIC_OUTPUT.to_owned(),
          port: "output".to_owned(),
          data: None,
        },
        default: None
      }
    );
    Ok(())
  }
}
