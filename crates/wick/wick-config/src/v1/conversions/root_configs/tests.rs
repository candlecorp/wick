use option_utils::OptionUtils;

use crate::config::{test_case, TemplateConfig};
use crate::error::ManifestError;
use crate::utils::VecTryMapInto;
use crate::v1;
type Result<T> = std::result::Result<T, ManifestError>;

impl TryFrom<v1::TestDefinition> for test_case::TestCase {
  type Error = crate::Error;
  fn try_from(value: v1::TestDefinition) -> Result<Self> {
    Ok(Self {
      name: value.name,
      operation: value.operation,
      inputs: value.inputs.try_map_into()?,
      outputs: value.outputs.try_map_into()?,
      inherent: value.inherent.map_into(),
      config: value.with.map_into(),
    })
  }
}

impl TryFrom<v1::PacketData> for test_case::PacketData {
  type Error = crate::Error;
  fn try_from(value: v1::PacketData) -> Result<Self> {
    Ok(match value {
      v1::PacketData::SuccessPacket(v) => test_case::PacketData::SuccessPacket(v.try_into()?),
      v1::PacketData::ErrorPacket(v) => test_case::PacketData::ErrorPacket(v.try_into()?),
      v1::PacketData::SignalPacket(v) => test_case::PacketData::SuccessPacket(v.try_into()?),
    })
  }
}

impl TryFrom<test_case::PacketData> for v1::PacketData {
  type Error = crate::Error;
  fn try_from(value: test_case::PacketData) -> Result<Self> {
    Ok(match value {
      test_case::PacketData::SuccessPacket(v) => v1::PacketData::SuccessPacket(v.into()),
      test_case::PacketData::ErrorPacket(v) => v1::PacketData::ErrorPacket(v.try_into()?),
    })
  }
}

impl TryFrom<v1::TestPacketData> for test_case::TestPacketData {
  type Error = crate::Error;
  fn try_from(value: v1::TestPacketData) -> Result<Self> {
    Ok(match value {
      v1::TestPacketData::SuccessPacket(v) => test_case::TestPacketData::SuccessPacket(v.try_into()?),
      v1::TestPacketData::ErrorPacket(v) => test_case::TestPacketData::ErrorPacket(v.try_into()?),
      v1::TestPacketData::PacketAssertionDef(v) => test_case::TestPacketData::PacketAssertion(v.try_into()?),
      v1::TestPacketData::SignalPacket(v) => test_case::TestPacketData::SuccessPacket(v.try_into()?),
    })
  }
}

impl TryFrom<test_case::TestPacketData> for v1::TestPacketData {
  type Error = crate::Error;
  fn try_from(value: test_case::TestPacketData) -> Result<Self> {
    Ok(match value {
      test_case::TestPacketData::SuccessPacket(v) => v1::TestPacketData::SuccessPacket(v.into()),
      test_case::TestPacketData::ErrorPacket(v) => v1::TestPacketData::ErrorPacket(v.try_into()?),
      test_case::TestPacketData::PacketAssertion(v) => v1::TestPacketData::PacketAssertionDef(v.try_into()?),
    })
  }
}

impl TryFrom<v1::PacketAssertionDef> for test_case::PacketAssertionDef {
  type Error = crate::Error;
  fn try_from(value: v1::PacketAssertionDef) -> Result<Self> {
    Ok(Self {
      port: value.name,
      assertions: value.assertions.try_map_into()?,
    })
  }
}

impl TryFrom<test_case::PacketAssertionDef> for v1::PacketAssertionDef {
  type Error = crate::Error;
  fn try_from(value: test_case::PacketAssertionDef) -> Result<Self> {
    Ok(Self {
      name: value.port,
      assertions: value.assertions.try_map_into()?,
    })
  }
}

impl TryFrom<test_case::PacketAssertion> for v1::PacketAssertion {
  type Error = crate::Error;
  fn try_from(value: test_case::PacketAssertion) -> Result<Self> {
    Ok(Self {
      path: value.path,
      operator: value.operator.into(),
      value: value.value,
    })
  }
}

impl TryFrom<v1::PacketAssertion> for test_case::PacketAssertion {
  type Error = crate::Error;
  fn try_from(value: v1::PacketAssertion) -> Result<Self> {
    Ok(Self {
      path: value.path,
      operator: value.operator.into(),
      value: value.value,
    })
  }
}

impl From<test_case::AssertionOperator> for v1::AssertionOperator {
  fn from(value: test_case::AssertionOperator) -> Self {
    match value {
      test_case::AssertionOperator::Equals => Self::Equals,
      test_case::AssertionOperator::LessThan => Self::LessThan,
      test_case::AssertionOperator::GreaterThan => Self::GreaterThan,
      test_case::AssertionOperator::Regex => Self::Regex,
      test_case::AssertionOperator::Contains => Self::Contains,
    }
  }
}

