use std::collections::HashMap;
use std::convert::{
  TryFrom,
  TryInto,
};
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{
  Deserialize,
  Serialize,
};
use vino_packet::error::DeserializationError;
use vino_packet::v0::Payload;
use vino_packet::Packet;
use vino_transport::{
  Failure,
  MessageTransport,
  TransportMap,
  TransportWrapper,
};
use vino_types::signatures::*;

use crate::error::RpcError;
use crate::generated::vino::component::ComponentKind;
use crate::rpc::{
  message_kind,
  MessageKind,
  Output,
};
use crate::{
  Error,
  Result,
};

/// Important statistics for the hosted components.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Statistics {
  /// The name of the component.
  pub name: String,
  /// The number of times a component has been called.
  pub runs: u32,
  /// The number of times the component resulted in an unrecoverable error.
  pub errors: u32,
  /// Execution duration statistics.
  pub execution_duration: Option<DurationStatistics>,
}

mod as_micros {
  use std::convert::TryInto;
  use std::time::Duration;

  use serde::{
    Deserialize,
    Deserializer,
    Serializer,
  };

  pub(crate) fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_u64(duration.as_micros().try_into().unwrap_or(u64::MAX))
  }
  pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
  where
    D: Deserializer<'de>,
  {
    let micros = u64::deserialize(deserializer)?;
    Ok(Duration::from_micros(micros))
  }
}

/// Duration related statistics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DurationStatistics {
  /// The maximum duration.
  #[serde(with = "as_micros")]
  pub max_time: Duration,
  /// The minimum duration.
  #[serde(with = "as_micros")]
  pub min_time: Duration,
  /// The average duration.
  #[serde(with = "as_micros")]
  pub average: Duration,
}

impl From<HostedType> for crate::rpc::Component {
  fn from(v: HostedType) -> Self {
    match v {
      HostedType::Component(v) => v.into(),
      HostedType::Schematic(v) => v.into(),
    }
  }
}

impl TryFrom<crate::rpc::Component> for HostedType {
  type Error = Error;

  fn try_from(value: crate::rpc::Component) -> Result<Self> {
    let kind =
      ComponentKind::from_i32(value.kind).ok_or_else(|| Error::InvalidComponentKind(value.kind))?;

    match kind {
      ComponentKind::Component => Ok(HostedType::Component(ComponentSignature {
        name: value.name,
        inputs: value.inputs.into_iter().map(From::from).collect(),
        outputs: value.outputs.into_iter().map(From::from).collect(),
      })),
      ComponentKind::Schematic => Ok(HostedType::Schematic(SchematicSignature {
        name: value.name,
        inputs: value.inputs.into_iter().map(From::from).collect(),
        outputs: value.outputs.into_iter().map(From::from).collect(),
        providers: value.providers.into_iter().map(From::from).collect(),
      })),
    }
  }
}

impl From<crate::generated::vino::Provider> for ProviderSignature {
  fn from(v: crate::generated::vino::Provider) -> Self {
    Self {
      name: v.name,
      components: v.components.into_iter().map(From::from).collect(),
    }
  }
}

impl From<crate::generated::vino::Component> for ComponentSignature {
  fn from(v: crate::generated::vino::Component) -> Self {
    Self {
      name: v.name,
      inputs: v.inputs.into_iter().map(From::from).collect(),
      outputs: v.outputs.into_iter().map(From::from).collect(),
    }
  }
}

impl From<ComponentSignature> for crate::generated::vino::Component {
  fn from(v: ComponentSignature) -> Self {
    Self {
      name: v.name,
      kind: crate::rpc::component::ComponentKind::Component.into(),
      inputs: v.inputs.into_iter().map(From::from).collect(),
      outputs: v.outputs.into_iter().map(From::from).collect(),
      providers: vec![],
    }
  }
}

impl From<SchematicSignature> for crate::generated::vino::Component {
  fn from(v: SchematicSignature) -> Self {
    Self {
      name: v.name,
      kind: crate::rpc::component::ComponentKind::Schematic.into(),
      inputs: v.inputs.into_iter().map(From::from).collect(),
      outputs: v.outputs.into_iter().map(From::from).collect(),
      providers: v.providers.into_iter().map(From::from).collect(),
    }
  }
}

