use bytes::BufMut;
use serde_json::Value;
use sqlx::encode::IsNull;
use sqlx::postgres::PgTypeInfo;
use sqlx::{Encode, Postgres};
use wick_interface_types::Type;
use wick_packet::parse_date;

use crate::sql_wrapper::SqlWrapper;

macro_rules! convert_int {
  ($ty:ty, $v:expr, $buf:expr) => {{
    #[allow(trivial_numeric_casts)]
    let v = $v.as_i64().unwrap() as $ty;
    Encode::<Postgres>::encode(v, $buf)
  }};
}

macro_rules! convert_uint {
  ($ty:ty, $v:expr, $buf:expr) => {{
    #[allow(trivial_numeric_casts)]
    let v = $v.as_u64().unwrap() as $ty;
    Encode::<Postgres>::encode(v, $buf)
  }};
}

impl<'q> Encode<'q, Postgres> for SqlWrapper {
  fn encode_by_ref(&self, buf: &mut <Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> IsNull {
    let sig = self.0.type_signature();
    let v = self.0.inner();
    let res = match sig {
      Type::I8 => convert_int!(i8, v, buf),
      Type::I16 => convert_int!(i16, v, buf),
      Type::I32 => convert_int!(i32, v, buf),
      Type::I64 => convert_int!(i64, v, buf),
      Type::U8 => convert_uint!(i8, v, buf),
      Type::U16 => convert_uint!(i16, v, buf),
      Type::U32 => convert_uint!(i32, v, buf),
      Type::U64 => convert_uint!(i64, v, buf),
      Type::F32 => convert_float(v, buf),
      Type::F64 => convert_float(v, buf),
      Type::Bool => {
        let v = v.as_bool().unwrap();
        Encode::<Postgres>::encode(v, buf)
      }
      Type::String => {
        let v = v.as_str().unwrap();
        Encode::<Postgres>::encode(v, buf)
      }
      Type::Datetime => {
        let datetime = parse_date(v.as_str().unwrap()).unwrap();
        Encode::<Postgres>::encode(datetime, buf)
      }
      Type::Named(_) => unimplemented!("custom types not yet handled"),
      Type::Bytes => encode_array(&Type::U8, v, buf),
      Type::List { ty } => encode_array(ty, v, buf),
      Type::Optional { .. } => {
        if v.is_null() {
          buf.put_u8(0);
          IsNull::Yes
        } else {
          Encode::<Postgres>::encode(v, buf)
        }
      }
      Type::Map { .. } => {
        let v = v.as_object().unwrap();
        buf.put_u32(v.len() as u32);
        for (k, v) in v {
          let _ = Encode::<Postgres>::encode(k, buf);
          let _ = Encode::<Postgres>::encode(v, buf);
        }
        IsNull::No
      }
      #[allow(deprecated)]
      Type::Link { .. } => unimplemented!("links not handled"),
      Type::Object => unimplemented!("objects not yet handled"),
      Type::AnonymousStruct(_) => unimplemented!("anonymous structs not yet handled"),
    };
    res
  }
}

fn encode_array(
  ty: &Type,
  v: &Value,
  buf: &mut <Postgres as sqlx::database::HasArguments<'_>>::ArgumentBuffer,
) -> IsNull {
  if matches!(
    ty,
    Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::I8 | Type::I16 | Type::I32 | Type::I64
  ) {
    let mut array = Vec::new();
    for v in v.as_array().unwrap() {
      array.push(v.as_i64().unwrap());
    }
    return Encode::<Postgres>::encode(array, buf);
  } else if ty == &Type::String {
    let mut array = Vec::new();
    for v in v.as_array().unwrap() {
      array.push(v.as_str().unwrap());
    }
    return Encode::<Postgres>::encode(array, buf);
  } else if matches!(ty, &Type::F32 | &Type::F64) {
    let mut array = Vec::new();
    for v in v.as_array().unwrap() {
      array.push(v.as_f64().unwrap());
    }
    return Encode::<Postgres>::encode(array, buf);
  }
  unreachable!()
}

fn convert_float(v: &Value, buf: &mut <Postgres as sqlx::database::HasArguments<'_>>::ArgumentBuffer) -> IsNull {
  let v: f32 = v.as_f64().unwrap() as f32;
  Encode::<Postgres>::encode(v, buf)
}

impl sqlx::Type<Postgres> for SqlWrapper {
  fn type_info() -> PgTypeInfo {
    PgTypeInfo::with_name("unknown")
  }
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use serde_json::json;

  use super::*;

  #[test]
  fn test_ints() -> Result<()> {
    let mut buf = <Postgres as sqlx::database::HasArguments<'_>>::ArgumentBuffer::default();
    convert_int!(i32, &json!(i32::MAX), &mut buf);
    convert_int!(i32, &json!(u32::MAX), &mut buf);
    convert_int!(i64, &json!(i64::MAX), &mut buf);
    convert_int!(i16, &json!(i16::MAX), &mut buf);
    convert_int!(i16, &json!(u16::MAX), &mut buf);
    convert_int!(i8, &json!(i8::MAX), &mut buf);
    convert_int!(i8, &json!(u8::MAX), &mut buf);
    Ok(())
  }
}
