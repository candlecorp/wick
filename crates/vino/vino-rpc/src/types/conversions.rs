use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use std::time::Duration;

use vino_transport::{Serialized, TransportMap};
use wasmflow_entity::Entity;
use wasmflow_interface::{self as vino};
use wasmflow_packet::Packet;

use crate::error::RpcError;
use crate::rpc::{InternalType, StructType};
use crate::{rpc, DurationStatistics};

type Result<T> = std::result::Result<T, RpcError>;

impl TryFrom<vino::HostedType> for rpc::ProviderSignature {
  type Error = RpcError;

  fn try_from(v: vino::HostedType) -> Result<Self> {
    Ok(match v {
      vino::HostedType::Provider(v) => v.try_into()?,
    })
  }
}

impl TryFrom<vino::HostedType> for rpc::HostedType {
  type Error = RpcError;

  fn try_from(value: vino::HostedType) -> Result<Self> {
    use rpc::hosted_type::Type;
    Ok(match value {
      vino::HostedType::Provider(p) => Self {
        r#type: Some(Type::Provider(p.try_into()?)),
      },
    })
  }
}

impl TryFrom<rpc::HostedType> for vino::HostedType {
  type Error = RpcError;

  fn try_from(value: rpc::HostedType) -> Result<Self> {
    use rpc::hosted_type::Type;
    match value.r#type {
      Some(v) => match v {
        Type::Provider(sig) => Ok(vino::HostedType::Provider(sig.try_into()?)),
      },
      None => Err(RpcError::InvalidSignature),
    }
  }
}

impl TryFrom<rpc::ProviderSignature> for vino::ProviderSignature {
  type Error = RpcError;

