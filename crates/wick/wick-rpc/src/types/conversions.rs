use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use std::time::Duration;

use option_utils::OptionUtils;
use wick_interface_types as wick;
use wick_packet::{Entity, InherentData, Metadata, Packet, PacketStream, WickMetadata};

use crate::error::RpcError;
use crate::{rpc, DurationStatistics};

type Result<T> = std::result::Result<T, RpcError>;

impl TryFrom<rpc::ComponentSignature> for wick::ComponentSignature {
  type Error = RpcError;

  fn try_from(v: rpc::ComponentSignature) -> Result<Self> {
    Ok(Self {
      name: Some(v.name),
      format: match v.format {
        0 => wick::ComponentVersion::V0,
        1 => wick::ComponentVersion::V1,
        _ => {
          return Err(RpcError::Component(format!(
            "Invalid component version ({}) for this runtime",
            v.format
          )))
        }
      },
      metadata: v.metadata.try_map_into()?.ok_or(RpcError::MissingFeatures)?,
      wellknown: v
        .wellknown
        .into_iter()
        .map(|v| {
          Ok(wick::WellKnownSchema {
            capabilities: v.capabilities,
            url: v.url,
            schema: v.schema.unwrap().try_into()?,
          })
        })
        .collect::<Result<Vec<_>>>()?,
      operations: convert_list(v.operations)?,
      types: convert_list(v.types)?,
      config: convert_list(v.config)?,
    })
  }
}

impl TryFrom<rpc::ComponentMetadata> for wick::ComponentMetadata {
  type Error = RpcError;
  fn try_from(v: rpc::ComponentMetadata) -> Result<Self> {
    Ok(Self { version: v.version })
  }
}

impl TryFrom<wick::ComponentMetadata> for rpc::ComponentMetadata {
  type Error = RpcError;
  fn try_from(v: wick::ComponentMetadata) -> Result<Self> {
    Ok(Self { version: v.version })
  }
}

impl TryFrom<rpc::Operation> for wick::OperationSignature {
  type Error = RpcError;
  fn try_from(v: rpc::Operation) -> Result<Self> {
    Ok(Self {
      name: v.name,
      inputs: convert_list(v.inputs)?,
      outputs: convert_list(v.outputs)?,
    })
  }
}

impl TryFrom<wick::OperationSignature> for rpc::Operation {
  type Error = RpcError;
  fn try_from(v: wick::OperationSignature) -> Result<Self> {
    Ok(Self {
      name: v.name,
      kind: rpc::operation::OperationKind::Operation.into(),
      inputs: convert_list(v.inputs)?,
      outputs: convert_list(v.outputs)?,
    })
  }
}

impl TryFrom<wick::ComponentSignature> for rpc::ComponentSignature {
  type Error = RpcError;

  fn try_from(v: wick::ComponentSignature) -> Result<Self> {
    Ok(Self {
      name: v.name.unwrap_or_default(),
      format: v.format.into(),
      metadata: Some(v.metadata.try_into()?),
      wellknown: v
        .wellknown
        .into_iter()
        .map(|v| {
          Ok(rpc::WellKnownSchema {
            capabilities: v.capabilities,
            url: v.url,
            schema: Some(v.schema.try_into()?),
          })
        })
        .collect::<Result<Vec<_>>>()?,
      operations: convert_list(v.operations)?,
      types: convert_list(v.types)?,
      config: convert_list(v.config)?,
    })
  }
}

impl From<crate::Statistics> for rpc::Statistic {
  fn from(v: crate::Statistics) -> Self {
    Self {
      name: v.name,
      runs: v.runs,
      errors: v.errors,
      execution_statistics: v.execution_duration.map(From::from),
    }
  }
}

impl From<rpc::Statistic> for crate::Statistics {
  fn from(v: rpc::Statistic) -> Self {
    Self {
      name: v.name,
      runs: v.runs,
      errors: v.errors,
      execution_duration: v.execution_statistics.map(From::from),
    }
  }
}

