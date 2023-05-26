use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_while1};
use nom::character::complete::{alpha1, alphanumeric1, char, multispace0};
use nom::character::is_alphabetic;
use nom::combinator::{opt, recognize};
use nom::error::ParseError;
use nom::multi::{many0, many0_count, separated_list1};
use nom::sequence::{delimited, pair, terminated, tuple};
use nom::IResult;

use crate::{Field, TypeSignature};

#[derive(Debug, Clone)]
pub enum ParserError {
  Fail(String),
  UnexpectedToken,
}

impl std::error::Error for ParserError {}
impl std::fmt::Display for ParserError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ParserError::Fail(v) => write!(f, "Could not parse {} into TypeSignature", v),
      ParserError::UnexpectedToken => write!(f, "Unexpected token"),
    }
  }
}

fn ws<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
  F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
  delimited(multispace0, inner, multispace0)
}

pub fn parse(input: &str) -> Result<TypeSignature, ParserError> {
  _parse(input)
    .map(|(_, t)| t)
    .map_err(|_| ParserError::Fail(input.to_owned()))
}

fn square_brackets(input: &str) -> IResult<&str, &str> {
  delimited(char('['), multispace0, char(']'))(input)
}

fn struct_type(input: &str) -> IResult<&str, TypeSignature> {
  let (i, (v)) = delimited(char('{'), separated_list1(char(','), ws(key_type_pair)), char('}'))(input)?;
  let v: Vec<_> = v.into_iter().map(|(s, t)| Field::new(s, t)).collect();
  Ok((i, TypeSignature::AnonymousStruct(v)))
}

fn map_type(input: &str) -> IResult<&str, TypeSignature> {
  let (i, ((_, ty))) = delimited(char('{'), ws(map_key_type_pair), char('}'))(input)?;

  Ok((
    i,
    TypeSignature::Map {
      key: Box::new(TypeSignature::String),
      value: Box::new(ty),
    },
  ))
}

fn key_type_pair(input: &str) -> IResult<&str, (&str, TypeSignature)> {
  let (i, (key, _, t)) = tuple((identifier, ws(char(':')), valid_type))(input)?;
  Ok((i, (key, t)))
}

fn map_key_type_pair(input: &str) -> IResult<&str, (&str, TypeSignature)> {
  let (i, (key, _, t)) = tuple((tag("string"), ws(char(':')), valid_type))(input)?;
  Ok((i, (key, t)))
}

fn identifier(input: &str) -> IResult<&str, &str> {
  recognize(pair(
    alt((alpha1, tag("_"))),
    many0_count(alt((alphanumeric1, tag("_")))),
  ))(input)
}

fn list_type(input: &str) -> IResult<&str, TypeSignature> {
  let (i, (t, _)) = pair(typename, ws(square_brackets))(input)?;

  Ok((i, TypeSignature::List { ty: Box::new(t) }))
}

fn valid_type(input: &str) -> IResult<&str, TypeSignature> {
  alt((map_type, struct_type, list_type, typename))(input)
}

fn typename(input: &str) -> IResult<&str, TypeSignature> {
  let (i, t) = alt((
    recognize(pair(many0(terminated(identifier, tag("::"))), identifier)),
    identifier,
  ))(input)?;
  let t = match t {
    "bool" => TypeSignature::Bool,
    "i8" => TypeSignature::I8,
    "i16" => TypeSignature::I16,
    "i32" => TypeSignature::I32,
    "i64" => TypeSignature::I64,
    "u8" => TypeSignature::U8,
    "u16" => TypeSignature::U16,
    "u32" => TypeSignature::U32,
    "u64" => TypeSignature::U64,
    "f32" => TypeSignature::F32,
    "f64" => TypeSignature::F64,
    "bytes" => TypeSignature::Bytes,
    "string" => TypeSignature::String,
    "datetime" => TypeSignature::Datetime,
    "object" => TypeSignature::Object,
    x => TypeSignature::Custom(x.to_owned()),
  };
  Ok((i, t))
}

