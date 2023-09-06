#![allow(missing_docs)]
use std::collections::HashMap;
use std::path::Path;

// delete when we move away from the `property` crate.
use liquid_json::LiquidJsonValue;
use wick_packet::{InherentData, RuntimeConfig};

use super::template_config::Renderable;
use crate::config::{LiquidJsonConfig, TemplateConfig};
use crate::error::ManifestError;

#[derive(Debug, Clone, PartialEq, property::Property, serde::Serialize, derive_builder::Builder)]
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
  pub(crate) inputs: Vec<PacketData>,
  /// The expected outputs of the operation.
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) outputs: Vec<TestPacketData>,
}

impl Renderable for TestCase {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    if let Some(config) = self.config.as_mut() {
      config.set_value(Some(config.render(
        source,
        root_config,
        None,
        env,
        Some(&InherentData::unsafe_default()),
      )?));
    }
    Ok(())
  }
}

#[derive(Debug, Default, Clone, PartialEq, Copy, property::Property, serde::Serialize, derive_builder::Builder)]
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
pub enum PacketData {
  /// A variant representing a [SuccessPayload] type.
  #[serde(rename = "success")]
  SuccessPacket(SuccessPayload),
  /// A variant representing a [ErrorPayload] type.
  #[serde(rename = "error")]
  ErrorPacket(ErrorPayload),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
/// Either a success packet or an error packet.
#[serde(rename_all = "kebab-case")]
pub enum TestPacketData {
  /// A variant representing a [SuccessPayload] type.
  #[serde(rename = "success")]
  SuccessPacket(SuccessPayload),
  /// A variant representing a [PacketAssertion] type.
  #[serde(rename = "contains")]
  PacketAssertion(PacketAssertionDef),
  /// A variant representing a [ErrorPayload] type.
  #[serde(rename = "error")]
  ErrorPacket(ErrorPayload),
}

impl PacketData {
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
      PacketData::SuccessPacket(data) => &data.port,
      PacketData::ErrorPacket(data) => &data.port,
    }
  }

  /// Get the flags for the packet.
  #[must_use]
  pub const fn flag(&self) -> MaybePacketFlag {
    MaybePacketFlag(match self {
      PacketData::SuccessPacket(data) => data.flag,
      PacketData::ErrorPacket(data) => data.flag,
    })
  }

  /// Get the data for the packet.
  #[must_use]
  pub const fn data(&self) -> Option<&LiquidJsonValue> {
    match self {
      PacketData::SuccessPacket(data) => data.data.as_ref(),
      PacketData::ErrorPacket(_) => None,
    }
  }
}

impl TestPacketData {
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
      Self::SuccessPacket(data) => &data.port,
      Self::ErrorPacket(data) => &data.port,
      Self::PacketAssertion(data) => &data.port,
    }
  }

  /// Get the data for the packet.
  #[must_use]
  pub const fn data(&self) -> Option<&LiquidJsonValue> {
    match self {
      Self::SuccessPacket(data) => data.data.as_ref(),
      Self::ErrorPacket(_) => None,
      Self::PacketAssertion(_) => None,
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
  pub const fn is_done(&self) -> bool {
    matches!(self.0, Some(PacketFlag::Done))
  }

  /// Check if the flag is set to `Open`.
  #[must_use]
  pub const fn is_open(&self) -> bool {
    matches!(self.0, Some(PacketFlag::Open))
  }

  /// Check if the flag is set to `Close`.
  #[must_use]
  pub const fn is_close(&self) -> bool {
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

#[derive(Debug, Clone, serde::Serialize, PartialEq, property::Property)]
#[property(get(public), set(private), mut(disable))]
/// A test case for a component's operation that uses loose equality for comparing data.
pub struct PacketAssertionDef {
  /// The name of the input or output this packet is going to or coming from.
  pub(crate) port: String,

  /// An assertion to test against the packet.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) assertions: Vec<PacketAssertion>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, property::Property)]
#[property(get(public), set(private), mut(disable))]
/// A packet assertion.
pub struct PacketAssertion {
  /// The optional path to a value in the packet to assert against.

  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) path: Option<String>,

  /// The operation to use when asserting against a packet.
  pub(crate) operator: AssertionOperator,

  /// A value or object combine with the operator to assert against a packet value.
  pub(crate) value: LiquidJsonValue,
}

#[derive(Debug, Clone, Copy, serde::Serialize, PartialEq)]
/// An operation that drives the logic in a packet assertion.
pub enum AssertionOperator {
  Equals = 0,
  LessThan = 1,
  GreaterThan = 2,
  Regex = 3,
  Contains = 4,
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
