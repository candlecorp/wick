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
use crate::port::PacketWrapper;
use crate::rpc::{
  OutputKind,
  OutputSignal,
};
use crate::{
  Error,
  Result,
};

/// The signature of a Vino component, including its input and output types.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ComponentSignature {
  /// The name of the component.
  pub name: String,
  /// A list of input signatures
  pub inputs: Vec<PortSignature>,
  /// A list of output signatures
  pub outputs: Vec<PortSignature>,
}

/// The signature of an individual port
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PortSignature {
  /// Name of the port
  pub name: String,

  /// The data type of the port
  // TODO: Need to turn this into a more complex representation of port types
  pub type_string: String,
}

impl PortSignature {
  /// Constructor
  #[must_use]
  pub fn new(name: String, type_string: String) -> Self {
    Self { name, type_string }
  }
}

impl From<(&str, &str)> for PortSignature {
  fn from(tup: (&str, &str)) -> Self {
    let (name, type_string) = tup;
    Self {
      name: name.to_owned(),
      type_string: type_string.to_owned(),
    }
  }
}

impl Display for PortSignature {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("{}: {}", self.name, self.type_string))
  }
}

/// Signature for Providers
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ProviderSignature {
  /// Name of the provider
  pub name: String,
  /// A list of [ComponentSignature]s the provider hosts.
  pub components: Vec<ComponentSignature>,
}

/// Signature for schematics, their ports, and their providers
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SchematicSignature {
  /// Name of the schematic
  pub name: String,
  /// A list of input ports
  pub inputs: Vec<PortSignature>,
  /// A list of output ports
  pub outputs: Vec<PortSignature>,
  /// A list of providers running on the schematic
  pub providers: Vec<ProviderSignature>,
}

/// An enum representing the types of components that can be hosted
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum HostedType {
  /// A hosted component
  Component(ComponentSignature),
  /// A hosted schematic
  Schematic(SchematicSignature),
}

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

/// The return type of RpcHandler requests
pub type BoxedPacketStream = Pin<Box<dyn Stream<Item = PacketWrapper> + Send>>;

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