impl From<ProviderSignature> for crate::generated::vino::Provider {
  fn from(v: ProviderSignature) -> Self {
    Self {
      name: v.name,
      components: v.components.into_iter().map(From::from).collect(),
    }
  }
}

impl From<PortSignature> for crate::generated::vino::component::Port {
  fn from(v: PortSignature) -> Self {
    Self {
      name: v.name,
      r#type: v.type_string,
    }
  }
}

impl From<crate::generated::vino::component::Port> for PortSignature {
  fn from(v: crate::generated::vino::component::Port) -> Self {
    Self {
      name: v.name,
      type_string: v.r#type,
    }
  }
}

impl From<Statistics> for crate::generated::vino::Statistic {
  fn from(v: Statistics) -> Self {
    Self {
      name: v.name,
      runs: v.runs,
      errors: v.errors,
      execution_statistics: v.execution_duration.map(From::from),
    }
  }
}

impl From<crate::generated::vino::Statistic> for Statistics {
  fn from(v: crate::generated::vino::Statistic) -> Self {
    Self {
      name: v.name,
      runs: v.runs,
      errors: v.errors,
      execution_duration: v.execution_statistics.map(From::from),
    }
  }
}

impl From<crate::generated::vino::DurationStatistics> for DurationStatistics {
  fn from(dur: crate::generated::vino::DurationStatistics) -> Self {
    Self {
      average: Duration::from_micros(dur.average),
      min_time: Duration::from_micros(dur.min),
      max_time: Duration::from_micros(dur.max),
    }
  }
}

impl From<DurationStatistics> for crate::generated::vino::DurationStatistics {
  fn from(dur: DurationStatistics) -> Self {
    Self {
      average: dur.average.as_micros().try_into().unwrap_or(u64::MAX),
      min: dur.min_time.as_micros().try_into().unwrap_or(u64::MAX),
      max: dur.max_time.as_micros().try_into().unwrap_or(u64::MAX),
    }
  }
}

#[allow(clippy::from_over_into)]
impl Into<Packet> for MessageKind {
  fn into(self) -> Packet {
    use crate::rpc::message_kind::{
      Data,
      Kind,
      OutputSignal,
    };
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
          _ => Packet::V0(Payload::Error(format!(
            "Invalid Signal received: {:?}",
            self.data
          ))),
        },
        _ => Packet::V0(Payload::Error(format!(
          "Invalid Signal received: {:?}",
          self.data
        ))),
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

impl MessageKind {
  /// Converts a [MessageKind] into a [Packet].
  #[must_use]
  pub fn into_packet(self) -> Packet {
    self.into()
  }
  /// Converts a [MessageKind] into a [MessageTransport].
  #[must_use]
  pub fn into_transport(self) -> Packet {
    self.into()
  }
  /// Attempt to deserialize a [MessageKind] into the destination type.
  pub fn try_into<T: DeserializeOwned>(self) -> std::result::Result<T, DeserializationError> {
    self.into_packet().try_into()
  }

  /// Utility function to determine if [MessageKind] is a Signal.
  #[must_use]
  pub fn is_signal(&self) -> bool {
    let kind: Option<message_kind::Kind> = message_kind::Kind::from_i32(self.kind);
    matches!(kind, Some(message_kind::Kind::Signal))
  }
}

impl Output {
  /// Utility function to determine if [MessageKind] is a Signal.
  #[must_use]
  pub fn is_signal(&self) -> bool {
    let num = self.payload.as_ref().map_or(-1, |p| p.kind);
    let kind: Option<message_kind::Kind> = message_kind::Kind::from_i32(num);
    matches!(kind, Some(message_kind::Kind::Signal))
  }

  /// Convert the Output to JSON object value. This will not fail. If there is an error, the return value will be a serialized wrapper for a [MessageTransport::Error].
  #[must_use]
  pub fn into_json(self) -> serde_json::Value {
    let transport: TransportWrapper = self.into();
    transport.into_json()
  }

