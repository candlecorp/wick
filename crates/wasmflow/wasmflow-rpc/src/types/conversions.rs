use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use std::time::Duration;

use wasmflow_sdk::v1 as sdk;
use wasmflow_sdk::v1::packet::Packet;
use wasmflow_sdk::v1::transport::TransportMap;
use wasmflow_sdk::v1::{types as wasmflow, Entity};

use crate::error::RpcError;
use crate::rpc::{InternalType, StructType};
use crate::{rpc, DurationStatistics};

type Result<T> = std::result::Result<T, RpcError>;

impl TryFrom<wasmflow::HostedType> for rpc::CollectionSignature {
  type Error = RpcError;

  fn try_from(v: wasmflow::HostedType) -> Result<Self> {
    Ok(match v {
      wasmflow::HostedType::Collection(v) => v.try_into()?,
    })
  }
}

impl TryFrom<wasmflow::HostedType> for rpc::HostedType {
  type Error = RpcError;

  fn try_from(value: wasmflow::HostedType) -> Result<Self> {
    use rpc::hosted_type::Type;
    Ok(match value {
      wasmflow::HostedType::Collection(p) => Self {
        r#type: Some(Type::Collection(p.try_into()?)),
      },
    })
  }
}

impl TryFrom<rpc::HostedType> for wasmflow::HostedType {
  type Error = RpcError;

  fn try_from(value: rpc::HostedType) -> Result<Self> {
    use rpc::hosted_type::Type;
    match value.r#type {
      Some(v) => match v {
        Type::Collection(sig) => Ok(wasmflow::HostedType::Collection(sig.try_into()?)),
      },
      None => Err(RpcError::InvalidSignature),
    }
  }
}

impl TryFrom<rpc::CollectionSignature> for wasmflow::CollectionSignature {
  type Error = RpcError;

  fn try_from(v: rpc::CollectionSignature) -> Result<Self> {
    Ok(Self {
      name: Some(v.name),
      features: v
        .features
        .map(|v| v.try_into())
        .transpose()?
        .ok_or(RpcError::MissingFeatures)?,
      version: v.version,
      format: v.format,
      wellknown: v
        .wellknown
        .into_iter()
        .map(|v| {
          Ok(wasmflow::WellKnownSchema {
            capabilities: v.capabilities,
            url: v.url,
            schema: v.schema.unwrap().try_into()?,
          })
        })
        .collect::<Result<Vec<_>>>()?,
      components: to_componentmap(v.components)?,
      types: to_typemap(v.types)?,
      config: to_typemap(v.config)?,
    })
  }
}

impl TryFrom<rpc::CollectionFeatures> for wasmflow::CollectionFeatures {
  type Error = RpcError;
  fn try_from(v: rpc::CollectionFeatures) -> Result<Self> {
    Ok(Self {
      streaming: v.streaming,
      stateful: v.stateful,
      version: match v.version {
        0 => wasmflow::CollectionVersion::V0,
        _ => {
          return Err(RpcError::CollectionError(format!(
            "Invalid collection version ({}) for this runtime",
            v.version
          )))
        }
      },
    })
  }
}

impl TryFrom<wasmflow::CollectionFeatures> for rpc::CollectionFeatures {
  type Error = RpcError;
  fn try_from(v: wasmflow::CollectionFeatures) -> Result<Self> {
    Ok(Self {
      streaming: v.streaming,
      stateful: v.stateful,
      version: v.version.into(),
    })
  }
}

impl TryFrom<rpc::Component> for wasmflow::ComponentSignature {
  type Error = RpcError;
  fn try_from(v: rpc::Component) -> Result<Self> {
    Ok(Self {
      name: v.name,
      inputs: to_fieldmap(v.inputs)?,
      outputs: to_fieldmap(v.outputs)?,
    })
  }
}

impl TryFrom<wasmflow::ComponentSignature> for rpc::Component {
  type Error = RpcError;
  fn try_from(v: wasmflow::ComponentSignature) -> Result<Self> {
    Ok(Self {
      name: v.name,
      kind: rpc::component::ComponentKind::Component.into(),
      inputs: from_fieldmap(v.inputs)?,
      outputs: from_fieldmap(v.outputs)?,
    })
  }
}

impl TryFrom<wasmflow::CollectionSignature> for rpc::CollectionSignature {
  type Error = RpcError;

