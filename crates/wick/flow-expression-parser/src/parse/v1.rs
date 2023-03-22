use regex::Regex;

use crate::{parse, Error};

lazy_static::lazy_static! {
    pub(crate) static ref CONNECTION_TARGET_REGEX: Regex = Regex::new(&format!(r"^({}|{}|{}|{}|{}|[a-zA-Z][a-zA-Z0-9_]+)(?:\.(\w+))?$", DEFAULT_ID, parse::SCHEMATIC_INPUT, parse::SCHEMATIC_OUTPUT, parse::NS_LINK, parse::CORE_ID)).unwrap();
}

/// The separator in a connection between connection targets.
pub static CONNECTION_SEPARATOR: &str = "->";

/// The reserved identifier representing an as-of-yet-undetermined default value.
const DEFAULT_ID: &str = "<>";

type Result<T> = std::result::Result<T, Error>;

/// Parse a string as connection target pieces.
pub fn parse_target(s: &str) -> Result<(Option<&str>, Option<&str>)> {
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

/// Parse a connection target, injecting defaults where applicable.
pub fn parse_connection_target(s: &str) -> Result<(&str, &str)> {
  let (t_ref, t_port) = parse_target(s)?;
  Ok((t_ref.unwrap_or(DEFAULT_ID), t_port.unwrap_or(DEFAULT_ID)))
}

type ConnectionDefinitionParts = (String, String, Option<serde_json::Value>);

fn parse_from_or_sender(from: &str, default_port: Option<&str>) -> Result<ConnectionDefinitionParts> {
  match parse_target(from) {
    Ok((from_ref, from_port)) => Ok((
      match from_ref {
        Some(DEFAULT_ID) => parse::SCHEMATIC_INPUT,
        Some(v) => v,
        None => return Err(Error::NoDefaultReference(from.to_owned())),
      }
      .to_owned(),
      from_port
        .or(default_port)
        .ok_or_else(|| Error::NoDefaultPort(from.to_owned()))?
        .to_owned(),
      None,
    )),
    // Validating JSON by parsing into a serde_json::Value is recommended by the docs
    Err(_e) => match serde_json::from_str::<serde_json::Value>(from) {
      Ok(_) => Ok((
        parse::SENDER_ID.to_owned(),
        parse::SENDER_PORT.to_owned(),
        Some(serde_json::from_str(from.trim()).map_err(|e| Error::InvalidSenderData(e.to_string()))?),
      )),
      Err(_e) => Err(Error::ConnectionTargetSyntax(from.to_owned())),
    },
  }
}

/// Parse a string as a connection and return its parts.
pub fn parse_connection(s: &str) -> Result<(ConnectionDefinitionParts, ConnectionDefinitionParts)> {
  let s = s.trim();
  s.split_once(CONNECTION_SEPARATOR).map_or_else(
    || Err(Error::ConnectionDefinitionSyntax(s.to_owned())),
    |(from, to)| {
      let (to_ref, to_port) = parse_target(to.trim())?;
      let from = parse_from_or_sender(from.trim(), to_port)?;
      let to = (
        match to_ref {
          Some(DEFAULT_ID) => parse::SCHEMATIC_OUTPUT,
          Some(v) => v,
          None => return Err(Error::NoDefaultReference(s.to_owned())),
        }
        .to_owned(),
        to_port
          .map(|s| s.to_owned())
          .or_else(|| Some(from.1.clone()))
          .ok_or_else(|| Error::NoDefaultPort(s.to_owned()))?,
        None,
      );
      Ok((from, to))
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
      (
        ("ref1".to_owned(), "in".to_owned(), None,),
        ("ref2".to_owned(), "out".to_owned(), None,),
      )
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_bare_num_default() -> Result<()> {
    let parsed = parse_connection("5 -> ref2.out")?;
    let num = 5;

    assert_eq!(
      parsed,
      (
        (
          parse::SENDER_ID.to_owned(),
          parse::SENDER_PORT.to_owned(),
          Some(num.into()),
        ),
        ("ref2".to_owned(), "out".to_owned(), None,),
      )
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_default_input_named_port() -> Result<()> {
    let parsed = parse_connection("<>.in->ref2.out")?;
    assert_eq!(
      parsed,
      (
        (parse::SCHEMATIC_INPUT.to_owned(), "in".to_owned(), None,),
        ("ref2".to_owned(), "out".to_owned(), None,),
      )
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_default_output_named_port() -> Result<()> {
    let parsed = parse_connection("ref1.in-><>.out")?;
    assert_eq!(
      parsed,
      (
        ("ref1".to_owned(), "in".to_owned(), None,),
        (parse::SCHEMATIC_OUTPUT.to_owned(), "out".to_owned(), None,),
      )
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_default_output() -> Result<()> {
    let parsed = parse_connection("ref1.port-><>")?;
    assert_eq!(
      parsed,
      (
        ("ref1".to_owned(), "port".to_owned(), None,),
        (parse::SCHEMATIC_OUTPUT.to_owned(), "port".to_owned(), None,),
      )
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_default_input() -> Result<()> {
    let parsed = parse_connection("<> -> ref1.port")?;
    assert_eq!(
      parsed,
      (
        (parse::SCHEMATIC_INPUT.to_owned(), "port".to_owned(), None,),
        ("ref1".to_owned(), "port".to_owned(), None,),
      )
    );
    Ok(())
  }

  #[test_logger::test]
  fn test_connection_with_default_data() -> Result<()> {
    let parsed = parse_connection(r#""default"->ref1.port"#)?;
    assert_eq!(
      parsed,
      (
        (
          parse::SENDER_ID.to_owned(),
          parse::SENDER_PORT.to_owned(),
          Some(Value::from_str(r#""default""#)?),
        ),
        ("ref1".to_owned(), "port".to_owned(), None,),
      )
    );
    Ok(())
  }

  #[test_logger::test]
  fn regression_1() -> Result<()> {
    let parsed = parse_connection(r#""1234512345" -> <>.output"#)?;
    assert_eq!(
      parsed,
      (
        (
          parse::SENDER_ID.to_owned(),
          parse::SENDER_PORT.to_owned(),
          Some(Value::from_str(r#""1234512345""#)?),
        ),
        (parse::SCHEMATIC_OUTPUT.to_owned(), "output".to_owned(), None,),
      )
    );
    Ok(())
  }
}
