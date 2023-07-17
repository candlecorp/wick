use std::str::FromStr;

use flow_expression_parser::ast::{self};
use serde::Deserialize;

use crate::error::ManifestError;
use crate::{v1, Error};

type Result<T> = std::result::Result<T, Error>;

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

pub(crate) fn vec_component_operation<'de, D>(
  deserializer: D,
) -> std::result::Result<Vec<crate::v1::ComponentOperationExpression>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct Visitor;
  impl<'de> serde::de::Visitor<'de> for Visitor {
    type Value = Vec<crate::v1::ComponentOperationExpression>;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "a list of operations")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(
      self,
      mut seq: A,
    ) -> std::result::Result<Vec<crate::v1::ComponentOperationExpression>, A::Error> {
      let mut v = vec![];
      while let Some(thing) = seq.next_element::<serde_value::Value>()? {
        let result = match thing {
          serde_value::Value::String(s) => crate::v1::ComponentOperationExpression::from_str(&s)
            .map_err(|e| serde::de::Error::custom(e.to_string()))?,
          serde_value::Value::Map(map) => crate::v1::ComponentOperationExpression::deserialize(
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

  deserializer.deserialize_seq(Visitor)
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
      with: None,
      timeout: None,
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

    fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Ok(crate::v1::ComponentDefinition::ComponentReference(
        crate::v1::ComponentReference { id: v },
      ))
    }

    fn visit_map<A>(self, map: A) -> std::result::Result<Self::Value, A::Error>
    where
      A: serde::de::MapAccess<'de>,
    {
      crate::v1::ComponentDefinition::deserialize(serde::de::value::MapAccessDeserializer::new(map))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> std::result::Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Ok(crate::v1::ComponentDefinition::ComponentReference(
        crate::v1::ComponentReference { id: v.to_owned() },
      ))
    }
  }

  deserializer.deserialize_any(Visitor)
}
