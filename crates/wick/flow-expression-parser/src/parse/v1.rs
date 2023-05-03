use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1, char, multispace0};
use nom::combinator::{eof, map, recognize};
use nom::error::ParseError;
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, terminated};
use nom::IResult;

use crate::ast::{ConnectionExpression, ConnectionTargetExpression, FlowExpression, FlowProgram, InstanceTarget};
use crate::Error;

/// The separator in a connection between connection targets.
pub static CONNECTION_SEPARATOR: &str = "->";

/// The reserved identifier representing an as-of-yet-undetermined default value.
const DEFAULT_ID: &str = "<>";

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

/// Parse a string into a [FlowProgram].
pub fn parse(input: &str) -> Result<FlowProgram, ParserError> {
  _parse(input).map(|(_, t)| t).map_err(|_| ParserError::Fail)
}

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

fn connection_target_expression(input: &str) -> IResult<&str, (InstanceTarget, Option<&str>)> {
  pair(terminated(instance, char('.')), identifier)(input).map(|(i, v)| (i, (v.0, Some(v.1))))
}

fn portless_target_expression(input: &str) -> IResult<&str, (InstanceTarget, Option<&str>)> {
  instance(input).map(|(i, v)| (i, (v, None)))
}

fn connection_expression(input: &str) -> IResult<&str, ConnectionExpression> {
  let (i, (from, to)) = pair(
    terminated(
      alt((connection_target_expression, portless_target_expression)),
      ws(tag(CONNECTION_SEPARATOR)),
    ),
    ws(alt((connection_target_expression, portless_target_expression))),
  )(input)?;
  let (i, (from, to)) = match (from.1, to.1) {
    (None, Some(to_port)) => (i, ((from.0, to_port), (to.0, to_port))),
    (Some(from_port), None) => (i, ((from.0, from_port), (to.0, from_port))),
    (None, None) => {
      return Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Verify,
      )))
    }
    _ => (i, ((from.0, from.1.unwrap()), (to.0, to.1.unwrap()))),
  };
  Ok((
    i,
    ConnectionExpression::new(
      ConnectionTargetExpression::new(from.0, from.1, Default::default()),
      ConnectionTargetExpression::new(to.0, to.1, Default::default()),
    ),
  ))
}

pub(crate) fn flow_expression(input: &str) -> IResult<&str, FlowExpression> {
  let (i, expr) = terminated(
    map(connection_expression, FlowExpression::ConnectionExpression),
    alt((eof, ws(tag(";")))),
  )(input)?;
  Ok((i, expr))
}

fn _parse(input: &str) -> IResult<&str, FlowProgram> {
  let (i, expressions) = many0(flow_expression)(input)?;

  Ok((i, FlowProgram::new(expressions)))
}

fn ws<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
  F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
  delimited(multispace0, inner, multispace0)
}

/// Parse a string as connection target pieces.
pub fn parse_target(s: &str) -> Result<(String, Option<&str>), Error> {
  let (_, (c, o)) = alt((connection_target_expression, portless_target_expression))(s)
    .map_err(|e| Error::ConnectionTargetSyntax(s.to_owned(), e.to_string()))?;
  Ok((c.to_string(), o))
}

/// Parse a string into an InstanceTarget
pub(crate) fn parse_instance(s: &str) -> Result<InstanceTarget, Error> {
  let (_, c) = instance(s).map_err(|_e| Error::ComponentIdError(s.to_owned()))?;
  Ok(c)
}

/// Parse a string as a connection and return its parts.
pub fn parse_connection_expression(s: &str) -> Result<ConnectionExpression, Error> {
  let (_, connection) =
    connection_expression(s).map_err(|e| Error::ConnectionTargetSyntax(s.to_owned(), e.to_string()))?;
  Ok(connection)
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
  #[case("input.foo", (InstanceTarget::named("input"), Some("foo")))]
  #[case("ref.foo", (InstanceTarget::named("ref"), Some("foo")))]
  #[case("<>.foo", (InstanceTarget::Default, Some("foo")))]
  fn connection_target_expression_tester(
    #[case] input: &'static str,
    #[case] expected: (InstanceTarget, Option<&str>),
  ) -> Result<()> {
    let (i, t) = connection_target_expression(input)?;
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
    let conn = ConnectionExpression::new(
      ConnectionTargetExpression::new(expected.0 .0, expected.0 .1, Default::default()),
      ConnectionTargetExpression::new(expected.1 .0, expected.1 .1, Default::default()),
    );

    assert_eq!(t, conn);
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
}
