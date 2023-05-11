use bytes::BufMut;
use serde_json::Value;
use sqlx::encode::IsNull;
use sqlx::postgres::PgTypeInfo;
use sqlx::{Encode, Postgres};
use wick_interface_types::TypeSignature;

use crate::sql_wrapper::SqlWrapper;

impl<'q> Encode<'q, Postgres> for SqlWrapper {
  fn encode_by_ref(&self, buf: &mut <Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> IsNull {
    let sig = self.0.type_signature();
    let v = self.0.inner();
    let res = match sig {
      TypeSignature::I8 => convert_int::<i8>(v, buf),
      TypeSignature::I16 => convert_int::<i16>(v, buf),
      TypeSignature::I32 => convert_int::<i32>(v, buf),
      TypeSignature::I64 => convert_int::<i64>(v, buf),
      TypeSignature::U8 => convert_int::<i8>(v, buf),
      TypeSignature::U16 => convert_int::<i16>(v, buf),
      TypeSignature::U32 => convert_int::<i32>(v, buf),
      TypeSignature::U64 => panic!("u64 not yet handled"), //convert_int::<i64>(v, buf),
      TypeSignature::F32 => convert_float(v, buf),
      TypeSignature::F64 => convert_float(v, buf),
      TypeSignature::Bool => {
        let v = v.as_bool().unwrap();
        Encode::<Postgres>::encode(v, buf)
      }
      TypeSignature::String => {
        let v = v.as_str().unwrap();
        Encode::<Postgres>::encode(v, buf)
      }
      TypeSignature::Datetime => todo!(),
      TypeSignature::Custom(_) => unimplemented!(),
      TypeSignature::Ref { .. } => unimplemented!(),
      TypeSignature::Bytes => encode_array(&TypeSignature::U8, v, buf),
      TypeSignature::List { ty } => encode_array(ty, v, buf),
      TypeSignature::Optional { .. } => {
        if v.is_null() {
          buf.put_u8(0);
          IsNull::Yes
        } else {
          Encode::<Postgres>::encode(v, buf)
        }
      }
      TypeSignature::Map { .. } => {
        let v = v.as_object().unwrap();
        buf.put_u32(v.len() as u32);
        for (k, v) in v {
          let _ = Encode::<Postgres>::encode(k, buf);
          let _ = Encode::<Postgres>::encode(v, buf);
        }
        IsNull::No
      }
      TypeSignature::Link { .. } => unimplemented!(),
      TypeSignature::Object => unimplemented!(),
      TypeSignature::AnonymousStruct(_) => unimplemented!(),
    };
    res
  }
}

fn encode_array(
  ty: &TypeSignature,
  v: &Value,
  buf: &mut <Postgres as sqlx::database::HasArguments<'_>>::ArgumentBuffer,
) -> IsNull {
  if matches!(
    ty,
    TypeSignature::U8
      | TypeSignature::U16
      | TypeSignature::U32
      | TypeSignature::U64
      | TypeSignature::I8
      | TypeSignature::I16
      | TypeSignature::I32
      | TypeSignature::I64
  ) {
    let mut array = Vec::new();
    for v in v.as_array().unwrap() {
      array.push(v.as_i64().unwrap());
    }
    return Encode::<Postgres>::encode(array, buf);
  } else if ty == &TypeSignature::String {
    let mut array = Vec::new();
    for v in v.as_array().unwrap() {
      array.push(v.as_str().unwrap());
    }
    return Encode::<Postgres>::encode(array, buf);
  } else if matches!(ty, &TypeSignature::F32 | &TypeSignature::F64) {
    let mut array = Vec::new();
    for v in v.as_array().unwrap() {
      array.push(v.as_f64().unwrap());
    }
    return Encode::<Postgres>::encode(array, buf);
  }
  unreachable!()
}

fn convert_int<'q, T>(v: &Value, buf: &mut <Postgres as sqlx::database::HasArguments<'_>>::ArgumentBuffer) -> IsNull
where
  T: TryFrom<i64>,
  T::Error: std::fmt::Debug,
  T: Encode<'q, Postgres>,
{
  let v: T = v.as_i64().unwrap().try_into().unwrap();
  Encode::<Postgres>::encode(v, buf)
}

fn convert_float(v: &Value, buf: &mut <Postgres as sqlx::database::HasArguments<'_>>::ArgumentBuffer) -> IsNull {
  let v = v.as_f64().unwrap();
  Encode::<Postgres>::encode(v, buf)
}

impl sqlx::Type<Postgres> for SqlWrapper {
  fn type_info() -> PgTypeInfo {
    PgTypeInfo::with_name("unknown")
  }
}
