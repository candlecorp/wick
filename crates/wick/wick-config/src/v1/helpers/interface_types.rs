use wick_interface_types as wick;

use super::*;
use crate::error::ManifestError;
use crate::utils::VecTryMapInto;
use crate::v1;

impl TryFrom<v1::TypeDefinition> for wick::TypeDefinition {
  type Error = ManifestError;

  fn try_from(value: v1::TypeDefinition) -> Result<Self, Self::Error> {
    Ok(match value {
      v1::TypeDefinition::StructSignature(v) => wick::TypeDefinition::Struct(v.try_into()?),
      v1::TypeDefinition::EnumSignature(v) => wick::TypeDefinition::Enum(v.try_into()?),
      v1::TypeDefinition::UnionSignature(v) => wick::TypeDefinition::Union(v.try_into()?),
    })
  }
}

impl TryFrom<wick::TypeDefinition> for v1::TypeDefinition {
  type Error = ManifestError;

  fn try_from(value: wick::TypeDefinition) -> Result<Self, Self::Error> {
    Ok(match value {
      wick::TypeDefinition::Struct(v) => v1::TypeDefinition::StructSignature(v.try_into()?),
      wick::TypeDefinition::Enum(v) => v1::TypeDefinition::EnumSignature(v.try_into()?),
      wick::TypeDefinition::Union(v) => v1::TypeDefinition::UnionSignature(v.try_into()?),
    })
  }
}

impl TryFrom<v1::UnionSignature> for wick::UnionDefinition {
  type Error = ManifestError;

  fn try_from(value: v1::UnionSignature) -> Result<Self, Self::Error> {
    Ok(Self::new(value.name, value.types.try_map_into()?, value.description))
  }
}

impl TryFrom<wick::UnionDefinition> for v1::UnionSignature {
  type Error = ManifestError;

  fn try_from(value: wick::UnionDefinition) -> Result<Self, Self::Error> {
    Ok(Self {
      name: value.name,
      description: value.description,
      types: value.types.try_map_into()?,
    })
  }
}

impl TryFrom<v1::StructSignature> for wick::StructDefinition {
  type Error = ManifestError;

  fn try_from(value: v1::StructSignature) -> Result<Self, Self::Error> {
    Ok(Self::new(value.name, value.fields.try_map_into()?, value.description))
  }
}

impl TryFrom<wick::StructDefinition> for v1::StructSignature {
  type Error = ManifestError;

  fn try_from(value: wick::StructDefinition) -> Result<Self, Self::Error> {
    Ok(Self {
      name: value.name,
      description: value.description,
      fields: value.fields.try_map_into()?,
    })
  }
}

impl TryFrom<v1::EnumSignature> for wick::EnumDefinition {
  type Error = ManifestError;

  fn try_from(value: v1::EnumSignature) -> Result<Self, Self::Error> {
    Ok(Self::new(value.name, value.variants.try_map_into()?, value.description))
  }
}

impl TryFrom<wick::EnumDefinition> for v1::EnumSignature {
  type Error = ManifestError;

  fn try_from(value: wick::EnumDefinition) -> Result<Self, Self::Error> {
    Ok(Self {
      name: value.name,
      description: value.description,
      variants: value.variants.try_map_into()?,
    })
  }
}

impl TryFrom<v1::EnumVariant> for wick::EnumVariant {
  type Error = ManifestError;

  fn try_from(value: v1::EnumVariant) -> Result<Self, Self::Error> {
    Ok(Self::new(value.name, value.index, value.value, value.description))
  }
}

impl TryFrom<wick::EnumVariant> for v1::EnumVariant {
  type Error = ManifestError;

  fn try_from(value: wick::EnumVariant) -> Result<Self, Self::Error> {
    Ok(Self {
      name: value.name,
      description: value.description,
      index: value.index,
      value: value.value,
    })
  }
}

impl TryFrom<v1::Field> for wick::Field {
  type Error = ManifestError;

  fn try_from(value: v1::Field) -> Result<Self, Self::Error> {
    Ok(Self::new_with_description(
      value.name,
      value.ty.try_into()?,
      value.description,
    ))
  }
}

