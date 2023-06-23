#![allow(deprecated)]
#[cfg(feature = "typeid")]
use std::any::TypeId;
use std::error::Error;
use std::str::FromStr;

mod enum_def;
mod struct_def;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use self::enum_def::{EnumDefinition, EnumVariant};
pub use self::struct_def::StructDefinition;
use crate::Field;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[must_use]
/// A valid type definition.
#[serde(tag = "type")]
pub enum TypeDefinition {
  /// A struct definition.
  #[serde(rename = "struct")]
  Struct(StructDefinition),
  /// An enum definition.
  #[serde(rename = "enum")]
  Enum(EnumDefinition),
}

impl TypeDefinition {
  /// Get the name of the type.
  #[must_use]
  pub fn name(&self) -> &str {
    match self {
      TypeDefinition::Struct(s) => &s.name,
      TypeDefinition::Enum(e) => &e.name,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[must_use]
/// Enum of valid types.
pub enum Type {
  /// I8 type.
  I8,
  /// I16 type.
  I16,
  /// I32 type.
  I32,
  /// I64 type.
  I64,
  /// u8 type.
  U8,
  /// u16 type.
  U16,
  /// u32 type.
  U32,
  /// u64 type.
  U64,
  /// f32 type.
  F32,
  /// f64 type.
  F64,
  /// Boolean type.
  Bool,
  /// String type.
  String,
  /// Date type.
  Datetime,
  /// Raw bytes.
  Bytes,
  /// A reference to another type.
  Named(String),
  /// A list type
  List {
    /// The type of the list's elements
    #[serde(rename = "type")]
    #[cfg_attr(feature = "parser", serde(deserialize_with = "crate::types::box_type_signature"))]
    #[cfg_attr(
      feature = "yaml",
      serde(serialize_with = "serde_yaml::with::singleton_map::serialize")
    )]
    ty: Box<Type>,
  },
  /// A type representing an optional value.
  Optional {
    /// The actual type that is optional.
    #[serde(rename = "type")]
    #[cfg_attr(feature = "parser", serde(deserialize_with = "crate::types::box_type_signature"))]
    #[cfg_attr(
      feature = "yaml",
      serde(serialize_with = "serde_yaml::with::singleton_map::serialize")
    )]
    ty: Box<Type>,
  },
  /// A HashMap-like type.
  Map {
    /// The type of the map's keys.
    #[cfg_attr(feature = "parser", serde(deserialize_with = "crate::types::box_type_signature"))]
    #[cfg_attr(
      feature = "yaml",
      serde(serialize_with = "serde_yaml::with::singleton_map::serialize")
    )]
    key: Box<Type>,
    /// The type of the map's values.
    #[cfg_attr(feature = "parser", serde(deserialize_with = "crate::types::box_type_signature"))]
    #[cfg_attr(
      feature = "yaml",
      serde(serialize_with = "serde_yaml::with::singleton_map::serialize")
    )]
    value: Box<Type>,
  },
  /// A type representing a link to another collection.
  #[deprecated = "Links are deprecated, use the require/provides interface instead."]
  Link {
    /// The schemas that must be provided with the linked collection.
    #[serde(default)]
    schemas: Vec<String>,
  },
  /// A JSON-like key/value map.
  Object,
  /// An inline, anonymous struct interface.
  AnonymousStruct(
    /// A list of fields in the struct.
    Vec<Field>,
  ),
}

impl Type {
  #[must_use]
  #[cfg(feature = "typeid")]
  pub fn to_type_id(&self) -> TypeId {
    match self {
      Type::I8 => TypeId::of::<i8>(),
      Type::I16 => TypeId::of::<i16>(),
      Type::I32 => TypeId::of::<i32>(),
      Type::I64 => TypeId::of::<i64>(),
      Type::U8 => TypeId::of::<u8>(),
      Type::U16 => TypeId::of::<u16>(),
      Type::U32 => TypeId::of::<u32>(),
      Type::U64 => TypeId::of::<u64>(),
      Type::F32 => TypeId::of::<f32>(),
      Type::F64 => TypeId::of::<f64>(),
      Type::Bool => TypeId::of::<bool>(),
      Type::String => TypeId::of::<String>(),
      Type::Datetime => TypeId::of::<String>(),
      Type::Bytes => TypeId::of::<Vec<u8>>(),
      Type::Named(_) => TypeId::of::<Value>(),
      Type::List { .. } => TypeId::of::<Vec<Box<dyn std::any::Any>>>(),
      Type::Optional { .. } => TypeId::of::<Option<Box<dyn std::any::Any>>>(),
      Type::Map { .. } => TypeId::of::<std::collections::HashMap<Box<dyn std::any::Any>, Box<dyn std::any::Any>>>(),
      Type::Link { .. } => TypeId::of::<Value>(),
      Type::Object => TypeId::of::<Value>(),
      Type::AnonymousStruct(_) => unimplemented!(),
    }
  }

