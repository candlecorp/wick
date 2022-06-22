use std::collections::HashMap;
use std::str::FromStr;

use regex::Regex;
use serde::Deserialize;

use crate::{parse, v1, Error};

lazy_static::lazy_static! {
    pub(crate) static ref CONNECTION_TARGET_REGEX: Regex = Regex::new(&format!(r"^({}|{}|{}|{}|{}|[a-zA-Z][a-zA-Z0-9_]+)(?:\.(\w+))?$", DEFAULT_ID, parse::SCHEMATIC_INPUT, parse::SCHEMATIC_OUTPUT, parse::NS_LINK, parse::CORE_ID)).unwrap();
}

pub(crate) static CONNECTION_SEPARATOR: &str = "->";

/// The reserved identifier representing an as-of-yet-undetermined default value.
const DEFAULT_ID: &str = "<>";

type Result<T> = std::result::Result<T, Error>;

pub(crate) fn parse_target(s: &str) -> Result<(Option<&str>, Option<&str>)> {
  CONNECTION_TARGET_REGEX.captures(s.trim()).map_or_else(
    || Err(Error::ConnectionTargetSyntax(s.to_owned())),
    |captures| {
      Ok((
        captures.get(1).map(|m| m.as_str().trim()),
        captures.get(2).map(|m| m.as_str().trim()),
      ))
    },
  )
}

pub(crate) fn parse_connection_target(s: &str) -> Result<v1::ConnectionTargetDefinition> {
  let (t_ref, t_port) = parse_target(s)?;
  Ok(v1::ConnectionTargetDefinition {
    instance: t_ref.unwrap_or(DEFAULT_ID).to_owned(),
    port: t_port.unwrap_or(DEFAULT_ID).to_owned(),
    data: None,
  })
}

fn parse_from_or_sender(from: &str, default_port: Option<&str>) -> Result<v1::ConnectionTargetDefinition> {
  match parse_target(from) {
    Ok((from_ref, from_port)) => Ok(v1::ConnectionTargetDefinition {
      port: from_port
        .or(default_port)
        .ok_or_else(|| Error::NoDefaultPort(from.to_owned()))?
        .to_owned(),
      instance: match from_ref {
        Some(DEFAULT_ID) => parse::SCHEMATIC_INPUT,
        Some(v) => v,
        None => return Err(Error::NoDefaultReference(from.to_owned())),
      }
      .to_owned(),
      data: None,
    }),
    // Validating JSON by parsing into a serde_json::Value is recommended by the docs
    Err(_e) => match serde_json::from_str::<serde_json::Value>(from) {
      Ok(_) => Ok(v1::ConnectionTargetDefinition {
        instance: parse::SENDER_ID.to_owned(),
        port: parse::SENDER_PORT.to_owned(),
        data: Some(serde_json::from_str(from.trim()).map_err(|e| Error::InvalidSenderData(e.to_string()))?),
      }),
      Err(_e) => Err(Error::ConnectionTargetSyntax(from.to_owned())),
    },
  }
}

pub(crate) fn parse_connection(s: &str) -> Result<v1::ConnectionDefinition> {
  let s = s.trim();
  s.split_once(CONNECTION_SEPARATOR).map_or_else(
    || Err(Error::ConnectionDefinitionSyntax(s.to_owned())),
    |(from, to)| {
      let (to_ref, to_port) = parse_target(to.trim())?;
      let from = parse_from_or_sender(from.trim(), to_port)?;
      let to = v1::ConnectionTargetDefinition {
        port: to_port
          .map(|s| s.to_owned())
          .or_else(|| Some(from.port.clone()))
          .ok_or_else(|| Error::NoDefaultPort(s.to_owned()))?,
        instance: match to_ref {
          Some(DEFAULT_ID) => parse::SCHEMATIC_OUTPUT,
          Some(v) => v,
          None => return Err(Error::NoDefaultReference(s.to_owned())),
        }
        .to_owned(),
        data: None,
      };
      Ok(v1::ConnectionDefinition {
        from,
        to,
        default: None,
      })
    },
  )
}

impl FromStr for crate::v1::ComponentDefinition {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    Ok(Self {
      id: s.to_owned(),
      config: None,
    })
  }
}

impl FromStr for crate::v1::CollectionDefinition {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    Ok(Self {
      kind: v1::CollectionKind::WASM,
      reference: s.to_owned(),
      config: serde_json::Value::Null,
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
) -> std::result::Result<HashMap<String, crate::v1::ComponentDefinition>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct ComponentDefinitionVisitor;
  impl<'de> serde::de::Visitor<'de> for ComponentDefinitionVisitor {
    type Value = HashMap<String, crate::v1::ComponentDefinition>;
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
            crate::v1::ComponentDefinition::from_str(&s).map_err(|e| serde::de::Error::custom(e.to_string()))?
          }
          serde_value::Value::Map(map) => crate::v1::ComponentDefinition::deserialize(
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

  deserializer.deserialize_map(ComponentDefinitionVisitor)
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

pub(crate) fn collection_shortform<'de, D>(
  deserializer: D,
) -> std::result::Result<HashMap<String, crate::v1::CollectionDefinition>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct CollectionDefinitionVisitor;
  impl<'de> serde::de::Visitor<'de> for CollectionDefinitionVisitor {
    type Value = HashMap<String, crate::v1::CollectionDefinition>;
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
            crate::v1::CollectionDefinition::from_str(&s).map_err(|e| serde::de::Error::custom(e.to_string()))?
          }
          serde_value::Value::Map(map) => crate::v1::CollectionDefinition::deserialize(
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

  deserializer.deserialize_map(CollectionDefinitionVisitor)
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use anyhow::Result;
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
