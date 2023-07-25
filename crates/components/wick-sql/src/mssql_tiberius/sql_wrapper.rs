use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde_json::{Number, Value};
use tiberius::{ColumnData, FromSql, IntoSql};
use wick_packet::{parse_date, TypeWrapper};

use crate::common::sql_wrapper::SqlWrapper;

#[derive(thiserror::Error, Debug, Copy, Clone)]
pub enum MsSqlConversionError {
  #[error("i8")]
  I8,
  #[error("i16")]
  I16,
  #[error("i32")]
  I32,
  #[error("i64")]
  I64,
  #[error("u8")]
  U8,
  #[error("u16")]
  U16,
  #[error("u32")]
  U32,
  #[error("u64")]
  U64,
  #[error("f32")]
  F32,
  #[error("f64")]
  F64,
  #[error("bool")]
  Bool,
  #[error("string")]
  String,
  #[error("datetime")]
  Datetime,
  #[error("bytes")]
  Bytes,
  #[error("named")]
  Named,
  #[error("list")]
  List,
  #[error("optional")]
  Optional,
  #[error("map")]
  Map,
  #[error("link")]
  Link,
  #[error("object")]
  Object,
  #[error("anonymous struct")]
  AnonymousStruct,
}

// The converted types that we *know* we can encode into SQL.
pub(super) enum MsSqlWrapper {
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

impl<'a> IntoSql<'a> for MsSqlWrapper {
  fn into_sql(self) -> ColumnData<'a> {
    match self {
      MsSqlWrapper::I8(v) => v.into_sql(),
      MsSqlWrapper::I16(v) => v.into_sql(),
      MsSqlWrapper::I32(v) => v.into_sql(),
      MsSqlWrapper::I64(v) => v.into_sql(),
      MsSqlWrapper::U8(v) => v.into_sql(),
      MsSqlWrapper::U16(v) => v.into_sql(),
      MsSqlWrapper::U32(v) => v.into_sql(),
      MsSqlWrapper::U64(v) => v.into_sql(),
      MsSqlWrapper::F32(v) => v.into_sql(),
      MsSqlWrapper::F64(v) => v.into_sql(),
      MsSqlWrapper::Bool(v) => v.into_sql(),
      MsSqlWrapper::String(v) => v.into_sql(),
      MsSqlWrapper::Datetime(v) => v.into_sql(),
    }
  }
}

impl TryFrom<&SqlWrapper> for MsSqlWrapper {
  type Error = MsSqlConversionError;

  fn try_from(wrapper: &SqlWrapper) -> Result<Self, Self::Error> {
    convert(wrapper)
  }
}

