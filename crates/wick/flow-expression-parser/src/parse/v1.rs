mod parsers;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1, char, digit1, multispace0};
use nom::combinator::{eof, map, opt, recognize};
use nom::error::ParseError;
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::IResult;

use crate::ast::{
  BlockExpression,
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

pub(crate) fn identifier(input: &str) -> IResult<&str, &str> {
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
  let (i, (path_parts, id)) = pair(operation_path, opt(inline_id))(input)?;
  Ok((
    i,
    id.map_or_else(
      || InstanceTarget::anonymous_path(path_parts),
      |id| InstanceTarget::path(path_parts, id),
    ),
  ))
}

fn instance(input: &str) -> IResult<&str, InstanceTarget> {
  alt((component_path, component_id))(input)
}

pub(crate) fn instance_port(input: &str) -> IResult<&str, InstancePort> {
  let (i, (name, parts)) = pair(
    identifier,
    many0(preceded(
      char('.'),
      alt((
        map(identifier, |r: &str| r.to_owned()),
        map(digit1, |r: &str| r.to_owned()),
        parsers::parse_string,
      )),
    )),
  )(input)?;
  if parts.is_empty() {
    Ok((i, InstancePort::Named(name.to_owned())))
  } else {
    Ok((i, InstancePort::Path(name.to_owned(), parts.into_iter().collect())))
  }
}

fn connection_target_expression(input: &str) -> IResult<&str, (InstanceTarget, InstancePort)> {
  pair(terminated(instance, char('.')), instance_port)(input).map(|(i, v)| (i, (v.0, v.1)))
}

fn portless_target_expression(input: &str) -> IResult<&str, (InstanceTarget, InstancePort)> {
  instance(input).map(|(i, v)| (i, (v, InstancePort::None)))
}

fn connection_expression_sequence(input: &str) -> IResult<&str, FlowExpression> {
  let (i, (from, hops)) = pair(
    alt((connection_target_expression, portless_target_expression)),
    many1(preceded(
      ws(tag(CONNECTION_SEPARATOR)),
      ws(alt((connection_target_expression, portless_target_expression))),
    )),
  )(input)?;

  let mut connections = Vec::new();

  let mut last_hop = from;
  last_hop.0.ensure_id();
  for mut hop in hops {
    hop.0.ensure_id();
    connections.push(FlowExpression::ConnectionExpression(Box::new(connect(
      last_hop,
      hop.clone(),
    ))));
    last_hop = hop;
  }

  if connections.len() == 1 {
    Ok((i, connections.remove(0)))
  } else {
    Ok((i, FlowExpression::BlockExpression(BlockExpression::new(connections))))
  }
}

fn connect(from: (InstanceTarget, InstancePort), to: (InstanceTarget, InstancePort)) -> ConnectionExpression {
  let (from, to) = match (from, to) {
    // if we have a known-default upstream and a port on the downstream, use the downstream port's name
    ((from, InstancePort::None), (to, to_port))
      if matches!(from, InstanceTarget::Input | InstanceTarget::Default)
        && matches!(to_port, InstancePort::Named(_) | InstancePort::Path(_, _)) =>
    {
      ((from, InstancePort::named(to_port.name().unwrap())), (to, to_port))
    }
    // if we have a known-default downstream and a port on the upstream, use the upstream port's name
    ((from, from_port), (to, InstancePort::None))
      if matches!(to, InstanceTarget::Output | InstanceTarget::Default)
        && matches!(from_port, InstancePort::Named(_) | InstancePort::Path(_, _)) =>
    {
      let to_port = InstancePort::named(from_port.name().unwrap());
      ((from, from_port), (to, to_port))
    }
    // otherwise, pass it along for the next processor to deal with.
    x => x,
  };
  ConnectionExpression::new(
    ConnectionTargetExpression::new(from.0, from.1),
    ConnectionTargetExpression::new(to.0, to.1),
  )
}

pub(crate) fn flow_expression(input: &str) -> IResult<&str, FlowExpression> {
  let (i, expr) = terminated(connection_expression_sequence, alt((eof, ws(tag(";")))))(input)?;
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

/// Parse a string into an InstanceTarget
pub(crate) fn parse_instance(s: &str) -> Result<InstanceTarget, Error> {
  let (_, c) = instance(s).map_err(|_e| Error::ComponentIdError(s.to_owned()))?;
  Ok(c)
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
  use crate::ast::{set_seed, BlockExpression};
  // use crate::ast::BlockExpression;

  #[rstest]
  #[case("<>", InstTgt::Default)]
  #[case("<input>", InstTgt::Input)]
  #[case("<output>", InstTgt::Output)]
  #[case("core", InstTgt::Core)]
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
  #[case("comp::op[FOO].foo", (InstTgt::path("comp::op","FOO"), InstPort::named("foo")))]
  #[case("That.bar", (InstTgt::Named("That".to_owned()), InstPort::named("bar")))]
  #[case("<>.input", (InstTgt::Default, InstPort::named("input")))]
  #[case("input.foo", (InstTgt::named("input"), InstPort::named("foo")))]
  #[case("ref.foo", (InstTgt::named("ref"), InstPort::named("foo")))]
  #[case("<>.foo", (InstTgt::Default, InstPort::named("foo")))]
  fn connection_target_expression_tester(
    #[case] input: &'static str,
    #[case] expected: (InstTgt, InstancePort),
  ) -> Result<()> {
    let (i, t) = connection_target_expression(input)?;
    assert_eq!(t, expected);
    assert_eq!(i, "");
    Ok(())
  }

  #[rstest]
  #[case("foo", InstancePort::named("foo"))]
  #[case("foo.hey", InstancePort::path("foo", vec!["hey".to_owned()]))]
  #[case("foo.hey.0.this", InstancePort::path("foo", vec!["hey".to_owned(),"0".to_owned(),"this".to_owned()]))]
  #[case("input.\"Raw String Field #\"", InstancePort::path("input", vec!["Raw String Field #".to_owned()]))]
  fn test_instance_port(#[case] input: &'static str, #[case] expected: InstancePort) -> Result<()> {
    let (i, actual) = instance_port(input)?;

    assert_eq!(expected, actual);
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
  #[case("<>.anything -> drop",((InstTgt::Default, "anything"),(InstTgt::Null(None), InstancePort::None)))]
  fn connection_parts(
    #[case] input: &'static str,
    #[case] expected: ((InstTgt, impl Into<InstancePort>), (InstTgt, impl Into<InstancePort>)),
  ) -> Result<()> {
    let (i, t) = connection_expression_sequence(input)?;
    let expected = FlowExpression::ConnectionExpression(Box::new(CE::new(
      CTE::new(expected.0 .0, expected.0 .1.into()),
      CTE::new(expected.1 .0, expected.1 .1.into()),
    )));

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

  fn flow_expr_conn(from_tgt: InstTgt, from_port: InstPort, to_tgt: InstTgt, to_port: InstPort) -> FE {
    FE::connection(CE::new(CTE::new(from_tgt, from_port), CTE::new(to_tgt, to_port)))
  }

  fn flow_block<const K: usize>(seed: u64, hops: impl FnOnce() -> [(InstTgt, InstPort); K]) -> FE {
    set_seed(seed);
    let mut connections = Vec::new();
    let mut last_hop: Option<(InstanceTarget, InstancePort)> = None;
    let hops = hops();
    for hop in hops {
      if let Some(last) = last_hop.take() {
        last_hop = Some(hop.clone());
        connections.push(FE::connection(CE::new(
          CTE::new(last.0, last.1),
          CTE::new(hop.0, hop.1),
        )));
      } else {
        last_hop = Some(hop);
      }
    }
    set_seed(seed);
    FE::block(BlockExpression::new(connections))
  }

  mod rng_limited {
    use pretty_assertions::assert_eq;

    use super::*;

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
    #[case(
      "this.output.\"field\" -> <>.output",
      flow_expr_conn(InstTgt::named("this"), InstPort::path("output",vec!["field".to_owned()]), InstTgt::Default, InstPort::named("output"))
    )]
    #[case(
      "this.output.\"field with spaces and \\\" quotes and symbols ,*|#\" -> <>.output",
      flow_expr_conn(InstTgt::named("this"), InstPort::path("output",vec!["field with spaces and \" quotes and symbols ,*|#".to_owned()]), InstTgt::Default, InstPort::named("output"))
    )]
    #[case(
      "this -> that",
      flow_expr_conn(InstTgt::named("this"), InstPort::None, InstTgt::named("that"), InstPort::None)
    )]
    #[case(
      "this -> that -> another",
      flow_block(0,||[
        (InstTgt::named("this"), InstPort::None),
        (InstTgt::named("that"), InstPort::None),
        (InstTgt::named("another"), InstPort::None)
      ])
    )]
    #[case(
      "<> -> test::reverse -> test::uppercase -> <>",
      flow_block(0,||[
        (InstTgt::Input, InstPort::None),
        (InstTgt::generated_path("test::reverse"), InstPort::None),
        (InstTgt::generated_path("test::uppercase"), InstPort::None),
        (InstTgt::Output, InstPort::None)
        ])
    )]
    #[case(
      "test::in -> test::middle -> test::out",
      flow_block(0,||[
        (InstTgt::generated_path("test::in"), InstPort::None),
        (InstTgt::generated_path("test::middle"), InstPort::None),
        (InstTgt::generated_path("test::out"), InstPort::None),
        ])
    )]

    fn test_flow_expression(#[case] input: &'static str, #[case] expected: FE) -> Result<()> {
      set_seed(0);
      let (t, actual) = flow_expression(input)?;
      println!("expected: {:?}", expected);
      println!("actual: {:?}", actual);
      assert_eq!(actual, expected);
      match actual {
        FlowExpression::ConnectionExpression(_) => {
          // no extra tests
        }
        FlowExpression::BlockExpression(block) => {
          // backup tests to ensure that the flow has the right... flow.
          let mut last: Option<&ConnectionExpression> = None;
          let inner = block.inner();

          for expr in inner {
            let this_con = expr.as_connection().unwrap();
            if let Some(last) = last {
              // assert the last downstream is the current's upstream
              assert_eq!(last.to(), this_con.from());
              // assert that each hop has a different ID
              assert_ne!(last.to().instance(), this_con.to().instance());
            }
            last = Some(this_con);
          }
        }
      }

      assert_eq!(t, "");
      Ok(())
    }
  }
}
