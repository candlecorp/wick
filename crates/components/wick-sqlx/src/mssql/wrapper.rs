use std::borrow::BorrowMut;
use std::mem;

use bytes::BufMut;
use chrono::{NaiveDate, NaiveTime};
use serde_json::Value;
use sqlx::encode::IsNull;
use sqlx::mssql::MssqlTypeInfo;
use sqlx::{Encode, Mssql, Type};
use wick_interface_types::TypeSignature;
use wick_packet::{parse_date, TypeWrapper};

use crate::sql_wrapper::SqlWrapper;

impl<'q> Encode<'q, Mssql> for SqlWrapper {
  #[inline]
  fn size_hint(&self) -> usize {
    match self.0.type_signature() {
      TypeSignature::I8 => mem::size_of::<i8>(),
      TypeSignature::I16 => mem::size_of::<i16>(),
      TypeSignature::I32 => mem::size_of::<i32>(),
      TypeSignature::I64 => mem::size_of::<i64>(),
      TypeSignature::U8 => mem::size_of::<u8>(),
      TypeSignature::U16 => mem::size_of::<u16>(),
      TypeSignature::U32 => mem::size_of::<u32>(),
      TypeSignature::U64 => mem::size_of::<u64>(),
      TypeSignature::F32 => mem::size_of::<f32>(),
      TypeSignature::F64 => mem::size_of::<f64>(),
      TypeSignature::Bool => mem::size_of::<bool>(),
      TypeSignature::String => (self.0.inner().as_str().unwrap().len() + 1) * mem::size_of::<u16>(),
      TypeSignature::Datetime => mem::size_of::<u64>(),
      TypeSignature::Bytes => self.0.inner().as_array().unwrap().len() * mem::size_of::<u8>(),
      TypeSignature::Custom(_) => unimplemented!("Custom types are not supported yet"),
      TypeSignature::Ref { .. } => unimplemented!("References are not supported"),
      TypeSignature::List { .. } => unimplemented!("Lists are not supported yet"),
      TypeSignature::Optional { .. } => unimplemented!("Optional values are not supported yet"),
      TypeSignature::Map { .. } => unimplemented!("Maps are not supported yet"),
      TypeSignature::Link { .. } => unimplemented!("Component references are not supported"),
      TypeSignature::Object => unimplemented!("Objects are not supported yet"),
      TypeSignature::AnonymousStruct(_) => unimplemented!("Anonymous structs are not supported yet"),
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
      TypeSignature::I8 => convert_int::<i8>(v, buf).unwrap(),
      TypeSignature::I16 => convert_int::<i16>(v, buf).unwrap(),
      TypeSignature::I32 => {
        let v: u32 = v.as_i64().unwrap().try_into().unwrap();
        buf.extend(v.to_le_bytes());
        IsNull::No
      }
      TypeSignature::I64 => convert_int::<i64>(v, buf).unwrap(),
      TypeSignature::U8 => convert_int::<i8>(v, buf).unwrap(),
      TypeSignature::U16 => convert_int::<i16>(v, buf).unwrap(),
      TypeSignature::U32 => convert_int::<i32>(v, buf).unwrap(),
      TypeSignature::U64 => convert_int::<i64>(v, buf).unwrap(),
      TypeSignature::F32 => convert_float(v, buf),
      TypeSignature::F64 => convert_float(v, buf),
      TypeSignature::Bool => {
        let v = v.as_bool().unwrap();
        Encode::<Mssql>::encode(v, buf)
      }
      TypeSignature::String => {
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
      TypeSignature::Datetime => {
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
      TypeSignature::Custom(_) => unimplemented!(),
      TypeSignature::Ref { .. } => unimplemented!(),
      TypeSignature::Bytes => encode_array(&TypeSignature::U8, v, buf),
      TypeSignature::List { ty } => encode_array(ty, v, buf),
      TypeSignature::Optional { ty } => {
        if v.is_null() {
          buf.put_u8(0);
          IsNull::Yes
        } else {
          Encode::<Mssql>::encode(SqlWrapper(TypeWrapper::new(*ty.clone(), v.clone())), buf)
        }
      }
      TypeSignature::Map { value, .. } => {
        let v = v.as_object().unwrap();
        buf.put_u32(v.len() as u32);
        for (k, v) in v {
          let _ = Encode::<Mssql>::encode(k, buf);
          let _ = Encode::<Mssql>::encode(SqlWrapper(TypeWrapper::new(*value.clone(), v.clone())), buf);
        }
        IsNull::No
      }
      TypeSignature::Link { .. } => unimplemented!(),
      TypeSignature::Object => unimplemented!(),
      TypeSignature::AnonymousStruct(_) => unimplemented!(),
    }
  }
}

fn encode_array(
  _ty: &TypeSignature,
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

impl Type<Mssql> for SqlWrapper {
  fn type_info() -> MssqlTypeInfo {
    <i32 as Type<Mssql>>::type_info()
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
