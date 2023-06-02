use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde_json::{Number, Value};
use tiberius::{ColumnData, FromSql, IntoSql};
use wick_packet::{parse_date, TypeWrapper};

#[derive(Debug, Clone)]
pub(crate) struct SqlWrapper(pub(crate) TypeWrapper);

impl<'a> IntoSql<'a> for SqlWrapper {
  fn into_sql(self) -> ColumnData<'a> {
    let v = self.0.inner();
    println!("type sig is {:?}", self.0.type_signature());
    match self.0.type_signature() {
      wick_interface_types::Type::I8 => {
        let v = to_int::<i16>(v).unwrap();
        v.into_sql()
      }
      wick_interface_types::Type::I16 => to_int::<i16>(v).unwrap().into_sql(),
      wick_interface_types::Type::I32 => to_int::<i32>(v).unwrap().into_sql(),
      wick_interface_types::Type::I64 => to_int::<i64>(v).unwrap().into_sql(),
      wick_interface_types::Type::U8 => to_uint::<u8>(v).unwrap().into_sql(),
      wick_interface_types::Type::U16 => to_int::<i16>(v).unwrap().into_sql(),
      wick_interface_types::Type::U32 => to_int::<i32>(v).unwrap().into_sql(),
      wick_interface_types::Type::U64 => to_int::<i64>(v).unwrap().into_sql(),
      wick_interface_types::Type::F32 => v.as_f64().unwrap().into_sql(),
      wick_interface_types::Type::F64 => v.as_f64().unwrap().into_sql(),
      wick_interface_types::Type::Bool => v.as_bool().unwrap().into_sql(),
      wick_interface_types::Type::String => v.as_str().unwrap().to_owned().into_sql(),
      wick_interface_types::Type::Datetime => parse_date(v.as_str().unwrap()).into_sql(),
      wick_interface_types::Type::Bytes => unimplemented!("Bytes are not supported yet."),
      wick_interface_types::Type::Named(_) => unimplemented!("Custom types are not supported yet."),
      wick_interface_types::Type::List { .. } => unimplemented!("Lists are not supported yet."),
      wick_interface_types::Type::Optional { .. } => unimplemented!("Optional values are not supported yet."),
      wick_interface_types::Type::Map { .. } => unimplemented!("Maps are not supported yet."),
      #[allow(deprecated)]
      wick_interface_types::Type::Link { .. } => unimplemented!("Links are not supported yet."),
      wick_interface_types::Type::Object => unimplemented!("Objects are not supported yet."),
      wick_interface_types::Type::AnonymousStruct(_) => {
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
      ColumnData::DateTime(_)
      | ColumnData::SmallDateTime(_)
      | ColumnData::DateTime2(_)
      | ColumnData::DateTimeOffset(_) => NaiveDateTime::from_sql(col)?.map(|d| Value::String(d.to_string())),

      ColumnData::Time(_) => NaiveTime::from_sql(col)?.map(|d| Value::String(d.to_string())),
      ColumnData::Date(_) => NaiveDate::from_sql(col)?.map(|d| Value::String(d.to_string())),
    };
    Ok(value.map(FromSqlWrapper))
  }
}
