// // #[derive(Debug, Clone, PartialEq)]
// // #[must_use]
// // /// TypeSignature with a contained value.
// // pub enum WrappedType<L, O, MK, MV>
// // where
// //   MK: Eq + std::hash::Hash,
// // {
// //   /// I8 type.
// //   I8(i8),
// //   /// I16 type.
// //   I16(i16),
// //   /// I32 type.
// //   I32(i32),
// //   /// I64 type.
// //   I64(i64),
// //   /// u8 type.
// //   U8(u8),
// //   /// u16 type.
// //   U16(u16),
// //   /// u32 type.
// //   U32(u32),
// //   /// u64 type.
// //   U64(u64),
// //   /// f32 type.
// //   F32(f32),
// //   /// f64 type.
// //   F64(f64),
// //   /// Boolean type.
// //   Bool(bool),
// //   /// String type.
// //   String(String),
// //   /// Date type.
// //   Datetime(String),
// //   /// Raw bytes.
// //   Bytes(Vec<u8>),
// //   /// Any valid value.
// //   Value(Value),
// //   /// A custom type name.
// //   Custom(Value),
// //   /// A list type
// //   List(Vec<L>),
// //   /// A type representing an optional value.
// //   Optional(Option<O>),
// //   /// A HashMap-like type.
// //   Map(std::collections::HashMap<MK, MV>),
// //   /// A type representing a link to another collection.
// //   Link(Value),
// //   /// A JSON-like key/value map.
// //   Struct(Value),
// // }

use serde_json::Value;
use wick_interface_types::TypeSignature;

#[derive(Debug, Clone)]
#[must_use]
pub struct TypeWrapper(TypeSignature, Value);

impl TypeWrapper {
  /// Create a new TypeWrapper.
  pub fn new(ty: TypeSignature, val: Value) -> Self {
    Self(ty, val)
  }

  pub fn type_signature(&self) -> &TypeSignature {
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

// #[derive(Debug)]
// #[must_use]
// pub struct TypeWrapper(TypeSignature, Box<dyn Any + Sync + Send>);

// impl TypeWrapper {
//   /// Create a new TypeWrapper.
//   pub fn new(ty: TypeSignature, val: impl Any + Sync + Send) -> Self {
//     Self(ty, Box::new(val))
//   }

//   pub fn type_signature(&self) -> &TypeSignature {
//     &self.0
//   }

//   #[must_use]
//   pub fn into_inner(self) -> Box<dyn Any + Sync + Send> {
//     self.1
//   }
// }