impl From<rpc::DurationStatistics> for DurationStatistics {
  fn from(dur: rpc::DurationStatistics) -> Self {
    Self {
      average_time: Duration::from_micros(dur.average),
      min_time: Duration::from_micros(dur.min),
      max_time: Duration::from_micros(dur.max),
      total_time: Duration::from_micros(dur.total),
    }
  }
}

impl From<DurationStatistics> for rpc::DurationStatistics {
  fn from(dur: DurationStatistics) -> Self {
    Self {
      average: dur.average_time.as_micros().try_into().unwrap_or(u64::MAX),
      min: dur.min_time.as_micros().try_into().unwrap_or(u64::MAX),
      max: dur.max_time.as_micros().try_into().unwrap_or(u64::MAX),
      total: dur.total_time.as_micros().try_into().unwrap_or(u64::MAX),
    }
  }
}

impl From<Packet> for rpc::Packet {
  fn from(value: Packet) -> Self {
    let md = rpc::Metadata {
      flags: value.flags().into(),
      port: value.port().to_owned(),
      index: value.index(),
    };
    rpc::Packet {
      data: Some(value.payload.into()),
      metadata: Some(md),
    }
  }
}

impl From<rpc::Metadata> for Metadata {
  fn from(v: rpc::Metadata) -> Self {
    Self {
      index: v.index,
      extra: Some(WickMetadata::new(v.port, 0).encode()),
    }
  }
}

impl From<wick_packet::Invocation> for rpc::Invocation {
  fn from(inv: wick_packet::Invocation) -> Self {
    Self {
      origin: inv.origin.url(),
      target: inv.target.url(),
      id: inv.id.as_hyphenated().to_string(),
      tx_id: inv.tx_id.as_hyphenated().to_string(),
      inherent: Some(rpc::InherentData {
        seed: inv.inherent.seed,
        timestamp: inv.inherent.timestamp,
      }),
    }
  }
}

impl TryFrom<rpc::Invocation> for wick_packet::Invocation {
  type Error = RpcError;
  fn try_from(inv: rpc::Invocation) -> Result<Self> {
    let inherent = inv.inherent.ok_or(RpcError::NoInherentData)?;
    Ok(Self {
      origin: Entity::from_str(&inv.origin).map_err(|_e| RpcError::TypeConversion)?,
      target: Entity::from_str(&inv.target).map_err(|_e| RpcError::TypeConversion)?,

      id: uuid::Uuid::from_str(&inv.id).map_err(|e| RpcError::UuidParseError(inv.id, e))?,
      tx_id: uuid::Uuid::from_str(&inv.tx_id).map_err(|e| RpcError::UuidParseError(inv.tx_id, e))?,
      inherent: InherentData {
        seed: inherent.seed,
        timestamp: inherent.timestamp,
      },
      span: tracing::Span::current(),
      packets: PacketStream::empty(),
    })
  }
}

impl TryFrom<wick::Type> for rpc::TypeSignature {
  type Error = RpcError;
  fn try_from(t: wick::Type) -> Result<Self> {
    use rpc::simple_type::PrimitiveType;
    use rpc::type_signature::Signature;
    use rpc::{InnerType, MapType};
    let sig: Signature = match t {
      wick::Type::I8 => PrimitiveType::I8.into(),
      wick::Type::I16 => PrimitiveType::I16.into(),
      wick::Type::I32 => PrimitiveType::I32.into(),
      wick::Type::I64 => PrimitiveType::I64.into(),
      wick::Type::U8 => PrimitiveType::U8.into(),
      wick::Type::U16 => PrimitiveType::U16.into(),
      wick::Type::U32 => PrimitiveType::U32.into(),
      wick::Type::U64 => PrimitiveType::U64.into(),
      wick::Type::F32 => PrimitiveType::F32.into(),
      wick::Type::F64 => PrimitiveType::F64.into(),
      wick::Type::Bool => PrimitiveType::Bool.into(),
      wick::Type::String => PrimitiveType::String.into(),
      wick::Type::Datetime => PrimitiveType::Datetime.into(),
      wick::Type::Bytes => PrimitiveType::Bytes.into(),
      wick::Type::Object => PrimitiveType::Object.into(),
      wick::Type::Named(v) => Signature::Named(v),
      wick::Type::List { ty } => Signature::List(Box::new(InnerType {
        r#type: Some(ty.try_into()?),
      })),
      wick::Type::Optional { ty } => Signature::Optional(Box::new(InnerType {
        r#type: Some(ty.try_into()?),
      })),
      wick::Type::Map { key, value } => Signature::Map(Box::new(MapType {
        key_type: Some(key.try_into()?),
        value_type: Some(value.try_into()?),
      })),
      wick::Type::AnonymousStruct(v) => Signature::AnonymousStruct(rpc::AnonymousStruct {
        fields: convert_list(v)?,
      }),
      #[allow(deprecated)]
      wick::Type::Link { .. } => unimplemented!(),
    };
    Ok(Self { signature: Some(sig) })
  }
}