  #[cfg(feature = "value")]
  pub fn coerce_str<'a>(&self, value: &'a str) -> Result<Value, &'a str> {
    let val = match self {
      Type::String => Value::String(value.to_owned()),
      Type::U8
      | Type::U16
      | Type::U32
      | Type::U64
      | Type::I8
      | Type::I16
      | Type::I32
      | Type::I64
      | Type::F32
      | Type::F64 => Value::Number(value.parse().map_err(|_| value)?),
      Type::Bool => Value::Bool(value.parse().map_err(|_| value)?),
      Type::Object => match serde_json::from_str(value) {
        Ok(v) => v,
        Err(_) => serde_json::from_str(&format!("\"{}\"", value)).map_err(|_| value)?,
      },
      Type::List { ty } => {
        let val: Value = serde_json::from_str(value).map_err(|_| value)?;
        if val.is_array() {
          val
        } else {
          Value::Array(vec![ty.coerce_str(value)?])
        }
      }
      Type::Datetime => Value::String(value.to_owned()),
      Type::Bytes => Value::String(value.to_owned()),
      Type::Named(_) => Value::Object(serde_json::from_str(value).map_err(|_| value)?),
      Type::Optional { ty } => {
        return Ok(ty.coerce_str(value).unwrap_or(Value::Null));
      }
      Type::Map { .. } => serde_json::from_str(value).map_err(|_| value)?,
      Type::Link { .. } => unimplemented!(),
      Type::AnonymousStruct(_) => Value::Object(serde_json::from_str(value).map_err(|_| value)?),
    };
    Ok(val)
  }
}

impl std::fmt::Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Type::I8 => f.write_str("i8"),
      Type::I16 => f.write_str("i16"),
      Type::I32 => f.write_str("i32"),
      Type::I64 => f.write_str("i64"),
      Type::U8 => f.write_str("u8"),
      Type::U16 => f.write_str("u16"),
      Type::U32 => f.write_str("u32"),
      Type::U64 => f.write_str("u64"),
      Type::F32 => f.write_str("f32"),
      Type::F64 => f.write_str("f64"),
      Type::Bool => f.write_str("bool"),
      Type::String => f.write_str("string"),
      Type::Datetime => f.write_str("datetime"),
      Type::Bytes => f.write_str("bytes"),
      Type::Named(v) => f.write_str(v),
      Type::List { ty } => write!(f, "{}[]", ty),
      Type::Optional { ty } => write!(f, "{}?", ty),
      Type::Map { key, value } => write!(f, "{{{}:{}}}", key, value),
      Type::Link { .. } => todo!(),
      Type::Object => f.write_str("object"),
      Type::AnonymousStruct(_) => todo!(),
    }
  }
}

#[derive(Debug)]
/// Error returned when attempting to convert an invalid source into a Wick type.
pub struct ParseError(String);

impl Error for ParseError {}

impl std::fmt::Display for ParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Could not parse {} into a TypeSignature.", self.0)
  }
}

#[cfg(feature = "parser")]
impl FromStr for Type {
  type Err = ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    crate::parser::parse(s).map_err(|_e| ParseError(s.to_owned()))
  }
}

#[cfg(feature = "parser")]
pub(crate) fn deserialize_type<'de, D>(deserializer: D) -> Result<Type, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct TypeSignatureVisitor;

  impl<'de> serde::de::Visitor<'de> for TypeSignatureVisitor {
    type Value = Type;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a TypeSignature definition")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Type::from_str(s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
      A: serde::de::MapAccess<'de>,
    {
      Type::deserialize(serde::de::value::MapAccessDeserializer::new(map))
    }
  }

  deserializer.deserialize_any(TypeSignatureVisitor)
}

#[cfg(feature = "parser")]
pub(crate) fn box_type_signature<'de, D>(deserializer: D) -> Result<Box<Type>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct TypeVisitor;

  impl<'de> serde::de::Visitor<'de> for TypeVisitor {
    type Value = Box<Type>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a TypeSignature definition")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Type::from_str(s)
        .map(Box::new)
        .map_err(|e| serde::de::Error::custom(e.to_string()))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
      A: serde::de::MapAccess<'de>,
    {
      Type::deserialize(serde::de::value::MapAccessDeserializer::new(map)).map(Box::new)
    }
  }

  deserializer.deserialize_any(TypeVisitor)
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use serde_json::json;

  use super::{Type as TS, *};

  fn b<T>(el: T) -> Box<T> {
    Box::new(el)
  }

  #[test]
  fn test_decode() -> Result<()> {
    let ty: Type = serde_json::from_str(r#""object""#)?;
    assert_eq!(ty, Type::Object);
    let ty: Field = serde_json::from_str(r#"{"name": "foo", "type": "object"}"#)?;
    assert_eq!(ty.name, "foo");
    assert_eq!(ty.ty, Type::Object);
    Ok(())
  }

  #[cfg(feature = "value")]
  #[rstest::rstest]
  #[case(TS::String, "foo", json!("foo"))]
  #[case(TS::U32, "48", json!(48))]
  #[case(TS::List{ty:b(TS::U32)}, "48", json!([48]))]
  #[case(TS::List{ty:b(TS::String)}, "48", json!(["48"]))]
  fn test_coerce(#[case] ty: Type, #[case] string: &str, #[case] json: Value) -> Result<()> {
    let val = ty.coerce_str(string).unwrap();

    assert_eq!(val, json);
    Ok(())
  }
}
