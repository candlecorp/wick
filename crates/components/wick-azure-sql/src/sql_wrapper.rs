use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde_json::{Number, Value};
use tiberius::{ColumnData, FromSql, IntoSql};
use wick_packet::TypeWrapper;

use crate::data::parse_date;

#[derive(Debug, Clone)]
pub(crate) struct SqlWrapper(pub(crate) TypeWrapper);

impl<'a> IntoSql<'a> for SqlWrapper {
  fn into_sql(self) -> ColumnData<'a> {
    let v = self.0.inner();
    match self.0.type_signature() {
      wick_interface_types::TypeSignature::I8 => {
        let v = to_int::<i16>(v).unwrap();
        v.into_sql()
      }
      wick_interface_types::TypeSignature::I16 => to_int::<i16>(v).unwrap().into_sql(),
      wick_interface_types::TypeSignature::I32 => to_int::<i32>(v).unwrap().into_sql(),
      wick_interface_types::TypeSignature::I64 => to_int::<i64>(v).unwrap().into_sql(),
      wick_interface_types::TypeSignature::U8 => to_uint::<u8>(v).unwrap().into_sql(),
      wick_interface_types::TypeSignature::U16 => to_int::<i16>(v).unwrap().into_sql(),
      wick_interface_types::TypeSignature::U32 => to_int::<i32>(v).unwrap().into_sql(),
      wick_interface_types::TypeSignature::U64 => to_int::<i64>(v).unwrap().into_sql(),
      wick_interface_types::TypeSignature::F32 => v.as_f64().unwrap().into_sql(),
      wick_interface_types::TypeSignature::F64 => v.as_f64().unwrap().into_sql(),
      wick_interface_types::TypeSignature::Bool => v.as_bool().unwrap().into_sql(),
      wick_interface_types::TypeSignature::String => v.as_str().unwrap().to_owned().into_sql(),
      wick_interface_types::TypeSignature::Datetime => parse_date(v.as_str().unwrap()).into_sql(),
      wick_interface_types::TypeSignature::Bytes => unimplemented!("Bytes are not supported yet."),
      wick_interface_types::TypeSignature::Custom(_) => unimplemented!("Custom types are not supported yet."),
      wick_interface_types::TypeSignature::Ref { .. } => unimplemented!("References are not supported yet."),
      wick_interface_types::TypeSignature::List { .. } => unimplemented!("Lists are not supported yet."),
      wick_interface_types::TypeSignature::Optional { .. } => unimplemented!("Optional values are not supported yet."),
      wick_interface_types::TypeSignature::Map { .. } => unimplemented!("Maps are not supported yet."),
      wick_interface_types::TypeSignature::Link { .. } => unimplemented!("Links are not supported yet."),
      wick_interface_types::TypeSignature::Object => unimplemented!("Objects are not supported yet."),
      wick_interface_types::TypeSignature::AnonymousStruct(_) => {
        unimplemented!("Anonymous structs are not supported yet.")
      }
    }
  }
}

fn to_int<T>(v: &Value) -> Result<T, T::Error>
where
  T: TryFrom<i64>,
  T::Error: std::fmt::Debug,
  T: std::fmt::Debug,
{
  v.as_i64().unwrap().try_into()
}

fn to_uint<T>(v: &Value) -> Result<T, T::Error>
where
  T: TryFrom<u64>,
  T::Error: std::fmt::Debug,
  T: std::fmt::Debug,
{
  v.as_u64().unwrap().try_into()
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
      ColumnData::DateTime(v) => v.map(|v| {
        NaiveDateTime::new(
          NaiveDate::from_num_days_from_ce_opt(v.days()).unwrap(),
          NaiveTime::from_num_seconds_from_midnight_opt(v.seconds_fragments() / 300, 0).unwrap(),
        )
        .to_string()
        .into()
      }),
      ColumnData::SmallDateTime(v) => v.map(|v| {
        NaiveDateTime::new(
          NaiveDate::from_num_days_from_ce_opt(v.days() as _).unwrap(),
          NaiveTime::from_num_seconds_from_midnight_opt((v.seconds_fragments() / 300) as _, 0).unwrap(),
        )
        .to_string()
        .into()
      }),

      ColumnData::Time(_) => unimplemented!("time is not supported yet"),
      ColumnData::Date(v) => v.map(|v| {
        NaiveDateTime::new(
          NaiveDate::from_num_days_from_ce_opt(v.days() as _).unwrap(),
          NaiveTime::default(),
        )
        .to_string()
        .into()
      }),
      ColumnData::DateTime2(_) => unimplemented!("DateTime2 is not supported yet"),
      ColumnData::DateTimeOffset(_) => unimplemented!("DateTimeOffset is not supported yet"),
    };
    Ok(value.map(FromSqlWrapper))
  }
}