fn _parse(input: &str) -> IResult<&str, TypeSignature> {
  let (i, (t, optional)) = pair(alt((list_type, map_type, struct_type, typename)), opt(tag("?")))(input)?;
  if optional.is_some() {
    Ok((i, TypeSignature::Optional { ty: Box::new(t) }))
  } else {
    Ok((i, t))
  }
}

#[macro_export]
macro_rules! fields {
  (@single $($x:tt)*) => (());
  (@count $($rest:expr),*) => (<[()]>::len(&[$($crate::fields!(@single $rest)),*]));

  // ($($key:expr => $value:expr,)+) => { $crate::typemap!($($key => $value),+) };
  ($($key:expr => $value:expr),* $(,)?) => {
      {
          let _cap = $crate::fields!(@count $(stringify!($key)),*);
          let mut _map = ::std::vec::Vec::with_capacity(_cap);
          $(
              let _ = _map.push($crate::Field::new($key, $crate::parse($value).unwrap()));
          )*
          _map
      }
  };
}

#[macro_export]
macro_rules! operation {
  ($name:expr => {
    inputs: {$($ikey:expr => $ivalue:expr),* $(,)?},
    outputs: {$($okey:expr => $ovalue:expr),* $(,)?},
  }) => {
    $crate::OperationSignature {
      name: $name.to_owned(),
      inputs: $crate::fields! {$($ikey => $ivalue),*},
      outputs: $crate::fields! {$($okey => $ovalue),*},
    }
  };
}

#[macro_export]
macro_rules! component {
  (
    name: $name:expr,
    version: $version:expr,
    operations: {
      $($opname:expr => {
        inputs: {$($ikey:expr => $ivalue:expr),* $(,)?},
        outputs: {$($okey:expr => $ovalue:expr),* $(,)?},
      }),* $(,)?
    }
  ) => {{
    let mut ops = std::vec::Vec::default();
    $(
      let _ = ops.push($crate::operation!($opname => {inputs: {$($ikey => $ivalue),*}, outputs: {$($okey => $ovalue),*},}));
    )*;

    $crate::component! {
      name: $name,
      version: $version,
      operations: ops,
    }
  }};
  (
    name: $name:expr,
    version: $version:expr,
    operations: $ops:expr,
  ) => {{
    $crate::ComponentSignature {
      name: Some($name.to_owned()),
      metadata: $crate::ComponentMetadata::new($version),
      operations: $ops,
      ..Default::default()
    }
  }};
}

#[cfg(test)]
mod test {
  use std::collections::HashMap;

  use anyhow::Result;

  use super::*;
  use crate::Field;

  #[rstest::rstest]
  #[case("bool[]", TypeSignature::Bool)]
  #[case("bool []", TypeSignature::Bool)]
  #[case("bool [ ]", TypeSignature::Bool)]
  #[case("string[]", TypeSignature::String)]
  fn test_list_variants(#[case] input: &'static str, #[case] expected: TypeSignature) -> Result<()> {
    let (i, t) = list_type(input)?;
    assert_eq!(t, TypeSignature::List { ty: Box::new(expected) });
    Ok(())
  }

  #[rstest::rstest]
  #[case("{ myBool : bool }", TypeSignature::Bool)]
  #[case("{myBool:bool}", TypeSignature::Bool)]
  #[case("{ myBool : bool }", TypeSignature::Bool)]
  fn test_struct_variants(#[case] input: &'static str, #[case] expected: TypeSignature) -> Result<()> {
    let (i, t) = struct_type(input)?;
    let fields = [Field::new("myBool", TypeSignature::Bool)];
    assert_eq!(t, TypeSignature::AnonymousStruct(fields.into()));
    Ok(())
  }