  fn try_from(v: rpc::ProviderSignature) -> Result<Self> {
    Ok(Self {
      name: Some(v.name),
      version: v.version,
      format: v.format,
      wellknown: v
        .wellknown
        .into_iter()
        .map(|v| {
          Ok(wasmflow_interface::WellKnownSchema {
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

impl TryFrom<rpc::Component> for vino::ComponentSignature {
  type Error = RpcError;
  fn try_from(v: rpc::Component) -> Result<Self> {
    Ok(Self {
      name: v.name,
      inputs: to_fieldmap(v.inputs)?,
      outputs: to_fieldmap(v.outputs)?,
    })
  }
}

impl TryFrom<vino::ComponentSignature> for rpc::Component {
  type Error = RpcError;
  fn try_from(v: vino::ComponentSignature) -> Result<Self> {
    Ok(Self {
      name: v.name,
      kind: rpc::component::ComponentKind::Component.into(),
      inputs: from_fieldmap(v.inputs)?,
      outputs: from_fieldmap(v.outputs)?,
    })
  }
}

impl TryFrom<vino::ProviderSignature> for rpc::ProviderSignature {
  type Error = RpcError;

  fn try_from(v: vino::ProviderSignature) -> Result<Self> {
    Ok(Self {
      name: v.name.unwrap_or_default(),
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
    use wasmflow_packet::v1;

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
            rpc::signal::OutputSignal::State => {
              let payload = v.payload.and_then(|v| v.data);
              match payload {
                Some(rpc::payload_data::Data::Messagepack(v)) => v1::Packet::Success(v1::Serialized::MessagePack(v)),
                Some(rpc::payload_data::Data::Json(v)) => v1::Packet::Success(v1::Serialized::Json(v)),
                None => v1::Packet::error("Invalid RPC packet. Received Signal packet but no payload."),
              }
            }
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

impl TryFrom<wasmflow_invocation::Invocation> for rpc::Invocation {
  type Error = RpcError;
  fn try_from(inv: wasmflow_invocation::Invocation) -> Result<Self> {
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
      state: normalize_serialization_out(inv.state)?,
    })
  }
}

impl TryFrom<vino_transport::Serialized> for rpc::Serialized {
  type Error = RpcError;
  fn try_from(v: vino_transport::Serialized) -> Result<Self> {
    let result = match v {
      Serialized::MessagePack(v) => rpc::Serialized {
        payload: Some(rpc::PayloadData {
          data: Some(rpc::payload_data::Data::Messagepack(v)),
        }),
      },
      Serialized::Struct(v) => rpc::Serialized {
        payload: Some(rpc::PayloadData {
          data: Some(rpc::payload_data::Data::Messagepack(
            wasmflow_codec::messagepack::serialize(&v).unwrap(),
          )),
        }),
      },
      Serialized::Json(v) => rpc::Serialized {
        payload: Some(rpc::PayloadData {
          data: Some(rpc::payload_data::Data::Json(v)),
        }),
      },
    };
    Ok(result)
  }
}

impl TryFrom<rpc::Serialized> for vino_transport::Serialized {
  type Error = RpcError;
  fn try_from(v: rpc::Serialized) -> Result<Self> {
    let data = v.payload.and_then(|v| v.data);

    match data {
      Some(rpc::payload_data::Data::Messagepack(v)) => Ok(Serialized::MessagePack(v)),
      Some(rpc::payload_data::Data::Json(v)) => Ok(Serialized::Json(v)),
      None => Err(RpcError::Internal(
        "Invalid RPC message, serialized data did not contain a payload",
      )),
    }
  }
}

fn normalize_serialization_in(packet: Option<rpc::Serialized>) -> Result<Option<vino_transport::Serialized>> {
  match packet {
    Some(packet) => Ok(Some(packet.try_into()?)),
    None => Ok(None),
  }
}

fn normalize_serialization_out(packet: Option<vino_transport::Serialized>) -> Result<Option<rpc::Serialized>> {
  match packet {
    Some(packet) => Ok(Some(packet.try_into()?)),
    None => Ok(None),
  }
}

impl TryFrom<rpc::Invocation> for wasmflow_invocation::Invocation {
  type Error = RpcError;
  fn try_from(inv: rpc::Invocation) -> Result<Self> {
    let config = normalize_serialization_in(inv.config)?;
    let state = normalize_serialization_in(inv.state)?;
    Ok(Self {
      origin: Entity::from_str(&inv.origin)?,
      target: Entity::from_str(&inv.target)?,
      payload: convert_messagekind_map(inv.payload),
      id: uuid::Uuid::from_str(&inv.id)?,
      tx_id: uuid::Uuid::from_str(&inv.tx_id)?,
      inherent: inv.inherent.map(|d| wasmflow_invocation::InherentData {
        seed: d.seed,
        timestamp: d.timestamp,
      }),
      config,
      state,
    })
  }
}

impl TryFrom<vino::TypeSignature> for rpc::TypeSignature {
  type Error = RpcError;
  fn try_from(t: vino::TypeSignature) -> Result<Self> {
    use rpc::simple_type::WidlType;
    use rpc::type_signature::Signature;
    use rpc::{LinkType, ListType, MapType, OptionalType, RefType};
    let sig: Signature = match t {
      vino::TypeSignature::I8 => WidlType::I8.into(),
      vino::TypeSignature::I16 => WidlType::I16.into(),
      vino::TypeSignature::I32 => WidlType::I32.into(),
      vino::TypeSignature::I64 => WidlType::I64.into(),
      vino::TypeSignature::U8 => WidlType::U8.into(),
      vino::TypeSignature::U16 => WidlType::U16.into(),
      vino::TypeSignature::U32 => WidlType::U32.into(),
      vino::TypeSignature::U64 => WidlType::U64.into(),
      vino::TypeSignature::F32 => WidlType::F32.into(),
      vino::TypeSignature::F64 => WidlType::F64.into(),
      vino::TypeSignature::Bool => WidlType::Bool.into(),
      vino::TypeSignature::String => WidlType::String.into(),
      vino::TypeSignature::Datetime => WidlType::Datetime.into(),
      vino::TypeSignature::Bytes => WidlType::Bytes.into(),
      vino::TypeSignature::Value => WidlType::Value.into(),
      vino::TypeSignature::Internal(t) => match t {
        vino::InternalType::ComponentInput => Signature::Internal(InternalType::ComponentInput.into()),
      },
      vino::TypeSignature::Ref { reference } => Signature::Ref(RefType { r#ref: reference }),
      vino::TypeSignature::List { element } => Signature::List(Box::new(ListType {
        r#type: Some(element.try_into()?),
      })),
      vino::TypeSignature::Optional { option } => Signature::Optional(Box::new(OptionalType {
        r#type: Some(option.try_into()?),
      })),
      vino::TypeSignature::Map { key, value } => Signature::Map(Box::new(MapType {
        key_type: Some(key.try_into()?),
        value_type: Some(value.try_into()?),
      })),
      vino::TypeSignature::Link { schemas } => Signature::Link(LinkType { schemas }),
      vino::TypeSignature::Struct => Signature::Struct(StructType {}),
    };
    Ok(Self { signature: Some(sig) })
  }
}

impl TryFrom<rpc::TypeSignature> for vino::TypeSignature {
  type Error = RpcError;
  fn try_from(t: rpc::TypeSignature) -> Result<Self> {
    use rpc::simple_type::WidlType;
    use rpc::type_signature::Signature;

    type DestType = vino::TypeSignature;
    let err = Err(RpcError::InvalidSignature);
    let sig = match t.signature {
      Some(sig) => match sig {
        Signature::Simple(t) => {
          let t = WidlType::from_i32(t.r#type);
          match t {
            Some(t) => match t {
              WidlType::I8 => DestType::I8,
              WidlType::I16 => DestType::I16,
              WidlType::I32 => DestType::I32,
              WidlType::I64 => DestType::I64,

              WidlType::U8 => DestType::U8,
              WidlType::U16 => DestType::U16,
              WidlType::U32 => DestType::U32,
              WidlType::U64 => DestType::U64,

              WidlType::F32 => DestType::F32,
              WidlType::F64 => DestType::F64,

              WidlType::Bool => DestType::Bool,
              WidlType::String => DestType::String,
              WidlType::Datetime => DestType::Datetime,
              WidlType::Bytes => DestType::Bytes,
              WidlType::Value => DestType::Value,
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
              InternalType::ComponentInput => DestType::Internal(wasmflow_interface::InternalType::ComponentInput),
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

impl TryFrom<&vino::TypeSignature> for rpc::TypeSignature {
  type Error = RpcError;
  fn try_from(t: &vino::TypeSignature) -> Result<Self> {
    t.clone().try_into()
  }
}

impl TryFrom<Box<vino::TypeSignature>> for Box<rpc::TypeSignature> {
  type Error = RpcError;
  fn try_from(t: Box<vino::TypeSignature>) -> Result<Self> {
    Ok(Box::new((*t).try_into()?))
  }
}

impl TryFrom<Box<rpc::TypeSignature>> for Box<vino::TypeSignature> {
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

impl From<rpc::simple_type::WidlType> for rpc::SimpleType {
  fn from(t: rpc::simple_type::WidlType) -> Self {
    Self { r#type: t.into() }
  }
}

impl From<rpc::simple_type::WidlType> for rpc::type_signature::Signature {
  fn from(t: rpc::simple_type::WidlType) -> Self {
    Self::Simple(rpc::SimpleType { r#type: t.into() })
  }
}

fn to_fieldmap(map: HashMap<String, rpc::TypeSignature>) -> Result<vino::FieldMap> {
  let mut tmap = HashMap::new();
  for (k, v) in map {
    tmap.insert(k, v.try_into()?);
  }
  Ok(tmap.into())
}

fn from_fieldmap(map: vino::FieldMap) -> Result<HashMap<String, rpc::TypeSignature>> {
  let mut tmap = HashMap::new();
  for (k, v) in map.into_inner() {
    tmap.insert(k, v.try_into()?);
  }
  Ok(tmap)
}

impl TryFrom<rpc::StructSignature> for vino::StructSignature {
  type Error = RpcError;
  fn try_from(v: rpc::StructSignature) -> Result<Self> {
    Ok(Self {
      name: v.name,
      fields: to_fieldmap(v.fields)?,
    })
  }
}

impl TryFrom<rpc::TypeDefinition> for vino::TypeDefinition {
  type Error = RpcError;
  fn try_from(v: rpc::TypeDefinition) -> Result<Self> {
    let typ = v.r#type.ok_or(RpcError::Internal("No type passed"))?;
    let result = match typ {
      rpc::type_definition::Type::Struct(v) => vino::TypeDefinition::Struct(v.try_into()?),
      rpc::type_definition::Type::Enum(v) => vino::TypeDefinition::Enum(v.try_into()?),
    };
    Ok(result)
  }
}

impl TryFrom<vino::TypeDefinition> for rpc::TypeDefinition {
  type Error = RpcError;
  fn try_from(v: vino::TypeDefinition) -> Result<Self> {
    let result = match v {
      vino::TypeDefinition::Struct(v) => rpc::TypeDefinition {
        r#type: Some(rpc::type_definition::Type::Struct(v.try_into()?)),
      },
      vino::TypeDefinition::Enum(v) => rpc::TypeDefinition {
        r#type: Some(rpc::type_definition::Type::Enum(v.try_into()?)),
      },
    };
    Ok(result)
  }
}

impl TryFrom<rpc::EnumSignature> for vino::EnumSignature {
  type Error = RpcError;
  fn try_from(v: rpc::EnumSignature) -> Result<Self> {
    Ok(vino::EnumSignature::new(
      v.name,
      v.values.into_iter().map(|v| v.try_into()).collect::<Result<Vec<_>>>()?,
    ))
  }
}

impl TryFrom<rpc::EnumVariant> for vino::EnumVariant {
  type Error = RpcError;
  fn try_from(v: rpc::EnumVariant) -> Result<Self> {
    Ok(vino::EnumVariant::new(v.name, v.index))
  }
}
fn to_typemap(map: HashMap<String, rpc::TypeDefinition>) -> Result<vino::TypeMap> {
  let mut tmap = HashMap::new();
  for (k, v) in map {
    tmap.insert(k, v.try_into()?);
  }
  Ok(tmap.into())
}

impl TryFrom<vino::StructSignature> for rpc::StructSignature {
  type Error = RpcError;
  fn try_from(v: vino::StructSignature) -> Result<Self> {
    Ok(Self {
      name: v.name,
      fields: from_fieldmap(v.fields)?,
    })
  }
}

impl TryFrom<vino::EnumSignature> for rpc::EnumSignature {
  type Error = RpcError;
  fn try_from(v: vino::EnumSignature) -> Result<Self> {
    Ok(Self {
      name: v.name,
      values: v.values.into_iter().map(|v| v.try_into()).collect::<Result<Vec<_>>>()?,
    })
  }
}

impl TryFrom<vino::EnumVariant> for rpc::EnumVariant {
  type Error = RpcError;
  fn try_from(v: vino::EnumVariant) -> Result<Self> {
    Ok(Self {
      name: v.name,
      index: v.index,
    })
  }
}

fn from_typemap(map: vino::TypeMap) -> Result<HashMap<String, rpc::TypeDefinition>> {
  let mut tmap = HashMap::new();
  for (k, v) in map.into_inner() {
    tmap.insert(k, v.try_into()?);
  }
  Ok(tmap)
}

fn to_componentmap(map: HashMap<String, rpc::Component>) -> Result<vino::ComponentMap> {
  let mut tmap = HashMap::new();
  for (k, v) in map {
    tmap.insert(k, v.try_into()?);
  }
  Ok(tmap.into())
}

fn from_componentmap(map: vino::ComponentMap) -> Result<HashMap<String, rpc::Component>> {
  let mut tmap = HashMap::new();
  for (k, v) in map.into_inner() {
    tmap.insert(k, v.try_into()?);
  }
  Ok(tmap)
}