fn convert(wrapper: &SqlWrapper) -> Result<MsSqlWrapper, MsSqlConversionError> {
  let ty = wrapper.0.type_signature().clone();
  let v = wrapper.0.inner();

  let data = match ty {
    wick_interface_types::Type::I8 => MsSqlWrapper::I16(Some(to_int::<i16>(v)?)),
    wick_interface_types::Type::I16 => MsSqlWrapper::I16(Some(to_int::<i16>(v)?)),
    wick_interface_types::Type::I32 => MsSqlWrapper::I32(Some(to_int::<i32>(v)?)),
    wick_interface_types::Type::I64 => MsSqlWrapper::I64(Some(to_int::<i64>(v)?)),
    wick_interface_types::Type::U8 => MsSqlWrapper::U8(Some(to_uint::<u8>(v)?)),
    wick_interface_types::Type::U16 => MsSqlWrapper::I16(Some(to_int::<i16>(v)?)),
    wick_interface_types::Type::U32 => MsSqlWrapper::I32(Some(to_int::<i32>(v)?)),
    wick_interface_types::Type::U64 => MsSqlWrapper::I64(Some(to_int::<i64>(v)?)),
    wick_interface_types::Type::F32 => MsSqlWrapper::F64(Some(v.as_f64().ok_or(MsSqlConversionError::F64)?)),
    wick_interface_types::Type::F64 => MsSqlWrapper::F64(Some(v.as_f64().ok_or(MsSqlConversionError::F64)?)),
    wick_interface_types::Type::Bool => MsSqlWrapper::Bool(Some(v.as_bool().ok_or(MsSqlConversionError::Bool)?)),
    wick_interface_types::Type::String => {
      MsSqlWrapper::String(Some(v.as_str().ok_or(MsSqlConversionError::String)?.to_owned()))
    }
    wick_interface_types::Type::Datetime => MsSqlWrapper::Datetime(Some(
      parse_date(v.as_str().ok_or(MsSqlConversionError::Datetime)?).map_err(|_| MsSqlConversionError::Datetime)?,
    )),
    wick_interface_types::Type::Bytes => return Err(MsSqlConversionError::Bytes),
    wick_interface_types::Type::Named(_) => return Err(MsSqlConversionError::Named),
    wick_interface_types::Type::List { .. } => return Err(MsSqlConversionError::List),
    wick_interface_types::Type::Optional { ty } => {
      if v.is_null() {
        match *ty {
          wick_interface_types::Type::I8 => MsSqlWrapper::I8(None),
          wick_interface_types::Type::I16 => MsSqlWrapper::I16(None),
          wick_interface_types::Type::I32 => MsSqlWrapper::I32(None),
          wick_interface_types::Type::I64 => MsSqlWrapper::I64(None),
          wick_interface_types::Type::U8 => MsSqlWrapper::U8(None),
          wick_interface_types::Type::U16 => MsSqlWrapper::U16(None),
          wick_interface_types::Type::U32 => MsSqlWrapper::U32(None),
          wick_interface_types::Type::U64 => MsSqlWrapper::U64(None),
          wick_interface_types::Type::F32 => MsSqlWrapper::F32(None),
          wick_interface_types::Type::F64 => MsSqlWrapper::F64(None),
          wick_interface_types::Type::Bool => MsSqlWrapper::Bool(None),
          wick_interface_types::Type::String => MsSqlWrapper::String(None),
          wick_interface_types::Type::Datetime => MsSqlWrapper::Datetime(None),
          wick_interface_types::Type::Bytes => return Err(MsSqlConversionError::Bytes),
          wick_interface_types::Type::Named(_) => return Err(MsSqlConversionError::Named),
          wick_interface_types::Type::List { .. } => return Err(MsSqlConversionError::List),
          wick_interface_types::Type::Optional { .. } => return Err(MsSqlConversionError::Optional),
          wick_interface_types::Type::Map { .. } => return Err(MsSqlConversionError::Map),
          #[allow(deprecated)]
          wick_interface_types::Type::Link { .. } => return Err(MsSqlConversionError::Link),
          wick_interface_types::Type::Object => return Err(MsSqlConversionError::Object),
          wick_interface_types::Type::AnonymousStruct(_) => return Err(MsSqlConversionError::AnonymousStruct),
        }
      } else {
        convert(&SqlWrapper(TypeWrapper::new(*ty, v.clone())))?
      }
    }
    wick_interface_types::Type::Map { .. } => return Err(MsSqlConversionError::Map),
    #[allow(deprecated)]
    wick_interface_types::Type::Link { .. } => return Err(MsSqlConversionError::Link),
    wick_interface_types::Type::Object => return Err(MsSqlConversionError::Object),
    wick_interface_types::Type::AnonymousStruct(_) => return Err(MsSqlConversionError::AnonymousStruct),
  };

  Ok(data)
}

fn to_int<T>(v: &Value) -> Result<T, MsSqlConversionError>
where
  T: TryFrom<i64>,
  T: std::fmt::Debug,
{
  v.as_i64().unwrap().try_into().map_err(|_| MsSqlConversionError::I64)
}

fn to_uint<T>(v: &Value) -> Result<T, MsSqlConversionError>
where
  T: TryFrom<u64>,
  T: std::fmt::Debug,
{
  v.as_u64().unwrap().try_into().map_err(|_| MsSqlConversionError::U64)
}

pub(crate) struct FromSqlWrapper(pub(crate) Value);

impl<'a> FromSql<'a> for FromSqlWrapper {
  fn from_sql(col: &'a ColumnData<'static>) -> tiberius::Result<Option<Self>> {
    let value: Option<Value> = match col {
      ColumnData::U8(v) => v.map(Into::into),
      ColumnData::I16(v) => v.map(Into::into),
      ColumnData::I32(v) => v.map(Into::into),
      ColumnData::I64(v) => v.map(Into::into),
      ColumnData::F32(v) => v.map(Into::into),
      ColumnData::F64(v) => v.map(Into::into),
      ColumnData::Bit(v) => v.map(Into::into),
      ColumnData::String(v) => v.clone().map(|v| v.into()),
      ColumnData::Guid(v) => v.map(|v| Value::from(v.to_string())),
      ColumnData::Binary(v) => v
        .clone()
        .map(|v| Value::Array(v.iter().copied().map(|v| Value::Number(Number::from(v))).collect())),
      ColumnData::Numeric(v) => v.map(|v| {
        let v: i64 = v.value().try_into().unwrap();
        Value::Number(Number::from(v))
      }),
      ColumnData::Xml(_) => unimplemented!(),
      ColumnData::DateTime(_) | ColumnData::SmallDateTime(_) | ColumnData::DateTime2(_) => {
        tiberius::time::chrono::NaiveDateTime::from_sql(col)?
          .map(|d| Value::String(DateTime::<Utc>::from_utc(d, Utc).to_rfc3339()))
      }
      ColumnData::DateTimeOffset(_) => DateTime::<Utc>::from_sql(col)?.map(|d| Value::String(d.to_rfc3339())),
      ColumnData::Time(_) => NaiveTime::from_sql(col)?.map(|d| Value::String(d.to_string())),
      ColumnData::Date(_) => NaiveDate::from_sql(col)?.map(|d| Value::String(d.to_string())),
    };
    Ok(value.map(FromSqlWrapper))
  }
}
