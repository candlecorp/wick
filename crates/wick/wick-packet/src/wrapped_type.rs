use serde_json::Value;
use wick_interface_types::Type;

use crate::Error;

#[derive(Debug, Clone)]
#[must_use]
pub struct TypeWrapper(Type, Value);

impl TypeWrapper {
  /// Create a new TypeWrapper.
  pub fn new(ty: Type, val: Value) -> Self {
    Self(ty, val)
  }

  pub fn type_signature(&self) -> &Type {
    &self.0
  }

  #[must_use]
  pub fn into_inner(self) -> Value {
    self.1
  }

  #[must_use]
  pub fn inner(&self) -> &Value {
    &self.1
  }
}

macro_rules! coersion_err {
  ($val:expr, $ty:expr) => {
    return Err(Error::Coersion {
      value: $val,
      desired: $ty,
    })
  };
}

pub(crate) fn coerce(val: Value, ty: &Type) -> Result<Value, Error> {
  let val = match ty {
    Type::I8
    | Type::I16
    | Type::I32
    | Type::I64
    | Type::U8
    | Type::U16
    | Type::U32
    | Type::U64
    | Type::F32
    | Type::F64 => match &val {
      Value::Number(_) => val,
      Value::String(v) => Value::Number(v.parse().map_err(|_| Error::Coersion {
        value: val,
        desired: ty.clone(),
      })?),
      _ => coersion_err!(val, ty.clone()),
    },
    Type::Bool => match &val {
      Value::Bool(_) => val,
      _ => coersion_err!(val, ty.clone()), // Todo: Coerce truthiness?
    },
    Type::String => {
      let string = match val {
        Value::Null => "".to_owned(),
        Value::Bool(v) => v.to_string(),
        Value::Number(v) => v.to_string(),
        Value::String(v) => v,
        _ => coersion_err!(val, ty.clone()),
      };

      Value::String(string)
    }
    Type::Datetime => match &val {
      Value::Number(_) => val,
      Value::String(_) => val,
      _ => coersion_err!(val, ty.clone()),
    },
    Type::Bytes => match &val {
      Value::String(_) => val,
      _ => coersion_err!(val, ty.clone()), // Todo: Coerce a [u8] to Base64Bytes?
    },
    Type::Named(_) => unimplemented!("named types"),
    Type::List { ty: inner_ty } => {
      let val = match val {
        Value::Array(v) => v,
        _ => coersion_err!(val, ty.clone()),
      };

      let mut out = Vec::with_capacity(val.len());
      for v in val {
        out.push(coerce(v, inner_ty)?);
      }

      Value::Array(out)
    }
    Type::Optional { ty: inner_ty } => {
      if val.is_null() {
        Value::Null
      } else {
        coerce(val, inner_ty)?
      }
    }
    Type::Map {
      value: inner_value_ty, ..
    } => {
      let obj = match val {
        Value::Object(v) => v,
        _ => coersion_err!(val, ty.clone()),
      };

      let mut out = serde_json::Map::with_capacity(obj.len());
      for (k, v) in obj {
        out.insert(k, coerce(v, inner_value_ty)?);
      }

      Value::Object(out)
    }
    #[allow(deprecated)]
    Type::Link { .. } => match &val {
      Value::String(_) => val,
      _ => coersion_err!(val, ty.clone()),
    },
    Type::Object => val,
    Type::AnonymousStruct(_) => unimplemented!(),
  };
  Ok(val)
}
