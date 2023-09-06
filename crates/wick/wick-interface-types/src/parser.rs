use std::fmt::Debug;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1, char, multispace0};
use nom::combinator::{opt, recognize};
use nom::error::ParseError;
use nom::multi::{many0, many0_count, separated_list1};
use nom::sequence::{delimited, pair, terminated, tuple};
use nom::IResult;

use crate::{Field, Type};

#[derive(Debug, Clone)]
#[non_exhaustive]
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

pub fn parse(input: &str) -> Result<Type, ParserError> {
  _parse(input)
    .map(|(_, t)| t)
    .map_err(|_| ParserError::Fail(input.to_owned()))
}

fn square_brackets(input: &str) -> IResult<&str, &str> {
  delimited(char('['), multispace0, char(']'))(input)
}

fn struct_type(input: &str) -> IResult<&str, Type> {
  let (i, v) = delimited(char('{'), separated_list1(char(','), ws(key_type_pair)), char('}'))(input)?;
  let v: Vec<_> = v.into_iter().map(|(s, t)| Field::new(s, t)).collect();
  Ok((i, Type::AnonymousStruct(v)))
}

fn map_type(input: &str) -> IResult<&str, Type> {
  let (i, (_, ty)) = delimited(char('{'), ws(map_key_type_pair), char('}'))(input)?;

  Ok((
    i,
    Type::Map {
      key: Box::new(Type::String),
      value: Box::new(ty),
    },
  ))
}

fn key_type_pair(input: &str) -> IResult<&str, (&str, Type)> {
  let (i, (key, _, t)) = tuple((identifier, ws(char(':')), valid_type))(input)?;
  Ok((i, (key, t)))
}

fn map_key_type_pair(input: &str) -> IResult<&str, (&str, Type)> {
  let (i, (key, _, t)) = tuple((tag("string"), ws(char(':')), valid_type))(input)?;
  Ok((i, (key, t)))
}

fn identifier(input: &str) -> IResult<&str, &str> {
  recognize(pair(
    alt((alpha1, tag("_"))),
    many0_count(alt((alphanumeric1, tag("_")))),
  ))(input)
}

fn list_type(input: &str) -> IResult<&str, Type> {
  let (i, (t, _)) = pair(typename, ws(square_brackets))(input)?;

  Ok((i, Type::List { ty: Box::new(t) }))
}

fn valid_type(input: &str) -> IResult<&str, Type> {
  alt((map_type, struct_type, list_type, typename))(input)
}

fn typename(input: &str) -> IResult<&str, Type> {
  let (i, t) = alt((
    recognize(pair(many0(terminated(identifier, tag("::"))), identifier)),
    identifier,
  ))(input)?;
  let t = match t {
    "bool" => Type::Bool,
    "i8" => Type::I8,
    "i16" => Type::I16,
    "i32" => Type::I32,
    "i64" => Type::I64,
    "u8" => Type::U8,
    "u16" => Type::U16,
    "u32" => Type::U32,
    "u64" => Type::U64,
    "int" => Type::I64,
    "uint" => Type::U64,
    "float" => Type::F64,
    "f32" => Type::F32,
    "f64" => Type::F64,
    "bytes" => Type::Bytes,
    "string" => Type::String,
    "datetime" => Type::Datetime,
    "object" => Type::Object,
    x => Type::Named(x.to_owned()),
  };
  Ok((i, t))
}

fn _parse(input: &str) -> IResult<&str, Type> {
  let (i, (t, optional)) = pair(alt((list_type, map_type, struct_type, typename)), opt(tag("?")))(input)?;
  if optional.is_some() {
    Ok((i, Type::Optional { ty: Box::new(t) }))
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
    config: {$($ckey:expr => $cvalue:expr),* $(,)?},
    inputs: {$($ikey:expr => $ivalue:expr),* $(,)?},
    outputs: {$($okey:expr => $ovalue:expr),* $(,)?},
  }) => {
    $crate::OperationSignature::new(
      $name.to_owned(),
      $crate::fields! {$($ikey => $ivalue),*},
      $crate::fields! {$($okey => $ovalue),*},
      $crate::fields! {$($ckey => $cvalue),*},
    )
  };
  ($name:expr => {
    inputs: {$($ikey:expr => $ivalue:expr),* $(,)?},
    outputs: {$($okey:expr => $ovalue:expr),* $(,)?},
  }) => {
    $crate::OperationSignature::new(
      $name.to_owned(),
      $crate::fields! {$($ikey => $ivalue),*},
      $crate::fields! {$($okey => $ovalue),*},
      Vec::new()
    )
  };
}

