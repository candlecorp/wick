use regex::Regex;

use crate::{v0, Error};

lazy_static::lazy_static! {
    pub(crate) static ref CONNECTION_TARGET_REGEX_V0: Regex = Regex::new(&format!(r"^({}|{}|{}|{}|{}|[a-zA-Z][a-zA-Z0-9_]*)(?:\[(\w*)\])?$", DEFAULT_ID, SCHEMATIC_INPUT, SCHEMATIC_OUTPUT, NS_LINK, CORE_ID)).unwrap();
}

pub(crate) static CONNECTION_SEPARATOR: &str = "=>";

/// The reserved identifier representing an as-of-yet-undetermined default value.
const DEFAULT_ID: &str = "<>";
/// The reserved reference name for schematic input. Used in schematic manifests to denote schematic input.
pub const SCHEMATIC_INPUT: &str = "<input>";
/// The reserved reference name for schematic output. Used in schematic manifests to denote schematic output.
pub const SCHEMATIC_OUTPUT: &str = "<output>";
/// The reserved reference name for a namespace link. Used in schematic manifests to pass a collection to a port by its namespace.
pub const NS_LINK: &str = "<link>";
/// The reserved port name to use when sending an asynchronous error from a component.
pub const COMPONENT_ERROR: &str = "<error>";
/// The reserved namespace for references to internal schematics.
pub const SELF_NAMESPACE: &str = "self";
/// The reserved name for components that send static data.
pub static SENDER_ID: &str = "core::sender";
/// The reserved name for data that Wasmflow injects itself.
pub static CORE_ID: &str = "<core>";
/// The name of SENDER's output port.
pub static SENDER_PORT: &str = "output";

type Result<T> = std::result::Result<T, Error>;

/// Parse a fully qualified component ID into its namespace & name parts.
pub fn parse_id(id: &str) -> Result<(&str, &str)> {
  if !id.contains("::") {
    Err(Error::ComponentIdError(id.to_owned()))
  } else {
    id.split_once("::")
      .ok_or_else(|| Error::ComponentIdError(id.to_owned()))
  }
}

pub(crate) fn parse_target_v0(s: &str) -> Result<(Option<&str>, Option<&str>)> {
  CONNECTION_TARGET_REGEX_V0.captures(s.trim()).map_or_else(
    || Err(Error::ConnectionTargetSyntax(s.to_owned())),
    |captures| {
      Ok((
        captures.get(1).map(|m| m.as_str().trim()),
        captures.get(2).map(|m| m.as_str().trim()),
      ))
    },
  )
}

pub(crate) fn parse_connection_target_v0(s: &str) -> Result<v0::ConnectionTargetDefinition> {
  let (t_ref, t_port) = parse_target_v0(s)?;
  Ok(v0::ConnectionTargetDefinition {
    instance: t_ref.unwrap_or(DEFAULT_ID).to_owned(),
    port: t_port.unwrap_or(DEFAULT_ID).to_owned(),
    data: None,
  })
}

fn parse_from_or_sender(from: &str, default_port: Option<&str>) -> Result<v0::ConnectionTargetDefinition> {
  match parse_target_v0(from) {
    Ok((from_ref, from_port)) => Ok(v0::ConnectionTargetDefinition {
      port: from_port
        .or(default_port)
        .ok_or_else(|| Error::NoDefaultPort(from.to_owned()))?
        .to_owned(),
      instance: match from_ref {
        Some(DEFAULT_ID) => SCHEMATIC_INPUT,
        Some(v) => v,
        None => return Err(Error::NoDefaultReference(from.to_owned())),
      }
      .to_owned(),
      data: None,
    }),
    // Validating JSON by parsing into a serde_json::Value is recommended by the docs
    Err(_e) => match serde_json::from_str::<serde_json::Value>(from) {
      Ok(_) => Ok(v0::ConnectionTargetDefinition {
        instance: SENDER_ID.to_owned(),
        port: SENDER_PORT.to_owned(),
        data: Some(serde_json::from_str(from.trim()).map_err(|e| Error::InvalidSenderData(e.to_string()))?),
      }),
      Err(_e) => Err(Error::ConnectionTargetSyntax(from.to_owned())),
    },
  }
}

