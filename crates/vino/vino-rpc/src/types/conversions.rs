use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use std::time::Duration;

use vino_entity::Entity;
use vino_packet::v0::Payload;
use vino_packet::Packet;
use vino_transport::TransportMap;
use vino_types::{self as vino, MapWrapper};

use crate::error::RpcError;
use crate::rpc::InternalType;
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
      components: to_componentmap(v.components)?,
      types: to_structmap(v.types)?,
    })
  }
}

impl TryFrom<rpc::Component> for vino::ComponentSignature {
  type Error = RpcError;
  fn try_from(v: rpc::Component) -> Result<Self> {
    Ok(Self {
      name: v.name,
      inputs: to_typemap(v.inputs)?,
      outputs: to_typemap(v.outputs)?,
    })
  }
}

impl TryFrom<vino::ComponentSignature> for rpc::Component {
  type Error = RpcError;
  fn try_from(v: vino::ComponentSignature) -> Result<Self> {
    Ok(Self {
      name: v.name,
      kind: rpc::component::ComponentKind::Component.into(),
      inputs: from_typemap(v.inputs)?,
      outputs: from_typemap(v.outputs)?,
    })
  }
}

impl TryFrom<vino::ProviderSignature> for rpc::ProviderSignature {
  type Error = RpcError;

  fn try_from(v: vino::ProviderSignature) -> Result<Self> {
    Ok(Self {
      name: v.name.unwrap_or_default(),
      components: from_componentmap(v.components)?,
      types: from_structmap(v.types)?,
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
impl Into<Packet> for rpc::MessageKind {
  fn into(self) -> Packet {
    use rpc::message_kind::{Data, Kind, OutputSignal};
    let kind: Kind = match Kind::from_i32(self.kind) {
      Some(v) => v,
      None => return Packet::V0(Payload::Error(format!("Invalid kind {}", self.kind))),
    };

    match kind {
      Kind::Invalid => Packet::V0(Payload::Invalid),
      Kind::Error => match self.data {
        Some(Data::Message(v)) => Packet::V0(Payload::Error(v)),
        _ => Packet::V0(Payload::Error(
          "Invalid Error output received: No message passed.".to_owned(),
        )),
      },
      Kind::Exception => match self.data {
        Some(Data::Message(v)) => Packet::V0(Payload::Error(v)),
        _ => Packet::V0(Payload::Error(
          "Invalid Error output received: No message passed.".to_owned(),
        )),
      },
      Kind::Test => Packet::V0(Payload::Invalid),
      Kind::MessagePack => match self.data {
        Some(Data::Messagepack(v)) => Packet::V0(Payload::MessagePack(v)),
        _ => Packet::V0(Payload::Error(
          "Invalid MessagePack output received: No data passed as 'bytes'.".to_owned(),
        )),
      },
      Kind::Signal => match self.data {
        Some(Data::Signal(v)) => match OutputSignal::from_i32(v) {
          Some(OutputSignal::Done) => Packet::V0(Payload::Done),
          Some(OutputSignal::OpenBracket) => Packet::V0(Payload::OpenBracket),
          Some(OutputSignal::CloseBracket) => Packet::V0(Payload::CloseBracket),
          _ => Packet::V0(Payload::Error(format!("Invalid Signal received: {:?}", self.data))),
        },
        _ => Packet::V0(Payload::Error(format!("Invalid Signal received: {:?}", self.data))),
      },
      Kind::Json => match self.data {
        Some(Data::Json(v)) => Packet::V0(Payload::Json(v)),
        _ => Packet::V0(Payload::Error(
          "Invalid JSON output received: No data passed as 'json'.".to_owned(),
        )),
      },
    }
  }
}

/// Converts a HashMap of [MessageKind] to a [TransportMap].
pub fn convert_messagekind_map(rpc_map: HashMap<String, rpc::MessageKind>) -> TransportMap {
  let mut transport_map = TransportMap::new();
  for (k, v) in rpc_map {
    transport_map.insert(k, v.into());
  }
  transport_map
}

/// Converts a [TransportMap] to a HashMap of [MessageKind].
#[must_use]
pub fn convert_transport_map(transport_map: TransportMap) -> HashMap<String, rpc::MessageKind> {
  let mut rpc_map: HashMap<String, rpc::MessageKind> = HashMap::new();
  for (k, v) in transport_map.into_inner() {
    rpc_map.insert(k, v.clone().into());
  }
  rpc_map
}

impl TryFrom<vino_transport::Invocation> for rpc::Invocation {
  type Error = RpcError;
  fn try_from(inv: vino_transport::Invocation) -> Result<Self> {
    Ok(Self {
      origin: inv.origin.url(),
      target: inv.target.url(),
      payload: convert_transport_map(inv.payload),
      id: inv.id,
      tx_id: inv.tx_id,
    })
  }
}

impl TryFrom<rpc::Invocation> for vino_transport::Invocation {
  type Error = RpcError;
  fn try_from(inv: rpc::Invocation) -> Result<Self> {
    Ok(Self {
      origin: Entity::from_str(&inv.origin)?,
      target: Entity::from_str(&inv.target)?,
      payload: convert_messagekind_map(inv.payload),
      id: inv.id,
      tx_id: inv.tx_id,
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
      vino::TypeSignature::Raw => WidlType::Raw.into(),
      vino::TypeSignature::Value => WidlType::Value.into(),
      vino::TypeSignature::ComponentInput => WidlType::Value.into(),
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
      vino::TypeSignature::Link { provider } => Signature::Link(LinkType {
        provider: provider.unwrap_or_default(),
      }),
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
              WidlType::U8 => DestType::U8,
              WidlType::I16 => DestType::I16,
              WidlType::U16 => DestType::U16,
              WidlType::I32 => DestType::I32,
              WidlType::U32 => DestType::U32,
              WidlType::I64 => DestType::I64,
              WidlType::U64 => DestType::U64,
              WidlType::F32 => DestType::F32,
              WidlType::F64 => DestType::F64,
              WidlType::Bool => DestType::Bool,
              WidlType::String => DestType::String,
              WidlType::Datetime => DestType::Datetime,
              WidlType::Bytes => DestType::Bytes,
              WidlType::Raw => DestType::Raw,
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
        Signature::Link(link) => DestType::Link {
          provider: (!link.provider.is_empty()).then(|| link.provider),
        },
        Signature::Internal(t) => {
          let t = InternalType::from_i32(t);
          match t {
            Some(t) => match t {
              InternalType::ComponentInput => DestType::ComponentInput,
            },
            None => todo!(),
          }
        }
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

fn to_typemap(map: HashMap<String, rpc::TypeSignature>) -> Result<vino::TypeMap> {
  let mut tmap = HashMap::new();
  for (k, v) in map {
    tmap.insert(k, v.try_into()?);
  }
  Ok(tmap.into())
}

fn from_typemap(map: vino::TypeMap) -> Result<HashMap<String, rpc::TypeSignature>> {
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
      fields: to_typemap(v.fields)?,
    })
  }
}

fn to_structmap(map: HashMap<String, rpc::StructSignature>) -> Result<vino::StructMap> {
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
      fields: from_typemap(v.fields)?,
    })
  }
}

fn from_structmap(map: vino::StructMap) -> Result<HashMap<String, rpc::StructSignature>> {
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
