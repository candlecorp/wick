use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1, char, multispace0};
use nom::combinator::recognize;
use nom::error::ParseError;
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, terminated};
use nom::IResult;
use serde_json::Value;

use crate::{Error, InstanceTarget};

/// The separator in a connection between connection targets.
pub static CONNECTION_SEPARATOR: &str = "->";

/// The reserved identifier representing an as-of-yet-undetermined default value.
const DEFAULT_ID: &str = "<>";

type ConnectionPair = (ConnectionDefinitionParts, ConnectionDefinitionParts);

#[derive(Debug, Clone, Copy)]
/// Errors that can occur during parsing.
pub enum ParserError {
  /// General parse failure.
  Fail,
  /// Unexpected token.
  UnexpectedToken,
}

impl std::error::Error for ParserError {}
impl std::fmt::Display for ParserError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ParserError::Fail => write!(f, "Parse failed"),
      ParserError::UnexpectedToken => write!(f, "Unexpected token"),
    }
  }
}

/// Parse a string into a list of connection pairs.
pub fn parse(input: &str) -> Result<Vec<ConnectionPair>, ParserError> {
  _parse(input).map(|(_, t)| t).map_err(|_| ParserError::Fail)
}

type PortParts<'a> = (InstanceTarget, &'a str);

fn component_id(input: &str) -> IResult<&str, InstanceTarget> {
  let (i, t) = recognize(alt((reserved_component_id, identifier)))(input)?;
  let t = match t {
    super::SCHEMATIC_INPUT => InstanceTarget::Input,
    super::SCHEMATIC_OUTPUT => InstanceTarget::Output,
    super::CORE_ID => InstanceTarget::Core,
    DEFAULT_ID => InstanceTarget::Default,
    super::NS_LINK => InstanceTarget::Link,
    name => InstanceTarget::Named(name.to_owned()),
  };
  Ok((i, t))
}

fn identifier(input: &str) -> IResult<&str, &str> {
  recognize(pair(alt((alpha1, tag("_"))), many0(alt((alphanumeric1, tag("_"))))))(input)
}

fn reserved_component_id(input: &str) -> IResult<&str, &str> {
  alt((
    tag(super::SCHEMATIC_INPUT),
    tag(super::SCHEMATIC_OUTPUT),
    tag(super::CORE_ID),
    tag(super::NS_LINK),
    tag(DEFAULT_ID),
  ))(input)
}

fn path(input: &str) -> IResult<&str, &str> {
  recognize(pair(many1(terminated(identifier, tag("::"))), identifier))(input)
}

fn inline_id(input: &str) -> IResult<&str, &str> {
  delimited(char('['), identifier, char(']'))(input)
}

fn component_path(input: &str) -> IResult<&str, InstanceTarget> {
  let (i, (path_parts, id)) = pair(path, inline_id)(input)?;
  Ok((i, InstanceTarget::Path(path_parts.to_owned(), id.to_owned())))
}

fn instance(input: &str) -> IResult<&str, InstanceTarget> {
  alt((component_path, component_id))(input)
}

fn port_reference_expression(input: &str) -> IResult<&str, (InstanceTarget, Option<&str>)> {
  pair(terminated(instance, char('.')), identifier)(input).map(|(i, v)| (i, (v.0, Some(v.1))))
}

fn portless_reference_expression(input: &str) -> IResult<&str, (InstanceTarget, Option<&str>)> {
  instance(input).map(|(i, v)| (i, (v, None)))
}

fn connection_expression(input: &str) -> IResult<&str, (PortParts, PortParts)> {
  let (i, (from, to)) = pair(
    terminated(
      alt((port_reference_expression, portless_reference_expression)),
      ws(tag(CONNECTION_SEPARATOR)),
    ),
    ws(alt((port_reference_expression, portless_reference_expression))),
  )(input)?;
  match (from.1, to.1) {
    (None, Some(to_port)) => Ok((i, ((from.0, to_port), (to.0, to_port)))),
    (Some(from_port), None) => Ok((i, ((from.0, from_port), (to.0, from_port)))),
    (None, None) => Err(nom::Err::Error(nom::error::Error::new(
      input,
      nom::error::ErrorKind::Verify,
    ))),
    _ => Ok((i, ((from.0, from.1.unwrap()), (to.0, to.1.unwrap())))),
  }
}

fn _parse(input: &str) -> IResult<&str, Vec<ConnectionPair>> {
  let (i, (from, to)) = connection_expression(input)?;
  Ok((
    i,
    vec![((from.0, from.1.to_owned(), None), (to.0, to.1.to_owned(), None))],
  ))
}

fn ws<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
  F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
  delimited(multispace0, inner, multispace0)
}

/// Parse a string as connection target pieces.
pub fn parse_target(s: &str) -> Result<(String, Option<&str>), Error> {
  let (_, (c, o)) = alt((port_reference_expression, portless_reference_expression))(s)
    .map_err(|e| Error::ConnectionTargetSyntax(s.to_owned(), e.to_string()))?;
  Ok((c.to_string(), o))
}

/// Parse a string into an InstanceTarget
pub fn parse_instance(s: &str) -> Result<InstanceTarget, Error> {
  let (_, c) = instance(s).map_err(|_e| Error::ComponentIdError(s.to_owned()))?;
  Ok(c)
}