impl TryFrom<rpc::TypeSignature> for wick::Type {
  type Error = RpcError;
  fn try_from(t: rpc::TypeSignature) -> Result<Self> {
    use rpc::simple_type::PrimitiveType;
    use rpc::type_signature::Signature;

    type DestType = wick::Type;
    let err = Err(RpcError::InvalidSignature);
    let sig = match t.signature {
      Some(sig) => match sig {
        Signature::Simple(t) => {
          let t = PrimitiveType::from_i32(t.r#type);
          match t {
            Some(t) => match t {
              PrimitiveType::I8 => DestType::I8,
              PrimitiveType::I16 => DestType::I16,
              PrimitiveType::I32 => DestType::I32,
              PrimitiveType::I64 => DestType::I64,

              PrimitiveType::U8 => DestType::U8,
              PrimitiveType::U16 => DestType::U16,
              PrimitiveType::U32 => DestType::U32,
              PrimitiveType::U64 => DestType::U64,

              PrimitiveType::F32 => DestType::F32,
              PrimitiveType::F64 => DestType::F64,

              PrimitiveType::Bool => DestType::Bool,
              PrimitiveType::String => DestType::String,
              PrimitiveType::Datetime => DestType::Datetime,
              PrimitiveType::Bytes => DestType::Bytes,
              PrimitiveType::Object => DestType::Object,
            },
            None => return err,
          }
        }
        Signature::Named(v) => DestType::Named(v),
        Signature::Map(map) => DestType::Map {
          key: match map.key_type {
            Some(v) => v.try_into()?,
            None => return err,
          },
          value: match map.value_type {
            Some(v) => v.try_into()?,
            None => return err,
          },
        },
        Signature::List(list) => DestType::List {
          ty: match list.r#type {
            Some(v) => v.try_into()?,
            None => return err,
          },
        },
        Signature::Optional(opt) => DestType::Optional {
          ty: match opt.r#type {
            Some(v) => v.try_into()?,
            None => return err,
          },
        },
        Signature::AnonymousStruct(v) => DestType::AnonymousStruct(convert_list(v.fields)?),
      },
      None => return err,
    };
    Ok(sig)
  }
}

impl TryFrom<&wick::Type> for rpc::TypeSignature {
  type Error = RpcError;
  fn try_from(t: &wick::Type) -> Result<Self> {
    t.clone().try_into()
  }
}

impl TryFrom<Box<wick::Type>> for Box<rpc::TypeSignature> {
  type Error = RpcError;
  fn try_from(t: Box<wick::Type>) -> Result<Self> {
    Ok(Box::new((*t).try_into()?))
  }
}

impl TryFrom<Box<rpc::TypeSignature>> for Box<wick::Type> {
  type Error = RpcError;
  fn try_from(t: Box<rpc::TypeSignature>) -> Result<Self> {
    Ok(Box::new((*t).try_into()?))
  }
}

impl From<rpc::SimpleType> for rpc::type_signature::Signature {
  fn from(t: rpc::SimpleType) -> Self {
    Self::Simple(t)
  }
}

