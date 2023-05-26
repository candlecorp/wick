use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1, char, digit1, multispace0};
use nom::combinator::{eof, recognize};
use nom::error::ParseError;
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::IResult;

use crate::ast::{
  ConnectionExpression,
  ConnectionTargetExpression,
  FlowExpression,
  FlowProgram,
  InstancePort,
  InstanceTarget,
};
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
    super::SCHEMATIC_NULL | "drop" => InstanceTarget::Null(None),
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
    tag(super::SCHEMATIC_NULL),
    tag("drop"),
    tag(super::CORE_ID),
    tag(super::NS_LINK),
    tag(DEFAULT_ID),
  ))(input)
}

fn operation_path(input: &str) -> IResult<&str, &str> {
  recognize(pair(many1(terminated(identifier, tag("::"))), identifier))(input)
}

fn inline_id(input: &str) -> IResult<&str, &str> {
  delimited(char('['), identifier, char(']'))(input)
}

fn component_path(input: &str) -> IResult<&str, InstanceTarget> {
  let (i, (path_parts, id)) = pair(operation_path, inline_id)(input)?;
  Ok((i, InstanceTarget::path(path_parts, id)))
}

fn instance(input: &str) -> IResult<&str, InstanceTarget> {
  alt((component_path, component_id))(input)
}

pub(crate) fn instance_port(input: &str) -> IResult<&str, InstancePort> {
  let (i, (name, parts)) = pair(identifier, many0(preceded(char('.'), alt((identifier, digit1)))))(input)?;
  if parts.is_empty() {
    Ok((i, InstancePort::Named(name.to_owned())))
  } else {
    Ok((
      i,
      InstancePort::Path(name.to_owned(), parts.into_iter().map(|x| x.to_owned()).collect()),
    ))
  }
}

fn connection_target_expression(input: &str) -> IResult<&str, (InstanceTarget, Option<InstancePort>)> {
  pair(terminated(instance, char('.')), instance_port)(input).map(|(i, v)| (i, (v.0, Some(v.1))))
}

fn portless_target_expression(input: &str) -> IResult<&str, (InstanceTarget, Option<InstancePort>)> {
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
  let (i, (from, to)) = match (from.1, to) {
    // if no port on the upstream but there's a port on the downstream, use the downstream port
    (None, (to, Some(InstancePort::Named(to_port)))) => (
      i,
      (
        (from.0, InstancePort::Named(to_port.clone())),
        (to, InstancePort::Named(to_port)),
      ),
    ),
    // if there's a port on the upstream and the downstream is the Null entity, use the port 'input'
    (Some(from_port), to @ (InstanceTarget::Null(_), _)) => (
      i,
      ((from.0, from_port), (to.0, InstancePort::Named("input".to_owned()))),
    ),
    // if there's a port on the upstream and the downstream is anything else, adopt the upstream port.
    (Some(InstancePort::Named(from_port)), (to, None)) => (
      i,
      (
        (from.0, InstancePort::Named(from_port.clone())),
        (to, InstancePort::Named(from_port)),
      ),
    ),
    // if there's no port on the upstream and no port on the downstream, error
    (None, (_, None)) => {
      return Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Verify,
      )))
    }
    // Otherwise we've got ports so let's use em.
    (from_port, to) => (i, ((from.0, from_port.unwrap()), (to.0, to.1.unwrap()))),
  };
  Ok((
    i,
    ConnectionExpression::new(
      ConnectionTargetExpression::new(from.0, from.1),
      ConnectionTargetExpression::new(to.0, to.1),
    ),
  ))
}

