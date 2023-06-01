use std::borrow::BorrowMut;
use std::mem;

use bytes::BufMut;
use chrono::{NaiveDate, NaiveTime};
use serde_json::Value;
use sqlx::encode::IsNull;
use sqlx::mssql::MssqlTypeInfo;
use sqlx::{Encode, Mssql, Type as SqlxType};
use wick_interface_types::Type;
use wick_packet::{parse_date, TypeWrapper};

use crate::sql_wrapper::SqlWrapper;

impl<'q> Encode<'q, Mssql> for SqlWrapper {
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
      Type::Custom(_) => unimplemented!("Custom types are not supported yet"),
      Type::Ref { .. } => unimplemented!("References are not supported"),
      Type::List { .. } => unimplemented!("Lists are not supported yet"),
      Type::Optional { .. } => unimplemented!("Optional values are not supported yet"),
      Type::Map { .. } => unimplemented!("Maps are not supported yet"),
      Type::Link { .. } => unimplemented!("Component references are not supported"),
      Type::Object => unimplemented!("Objects are not supported yet"),
      Type::AnonymousStruct(_) => unimplemented!("Anonymous structs are not supported yet"),
    }
  }

  // sqlx doesn't expose these types so we can't implement this on the public release.
  // Holding on to this for now in case we maintain a fork in the near future.
  // fn produces(&self) -> Option<MssqlTypeInfo> {
  //   let ty = match self.0.type_signature() {
  //     TypeSignature::I8 => DataType::SmallInt,
  //     TypeSignature::I16 => DataType::Int,
  //     TypeSignature::I32 => DataType::Int,
  //     TypeSignature::I64 => DataType::BigInt,
  //     TypeSignature::U8 => DataType::TinyInt,
  //     TypeSignature::U16 => DataType::SmallInt,
  //     TypeSignature::U32 => DataType::Int,
  //     TypeSignature::U64 => DataType::BigInt,
  //     TypeSignature::F32 => DataType::Real,
  //     TypeSignature::F64 => DataType::Float,
  //     TypeSignature::Bool => DataType::Bit,
  //     TypeSignature::String => DataType::NVarChar,
  //     TypeSignature::Datetime => DataType::DateTimeN,
  //     TypeSignature::Bytes => DataType::Binary,
  //     TypeSignature::Custom(_) => todo!(),
  //     TypeSignature::Ref { .. } => todo!(),
  //     TypeSignature::List { .. } => todo!(),
  //     TypeSignature::Optional { .. } => todo!(),
  //     TypeSignature::Map { .. } => todo!(),
  //     TypeSignature::Link { .. } => todo!(),
  //     TypeSignature::Object => todo!(),
  //     TypeSignature::AnonymousStruct(_) => todo!(),
  //   };
  //   let size = Encode::<Mssql>::size_hint(&self) as u32;

  //   Some(MssqlTypeInfo(TypeInfo::new(ty, size)))
  // }

  fn encode_by_ref(&self, buf: &mut <Mssql as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> IsNull {
    let sig = self.0.type_signature();
    let v = self.0.inner();
    match sig {
      Type::I8 => convert_int::<i8>(v, buf).unwrap(),
      Type::I16 => convert_int::<i16>(v, buf).unwrap(),
      Type::I32 => {
        let v: u32 = v.as_i64().unwrap().try_into().unwrap();
        buf.extend(v.to_le_bytes());
        IsNull::No
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
        Encode::<Mssql>::encode(v, buf)
      }
      Type::String => {
        let len_pos = buf.len();
        let v = v.as_str().unwrap().encode_utf16();
        buf.put_u16_le(0u16);
        for chr in v {
          buf.put_u16_le(chr);
        }
        let length = buf.len() - len_pos - 2;

        let dst: &mut [u8] = buf.borrow_mut();
        let mut dst = &mut dst[len_pos..];
        dst.put_u16_le(length as u16);
        IsNull::No
      }
      Type::Datetime => {
        let v = v.as_str().unwrap();
        let datetime = parse_date(v);

        let days_duration = datetime.date() - NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
        let ms_duration = datetime.time() - NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let days = days_duration.num_days() as i32;
        let ms: i32 = (ms_duration.num_milliseconds() / 300) as i32;
        buf.extend(&days.to_le_bytes());
        buf.extend_from_slice(&ms.to_le_bytes());

        IsNull::No
      }
      Type::Custom(_) => unimplemented!(),
      Type::Ref { .. } => unimplemented!(),
      Type::Bytes => encode_array(&Type::U8, v, buf),
      Type::List { ty } => encode_array(ty, v, buf),
      Type::Optional { ty } => {
        if v.is_null() {
          buf.put_u8(0);
          IsNull::Yes
        } else {
          Encode::<Mssql>::encode(SqlWrapper(TypeWrapper::new(*ty.clone(), v.clone())), buf)
        }
      }
      Type::Map { value, .. } => {
        let v = v.as_object().unwrap();
        buf.put_u32(v.len() as u32);
        for (k, v) in v {
          let _ = Encode::<Mssql>::encode(k, buf);
          let _ = Encode::<Mssql>::encode(SqlWrapper(TypeWrapper::new(*value.clone(), v.clone())), buf);
        }
        IsNull::No
      }
      Type::Link { .. } => unimplemented!(),
      Type::Object => unimplemented!(),
      Type::AnonymousStruct(_) => unimplemented!(),
    }
  }
}

fn encode_array(
  _ty: &Type,
  _v: &Value,
  _buf: &mut <Mssql as sqlx::database::HasArguments<'_>>::ArgumentBuffer,
) -> IsNull {
  unimplemented!()
}

fn convert_int<'q, T>(
  v: &Value,
  buf: &mut <Mssql as sqlx::database::HasArguments<'_>>::ArgumentBuffer,
) -> Result<IsNull, T::Error>
where
  T: TryFrom<i64>,
  T::Error: std::fmt::Debug,
  T: Encode<'q, Mssql>,
  T: std::fmt::Debug,
{
  let v: T = v.as_i64().unwrap().try_into()?;
  Ok(Encode::<Mssql>::encode(v, buf))
}

fn convert_float(v: &Value, buf: &mut <Mssql as sqlx::database::HasArguments<'_>>::ArgumentBuffer) -> IsNull {
  let v = v.as_f64().unwrap();
  Encode::<Mssql>::encode(v, buf)
}

impl SqlxType<Mssql> for SqlWrapper {
  fn type_info() -> MssqlTypeInfo {
    <i32 as SqlxType<Mssql>>::type_info()
  }

  fn compatible(_ty: &<Mssql as sqlx::Database>::TypeInfo) -> bool {
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
    let mut buf = <Mssql as sqlx::database::HasArguments<'_>>::ArgumentBuffer::default();
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