  fn try_from(v: wasmflow::CollectionSignature) -> Result<Self> {
    Ok(Self {
      name: v.name.unwrap_or_default(),
      features: Some(v.features.try_into()?),
      version: v.version,
      format: v.format,
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
      components: from_componentmap(v.components)?,
      types: from_typemap(v.types)?,
      config: from_typemap(v.config)?,
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
      average: Duration::from_micros(dur.average),
      min_time: Duration::from_micros(dur.min),
      max_time: Duration::from_micros(dur.max),
    }
  }
}

impl From<DurationStatistics> for rpc::DurationStatistics {
  fn from(dur: DurationStatistics) -> Self {
    Self {
      average: dur.average.as_micros().try_into().unwrap_or(u64::MAX),
      min: dur.min_time.as_micros().try_into().unwrap_or(u64::MAX),
      max: dur.max_time.as_micros().try_into().unwrap_or(u64::MAX),
    }
  }
}

#[allow(clippy::from_over_into)]
impl Into<Packet> for rpc::Packet {
  fn into(self) -> Packet {
    use rpc::packet::Data;
    use wasmflow_sdk::v1::packet::v1;

    if self.data.is_none() {
      return v1::Packet::error("Invalid RPC packet. Received message but no data.").into();
    }
    let data = self.data.unwrap();

    let packet = match data {
      Data::Success(v) => {
        let payload = v.payload.and_then(|v| v.data);
        match payload {
          Some(rpc::payload_data::Data::Messagepack(v)) => v1::Packet::Success(v1::Serialized::MessagePack(v)),
          Some(rpc::payload_data::Data::Json(v)) => v1::Packet::Success(v1::Serialized::Json(v)),
          None => v1::Packet::error("Invalid RPC packet. Received Success packet but no payload."),
        }
      }
      Data::Failure(v) => {
        let kind = rpc::failure::FailureKind::from_i32(v.r#type);
        match kind {
          Some(rpc::failure::FailureKind::Error) => v1::Packet::error(v.payload),
          Some(rpc::failure::FailureKind::Exception) => v1::Packet::exception(v.payload),
          None => v1::Packet::error(format!(
            "Invalid RPC packet. Received Failure packet with invalid kind '{}'.",
            v.r#type
          )),
        }
      }
      Data::Signal(v) => {
        let kind = rpc::signal::OutputSignal::from_i32(v.r#type);

        match kind {
          None => v1::Packet::error(format!(
            "Invalid RPC packet. Received Signal packet with invalid kind '{}'.",
            v.r#type
          )),
          Some(kind) => match kind {
            rpc::signal::OutputSignal::Done => v1::Packet::Signal(v1::Signal::Done),
            rpc::signal::OutputSignal::OpenBracket => v1::Packet::Signal(v1::Signal::OpenBracket),
            rpc::signal::OutputSignal::CloseBracket => v1::Packet::Signal(v1::Signal::CloseBracket),
          },
        }
      }
    };
    packet.into()
  }
}

/// Converts a HashMap of [MessageKind] to a [TransportMap].
pub fn convert_messagekind_map(rpc_map: HashMap<String, rpc::Packet>) -> TransportMap {
  let mut transport_map = TransportMap::default();
  for (k, v) in rpc_map {
    transport_map.insert(k, v.into());
  }
  transport_map
}

/// Converts a [TransportMap] to a HashMap of [MessageKind].
#[must_use]
pub fn convert_transport_map(transport_map: TransportMap) -> HashMap<String, rpc::Packet> {
  let mut rpc_map: HashMap<String, rpc::Packet> = HashMap::new();
  for (k, v) in transport_map.into_inner() {
    rpc_map.insert(k, v.clone().into());
  }
  rpc_map
}

impl TryFrom<sdk::Invocation> for rpc::Invocation {
  type Error = RpcError;
  fn try_from(inv: sdk::Invocation) -> Result<Self> {
    Ok(Self {
      origin: inv.origin.url(),
      target: inv.target.url(),
      payload: convert_transport_map(inv.payload),
      id: inv.id.as_hyphenated().to_string(),
      tx_id: inv.tx_id.as_hyphenated().to_string(),
      inherent: inv.inherent.map(|d| rpc::InherentData {
        seed: d.seed,
        timestamp: d.timestamp,
      }),
      config: normalize_serialization_out(inv.config)?,
    })
  }
}

impl TryFrom<sdk::transport::Serialized> for rpc::Serialized {
  type Error = RpcError;
  fn try_from(v: sdk::transport::Serialized) -> Result<Self> {
    let result = match v {
      sdk::transport::Serialized::MessagePack(v) => rpc::Serialized {
        payload: Some(rpc::PayloadData {
          data: Some(rpc::payload_data::Data::Messagepack(v)),
        }),
      },
      sdk::transport::Serialized::Struct(v) => rpc::Serialized {
        payload: Some(rpc::PayloadData {
          data: Some(rpc::payload_data::Data::Messagepack(
            sdk::codec::messagepack::serialize(&v).unwrap(),
          )),
        }),
      },
      sdk::transport::Serialized::Json(v) => rpc::Serialized {
        payload: Some(rpc::PayloadData {
          data: Some(rpc::payload_data::Data::Json(v)),
        }),
      },
    };
    Ok(result)
  }
}

impl TryFrom<rpc::Serialized> for sdk::transport::Serialized {
  type Error = RpcError;
  fn try_from(v: rpc::Serialized) -> Result<Self> {
    let data = v.payload.and_then(|v| v.data);

    match data {
      Some(rpc::payload_data::Data::Messagepack(v)) => Ok(sdk::transport::Serialized::MessagePack(v)),
      Some(rpc::payload_data::Data::Json(v)) => Ok(sdk::transport::Serialized::Json(v)),
      None => Err(RpcError::Internal(
        "Invalid RPC message, serialized data did not contain a payload",
      )),
    }
  }
}

fn normalize_serialization_in(packet: Option<rpc::Serialized>) -> Result<Option<sdk::transport::Serialized>> {
  match packet {
    Some(packet) => Ok(Some(packet.try_into()?)),
    None => Ok(None),
  }
}

fn normalize_serialization_out(packet: Option<sdk::transport::Serialized>) -> Result<Option<rpc::Serialized>> {
  match packet {
    Some(packet) => Ok(Some(packet.try_into()?)),
    None => Ok(None),
  }
}

impl TryFrom<rpc::Invocation> for sdk::Invocation {
  type Error = RpcError;
  fn try_from(inv: rpc::Invocation) -> Result<Self> {
    let config = normalize_serialization_in(inv.config)?;
    Ok(Self {
      origin: Entity::from_str(&inv.origin).map_err(|e| RpcError::SdkError(e.into()))?,
      target: Entity::from_str(&inv.target).map_err(|e| RpcError::SdkError(e.into()))?,
      payload: convert_messagekind_map(inv.payload),
      id: uuid::Uuid::from_str(&inv.id).map_err(|e| RpcError::UuidParseError(inv.id, e))?,
      tx_id: uuid::Uuid::from_str(&inv.tx_id).map_err(|e| RpcError::UuidParseError(inv.tx_id, e))?,
      inherent: inv.inherent.map(|d| sdk::InherentData {
        seed: d.seed,
        timestamp: d.timestamp,
      }),
      config,
    })
  }
}

impl TryFrom<wasmflow::TypeSignature> for rpc::TypeSignature {
  type Error = RpcError;
  fn try_from(t: wasmflow::TypeSignature) -> Result<Self> {
    use rpc::simple_type::ApexType;
    use rpc::type_signature::Signature;
    use rpc::{LinkType, ListType, MapType, OptionalType, RefType};
    let sig: Signature = match t {
      wasmflow::TypeSignature::I8 => ApexType::I8.into(),
      wasmflow::TypeSignature::I16 => ApexType::I16.into(),
      wasmflow::TypeSignature::I32 => ApexType::I32.into(),
      wasmflow::TypeSignature::I64 => ApexType::I64.into(),
      wasmflow::TypeSignature::U8 => ApexType::U8.into(),
      wasmflow::TypeSignature::U16 => ApexType::U16.into(),
      wasmflow::TypeSignature::U32 => ApexType::U32.into(),
      wasmflow::TypeSignature::U64 => ApexType::U64.into(),
      wasmflow::TypeSignature::F32 => ApexType::F32.into(),
      wasmflow::TypeSignature::F64 => ApexType::F64.into(),
      wasmflow::TypeSignature::Bool => ApexType::Bool.into(),
      wasmflow::TypeSignature::String => ApexType::String.into(),
      wasmflow::TypeSignature::Datetime => ApexType::Datetime.into(),
      wasmflow::TypeSignature::Bytes => ApexType::Bytes.into(),
      wasmflow::TypeSignature::Value => ApexType::Value.into(),
      wasmflow::TypeSignature::Internal(t) => match t {
        wasmflow::InternalType::ComponentInput => Signature::Internal(InternalType::ComponentInput.into()),
      },
      wasmflow::TypeSignature::Ref { reference } => Signature::Ref(RefType { r#ref: reference }),
      wasmflow::TypeSignature::List { element } => Signature::List(Box::new(ListType {
        r#type: Some(element.try_into()?),
      })),
      wasmflow::TypeSignature::Optional { option } => Signature::Optional(Box::new(OptionalType {
        r#type: Some(option.try_into()?),
      })),
      wasmflow::TypeSignature::Map { key, value } => Signature::Map(Box::new(MapType {
        key_type: Some(key.try_into()?),
        value_type: Some(value.try_into()?),
      })),
      wasmflow::TypeSignature::Link { schemas } => Signature::Link(LinkType { schemas }),
      wasmflow::TypeSignature::Struct => Signature::Struct(StructType {}),
    };
    Ok(Self { signature: Some(sig) })
  }
}

impl TryFrom<rpc::TypeSignature> for wasmflow::TypeSignature {
  type Error = RpcError;
  fn try_from(t: rpc::TypeSignature) -> Result<Self> {
    use rpc::simple_type::ApexType;
    use rpc::type_signature::Signature;

    type DestType = wasmflow::TypeSignature;
    let err = Err(RpcError::InvalidSignature);
    let sig = match t.signature {
      Some(sig) => match sig {
        Signature::Simple(t) => {
          let t = ApexType::from_i32(t.r#type);
          match t {
            Some(t) => match t {
              ApexType::I8 => DestType::I8,
              ApexType::I16 => DestType::I16,
              ApexType::I32 => DestType::I32,
              ApexType::I64 => DestType::I64,

              ApexType::U8 => DestType::U8,
              ApexType::U16 => DestType::U16,
              ApexType::U32 => DestType::U32,
              ApexType::U64 => DestType::U64,

              ApexType::F32 => DestType::F32,
              ApexType::F64 => DestType::F64,

              ApexType::Bool => DestType::Bool,
              ApexType::String => DestType::String,
              ApexType::Datetime => DestType::Datetime,
              ApexType::Bytes => DestType::Bytes,
              ApexType::Value => DestType::Value,
            },
            None => return err,
          }
        }
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
          element: match list.r#type {
            Some(v) => v.try_into()?,
            None => return err,
          },
        },
        Signature::Optional(opt) => DestType::Optional {
          option: match opt.r#type {
            Some(v) => v.try_into()?,
            None => return err,
          },
        },
        Signature::Ref(reference) => DestType::Ref {
          reference: reference.r#ref,
        },
        Signature::Link(link) => DestType::Link { schemas: link.schemas },
        Signature::Internal(t) => {
          let t = InternalType::from_i32(t);
          match t {
            Some(t) => match t {
              InternalType::ComponentInput => DestType::Internal(sdk::types::InternalType::ComponentInput),
            },
            None => todo!(),
          }
        }
        Signature::Struct(_) => DestType::Struct,
      },
      None => return err,
    };
    Ok(sig)
  }
}

impl TryFrom<&wasmflow::TypeSignature> for rpc::TypeSignature {
  type Error = RpcError;
  fn try_from(t: &wasmflow::TypeSignature) -> Result<Self> {
    t.clone().try_into()
  }
}

impl TryFrom<Box<wasmflow::TypeSignature>> for Box<rpc::TypeSignature> {
  type Error = RpcError;
  fn try_from(t: Box<wasmflow::TypeSignature>) -> Result<Self> {
    Ok(Box::new((*t).try_into()?))
  }
}

impl TryFrom<Box<rpc::TypeSignature>> for Box<wasmflow::TypeSignature> {
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

impl From<rpc::simple_type::ApexType> for rpc::SimpleType {
  fn from(t: rpc::simple_type::ApexType) -> Self {
    Self { r#type: t.into() }
  }
}

impl From<rpc::simple_type::ApexType> for rpc::type_signature::Signature {
  fn from(t: rpc::simple_type::ApexType) -> Self {
    Self::Simple(rpc::SimpleType { r#type: t.into() })
  }
}

fn to_fieldmap(map: HashMap<String, rpc::TypeSignature>) -> Result<wasmflow::FieldMap> {
  let mut tmap = HashMap::new();
  for (k, v) in map {
    tmap.insert(k, v.try_into()?);
  }
  Ok(tmap.into())
}

fn from_fieldmap(map: wasmflow::FieldMap) -> Result<HashMap<String, rpc::TypeSignature>> {
  let mut tmap = HashMap::new();
  for (k, v) in map.into_inner() {
    tmap.insert(k, v.try_into()?);
  }
  Ok(tmap)
}

impl TryFrom<rpc::StructSignature> for wasmflow::StructSignature {
  type Error = RpcError;
  fn try_from(v: rpc::StructSignature) -> Result<Self> {
    Ok(Self {
      name: v.name,
      fields: to_fieldmap(v.fields)?,
    })
  }
}

impl TryFrom<rpc::TypeDefinition> for wasmflow::TypeDefinition {
  type Error = RpcError;
  fn try_from(v: rpc::TypeDefinition) -> Result<Self> {
    let typ = v.r#type.ok_or(RpcError::Internal("No type passed"))?;
    let result = match typ {
      rpc::type_definition::Type::Struct(v) => wasmflow::TypeDefinition::Struct(v.try_into()?),
      rpc::type_definition::Type::Enum(v) => wasmflow::TypeDefinition::Enum(v.try_into()?),
    };
    Ok(result)
  }
}

impl TryFrom<wasmflow::TypeDefinition> for rpc::TypeDefinition {
  type Error = RpcError;
  fn try_from(v: wasmflow::TypeDefinition) -> Result<Self> {
    let result = match v {
      wasmflow::TypeDefinition::Struct(v) => rpc::TypeDefinition {
        r#type: Some(rpc::type_definition::Type::Struct(v.try_into()?)),
      },
      wasmflow::TypeDefinition::Enum(v) => rpc::TypeDefinition {
        r#type: Some(rpc::type_definition::Type::Enum(v.try_into()?)),
      },
    };
    Ok(result)
  }
}

impl TryFrom<rpc::EnumSignature> for wasmflow::EnumSignature {
  type Error = RpcError;
  fn try_from(v: rpc::EnumSignature) -> Result<Self> {
    Ok(wasmflow::EnumSignature::new(
      v.name,
      v.values.into_iter().map(|v| v.try_into()).collect::<Result<Vec<_>>>()?,
    ))
  }
}

impl TryFrom<rpc::EnumVariant> for wasmflow::EnumVariant {
  type Error = RpcError;
  fn try_from(v: rpc::EnumVariant) -> Result<Self> {
    Ok(wasmflow::EnumVariant::new(v.name, v.index))
  }
}
fn to_typemap(map: HashMap<String, rpc::TypeDefinition>) -> Result<wasmflow::TypeMap> {
  let mut tmap = HashMap::new();
  for (k, v) in map {
    tmap.insert(k, v.try_into()?);
  }
  Ok(tmap.into())
}

impl TryFrom<wasmflow::StructSignature> for rpc::StructSignature {
  type Error = RpcError;
  fn try_from(v: wasmflow::StructSignature) -> Result<Self> {
    Ok(Self {
      name: v.name,
      fields: from_fieldmap(v.fields)?,
    })
  }
}

impl TryFrom<wasmflow::EnumSignature> for rpc::EnumSignature {
  type Error = RpcError;
  fn try_from(v: wasmflow::EnumSignature) -> Result<Self> {
    Ok(Self {
      name: v.name,
      values: v.values.into_iter().map(|v| v.try_into()).collect::<Result<Vec<_>>>()?,
    })
  }
}

impl TryFrom<wasmflow::EnumVariant> for rpc::EnumVariant {
  type Error = RpcError;
  fn try_from(v: wasmflow::EnumVariant) -> Result<Self> {
    Ok(Self {
      name: v.name,
      index: v.index,
    })
  }
}

fn from_typemap(map: wasmflow::TypeMap) -> Result<HashMap<String, rpc::TypeDefinition>> {
  let mut tmap = HashMap::new();
  for (k, v) in map.into_inner() {
    tmap.insert(k, v.try_into()?);
  }
  Ok(tmap)
}

fn to_componentmap(map: HashMap<String, rpc::Component>) -> Result<wasmflow::ComponentMap> {
  let mut tmap = HashMap::new();
  for (k, v) in map {
    tmap.insert(k, v.try_into()?);
  }
  Ok(tmap.into())
}

fn from_componentmap(map: wasmflow::ComponentMap) -> Result<HashMap<String, rpc::Component>> {
  let mut tmap = HashMap::new();
  for (k, v) in map.into_inner() {
    tmap.insert(k, v.try_into()?);
  }
  Ok(tmap)
}