  /// Attempt to deserialize the payload into the destination type
  pub fn try_into<T: DeserializeOwned>(self) -> Result<T> {
    let transport: TransportWrapper = self.into();
    transport
      .try_into()
      .map_err(|e| RpcError::General(e.to_string()))
  }

  /// Convert the RPC output into a [TransportWrapper]
  pub fn into_transport_wrapper(self) -> TransportWrapper {
    self.into()
  }
}

impl From<Output> for TransportWrapper {
  fn from(v: Output) -> Self {
    Self {
      port: v.port,
      payload: v.payload.map_or(
        MessageTransport::Failure(Failure::Error("Could not decode RPC message".to_owned())),
        |p| p.into(),
      ),
    }
  }
}

impl From<MessageKind> for MessageTransport {
  fn from(v: MessageKind) -> Self {
    let packet: Packet = v.into();
    packet.into()
  }
}

impl From<MessageTransport> for MessageKind {
  fn from(v: MessageTransport) -> Self {
    let kind: i32 = match &v {
      MessageTransport::Success(v) => match v {
        vino_transport::message_transport::Success::MessagePack(_) => {
          message_kind::Kind::MessagePack
        }
        vino_transport::message_transport::Success::Serialized(_) => message_kind::Kind::Json,
        vino_transport::message_transport::Success::Json(_) => message_kind::Kind::Json,
      },
      MessageTransport::Failure(v) => match v {
        vino_transport::message_transport::Failure::Invalid => message_kind::Kind::Invalid,
        vino_transport::message_transport::Failure::Exception(_) => message_kind::Kind::Exception,
        vino_transport::message_transport::Failure::Error(_) => message_kind::Kind::Error,
      },
      MessageTransport::Signal(_) => message_kind::Kind::Signal,
    }
    .into();
    let data = match v {
      MessageTransport::Success(v) => match v {
        vino_transport::message_transport::Success::MessagePack(v) => {
          Some(message_kind::Data::Messagepack(v))
        }
        vino_transport::message_transport::Success::Serialized(val) => {
          match vino_codec::json::serialize(&val) {
            Ok(json) => Some(message_kind::Data::Json(json)),
            Err(e) => Some(message_kind::Data::Message(e.to_string())),
          }
        }
        vino_transport::message_transport::Success::Json(json) => {
          Some(message_kind::Data::Json(json))
        }
      },
      MessageTransport::Failure(v) => match v {
        vino_transport::message_transport::Failure::Invalid => None,
        vino_transport::message_transport::Failure::Exception(v) => {
          Some(message_kind::Data::Message(v))
        }
        vino_transport::message_transport::Failure::Error(v) => {
          Some(message_kind::Data::Message(v))
        }
      },
      MessageTransport::Signal(signal) => match signal {
        vino_transport::message_transport::MessageSignal::Done => Some(message_kind::Data::Signal(
          message_kind::OutputSignal::Done.into(),
        )),
        vino_transport::message_transport::MessageSignal::OpenBracket => Some(
          message_kind::Data::Signal(message_kind::OutputSignal::OpenBracket.into()),
        ),
        vino_transport::message_transport::MessageSignal::CloseBracket => Some(
          message_kind::Data::Signal(message_kind::OutputSignal::CloseBracket.into()),
        ),
      },
    };
    MessageKind { kind, data }
  }
}

/// Converts a HashMap of [MessageKind] to a [TransportMap].
pub fn convert_messagekind_map(rpc_map: &HashMap<String, MessageKind>) -> TransportMap {
  let mut transport_map = TransportMap::new();
  for (k, v) in rpc_map {
    transport_map.insert(k, v.clone().into());
  }
  transport_map
}

/// Converts a [TransportMap] to a HashMap of [MessageKind].
#[must_use]
pub fn convert_transport_map(transport_map: TransportMap) -> HashMap<String, MessageKind> {
  let mut rpc_map: HashMap<String, MessageKind> = HashMap::new();
  for (k, v) in transport_map.into_inner() {
    rpc_map.insert(k, v.clone().into());
  }
  rpc_map
}