#[macro_export]
macro_rules! component {
  (
    name: $name:expr,
    version: $version:expr,
    operations: {
      $($opname:expr => {
        config: {$($ckey:expr => $cvalue:expr),* $(,)?},
        inputs: {$($ikey:expr => $ivalue:expr),* $(,)?},
        outputs: {$($okey:expr => $ovalue:expr),* $(,)?},
      }),* $(,)?
    }
  ) => {{
    let mut ops = std::vec::Vec::default();
    $(
      let _ = ops.push($crate::operation!($opname => {config:{$($ckey => $cvalue),*}, inputs: {$($ikey => $ivalue),*}, outputs: {$($okey => $ovalue),*},}));
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
    operations: {
      $($opname:expr => {
        inputs: {$($ikey:expr => $ivalue:expr),* $(,)?},
        outputs: {$($okey:expr => $ovalue:expr),* $(,)?},
      }),* $(,)?
    }
  ) => {{
    let mut ops = std::vec::Vec::default();
    $(
      let _ = ops.push($crate::operation!($opname => {config:{}, inputs: {$($ikey => $ivalue),*}, outputs: {$($okey => $ovalue),*},}));
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
    $crate::ComponentSignature::new($name.to_owned(),$version.map(|v|v.into()),$ops,Vec::new(),Vec::new())
  }};
}

#[cfg(test)]
mod test {

  use anyhow::Result;

  use super::*;
  use crate::Field;

  #[rstest::rstest]
  #[case("bool[]", Type::Bool)]
  #[case("bool []", Type::Bool)]
  #[case("bool [ ]", Type::Bool)]
  #[case("string[]", Type::String)]
  fn test_list_variants(#[case] input: &'static str, #[case] expected: Type) -> Result<()> {
    let (_i, t) = list_type(input)?;
    assert_eq!(t, Type::List { ty: Box::new(expected) });
    Ok(())
  }

  #[rstest::rstest]
  #[case("{ myBool : bool }", Type::Bool)]
  #[case("{myBool:bool}", Type::Bool)]
  #[case("{ myBool : bool }", Type::Bool)]
  fn test_struct_variants(#[case] input: &'static str, #[case] expected: Type) -> Result<()> {
    let (_i, t) = struct_type(input)?;
    let fields = [Field::new("myBool", expected)];
    assert_eq!(t, Type::AnonymousStruct(fields.into()));
    Ok(())
  }

  #[rstest::rstest]
  #[case("bool", Type::Bool)]
  #[case("i8", Type::I8)]
  #[case("i16", Type::I16)]
  #[case("i32", Type::I32)]
  #[case("i64", Type::I64)]
  #[case("int", Type::I64)]
  #[case("uint", Type::U64)]
  #[case("float", Type::F64)]
  #[case("u8", Type::U8)]
  #[case("u16", Type::U16)]
  #[case("u32", Type::U32)]
  #[case("u64", Type::U64)]
  #[case("f32", Type::F32)]
  #[case("f64", Type::F64)]
  #[case("bytes", Type::Bytes)]
  #[case("string", Type::String)]
  #[case("datetime", Type::Datetime)]
  #[case("object", Type::Object)]
  #[case("myType", Type::Named("myType".to_owned()))]
  #[case("name::myType", Type::Named("name::myType".to_owned()))]
  fn test_parse_typename(#[case] as_str: &'static str, #[case] ty: Type) -> Result<()> {
    assert_eq!(typename(as_str)?, ("", ty));
    Ok(())
  }

  #[rstest::rstest]
  #[case("bool", Type::Bool)]
  #[case("i8", Type::I8)]
  #[case("i16", Type::I16)]
  #[case("i32", Type::I32)]
  #[case("i64", Type::I64)]
  #[case("u8", Type::U8)]
  #[case("u16", Type::U16)]
  #[case("u32", Type::U32)]
  #[case("u64", Type::U64)]
  #[case("f32", Type::F32)]
  #[case("f64", Type::F64)]
  #[case("bytes", Type::Bytes)]
  #[case("string", Type::String)]
  #[case("datetime", Type::Datetime)]
  #[case("object", Type::Object)]
  #[case("myType[]", Type::List{ty:Box::new(Type::Named("myType".to_owned()))})]
  #[case("{string: bool}", Type::Map{key:Box::new(Type::String),value:Box::new(Type::Bool)})]
  #[case("{string: bool[]}", Type::Map{key:Box::new(Type::String),value:Box::new(Type::List{ty:Box::new(Type::Bool)})})]
  #[case("{string: name::myType}", Type::Map{key:Box::new(Type::String),value:Box::new(Type::Named("name::myType".to_owned()))})]
  #[case("name::myType", Type::Named("name::myType".to_owned()))]
  fn test_parse_type(#[case] as_str: &'static str, #[case] ty: Type) -> Result<()> {
    assert_eq!(valid_type(as_str)?, ("", ty));
    Ok(())
  }

  #[rstest::rstest]
  #[case("bool", Type::Bool)]
  #[case("i8", Type::I8)]
  #[case("i16", Type::I16)]
  #[case("i32", Type::I32)]
  #[case("i64", Type::I64)]
  #[case("u8", Type::U8)]
  #[case("u16", Type::U16)]
  #[case("u32", Type::U32)]
  #[case("u64", Type::U64)]
  #[case("f32", Type::F32)]
  #[case("f32", Type::F32)]
  #[case("f64?", Type::Optional{ty:Box::new(Type::F64)} )]
  #[case("bytes", Type::Bytes)]
  #[case("string", Type::String)]
  #[case("datetime", Type::Datetime)]
  #[case("{string:string}", Type::Map { key: Box::new(Type::String), value: Box::new(Type::String) })]
  #[case("{string:string[]}", Type::Map { key: Box::new(Type::String), value: Box::new(Type::List{ty:Box::new(Type::String)}) })]
  #[case("object", Type::Object)]
  #[case("myType", Type::Named("myType".to_owned()))]
  #[case("name::myType", Type::Named("name::myType".to_owned()))]
  fn test_parse(#[case] as_str: &'static str, #[case] ty: Type) -> Result<()> {
    assert_eq!(parse(as_str)?, ty);
    Ok(())
  }
}
