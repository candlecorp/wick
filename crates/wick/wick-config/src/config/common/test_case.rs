#![allow(missing_docs)] // delete when we move away from the `property` crate.
use liquid_json::LiquidJsonValue;

use crate::config::{LiquidJsonConfig, TemplateConfig};

#[derive(Debug, Clone, PartialEq, property::Property, serde::Serialize, Builder)]
#[property(get(public), set(private), mut(disable))]
/// A test case for a component.
pub struct TestCase {
  /// The name of the test.
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) name: Option<String>,
  /// The operaton to test.
  #[builder(default, setter(into))]
  pub(crate) operation: String,
  /// The configuration for the operation being tested, if any.
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) config: Option<LiquidJsonConfig>,
  /// Inherent data to use for the test.
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) inherent: Option<InherentConfig>,
  /// The inputs to the test.
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) inputs: Vec<TestPacket>,
  /// The expected outputs of the operation.
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) outputs: Vec<TestPacket>,
}

#[derive(Debug, Default, Clone, PartialEq, Copy, property::Property, serde::Serialize, Builder)]
#[property(get(public), set(private), mut(disable))]
/// Data inherent to transactions.
pub struct InherentConfig {
  /// An RNG seed.
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) seed: Option<u64>,
  /// A timestamp.
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) timestamp: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
/// Either a success packet or an error packet.
#[serde(rename_all = "kebab-case")]
pub enum TestPacket {
  /// A variant representing a [SuccessPayload] type.
  #[serde(rename = "success")]
  SuccessPacket(SuccessPayload),
  /// A variant representing a [ErrorPayload] type.
  #[serde(rename = "error")]
  ErrorPacket(ErrorPayload),
}

impl TestPacket {
  /// Create a new success packet.
  #[must_use]
  pub fn success(port: impl Into<String>, data: Option<LiquidJsonValue>) -> Self {
    Self::SuccessPacket(SuccessPayload {
      port: port.into(),
      flag: None,
      data,
    })
  }

  /// Create a new error packet.
  #[must_use]
  pub fn error(port: impl Into<String>, error: impl Into<String>) -> Self {
    Self::ErrorPacket(ErrorPayload {
      port: port.into(),
      flag: None,
      error: TemplateConfig::new_template(error.into()),
    })
  }

  /// Create a new done packet.
  #[must_use]
  pub fn done(port: impl Into<String>) -> Self {
    Self::SuccessPacket(SuccessPayload {
      port: port.into(),
      flag: Some(PacketFlag::Done),
      data: None,
    })
  }

  /// Create a new open packet.
  #[must_use]
  pub fn open(port: impl Into<String>) -> Self {
    Self::SuccessPacket(SuccessPayload {
      port: port.into(),
      flag: Some(PacketFlag::Open),
      data: None,
    })
  }

  /// Create a new close packet.
  #[must_use]
  pub fn close(port: impl Into<String>) -> Self {
    Self::SuccessPacket(SuccessPayload {
      port: port.into(),
      flag: Some(PacketFlag::Close),
      data: None,
    })
  }

  /// Get the port name for the packet.
  #[must_use]
  pub fn port(&self) -> &str {
    match self {
      TestPacket::SuccessPacket(data) => &data.port,
      TestPacket::ErrorPacket(data) => &data.port,
    }
  }

  /// Get the flags for the packet.
  #[must_use]
  pub fn flag(&self) -> MaybePacketFlag {
    MaybePacketFlag(match self {
      TestPacket::SuccessPacket(data) => data.flag,
      TestPacket::ErrorPacket(data) => data.flag,
    })
  }

  /// Get the data for the packet.
  #[must_use]
  pub fn data(&self) -> Option<&LiquidJsonValue> {
    match self {
      TestPacket::SuccessPacket(data) => data.data.as_ref(),
      TestPacket::ErrorPacket(_) => None,
    }
  }
}

/// A utility wrapper for an [Option]-wrapped [PacketFlag] that allows for more ergonomic assertions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaybePacketFlag(Option<PacketFlag>);

impl std::ops::Deref for MaybePacketFlag {
  type Target = Option<PacketFlag>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl MaybePacketFlag {
  /// Check if the flag is set to `Done`.
  #[must_use]
  pub fn is_done(&self) -> bool {
    matches!(self.0, Some(PacketFlag::Done))
  }

  /// Check if the flag is set to `Open`.
  #[must_use]
  pub fn is_open(&self) -> bool {
    matches!(self.0, Some(PacketFlag::Open))
  }

  /// Check if the flag is set to `Close`.
  #[must_use]
  pub fn is_close(&self) -> bool {
    matches!(self.0, Some(PacketFlag::Close))
  }
}

#[derive(Debug, Clone, PartialEq, property::Property, serde::Serialize)]
#[property(get(public), set(private), mut(disable))]
/// A simplified representation of a Wick data packet & payload, used to write tests.
pub struct SuccessPayload {
  /// The name of the port to send the data to.
  pub(crate) port: String,
  /// The flag set on the packet.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) flag: Option<PacketFlag>,
  /// The data to send.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) data: Option<LiquidJsonValue>,
}

#[derive(Debug, Clone, PartialEq, property::Property, serde::Serialize)]
#[property(get(public), set(private), mut(disable))]
/// A simplified representation of a Wick error packet & payload, used to write tests.
pub struct ErrorPayload {
  /// The name of the port to send the data to.
  pub(crate) port: String,
  /// The flag set on the packet.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) flag: Option<PacketFlag>,
  /// The error message.
  pub(crate) error: TemplateConfig<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
/// Possible flags that can be set on a packet.
#[serde(rename_all = "kebab-case")]
pub enum PacketFlag {
  /// Indicates the port should be considered closed.
  Done = 0,
  /// Indicates the opening of a new substream context within the parent stream.
  Open = 1,
  /// Indicates the closing of a substream context within the parent stream.
  Close = 2,
}