  #[rstest::rstest]
  #[case("bool", TypeSignature::Bool)]
  #[case("i8", TypeSignature::I8)]
  #[case("i16", TypeSignature::I16)]
  #[case("i32", TypeSignature::I32)]
  #[case("i64", TypeSignature::I64)]
  #[case("u8", TypeSignature::U8)]
  #[case("u16", TypeSignature::U16)]
  #[case("u32", TypeSignature::U32)]
  #[case("u64", TypeSignature::U64)]
  #[case("f32", TypeSignature::F32)]
  #[case("f64", TypeSignature::F64)]
  #[case("bytes", TypeSignature::Bytes)]
  #[case("string", TypeSignature::String)]
  #[case("datetime", TypeSignature::Datetime)]
  #[case("object", TypeSignature::Object)]
  #[case("myType", TypeSignature::Custom("myType".to_owned()))]
  #[case("name::myType", TypeSignature::Custom("name::myType".to_owned()))]
  fn test_parse_typename(#[case] as_str: &'static str, #[case] ty: TypeSignature) -> Result<()> {
    assert_eq!(typename(as_str)?, ("", ty));
    Ok(())
  }

  #[rstest::rstest]
  #[case("bool", TypeSignature::Bool)]
  #[case("i8", TypeSignature::I8)]
  #[case("i16", TypeSignature::I16)]
  #[case("i32", TypeSignature::I32)]
  #[case("i64", TypeSignature::I64)]
  #[case("u8", TypeSignature::U8)]
  #[case("u16", TypeSignature::U16)]
  #[case("u32", TypeSignature::U32)]
  #[case("u64", TypeSignature::U64)]
  #[case("f32", TypeSignature::F32)]
  #[case("f64", TypeSignature::F64)]
  #[case("bytes", TypeSignature::Bytes)]
  #[case("string", TypeSignature::String)]
  #[case("datetime", TypeSignature::Datetime)]
  #[case("object", TypeSignature::Object)]
  #[case("myType[]", TypeSignature::List{ty:Box::new(TypeSignature::Custom("myType".to_owned()))})]
  #[case("{string: bool}", TypeSignature::Map{key:Box::new(TypeSignature::String),value:Box::new(TypeSignature::Bool)})]
  #[case("{string: bool[]}", TypeSignature::Map{key:Box::new(TypeSignature::String),value:Box::new(TypeSignature::List{ty:Box::new(TypeSignature::Bool)})})]
  #[case("{string: name::myType}", TypeSignature::Map{key:Box::new(TypeSignature::String),value:Box::new(TypeSignature::Custom("name::myType".to_owned()))})]
  #[case("name::myType", TypeSignature::Custom("name::myType".to_owned()))]
  fn test_parse_type(#[case] as_str: &'static str, #[case] ty: TypeSignature) -> Result<()> {
    assert_eq!(valid_type(as_str)?, ("", ty));
    Ok(())
  }

  #[rstest::rstest]
  #[case("bool", TypeSignature::Bool)]
  #[case("i8", TypeSignature::I8)]
  #[case("i16", TypeSignature::I16)]
  #[case("i32", TypeSignature::I32)]
  #[case("i64", TypeSignature::I64)]
  #[case("u8", TypeSignature::U8)]
  #[case("u16", TypeSignature::U16)]
  #[case("u32", TypeSignature::U32)]
  #[case("u64", TypeSignature::U64)]
  #[case("f32", TypeSignature::F32)]
  #[case("f32", TypeSignature::F32)]
  #[case("f64?", TypeSignature::Optional{ty:Box::new(TypeSignature::F64)} )]
  #[case("bytes", TypeSignature::Bytes)]
  #[case("string", TypeSignature::String)]
  #[case("datetime", TypeSignature::Datetime)]
  #[case("{string:string}", TypeSignature::Map { key: Box::new(TypeSignature::String), value: Box::new(TypeSignature::String) })]
  #[case("{string:string[]}", TypeSignature::Map { key: Box::new(TypeSignature::String), value: Box::new(TypeSignature::List{ty:Box::new(TypeSignature::String)}) })]
  #[case("object", TypeSignature::Object)]
  #[case("myType", TypeSignature::Custom("myType".to_owned()))]
  #[case("name::myType", TypeSignature::Custom("name::myType".to_owned()))]
  fn test_parse(#[case] as_str: &'static str, #[case] ty: TypeSignature) -> Result<()> {
    assert_eq!(parse(as_str)?, ty);
    Ok(())
  }
}