impl From<rpc::simple_type::PrimitiveType> for rpc::SimpleType {
  fn from(t: rpc::simple_type::PrimitiveType) -> Self {
    Self { r#type: t.into() }
  }
}

impl From<rpc::simple_type::PrimitiveType> for rpc::type_signature::Signature {
  fn from(t: rpc::simple_type::PrimitiveType) -> Self {
    Self::Simple(rpc::SimpleType { r#type: t.into() })
  }
}

impl TryFrom<rpc::StructSignature> for wick::StructDefinition {
  type Error = RpcError;
  fn try_from(v: rpc::StructSignature) -> Result<Self> {
    Ok(Self {
      name: v.name,
      fields: convert_list(v.fields)?,
      imported: false,
      description: Some(v.description),
    })
  }
}

impl TryFrom<rpc::TypeDefinition> for wick::TypeDefinition {
  type Error = RpcError;
  fn try_from(v: rpc::TypeDefinition) -> Result<Self> {
    let typ = v.r#type.ok_or(RpcError::Internal("No type passed"))?;
    let result = match typ {
      rpc::type_definition::Type::Struct(v) => wick::TypeDefinition::Struct(v.try_into()?),
      rpc::type_definition::Type::Enum(v) => wick::TypeDefinition::Enum(v.try_into()?),
    };
    Ok(result)
  }
}

impl TryFrom<wick::TypeDefinition> for rpc::TypeDefinition {
  type Error = RpcError;
  fn try_from(v: wick::TypeDefinition) -> Result<Self> {
    let result = match v {
      wick::TypeDefinition::Struct(v) => rpc::TypeDefinition {
        r#type: Some(rpc::type_definition::Type::Struct(v.try_into()?)),
      },
      wick::TypeDefinition::Enum(v) => rpc::TypeDefinition {
        r#type: Some(rpc::type_definition::Type::Enum(v.try_into()?)),
      },
    };
    Ok(result)
  }
}

impl TryFrom<rpc::EnumSignature> for wick::EnumDefinition {
  type Error = RpcError;
  fn try_from(v: rpc::EnumSignature) -> Result<Self> {
    Ok(wick::EnumDefinition::new(
      v.name,
      v.values.into_iter().map(|v| v.try_into()).collect::<Result<Vec<_>>>()?,
    ))
  }
}

impl TryFrom<rpc::Field> for wick::Field {
  type Error = RpcError;
  fn try_from(v: rpc::Field) -> Result<Self> {
    Ok(wick::Field::new(
      v.name,
      v.r#type.ok_or(RpcError::Internal("No type passed"))?.try_into()?,
    ))
  }
}

impl TryFrom<wick::Field> for rpc::Field {
  type Error = RpcError;
  fn try_from(v: wick::Field) -> Result<Self> {
    Ok(rpc::Field {
      name: v.name,
      r#type: Some(v.ty.try_into()?),
      description: v.description.unwrap_or_default(),
    })
  }
}

impl TryFrom<rpc::EnumVariant> for wick::EnumVariant {
  type Error = RpcError;
  fn try_from(v: rpc::EnumVariant) -> Result<Self> {
    Ok(wick::EnumVariant::new(v.name, v.index, v.value))
  }
}

impl TryFrom<wick::StructDefinition> for rpc::StructSignature {
  type Error = RpcError;
  fn try_from(v: wick::StructDefinition) -> Result<Self> {
    Ok(Self {
      name: v.name,
      fields: convert_list(v.fields)?,
      description: v.description.unwrap_or_default(),
    })
  }
}

impl TryFrom<wick::EnumDefinition> for rpc::EnumSignature {
  type Error = RpcError;
  fn try_from(v: wick::EnumDefinition) -> Result<Self> {
    Ok(Self {
      name: v.name,
      description: v.description.unwrap_or_default(),
      values: v
        .variants
        .into_iter()
        .map(|v| v.try_into())
        .collect::<Result<Vec<_>>>()?,
    })
  }
}

impl TryFrom<wick::EnumVariant> for rpc::EnumVariant {
  type Error = RpcError;
  fn try_from(v: wick::EnumVariant) -> Result<Self> {
    Ok(Self {
      name: v.name,
      index: v.index,
      value: v.value,
      description: v.description.unwrap_or_default(),
    })
  }
}

fn convert_list<TO>(list: Vec<impl TryInto<TO>>) -> Result<Vec<TO>> {
  list
    .into_iter()
    .map(|v| v.try_into().map_err(|_| RpcError::TypeConversion))
    .collect()
}
