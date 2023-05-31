use serde_json::Value;

#[derive(Debug, Clone, PartialEq, property::Property)]
#[property(get(public), set(private), mut(disable))]
/// A test case for a component.
pub struct TestCase {
  /// The name of the test.
  pub(crate) name: String,
  /// The operaton to test.
  pub(crate) operation: String,
  /// The configuration for the operation being tested, if any.
  pub(crate) config: Option<wick_packet::GenericConfig>,
  /// Inherent data to use for the test.
  pub(crate) inherent: Option<InherentConfig>,
  /// The inputs to the test.
  pub(crate) inputs: Vec<TestPacket>,
  /// The expected outputs of the operation.
  pub(crate) outputs: Vec<TestPacket>,
}

#[derive(Debug, Clone, PartialEq, Copy, property::Property)]
#[property(get(public), set(private), mut(disable))]
/// Data inherent to transactions.
pub struct InherentConfig {
  /// An RNG seed.
  pub(crate) seed: Option<u64>,
  /// A timestamp.
  pub(crate) timestamp: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
/// Either a success packet or an error packet.
pub enum TestPacket {
  /// A variant representing a [PayloadData] type.
  PayloadData(SuccessPayload),
  /// A variant representing a [ErrorData] type.
  ErrorData(ErrorPayload),
}

impl TestPacket {
  /// Get the port name for the packet.
  #[must_use]
  pub fn port(&self) -> &str {
    match self {
      TestPacket::PayloadData(data) => &data.port,
      TestPacket::ErrorData(data) => &data.port,
    }
  }

  /// Get the flags for the packet.
  #[must_use]
  pub fn flags(&self) -> Option<PacketFlags> {
    match self {
      TestPacket::PayloadData(data) => data.flags,
      TestPacket::ErrorData(data) => data.flags,
    }
  }

  /// Get the data for the packet.
  #[must_use]
  pub fn data(&self) -> Option<&Value> {
    match self {
      TestPacket::PayloadData(data) => data.data.as_ref(),
      TestPacket::ErrorData(_) => None,
    }
  }
}

#[derive(Debug, Clone, PartialEq, property::Property)]
#[property(get(public), set(private), mut(disable))]
/// A simplified representation of a Wick data packet & payload, used to write tests.
pub struct SuccessPayload {
  /// The name of the port to send the data to.
  pub(crate) port: String,
  /// Any flags set on the packet.
  pub(crate) flags: Option<PacketFlags>,
  /// The data to send.
  pub(crate) data: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, property::Property)]
#[property(get(public), set(private), mut(disable))]
pub struct ErrorPayload {
  /// The name of the port to send the data to.
  pub(crate) port: String,
  /// Any flags set on the packet.
  pub(crate) flags: Option<PacketFlags>,
  /// The error message.
  pub(crate) error: String,
}

#[derive(Debug, Clone, Copy, PartialEq, property::Property)]
#[property(get(public), set(private), mut(disable))]
/// Flags set on a packet.
pub struct PacketFlags {
  /// When set, indicates the port should be considered closed.
  pub(crate) done: bool,
  /// When set, indicates the opening of a new substream context within the parent stream.
  pub(crate) open: bool,
  /// When set, indicates the closing of a substream context within the parent stream.
  pub(crate) close: bool,
}
