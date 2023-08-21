use chrono::{DateTime, Utc};
use serde_json::Value;
use wick_packet::{parse_date, TypeWrapper};

use crate::error::ConversionError;

#[derive(Debug, PartialEq, serde::Serialize)]
pub(crate) enum ConvertedType {
  I8(Option<i16>),
  I16(Option<i16>),
  I32(Option<i32>),
  I64(Option<i64>),
  U8(Option<u8>),
  U16(Option<i32>),
  U32(Option<i64>),
  U64(Option<i64>),
  F32(Option<f32>),
  F64(Option<f64>),
  Bool(Option<bool>),
  String(Option<String>),
  Datetime(Option<DateTime<Utc>>),
}

pub(crate) fn convert(wrapper: &TypeWrapper) -> Result<ConvertedType, ConversionError> {
  let ty = wrapper.type_signature().clone();
  let v = wrapper.inner();

  let data = match ty {
    wick_interface_types::Type::I8 => ConvertedType::I16(Some(to_int::<i16>(v)?)),
    wick_interface_types::Type::I16 => ConvertedType::I16(Some(to_int::<i16>(v)?)),
    wick_interface_types::Type::I32 => ConvertedType::I32(Some(to_int::<i32>(v)?)),
    wick_interface_types::Type::I64 => ConvertedType::I64(Some(to_int::<i64>(v)?)),
    wick_interface_types::Type::U8 => ConvertedType::U8(Some(to_uint::<u8>(v)?)),
    wick_interface_types::Type::U16 => ConvertedType::I16(Some(to_int::<i16>(v)?)),
    wick_interface_types::Type::U32 => ConvertedType::I32(Some(to_int::<i32>(v)?)),
    wick_interface_types::Type::U64 => ConvertedType::I64(Some(to_int::<i64>(v)?)),
    wick_interface_types::Type::F32 => ConvertedType::F64(Some(v.as_f64().ok_or(ConversionError::F64)?)),
    wick_interface_types::Type::F64 => ConvertedType::F64(Some(v.as_f64().ok_or(ConversionError::F64)?)),
    wick_interface_types::Type::Bool => ConvertedType::Bool(Some(v.as_bool().ok_or(ConversionError::Bool)?)),
    wick_interface_types::Type::String => {
      ConvertedType::String(Some(v.as_str().ok_or(ConversionError::String)?.to_owned()))
    }
    wick_interface_types::Type::Datetime => ConvertedType::Datetime(Some(
      parse_date(v.as_str().ok_or(ConversionError::Datetime)?).map_err(|_| ConversionError::Datetime)?,
    )),
    wick_interface_types::Type::Bytes => return Err(ConversionError::Bytes),
    wick_interface_types::Type::Named(_) => return Err(ConversionError::Named),
    wick_interface_types::Type::List { .. } => return Err(ConversionError::List),
    wick_interface_types::Type::Optional { ty } => {
      if v.is_null() {
        match *ty {
          wick_interface_types::Type::I8 => ConvertedType::I8(None),
          wick_interface_types::Type::I16 => ConvertedType::I16(None),
          wick_interface_types::Type::I32 => ConvertedType::I32(None),
          wick_interface_types::Type::I64 => ConvertedType::I64(None),
          wick_interface_types::Type::U8 => ConvertedType::U8(None),
          wick_interface_types::Type::U16 => ConvertedType::U16(None),
          wick_interface_types::Type::U32 => ConvertedType::U32(None),
          wick_interface_types::Type::U64 => ConvertedType::U64(None),
          wick_interface_types::Type::F32 => ConvertedType::F32(None),
          wick_interface_types::Type::F64 => ConvertedType::F64(None),
          wick_interface_types::Type::Bool => ConvertedType::Bool(None),
          wick_interface_types::Type::String => ConvertedType::String(None),
          wick_interface_types::Type::Datetime => ConvertedType::Datetime(None),
          wick_interface_types::Type::Bytes => return Err(ConversionError::Bytes),
          wick_interface_types::Type::Named(_) => return Err(ConversionError::Named),
          wick_interface_types::Type::List { .. } => return Err(ConversionError::List),
          wick_interface_types::Type::Optional { .. } => return Err(ConversionError::Optional),
          wick_interface_types::Type::Map { .. } => return Err(ConversionError::Map),
          #[allow(deprecated)]
          wick_interface_types::Type::Link { .. } => return Err(ConversionError::Link),
          wick_interface_types::Type::Object => return Err(ConversionError::Object),
          wick_interface_types::Type::AnonymousStruct(_) => return Err(ConversionError::AnonymousStruct),
        }
      } else {
        convert(&TypeWrapper::new(*ty, v.clone()))?
      }
    }
    wick_interface_types::Type::Map { .. } => return Err(ConversionError::Map),
    #[allow(deprecated)]
    wick_interface_types::Type::Link { .. } => return Err(ConversionError::Link),
    wick_interface_types::Type::Object => return Err(ConversionError::Object),
    wick_interface_types::Type::AnonymousStruct(_) => return Err(ConversionError::AnonymousStruct),
  };

  Ok(data)
}

fn to_int<T>(v: &Value) -> Result<T, ConversionError>
where
  T: TryFrom<i64>,
  T: std::fmt::Debug,
{
  v.as_i64().unwrap().try_into().map_err(|_| ConversionError::I64)
}

fn to_uint<T>(v: &Value) -> Result<T, ConversionError>
where
  T: TryFrom<u64>,
  T: std::fmt::Debug,
{
  v.as_u64().unwrap().try_into().map_err(|_| ConversionError::U64)
}