pub(crate) fn parse_connection_v0(s: &str) -> Result<v0::ConnectionDefinition> {
  let s = s.trim();
  s.split_once(CONNECTION_SEPARATOR).map_or_else(
    || Err(Error::ConnectionDefinitionSyntax(s.to_owned())),
    |(from, to)| {
      let (to_ref, to_port) = parse_target_v0(to)?;
      let from = parse_from_or_sender(from, to_port)?;
      let to = v0::ConnectionTargetDefinition {
        port: to_port
          .map(|s| s.to_owned())
          .or_else(|| Some(from.port.clone()))
          .ok_or_else(|| Error::NoDefaultPort(s.to_owned()))?,
        instance: match to_ref {
          Some(DEFAULT_ID) => SCHEMATIC_OUTPUT,
          Some(v) => v,
          None => return Err(Error::NoDefaultReference(s.to_owned())),
        }
        .to_owned(),
        data: None,
      };
      Ok(v0::ConnectionDefinition {
        from,
        to,
        default: None,
      })
    },
  )
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
    let parsed = parse_target_v0("<input>[foo]")?;
    assert_eq!(parsed, (Some("<input>"), Some("foo")));
    Ok(())
  }

  #[test_logger::test]
  fn test_basic() -> Result<()> {
    let parsed = parse_target_v0("ref[foo]")?;
    assert_eq!(parsed, (Some("ref"), Some("foo")));
    Ok(())
  }

  #[test_logger::test]
  fn test_default_with_port() -> Result<()> {
    let parsed = parse_target_v0("<>[foo]")?;
    assert_eq!(parsed, (Some(DEFAULT_ID), Some("foo")));
    Ok(())
  }

  #[test_logger::test]
  fn test_default() -> Result<()> {
    let parsed = parse_target_v0("<>")?;
    assert_eq!(parsed, (Some(DEFAULT_ID), None));
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_basic() -> Result<()> {
    let parsed = parse_connection_v0("ref1[in]=>ref2[out]")?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: v0::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "in".to_owned(),
          data: None,
        },
        to: v0::ConnectionTargetDefinition {
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
    let parsed = parse_connection_v0("5 => ref2[out]")?;
    let num = 5;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: v0::ConnectionTargetDefinition {
          instance: SENDER_ID.to_owned(),
          port: SENDER_PORT.to_owned(),
          data: Some(num.into()),
        },
        to: v0::ConnectionTargetDefinition {
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
    let parsed = parse_connection_v0("<>[in]=>ref2[out]")?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: v0::ConnectionTargetDefinition {
          instance: SCHEMATIC_INPUT.to_owned(),
          port: "in".to_owned(),
          data: None,
        },
        to: v0::ConnectionTargetDefinition {
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
    let parsed = parse_connection_v0("ref1[in]=><>[out]")?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: v0::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "in".to_owned(),
          data: None,
        },
        to: v0::ConnectionTargetDefinition {
          instance: SCHEMATIC_OUTPUT.to_owned(),
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
    let parsed = parse_connection_v0("ref1[port]=><>")?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: v0::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "port".to_owned(),
          data: None,
        },
        to: v0::ConnectionTargetDefinition {
          instance: SCHEMATIC_OUTPUT.to_owned(),
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
    let parsed = parse_connection_v0("<>=>ref1[port]")?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: v0::ConnectionTargetDefinition {
          instance: SCHEMATIC_INPUT.to_owned(),
          port: "port".to_owned(),
          data: None,
        },
        to: v0::ConnectionTargetDefinition {
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
    let parsed = parse_connection_v0(r#""default"=>ref1[port]"#)?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: v0::ConnectionTargetDefinition {
          instance: SENDER_ID.to_owned(),
          port: SENDER_PORT.to_owned(),
          data: Some(Value::from_str(r#""default""#)?),
        },
        to: v0::ConnectionTargetDefinition {
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
    let parsed = parse_connection_v0(r#""1234512345" => <>[output]"#)?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: v0::ConnectionTargetDefinition {
          instance: SENDER_ID.to_owned(),
          port: SENDER_PORT.to_owned(),
          data: Some(Value::from_str(r#""1234512345""#)?),
        },
        to: v0::ConnectionTargetDefinition {
          instance: SCHEMATIC_OUTPUT.to_owned(),
          port: "output".to_owned(),
          data: None,
        },
        default: None
      }
    );
    Ok(())
  }
}