pub(crate) fn flow_expression(input: &str) -> IResult<&str, FlowExpression> {
  let (i, expr) = terminated(connection_expression, alt((eof, ws(tag(";")))))(input)?;
  Ok((i, FlowExpression::ConnectionExpression(Box::new(expr))))
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
pub fn parse_target(s: &str) -> Result<(String, Option<InstancePort>), Error> {
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

  use super::{
    ConnectionExpression as CE,
    ConnectionTargetExpression as CTE,
    FlowExpression as FE,
    InstancePort as InstPort,
    InstanceTarget as InstTgt,
    *,
  };
  // use crate::ast::BlockExpression;

  #[rstest]
  #[case("<>", InstTgt::Default)]
  #[case("<input>", InstTgt::Input)]
  #[case("<output>", InstTgt::Output)]
  #[case("<core>", InstTgt::Core)]
  #[case("heya", InstTgt::named("heya"))]
  #[case("this::that[A]", InstTgt::path("this::that", "A"))]
  fn test_component_id(#[case] input: &'static str, #[case] expected: InstTgt) -> Result<()> {
    let (i, t) = instance(input)?;
    assert_eq!(t, expected);
    assert_eq!(i, "");
    Ok(())
  }

  #[rstest]
  #[case("single")]
  #[case("<input>")]
  fn test_path_negative(#[case] input: &str) -> Result<()> {
    assert_err(&operation_path(input));
    Ok(())
  }

  #[rstest]
  #[case("comp::op", "comp::op")]
  #[case("THIS::That", "THIS::That")]
  #[allow(clippy::needless_pass_by_value)]
  fn path_tester(#[case] input: &'static str, #[case] expected: &str) -> Result<()> {
    let (i, t) = operation_path(input)?;
    assert_eq!(t, expected);
    assert_eq!(i, "");
    Ok(())
  }

  #[rstest]
  #[case("comp::op[FOO].foo", (InstTgt::path("comp::op","FOO"), Some(InstPort::named("foo"))))]
  #[case("That.bar", (InstTgt::Named("That".to_owned()), Some(InstPort::named("bar"))))]
  #[case("<>.input", (InstTgt::Default, Some(InstPort::named("input"))))]
  #[case("input.foo", (InstTgt::named("input"), Some(InstPort::named("foo"))))]
  #[case("ref.foo", (InstTgt::named("ref"), Some(InstPort::named("foo"))))]
  #[case("<>.foo", (InstTgt::Default, Some(InstPort::named("foo"))))]
  fn connection_target_expression_tester(
    #[case] input: &'static str,
    #[case] expected: (InstTgt, Option<InstancePort>),
  ) -> Result<()> {
    let (i, t) = connection_target_expression(input)?;
    assert_eq!(t, expected);
    assert_eq!(i, "");
    Ok(())
  }

  #[rstest]
  #[case("comp::op[INLINE].foo -> <>.output", ((InstTgt::path("comp::op","INLINE"), "foo"),(InstTgt::Default, "output")))]
  #[case("<> -> ref1.port", ((InstTgt::Default, "port"),(InstTgt::named("ref1"), "port")))]
  #[case("ref1.in -> ref2.out", ((InstTgt::named("ref1"), "in"),(InstTgt::named("ref2"), "out")))]
  #[case("<>.in->ref2.out", ((InstTgt::Default, "in"),(InstTgt::named("ref2"), "out")))]
  #[case("ref1.in-><>.out", ((InstTgt::named("ref1"), "in"),(InstTgt::Default, "out")))]
  #[case("ref1.port-><>", ((InstTgt::named("ref1"), "port"),(InstTgt::Default, "port")))]
  #[case("<> -> ref1.port", ((InstTgt::Default, "port"),(InstTgt::named("ref1"), "port")))]
  #[case("<> -> test::reverse[A].input",((InstTgt::Default, "input"),(InstTgt::path("test::reverse","A"), "input")))]
  #[case("<>.anything -> drop",((InstTgt::Default, "anything"),(InstTgt::Null(None), "input")))]
  fn connection_parts(#[case] input: &'static str, #[case] expected: ((InstTgt, &str), (InstTgt, &str))) -> Result<()> {
    let (i, t) = connection_expression(input)?;
    let conn = CE::new(
      CTE::new(expected.0 .0, InstPort::named(expected.0 .1)),
      CTE::new(expected.1 .0, InstPort::named(expected.1 .1)),
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

  fn flow_expr_conn(from_tgt: InstTgt, from_port: InstPort, to_tgt: InstTgt, to_port: InstPort) -> FE {
    FE::connection(CE::new(CTE::new(from_tgt, from_port), CTE::new(to_tgt, to_port)))
  }

  // fn flow_block(exprs: &[FE]) -> FE {
  //   FE::block(BlockExpression::new(exprs.to_vec()))
  // }

  #[rstest]
  #[case(
    "comp::op[INLINE].foo -> <>.output",
    flow_expr_conn(
      InstTgt::path("comp::op", "INLINE"),
      InstPort::named("foo"),
      InstTgt::Default,
      InstPort::named("output")
    )
  )]
  #[case(
    "this.output.field -> <>.output",
    flow_expr_conn(InstTgt::named("this"), InstPort::path("output",vec!["field".to_owned()]), InstTgt::Default, InstPort::named("output"))
  )]
  #[case(
    "this.output.field.other.0 -> <>.output",
    flow_expr_conn(InstTgt::named("this"), InstPort::path("output",vec!["field".to_owned(),"other".to_owned(), "0".to_owned()]), InstTgt::Default, InstPort::named("output"))
  )]
  // #[case(
  //   "this.output.field -> other.input",
  //   flow_block(&[
  //     flow_expr_conn(InstTgt::named("this"), "output", anon("core::pluck"), "input"),
  //     flow_expr_conn(anon("core::pluck"), "output",named("other"), "input",)
  //   ])
  // )]
  fn test_flow_expression(#[case] input: &'static str, #[case] expected: FE) -> Result<()> {
    let (t, actual) = flow_expression(input)?;
    assert_eq!(actual, expected);
    assert_eq!(t, "");
    Ok(())
  }
}
