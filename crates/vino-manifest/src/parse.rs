use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
  v0,
  Error,
};

lazy_static::lazy_static! {
    pub(crate) static ref CONNECTION_TARGET_REGEX_V0: Regex = Regex::new(&format!(r"^({}|{}|{}|\w*)(?:\[(\w*)\])?$", DEFAULT_ID, SCHEMATIC_INPUT, SCHEMATIC_OUTPUT)).unwrap();
}

pub(crate) static CONNECTION_SEPARATOR: &str = "=>";

/// The reserved identifier representing an as-of-yet-undetermined default value.
const DEFAULT_ID: &str = "<>";
/// The reserved reference name for schematic input. Used in schematic manifests to denote schematic input.
pub const SCHEMATIC_INPUT: &str = "<input>";
/// The reserved reference name for schematic output. Used in schematic manifests to denote schematic output.
pub const SCHEMATIC_OUTPUT: &str = "<output>";
/// The reserved port name to use when sending an asynchronous error from a component.
pub const COMPONENT_ERROR: &str = "<error>";
/// The reserved namespace for references to internal schematics.
pub const SELF_NAMESPACE: &str = "self";

type Result<T> = std::result::Result<T, Error>;

/// Parse a fully qualified component ID into its namespace & name parts
pub fn parse_id(id: &str) -> Result<(&str, &str)> {
  if !id.contains("::") {
    Err(Error::ComponentIdError(id.to_owned()))
  } else {
    id.rsplit_once("::")
      .ok_or_else(|| Error::ComponentIdError(id.to_owned()))
  }
}

pub(crate) fn parse_target_v0(s: &str) -> Result<(Option<&str>, Option<&str>)> {
  CONNECTION_TARGET_REGEX_V0.captures(s.trim()).map_or_else(
    || Err(Error::ConnectionTargetSyntax(s.to_owned())),
    |captures| {
      Ok((
        captures.get(1).map(|m| m.as_str()),
        captures.get(2).map(|m| m.as_str()),
      ))
    },
  )
}

pub(crate) fn parse_connection_target_v0(s: &str) -> Result<v0::ConnectionTargetDefinition> {
  let (t_ref, t_port) = parse_target_v0(s)?;
  Ok(v0::ConnectionTargetDefinition {
    instance: t_ref.unwrap_or(DEFAULT_ID).to_owned(),
    port: t_port.unwrap_or(DEFAULT_ID).to_owned(),
  })
}

fn parse_from_or_default(
  from: &str,
  default_port: Option<&str>,
) -> Result<(Option<v0::ConnectionTargetDefinition>, Option<String>)> {
  match parse_target_v0(from) {
    Ok((from_ref, from_port)) => Ok((
      Some(v0::ConnectionTargetDefinition {
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
      }),
      None,
    )),
    // Validating JSON by parsing into a serde_json::Value is recommended by the docs
    Err(_e) => match serde_json::from_str::<serde_json::Value>(from) {
      Ok(_) => Ok((None, Some(from.to_owned()))),
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
      let (from, default) = parse_from_or_default(from, to_port)?;
      let to = Some(v0::ConnectionTargetDefinition {
        port: to_port
          .map(|s| s.to_owned())
          .or_else(|| from.as_ref().map(|p| p.port.clone()))
          .ok_or_else(|| Error::NoDefaultPort(s.to_owned()))?,
        instance: match to_ref {
          Some(DEFAULT_ID) => SCHEMATIC_OUTPUT,
          Some(v) => v,
          None => return Err(Error::NoDefaultReference(s.to_owned())),
        }
        .to_owned(),
      });
      Ok(v0::ConnectionDefinition { from, to, default })
    },
  )
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use pretty_assertions::assert_eq;

  use super::*;
  #[test_env_log::test]
  fn test_reserved() -> Result<()> {
    let parsed = parse_target_v0("<input>[foo]")?;
    assert_eq!(parsed, (Some("<input>"), Some("foo")));
    Ok(())
  }

  #[test_env_log::test]
  fn test_basic() -> Result<()> {
    let parsed = parse_target_v0("ref[foo]")?;
    assert_eq!(parsed, (Some("ref"), Some("foo")));
    Ok(())
  }

  #[test_env_log::test]
  fn test_default_with_port() -> Result<()> {
    let parsed = parse_target_v0("<>[foo]")?;
    assert_eq!(parsed, (Some(DEFAULT_ID), Some("foo")));
    Ok(())
  }

  #[test_env_log::test]
  fn test_default() -> Result<()> {
    let parsed = parse_target_v0("<>")?;
    assert_eq!(parsed, (Some(DEFAULT_ID), None));
    Ok(())
  }

  #[test_env_log::test]
  fn test_connection_basic() -> Result<()> {
    let parsed = parse_connection_v0("ref1[in]=>ref2[out]")?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: Some(v0::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "in".to_owned()
        }),
        to: Some(v0::ConnectionTargetDefinition {
          instance: "ref2".to_owned(),
          port: "out".to_owned()
        }),
        default: None
      }
    );
    Ok(())
  }

  #[test_env_log::test]
  fn test_connection_default_input_named_port() -> Result<()> {
    let parsed = parse_connection_v0("<>[in]=>ref2[out]")?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: Some(v0::ConnectionTargetDefinition {
          instance: SCHEMATIC_INPUT.to_owned(),
          port: "in".to_owned()
        }),
        to: Some(v0::ConnectionTargetDefinition {
          instance: "ref2".to_owned(),
          port: "out".to_owned()
        }),
        default: None
      }
    );
    Ok(())
  }

  #[test_env_log::test]
  fn test_connection_default_output_named_port() -> Result<()> {
    let parsed = parse_connection_v0("ref1[in]=><>[out]")?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: Some(v0::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "in".to_owned()
        }),
        to: Some(v0::ConnectionTargetDefinition {
          instance: SCHEMATIC_OUTPUT.to_owned(),
          port: "out".to_owned()
        }),
        default: None
      }
    );
    Ok(())
  }

  #[test_env_log::test]
  fn test_connection_default_output() -> Result<()> {
    let parsed = parse_connection_v0("ref1[port]=><>")?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: Some(v0::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "port".to_owned()
        }),
        to: Some(v0::ConnectionTargetDefinition {
          instance: SCHEMATIC_OUTPUT.to_owned(),
          port: "port".to_owned()
        }),
        default: None
      }
    );
    Ok(())
  }

  #[test_env_log::test]
  fn test_connection_default_input() -> Result<()> {
    let parsed = parse_connection_v0("<>=>ref1[port]")?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: Some(v0::ConnectionTargetDefinition {
          instance: SCHEMATIC_INPUT.to_owned(),
          port: "port".to_owned()
        }),
        to: Some(v0::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "port".to_owned()
        }),
        default: None
      }
    );
    Ok(())
  }

  #[test_env_log::test]
  fn test_connection_with_default_data() -> Result<()> {
    let parsed = parse_connection_v0(r#""default"=>ref1[port]"#)?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: None,
        to: Some(v0::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "port".to_owned()
        }),
        default: Some(r#""default""#.to_owned())
      }
    );
    Ok(())
  }
}
