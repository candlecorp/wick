use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use serde_json::{Number, Value};
use tiberius::{ColumnData, FromSql, IntoSql};

use crate::common::sql_wrapper::ConvertedType;

impl<'a> IntoSql<'a> for ConvertedType {
  fn into_sql(self) -> ColumnData<'a> {
    match self {
      ConvertedType::I8(v) => v.into_sql(),
      ConvertedType::I16(v) => v.into_sql(),
      ConvertedType::I32(v) => v.into_sql(),
      ConvertedType::I64(v) => v.into_sql(),
      ConvertedType::U8(v) => v.into_sql(),
      ConvertedType::U16(v) => v.into_sql(),
      ConvertedType::U32(v) => v.into_sql(),
      ConvertedType::U64(v) => v.into_sql(),
      ConvertedType::F32(v) => v.into_sql(),
      ConvertedType::F64(v) => v.into_sql(),
      ConvertedType::Bool(v) => v.into_sql(),
      ConvertedType::String(v) => v.into_sql(),
      ConvertedType::Datetime(v) => v.into_sql(),
    }
  }
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
          .map(|d| Value::String(Utc.from_utc_datetime(&d).to_rfc3339()))
      }
      ColumnData::DateTimeOffset(_) => DateTime::<Utc>::from_sql(col)?.map(|d| Value::String(d.to_rfc3339())),
      ColumnData::Time(_) => NaiveTime::from_sql(col)?.map(|d| Value::String(d.to_string())),
      ColumnData::Date(_) => NaiveDate::from_sql(col)?.map(|d| Value::String(d.to_string())),
    };
    Ok(value.map(FromSqlWrapper))
  }
}