impl TryFrom<wick::Field> for v1::Field {
  type Error = ManifestError;

  fn try_from(value: wick::Field) -> Result<Self, Self::Error> {
    Ok(Self {
      name: value.name,
      description: value.description,
      ty: value.ty.try_into()?,
    })
  }
}

impl FromStr for v1::TypeSignature {
  type Err = ManifestError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    wick::parse(s).map_err(ManifestError::TypeParser)?.try_into()
  }
}

impl TryFrom<v1::TypeSignature> for wick::Type {
  type Error = ManifestError;

  fn try_from(value: v1::TypeSignature) -> Result<Self, Self::Error> {
    use wick::Type as TS;
    let v = match value {
      v1::TypeSignature::I8(_) => TS::I8,
      v1::TypeSignature::I16(_) => TS::I16,
      v1::TypeSignature::I32(_) => TS::I32,
      v1::TypeSignature::I64(_) => TS::I64,
      v1::TypeSignature::U8(_) => TS::U8,
      v1::TypeSignature::U16(_) => TS::U16,
      v1::TypeSignature::U32(_) => TS::U32,
      v1::TypeSignature::U64(_) => TS::U64,
      v1::TypeSignature::F32(_) => TS::F32,
      v1::TypeSignature::F64(_) => TS::F64,
      v1::TypeSignature::Bool(_) => TS::Bool,
      v1::TypeSignature::StringType(_) => TS::String,
      v1::TypeSignature::Optional(t) => TS::Optional {
        ty: Box::new((*t.ty).try_into()?),
      },
      v1::TypeSignature::Datetime(_) => TS::Datetime,
      v1::TypeSignature::Bytes(_) => TS::Bytes,
      v1::TypeSignature::Custom(v) => TS::Named(v.name),
      v1::TypeSignature::List(t) => TS::List {
        ty: Box::new((*t.ty).try_into()?),
      },
      v1::TypeSignature::Map(t) => TS::Map {
        key: Box::new((*t.key).try_into()?),
        value: Box::new((*t.value).try_into()?),
      },
      v1::TypeSignature::Object(_) => TS::Object,
    };
    Ok(v)
  }
}

impl TryFrom<wick::Type> for v1::TypeSignature {
  type Error = ManifestError;

  fn try_from(value: wick::Type) -> Result<Self, Self::Error> {
    use v1::TypeSignature as TS;
    let v = match value {
      wick::Type::I8 => TS::I8(v1::I8),
      wick::Type::I16 => TS::I16(v1::I16),
      wick::Type::I32 => TS::I32(v1::I32),
      wick::Type::I64 => TS::I64(v1::I64),
      wick::Type::U8 => TS::U8(v1::U8),
      wick::Type::U16 => TS::U16(v1::U16),
      wick::Type::U32 => TS::U32(v1::U32),
      wick::Type::U64 => TS::U64(v1::U64),
      wick::Type::F32 => TS::F32(v1::F32),
      wick::Type::F64 => TS::F64(v1::F64),
      wick::Type::Bool => TS::Bool(v1::Bool),
      wick::Type::String => TS::StringType(v1::StringType),
      wick::Type::Optional { ty } => TS::Optional(v1::Optional {
        ty: Box::new((*ty).try_into()?),
      }),
      wick::Type::Datetime => TS::Datetime(v1::Datetime {}),
      wick::Type::Bytes => TS::Bytes(v1::Bytes {}),
      wick::Type::Named(v) => TS::Custom(v1::Custom { name: v }),
      wick::Type::List { ty } => TS::List(v1::List {
        ty: Box::new((*ty).try_into()?),
      }),
      wick::Type::Map { key, value } => TS::Map(v1::Map {
        key: Box::new((*key).try_into()?),
        value: Box::new((*value).try_into()?),
      }),
      #[allow(deprecated)]
      wick::Type::Link { .. } => unimplemented!(),
      wick::Type::Object => TS::Object(v1::Object {}),
      wick::Type::AnonymousStruct(_) => unimplemented!(),
    };
    Ok(v)
  }
}
