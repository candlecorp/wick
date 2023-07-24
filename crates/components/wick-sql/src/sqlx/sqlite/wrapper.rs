use std::mem;

use serde_json::Value;
use sqlx::encode::IsNull;
use sqlx::sqlite::SqliteTypeInfo;
use sqlx::{Encode, Sqlite, Type as SqlxType};
use wick_interface_types::Type;
use wick_packet::{parse_date, TypeWrapper};

use crate::common::sql_wrapper::SqlWrapper;

impl<'q> Encode<'q, Sqlite> for SqlWrapper {
  #[inline]
  fn size_hint(&self) -> usize {
    match self.0.type_signature() {
      Type::I8 => mem::size_of::<i8>(),
      Type::I16 => mem::size_of::<i16>(),
      Type::I32 => mem::size_of::<i32>(),
      Type::I64 => mem::size_of::<i64>(),
      Type::U8 => mem::size_of::<u8>(),
      Type::U16 => mem::size_of::<u16>(),
      Type::U32 => mem::size_of::<u32>(),
      Type::U64 => mem::size_of::<u64>(),
      Type::F32 => mem::size_of::<f32>(),
      Type::F64 => mem::size_of::<f64>(),
      Type::Bool => mem::size_of::<bool>(),
      Type::String => (self.0.inner().as_str().unwrap().len() + 1) * mem::size_of::<u16>(),
      Type::Datetime => mem::size_of::<u64>(),
      Type::Bytes => self.0.inner().as_array().unwrap().len() * mem::size_of::<u8>(),
      Type::Named(_) => unimplemented!("Custom types are not supported yet"),
      Type::List { .. } => unimplemented!("Lists are not supported yet"),
      Type::Optional { .. } => unimplemented!("Optional values are not supported yet"),
      Type::Map { .. } => unimplemented!("Maps are not supported yet"),
      #[allow(deprecated)]
      Type::Link { .. } => unimplemented!("Component references are not supported"),
      Type::Object => unimplemented!("Objects are not supported yet"),
      Type::AnonymousStruct(_) => unimplemented!("Anonymous structs are not supported yet"),
    }
  }

  fn encode_by_ref<'a, 'b>(
    &'a self,
    buf: &'a mut <Sqlite as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
  ) -> IsNull {
    let sig = self.0.type_signature();
    let v = self.0.inner();
    match sig {
      Type::I8 => convert_int::<i8>(v, buf).unwrap(),
      Type::I16 => convert_int::<i16>(v, buf).unwrap(),
      Type::I32 => {
        let v: u32 = v.as_i64().unwrap().try_into().unwrap();
        Encode::<Sqlite>::encode(v, buf)
      }
      Type::I64 => convert_int::<i64>(v, buf).unwrap(),
      Type::U8 => convert_int::<i8>(v, buf).unwrap(),
      Type::U16 => convert_int::<i16>(v, buf).unwrap(),
      Type::U32 => convert_int::<i32>(v, buf).unwrap(),
      Type::U64 => convert_int::<i64>(v, buf).unwrap(),
      Type::F32 => convert_float(v, buf),
      Type::F64 => convert_float(v, buf),
      Type::Bool => {
        let v = v.as_bool().unwrap();
        Encode::<Sqlite>::encode(v, buf)
      }
      Type::String => {
        let v = v.as_str().unwrap().to_owned();
        Encode::<Sqlite>::encode(v, buf)
      }
      Type::Datetime => {
        let datetime = parse_date(v.as_str().unwrap()).unwrap();
        Encode::<Sqlite>::encode(datetime, buf)
      }
      Type::Named(_) => unimplemented!(),
      Type::Bytes => encode_array(&Type::U8, v, buf),
      Type::List { ty } => encode_array(ty, v, buf),
      Type::Optional { ty } => {
        if v.is_null() {
          IsNull::Yes
        } else {
          Encode::<Sqlite>::encode(SqlWrapper(TypeWrapper::new(*ty.clone(), v.clone())), buf)
        }
      }
      Type::Map { value, .. } => {
        let v = v.as_object().unwrap();
        let _ = Encode::<Sqlite>::encode(v.len() as u32, buf);

        for (k, v) in v {
          let _ = Encode::<Sqlite>::encode(k, buf);
          let _ = Encode::<Sqlite>::encode(SqlWrapper(TypeWrapper::new(*value.clone(), v.clone())), buf);
        }
        IsNull::No
      }
      #[allow(deprecated)]
      Type::Link { .. } => unimplemented!(),
      Type::Object => unimplemented!(),
      Type::AnonymousStruct(_) => unimplemented!(),
    }
  }
}

fn encode_array(
  _ty: &Type,
  _v: &Value,
  _buf: &mut <Sqlite as sqlx::database::HasArguments<'_>>::ArgumentBuffer,
) -> IsNull {
  unimplemented!()
}

fn convert_int<'q, T>(
  v: &Value,
  buf: &mut <Sqlite as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
) -> Result<IsNull, T::Error>
where
  T: TryFrom<i64>,
  T::Error: std::fmt::Debug,
  T: Encode<'q, Sqlite>,
  T: std::fmt::Debug,
{
  let v: T = v.as_i64().unwrap().try_into()?;
  Ok(Encode::<Sqlite>::encode(v, buf))
}

fn convert_float(v: &Value, buf: &mut <Sqlite as sqlx::database::HasArguments<'_>>::ArgumentBuffer) -> IsNull {
  let v = v.as_f64().unwrap();
  Encode::<Sqlite>::encode(v, buf)
}

impl SqlxType<Sqlite> for SqlWrapper {
  fn type_info() -> SqliteTypeInfo {
    <i32 as SqlxType<Sqlite>>::type_info()
  }

  fn compatible(_ty: &<Sqlite as sqlx::Database>::TypeInfo) -> bool {
    true
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use serde_json::json;

  use super::*;

  #[test]
  fn test_ints() -> Result<()> {
    let mut buf = <Sqlite as sqlx::database::HasArguments<'_>>::ArgumentBuffer::default();
    convert_int::<i32>(&json!(i32::MAX), &mut buf)?;
    convert_int::<i64>(&json!(u32::MAX), &mut buf)?;
    convert_int::<i64>(&json!(i64::MAX), &mut buf)?;
    convert_int::<i16>(&json!(i16::MAX), &mut buf)?;
    convert_int::<i32>(&json!(u16::MAX), &mut buf)?;
    convert_int::<i8>(&json!(i8::MAX), &mut buf)?;
    convert_int::<i16>(&json!(u8::MAX), &mut buf)?;

    Ok(())
  }
}
