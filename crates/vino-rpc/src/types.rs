use std::convert::TryFrom;
use std::fmt::Display;
use std::pin::Pin;

use futures::Stream;
use serde::{
  Deserialize,
  Serialize,
};
use vino_component::v0::Payload;
use vino_component::Packet;
use vino_transport::message_transport::MessageSignal;
use vino_transport::MessageTransport;

use crate::generated::vino::component::ComponentKind;
use crate::port::PortPacket;
use crate::rpc::{
  OutputKind,
  OutputSignal,
};
use crate::{
  Error,
  Result,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ComponentSignature {
  pub name: String,
  pub inputs: Vec<PortSignature>,
  pub outputs: Vec<PortSignature>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PortSignature {
  pub name: String,
  pub type_string: String,
}

impl PortSignature {
  pub fn new(name: String, type_string: String) -> Self {
    Self { name, type_string }
  }
}

impl From<(String, String)> for PortSignature {
  fn from(tup: (String, String)) -> Self {
    let (name, type_string) = tup;
    Self { name, type_string }
  }
}

impl Display for PortSignature {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("{}: {}", self.name, self.type_string))
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ProviderSignature {
  pub name: String,
  pub components: Vec<ComponentSignature>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SchematicSignature {
  pub name: String,
  pub inputs: Vec<PortSignature>,
  pub outputs: Vec<PortSignature>,
  pub providers: Vec<ProviderSignature>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum HostedType {
  Component(ComponentSignature),
  Schematic(SchematicSignature),
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Statistics {
  pub num_calls: u64,
  pub execution_duration: ExecutionStatistics,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ExecutionStatistics {
  pub max_time: usize,
  pub min_time: usize,
  pub average: usize,
}

pub type BoxedPacketStream = Pin<Box<dyn Stream<Item = PortPacket> + Send>>;

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
      execution_duration: ExecutionStatistics::default(),
    }
  }
}

#[allow(clippy::from_over_into)]
impl Into<Packet> for OutputKind {
  fn into(self) -> Packet {
    use crate::rpc::output_kind::Data;
    match self.data {
      Some(v) => match v {
        Data::Messagepack(v) => Packet::V0(Payload::MessagePack(v)),
        Data::Error(v) => Packet::V0(Payload::Error(v)),
        Data::Exception(v) => Packet::V0(Payload::Exception(v)),
        Data::Test(_) => Packet::V0(Payload::Invalid),
        Data::Invalid(_) => Packet::V0(Payload::Invalid),
        Data::Signal(signal) => match OutputSignal::from_i32(signal) {
          Some(OutputSignal::Close) => Packet::V0(Payload::Close),
          Some(OutputSignal::OpenBracket) => Packet::V0(Payload::OpenBracket),
          Some(OutputSignal::CloseBracket) => Packet::V0(Payload::CloseBracket),
          None => Packet::V0(Payload::Error("Sent an invalid signal".to_owned())),
        },
      },
      None => Packet::V0(Payload::Error(
        "Response received without output".to_owned(),
      )),
    }
  }
}

impl From<OutputKind> for MessageTransport {
  fn from(v: OutputKind) -> Self {
    use crate::rpc::output_kind::Data;
    match v.data {
      Some(v) => match v {
        Data::Messagepack(v) => MessageTransport::MessagePack(v),
        Data::Error(v) => MessageTransport::Error(v),
        Data::Exception(v) => MessageTransport::Exception(v),
        Data::Test(v) => MessageTransport::Test(v),
        Data::Invalid(_) => MessageTransport::Invalid,
        Data::Signal(signal) => match OutputSignal::from_i32(signal) {
          Some(OutputSignal::Close) => MessageTransport::Signal(MessageSignal::Close),
          Some(OutputSignal::OpenBracket) => MessageTransport::Signal(MessageSignal::CloseBracket),
          Some(OutputSignal::CloseBracket) => MessageTransport::Signal(MessageSignal::OpenBracket),
          None => MessageTransport::Error("Sent an invalid signal".to_owned()),
        },
      },
      None => MessageTransport::Error("Response received without output".to_owned()),
    }
  }
}
