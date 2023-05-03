use std::collections::HashMap;

use serde_json::Value;

use crate::{v0, Error};

/// The reserved identifier representing an as-of-yet-undetermined default value.
const DEFAULT_ID: &str = "<>";

type Result<T> = std::result::Result<T, Error>;

pub(crate) fn parse_target(s: &str) -> Result<(Option<&str>, Option<&str>)> {
  Ok(flow_expression_parser::parse::v0::parse_target(s)?)
}

pub(crate) fn parse_connection_target(s: &str) -> Result<v0::ConnectionTargetDefinition> {
  let (t_ref, t_port) = parse_target(s)?;
  Ok(v0::ConnectionTargetDefinition {
    instance: t_ref.unwrap_or(DEFAULT_ID).to_owned(),
    port: t_port.unwrap_or(DEFAULT_ID).to_owned(),
    data: Default::default(),
  })
}

pub(crate) fn parse_connection(s: &str) -> Result<v0::ConnectionDefinition> {
  let (from, to) = flow_expression_parser::parse::v0::parse_connection(s)?;
  Ok(v0::ConnectionDefinition {
    from: from.try_into()?,
    to: to.try_into()?,
  })
}

impl TryFrom<(String, String, Option<HashMap<String, Value>>)> for v0::ConnectionTargetDefinition {
  type Error = Error;

  fn try_from(value: (String, String, Option<HashMap<String, Value>>)) -> Result<Self> {
    Ok(Self {
      instance: value.0,
      port: value.1,
      data: value.2,
    })
  }
}

#[cfg(test)]
mod tests {

  use anyhow::Result;
  use flow_expression_parser::parse;
  use pretty_assertions::assert_eq;

  use super::*;
  #[test_logger::test]
  fn test_reserved() -> Result<()> {
    let parsed = parse_target("<input>[foo]")?;
    assert_eq!(parsed, (Some("<input>"), Some("foo")));
    Ok(())
  }

  #[test_logger::test]
  fn test_basic() -> Result<()> {
    let parsed = parse_target("ref[foo]")?;
    assert_eq!(parsed, (Some("ref"), Some("foo")));
    Ok(())
  }

  #[test_logger::test]
  fn test_default_with_port() -> Result<()> {
    let parsed = parse_target("<>[foo]")?;
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
    let parsed = parse_connection("ref1[in]=>ref2[out]")?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: v0::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "in".to_owned(),
          data: Default::default(),
        },
        to: v0::ConnectionTargetDefinition {
          instance: "ref2".to_owned(),
          port: "out".to_owned(),
          data: Default::default(),
        },
      }
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_bare_num_default() -> Result<()> {
    let parsed = parse_connection("5 => ref2[out]")?;
    let num = 5;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: v0::ConnectionTargetDefinition {
          instance: parse::SENDER_ID.to_owned(),
          port: parse::SENDER_PORT.to_owned(),
          data: Some(HashMap::from([("default".into(), num.into())])),
        },
        to: v0::ConnectionTargetDefinition {
          instance: "ref2".to_owned(),
          port: "out".to_owned(),
          data: Default::default(),
        },
      }
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_default_input_named_port() -> Result<()> {
    let parsed = parse_connection("<>[in]=>ref2[out]")?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: v0::ConnectionTargetDefinition {
          instance: parse::SCHEMATIC_INPUT.to_owned(),
          port: "in".to_owned(),
          data: Default::default(),
        },
        to: v0::ConnectionTargetDefinition {
          instance: "ref2".to_owned(),
          port: "out".to_owned(),
          data: Default::default(),
        },
      }
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_default_output_named_port() -> Result<()> {
    let parsed = parse_connection("ref1[in]=><>[out]")?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: v0::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "in".to_owned(),
          data: Default::default(),
        },
        to: v0::ConnectionTargetDefinition {
          instance: parse::SCHEMATIC_OUTPUT.to_owned(),
          port: "out".to_owned(),
          data: Default::default(),
        },
      }
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_default_output() -> Result<()> {
    let parsed = parse_connection("ref1[port]=><>")?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: v0::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "port".to_owned(),
          data: Default::default(),
        },
        to: v0::ConnectionTargetDefinition {
          instance: parse::SCHEMATIC_OUTPUT.to_owned(),
          port: "port".to_owned(),
          data: Default::default(),
        },
      }
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_default_input() -> Result<()> {
    let parsed = parse_connection("<>=>ref1[port]")?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: v0::ConnectionTargetDefinition {
          instance: parse::SCHEMATIC_INPUT.to_owned(),
          port: "port".to_owned(),
          data: Default::default(),
        },
        to: v0::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "port".to_owned(),
          data: Default::default(),
        },
      }
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_with_default_data() -> Result<()> {
    let parsed = parse_connection(r#""default"=>ref1[port]"#)?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: v0::ConnectionTargetDefinition {
          instance: parse::SENDER_ID.to_owned(),
          port: parse::SENDER_PORT.to_owned(),
          data: Some(HashMap::from([("default".into(), "default".into())])),
        },
        to: v0::ConnectionTargetDefinition {
          instance: "ref1".to_owned(),
          port: "port".to_owned(),
          data: Default::default(),
        },
      }
    );
    Ok(())
  }

  #[test_logger::test]
  fn regression_1() -> Result<()> {
    let parsed = parse_connection(r#""1234512345" => <>[output]"#)?;
    assert_eq!(
      parsed,
      v0::ConnectionDefinition {
        from: v0::ConnectionTargetDefinition {
          instance: parse::SENDER_ID.to_owned(),
          port: parse::SENDER_PORT.to_owned(),
          data: Some(HashMap::from([("default".into(), "1234512345".into())])),
        },
        to: v0::ConnectionTargetDefinition {
          instance: parse::SCHEMATIC_OUTPUT.to_owned(),
          port: "output".to_owned(),
          data: Default::default(),
        },
      }
    );
    Ok(())
  }
}