/// Parse a string as an instance.
pub fn parse_instance_pieces(s: &str) -> Result<(String, Option<&str>), Error> {
  let (_, (c, o)) = alt((port_reference_expression, portless_reference_expression))(s)
    .map_err(|e| Error::ConnectionTargetSyntax(s.to_owned(), e.to_string()))?;
  Ok((c.to_string(), o))
}

type ConnectionDefinitionParts = (InstanceTarget, String, Option<Value>);

/// Parse a string as a connection and return its parts.
pub fn parse_connection_pieces(s: &str) -> Result<ConnectionPair, Error> {
  let (_, ((from_ref, from_port), (to_ref, to_port))) =
    connection_expression(s).map_err(|e| Error::ConnectionTargetSyntax(s.to_owned(), e.to_string()))?;
  Ok((
    (from_ref.or(InstanceTarget::Input), from_port.to_owned(), None),
    (to_ref.or(InstanceTarget::Output), to_port.to_owned(), None),
  ))
}

#[cfg(test)]
mod tests {

  use anyhow::Result;
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  use super::*;

  #[rstest]
  #[case("<>", InstanceTarget::Default)]
  #[case("<input>", InstanceTarget::Input)]
  #[case("<output>", InstanceTarget::Output)]
  #[case("<core>", InstanceTarget::Core)]
  #[case("heya", InstanceTarget::Named("heya".to_owned()))]
  #[case("this::that[A]", InstanceTarget::Path("this::that".to_owned(),"A".to_owned()))]
  fn test_component_id(#[case] input: &'static str, #[case] expected: InstanceTarget) -> Result<()> {
    let (i, t) = instance(input)?;
    assert_eq!(t, expected);
    assert_eq!(i, "");
    Ok(())
  }

  #[rstest]
  #[case("single")]
  #[case("<input>")]
  fn test_path_negative(#[case] input: &str) -> Result<()> {
    assert_err(&path(input));
    Ok(())
  }

  #[rstest]
  #[case("comp::op", "comp::op")]
  #[case("THIS::That", "THIS::That")]
  #[allow(clippy::needless_pass_by_value)]
  fn path_tester(#[case] input: &'static str, #[case] expected: &str) -> Result<()> {
    let (i, t) = path(input)?;
    assert_eq!(t, expected);
    assert_eq!(i, "");
    Ok(())
  }

  #[rstest]
  #[case("comp::op[FOO].foo", (InstanceTarget::path("comp::op","FOO"), Some("foo")))]
  #[case("That.bar", (InstanceTarget::Named("That".to_owned()), Some("bar")))]
  #[case("<>.input", (InstanceTarget::Default, Some("input")))]
  fn op_expression_tester(#[case] input: &'static str, #[case] expected: (InstanceTarget, Option<&str>)) -> Result<()> {
    let (i, t) = port_reference_expression(input)?;
    assert_eq!(t, expected);
    assert_eq!(i, "");
    Ok(())
  }

  #[rstest]
  #[case("comp::op[INLINE].foo -> <>.output", ((InstanceTarget::path("comp::op","INLINE"), "foo"),(InstanceTarget::Default, "output")))]
  #[case("<> -> ref1.port", ((InstanceTarget::Default, "port"),(InstanceTarget::named("ref1"), "port")))]
  #[case("ref1.in -> ref2.out", ((InstanceTarget::named("ref1"), "in"),(InstanceTarget::named("ref2"), "out")))]
  #[case("<>.in->ref2.out", ((InstanceTarget::Default, "in"),(InstanceTarget::named("ref2"), "out")))]
  #[case("ref1.in-><>.out", ((InstanceTarget::named("ref1"), "in"),(InstanceTarget::Default, "out")))]
  #[case("ref1.port-><>", ((InstanceTarget::named("ref1"), "port"),(InstanceTarget::Default, "port")))]
  #[case("<> -> ref1.port", ((InstanceTarget::Default, "port"),(InstanceTarget::named("ref1"), "port")))]
  #[case("<> -> test::reverse[A].input",((InstanceTarget::Default, "input"),(InstanceTarget::path("test::reverse","A"), "input")))]
  fn connection_parts(
    #[case] input: &'static str,
    #[case] expected: ((InstanceTarget, &str), (InstanceTarget, &str)),
  ) -> Result<()> {
    let (i, t) = connection_expression(input)?;
    assert_eq!(t, expected);
    assert_eq!(i, "");
    Ok(())
  }

  fn assert_err<O, E>(item: &Result<O, E>)
  where
    E: std::fmt::Debug,
    O: std::fmt::Debug,
  {
    if !item.is_err() {
      panic!("Expected error, got {:?}", item);
    } else {
    }
  }

  #[test_logger::test]
  fn test_reserved() -> Result<()> {
    let parsed = parse_target("input.foo")?;
    assert_eq!(parsed, ("input".to_owned(), Some("foo")));
    Ok(())
  }

  #[test_logger::test]
  fn test_basic() -> Result<()> {
    let parsed = parse_target("ref.foo")?;
    assert_eq!(parsed, ("ref".to_owned(), Some("foo")));
    Ok(())
  }

  #[test_logger::test]
  fn test_default_with_port() -> Result<()> {
    let parsed = parse_target("<>.foo")?;
    assert_eq!(parsed, (DEFAULT_ID.to_owned(), Some("foo")));
    Ok(())
  }

  #[test_logger::test]
  fn test_default() -> Result<()> {
    let parsed = parse_target("<>")?;
    assert_eq!(parsed, (DEFAULT_ID.to_owned(), None));
    Ok(())
  }
}
