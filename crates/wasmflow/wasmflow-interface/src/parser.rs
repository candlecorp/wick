use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_while1};
use nom::character::complete::{alpha1, alphanumeric1, char, multispace0};
use nom::character::is_alphabetic;
use nom::combinator::recognize;
use nom::error::ParseError;
use nom::multi::{many0_count, separated_list1};
use nom::sequence::{delimited, pair, tuple};
use nom::IResult;

use crate::TypeSignature;

#[derive(Debug, Clone, Copy)]
pub enum ParserError {
  Fail,
  UnexpectedToken,
}

impl std::error::Error for ParserError {}
impl std::fmt::Display for ParserError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    todo!()
  }
}

fn ws<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
  F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
  delimited(multispace0, inner, multispace0)
}

pub fn parse(input: &str) -> Result<TypeSignature, ParserError> {
  _parse(input).map(|(_, t)| t).map_err(|_| ParserError::Fail)
}

fn square_brackets(input: &str) -> IResult<&str, &str> {
  delimited(char('['), multispace0, char(']'))(input)
}

fn struct_type(input: &str) -> IResult<&str, TypeSignature> {
  let (i, (v)) = delimited(char('{'), separated_list1(char(','), ws(key_type_pair)), char('}'))(input)?;
  let v: HashMap<String, TypeSignature> = v.into_iter().map(|(s, t)| (s.to_owned(), t)).collect();
  Ok((i, TypeSignature::AnonymousStruct(v.into())))
}

fn key_type_pair(input: &str) -> IResult<&str, (&str, TypeSignature)> {
  let (i, (key, _, t)) = tuple((identifier, ws(char(':')), typename))(input)?;
  Ok((i, (key, t)))
}

fn identifier(input: &str) -> IResult<&str, &str> {
  recognize(pair(
    alt((alpha1, tag("_"))),
    many0_count(alt((alphanumeric1, tag("_")))),
  ))(input)
}

fn list_type(input: &str) -> IResult<&str, TypeSignature> {
  let (i, (t, _)) = pair(identifier, ws(square_brackets))(input)?;
  let (i, t) = typename(t)?;
  Ok((i, TypeSignature::List { element: Box::new(t) }))
}

fn typename(input: &str) -> IResult<&str, TypeSignature> {
  let (i, t) = identifier(input)?;
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
    "value" => TypeSignature::Value,
    "string" => TypeSignature::String,
    "datetime" => TypeSignature::Datetime,
    x => TypeSignature::Custom(x.to_owned()),
  };
  Ok((i, t))
}

fn _parse(input: &str) -> IResult<&str, TypeSignature> {
  let (i, t) = alt((list_type, struct_type, typename))(input)?;
  Ok((i, t))
}

#[macro_export]
macro_rules! typemap {
  (@single $($x:tt)*) => (());
  (@count $($rest:expr),*) => (<[()]>::len(&[$($crate::typemap!(@single $rest)),*]));

  // ($($key:expr => $value:expr,)+) => { $crate::typemap!($($key => $value),+) };
  ($($key:expr => $value:expr),* $(,)?) => {
      {
          let _cap = $crate::typemap!(@count $(stringify!($key)),*);
          let mut _map = ::std::collections::HashMap::with_capacity(_cap);
          $(
              let _ = _map.insert($key.to_owned(), $crate::parse($value).unwrap());
          )*
          $crate::FieldMap::from(_map)
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
      index: 0,
      name: $name.to_owned(),
      inputs: $crate::typemap! {$($ikey => $ivalue),*},
      outputs: $crate::typemap! {$($okey => $ovalue),*},
    }
  };
}

#[macro_export]
macro_rules! component {
  ($name:expr => {
    version: $version:expr,
    operations: {
      $($opname:expr => {
        inputs: {$($ikey:expr => $ivalue:expr),* $(,)?},
        outputs: {$($okey:expr => $ovalue:expr),* $(,)?},
      }),* $(,)?
    }
  }) => {{
    let mut opmap = $crate::OperationMap::default();
    $(
      let _ = opmap.insert($opname.to_owned(), $crate::operation!($opname => {inputs: {$($ikey => $ivalue),*}, outputs: {$($okey => $ovalue),*},}));
    )*;

    $crate::CollectionSignature {
      name: Some($name.to_owned()),
      features: $crate::CollectionFeatures {
        streaming: false,
        stateful: true,
        version: $crate::CollectionVersion::V0,
      },
      format: 1,
      version: $version.to_owned(),
      types: std::collections::HashMap::from([]).into(),
      operations: opmap,
      wellknown: Vec::new(),
      config: $crate::TypeMap::new(),
    }
  }};
}

#[cfg(test)]
mod test {
  use std::collections::HashMap;

  use anyhow::Result;

  use super::*;
  use crate::FieldMap;

  #[test]
  fn test_list() -> Result<()> {
    test_list_variants("bool[]")?;
    test_list_variants("bool []")?;
    test_list_variants("bool [ ]")?;
    Ok(())
  }

  fn test_list_variants(input: &'static str) -> Result<()> {
    let (i, t) = list_type(input)?;
    assert_eq!(
      t,
      TypeSignature::List {
        element: Box::new(TypeSignature::Bool),
      }
    );
    Ok(())
  }

  #[test]
  fn test_struct() -> Result<()> {
    test_struct_variants("{ myBool : bool }")?;
    test_struct_variants("{myBool:bool}")?;
    test_struct_variants("{ myBool :bool}")?;
    Ok(())
  }

  fn test_struct_variants(input: &'static str) -> Result<()> {
    let (i, t) = struct_type(input)?;
    let fields: HashMap<String, TypeSignature> = [("myBool".to_owned(), TypeSignature::Bool)].into();
    assert_eq!(t, TypeSignature::AnonymousStruct(fields.into()));
    Ok(())
  }

  #[test]
  fn test_parse_type() -> Result<()> {
    assert_eq!(typename("bool")?, ("", TypeSignature::Bool));
    assert_eq!(typename("i8")?, ("", TypeSignature::I8));
    assert_eq!(typename("i16")?, ("", TypeSignature::I16));
    assert_eq!(typename("i32")?, ("", TypeSignature::I32));
    assert_eq!(typename("i64")?, ("", TypeSignature::I64));
    assert_eq!(typename("u8")?, ("", TypeSignature::U8));
    assert_eq!(typename("u16")?, ("", TypeSignature::U16));
    assert_eq!(typename("u32")?, ("", TypeSignature::U32));
    assert_eq!(typename("u64")?, ("", TypeSignature::U64));
    assert_eq!(typename("f32")?, ("", TypeSignature::F32));
    assert_eq!(typename("f64")?, ("", TypeSignature::F64));
    assert_eq!(typename("string")?, ("", TypeSignature::String));
    assert_eq!(typename("datetime")?, ("", TypeSignature::Datetime));
    assert_eq!(typename("bytes")?, ("", TypeSignature::Bytes));
    assert_eq!(typename("custom")?, ("", TypeSignature::Custom("custom".to_owned())));

    Ok(())
  }
}
