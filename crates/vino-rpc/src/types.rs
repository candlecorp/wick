use std::collections::HashMap;
use std::convert::TryFrom;
use std::pin::Pin;

use futures::Stream;
use serde::de::DeserializeOwned;
use serde::{
  Deserialize,
  Serialize,
};
use vino_component::error::DeserializationError;
use vino_component::v0::Payload;
use vino_component::Packet;
use vino_transport::message_transport::TransportMap;
use vino_transport::{
  InvocationTransport,
  MessageTransport,
};
use vino_types::signatures::*;

use crate::generated::vino::component::ComponentKind;
use crate::rpc::{
  message_kind,
  MessageKind,
};
use crate::{
  Error,
  Result,
};

/// Important statistics for the hosted components
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Statistics {
  /// The number of times a component has been called
  pub num_calls: u64,
  /// Execution duration statistics
  pub execution_duration: DurationStatistics,
}

/// Duration related statistics
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DurationStatistics {
  /// The maximum duration
  pub max_time: usize,
  /// The minimum duration
  pub min_time: usize,
  /// The average duration
  pub average: usize,
}

// /// The return type of RpcHandler requests
// pub type BoxedPacketStream = Pin<Box<dyn Stream<Item = InvocationPacket> + Send>>;

/// The return type of RpcHandler requests
pub type BoxedTransportStream = Pin<Box<dyn Stream<Item = InvocationTransport> + Send>>;

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
      ComponentKind::from_i32(value.kind).ok_or_else(|| Error::InvalidOutputKind(value.kind))?;

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
      num_calls: v.num_calls,
    }
  }
}

impl From<crate::generated::vino::Statistic> for Statistics {
  fn from(v: crate::generated::vino::Statistic) -> Self {
    Self {
      num_calls: v.num_calls,
      execution_duration: DurationStatistics::default(),
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
          Some(OutputSignal::Close) => Packet::V0(Payload::Close),
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
  /// Converts a [MessageKind] into a [Packet]
  #[must_use]
  pub fn into_packet(self) -> Packet {
    self.into()
  }
  /// Converts a [MessageKind] into a [MessageTransport]
  #[must_use]
  pub fn into_transport(self) -> Packet {
    self.into()
  }
  /// Attempt to deserialize a [MessageKind] into the destination type
  pub fn try_into<T: DeserializeOwned>(self) -> std::result::Result<T, DeserializationError> {
    self.into_packet().try_into()
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
    let kind: i32 = match v {
      MessageTransport::Invalid => message_kind::Kind::Invalid,
      MessageTransport::Exception(_) => message_kind::Kind::Exception,
      MessageTransport::Error(_) => message_kind::Kind::Error,
      MessageTransport::MessagePack(_) => message_kind::Kind::MessagePack,
      MessageTransport::Test(_) => message_kind::Kind::Test,
      MessageTransport::Signal(_) => message_kind::Kind::Signal,
      MessageTransport::Success(_) => message_kind::Kind::Json,
      MessageTransport::Json(_) => message_kind::Kind::Json,
    }
    .into();
    let data = match v {
      MessageTransport::Invalid => None,
      MessageTransport::Exception(v) => Some(message_kind::Data::Message(v)),
      MessageTransport::Error(v) => Some(message_kind::Data::Message(v)),
      MessageTransport::MessagePack(v) => Some(message_kind::Data::Messagepack(v)),
      MessageTransport::Test(v) => Some(message_kind::Data::Message(v)),
      MessageTransport::Signal(signal) => match signal {
        vino_transport::message_transport::MessageSignal::Close => Some(
          message_kind::Data::Signal(message_kind::OutputSignal::Close.into()),
        ),
        vino_transport::message_transport::MessageSignal::OpenBracket => Some(
          message_kind::Data::Signal(message_kind::OutputSignal::OpenBracket.into()),
        ),
        vino_transport::message_transport::MessageSignal::CloseBracket => Some(
          message_kind::Data::Signal(message_kind::OutputSignal::CloseBracket.into()),
        ),
      },
      MessageTransport::Success(val) => match vino_codec::json::serialize(&val) {
        Ok(json) => Some(message_kind::Data::Json(json)),
        Err(e) => Some(message_kind::Data::Message(e.to_string())),
      },
      MessageTransport::Json(json) => Some(message_kind::Data::Json(json)),
    };
    MessageKind { kind, data }
  }
}

/// Converts a HashMap of [MessageKind] to a [TransportMap]
pub fn convert_messagekind_map(rpc_map: &HashMap<String, MessageKind>) -> TransportMap {
  let mut transport_map = TransportMap::new();
  for (k, v) in rpc_map {
    transport_map.insert(k, v.clone().into());
  }
  transport_map
}

/// Converts a [TransportMap] to a HashMap of [MessageKind]
#[must_use]
pub fn convert_transport_map(transport_map: TransportMap) -> HashMap<String, MessageKind> {
  let mut rpc_map: HashMap<String, MessageKind> = HashMap::new();
  for (k, v) in transport_map.into_inner() {
    rpc_map.insert(k, v.clone().into());
  }
  rpc_map
}