impl From<v1::AssertionOperator> for test_case::AssertionOperator {
  fn from(value: v1::AssertionOperator) -> Self {
    match value {
      v1::AssertionOperator::Equals => Self::Equals,
      v1::AssertionOperator::LessThan => Self::LessThan,
      v1::AssertionOperator::GreaterThan => Self::GreaterThan,
      v1::AssertionOperator::Regex => Self::Regex,
      v1::AssertionOperator::Contains => Self::Contains,
    }
  }
}

impl TryFrom<v1::PacketFlags> for test_case::PacketFlag {
  type Error = crate::Error;
  fn try_from(value: v1::PacketFlags) -> Result<Self> {
    Ok(match value {
      v1::PacketFlags {
        done: true,
        close: false,
        open: false,
      } => Self::Done,
      v1::PacketFlags {
        done: false,
        close: true,
        open: false,
      } => Self::Close,
      v1::PacketFlags {
        done: false,
        close: false,
        open: true,
      } => Self::Open,
      _ => return Err(crate::Error::InvalidPacketFlags),
    })
  }
}

impl TryFrom<v1::SuccessPacket> for test_case::SuccessPayload {
  type Error = crate::Error;
  fn try_from(value: v1::SuccessPacket) -> Result<Self> {
    Ok(Self {
      port: value.name,
      flag: None,
      data: Some(value.value),
    })
  }
}

impl TryFrom<v1::SignalPacket> for test_case::SuccessPayload {
  type Error = crate::Error;
  fn try_from(value: v1::SignalPacket) -> Result<Self> {
    let mut val = Self {
      port: value.name,
      flag: value.flag.map_into(),
      data: None,
    };
    #[allow(deprecated)]
    if let Some(flags) = value.flags {
      if val.flag.is_some() {
        return Err(crate::Error::InvalidPacketFlags);
      }
      val.flag = Some(flags.try_into()?);
    }
    Ok(val)
  }
}

impl TryFrom<v1::ErrorPacket> for test_case::ErrorPayload {
  type Error = crate::Error;
  fn try_from(value: v1::ErrorPacket) -> Result<Self> {
    let mut val = Self {
      port: value.name,
      flag: value.flag.map_into(),
      error: TemplateConfig::new_template(value.error),
    };
    #[allow(deprecated)]
    if let Some(flags) = value.flags {
      if val.flag.is_some() {
        return Err(crate::Error::InvalidPacketFlags);
      }
      val.flag = Some(flags.try_into()?);
    }

    Ok(val)
  }
}

impl From<v1::InherentData> for test_case::InherentConfig {
  fn from(value: v1::InherentData) -> Self {
    Self {
      seed: value.seed,
      timestamp: value.timestamp,
    }
  }
}

impl TryFrom<test_case::TestCase> for v1::TestDefinition {
  type Error = crate::Error;
  fn try_from(value: test_case::TestCase) -> Result<Self> {
    Ok(Self {
      name: value.name,
      operation: value.operation,
      inputs: value.inputs.try_map_into()?,
      outputs: value.outputs.try_map_into()?,
      inherent: value.inherent.map_into(),
      with: value.config.map_into(),
    })
  }
}

impl TryFrom<test_case::ErrorPayload> for v1::ErrorPacket {
  type Error = crate::Error;
  fn try_from(value: test_case::ErrorPayload) -> Result<Self> {
    #[allow(deprecated)]
    Ok(Self {
      name: value.port,
      flag: value.flag.map_into(),
      flags: None,
      error: value.error.unrender()?,
    })
  }
}

impl From<test_case::SuccessPayload> for v1::SuccessPacket {
  fn from(value: test_case::SuccessPayload) -> Self {
    assert!(
      value.flag.is_none(),
      "internal error, bad conversion from wick success packet to v1 success packet"
    );
    Self {
      name: value.port,
      value: value.data.unwrap(),
    }
  }
}

impl From<test_case::SuccessPayload> for v1::SignalPacket {
  fn from(value: test_case::SuccessPayload) -> Self {
    #[allow(deprecated)]
    Self {
      name: value.port,
      flag: value.flag.map_into(),
      flags: None,
    }
  }
}

impl From<test_case::PacketFlag> for v1::PacketFlag {
  fn from(value: test_case::PacketFlag) -> Self {
    match value {
      test_case::PacketFlag::Done => Self::Done,
      test_case::PacketFlag::Close => Self::Close,
      test_case::PacketFlag::Open => Self::Open,
    }
  }
}

impl From<v1::PacketFlag> for test_case::PacketFlag {
  fn from(value: v1::PacketFlag) -> Self {
    match value {
      v1::PacketFlag::Done => Self::Done,
      v1::PacketFlag::Close => Self::Close,
      v1::PacketFlag::Open => Self::Open,
    }
  }
}

impl From<test_case::InherentConfig> for v1::InherentData {
  fn from(value: test_case::InherentConfig) -> Self {
    Self {
      seed: value.seed,
      timestamp: value.timestamp,
    }
  }
}
