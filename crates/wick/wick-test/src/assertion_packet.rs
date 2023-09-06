use either::Either;
use flow_component::Value;
use wick_config::config::test_case::{AssertionOperator, PacketData, TestPacketData};
use wick_config::config::LiquidJsonConfig;
use wick_packet::{Packet, RuntimeConfig};

use crate::utils::{gen_packet, ConfigError};
use crate::TestError;

pub(crate) trait ToPacket {
  fn to_packet(
    &self,
    root_config: Option<&RuntimeConfig>,
    op_config: Option<&RuntimeConfig>,
  ) -> Result<Packet, TestError>;
}

impl ToPacket for PacketData {
  fn to_packet(
    &self,
    root_config: Option<&RuntimeConfig>,
    op_config: Option<&RuntimeConfig>,
  ) -> Result<Packet, TestError> {
    gen_packet(
      match self {
        PacketData::SuccessPacket(l) => Either::Left(l),
        PacketData::ErrorPacket(r) => Either::Right(r),
      },
      root_config,
      op_config,
    )
  }
}

impl ToPacket for TestPacketData {
  fn to_packet(
    &self,
    root_config: Option<&RuntimeConfig>,
    op_config: Option<&RuntimeConfig>,
  ) -> Result<Packet, TestError> {
    gen_packet(
      match self {
        TestPacketData::SuccessPacket(l) => Either::Left(l),
        TestPacketData::PacketAssertion(_) => unreachable!("ContainsPacket can not be converted to a packet"),
        TestPacketData::ErrorPacket(r) => Either::Right(r),
      },
      root_config,
      op_config,
    )
  }
}

pub(crate) trait ToAssertionPacket {
  fn to_assertion_packet(
    &self,
    root_config: Option<&RuntimeConfig>,
    op_config: Option<&RuntimeConfig>,
  ) -> Result<TestKind, TestError>;
}

impl ToAssertionPacket for TestPacketData {
  fn to_assertion_packet(
    &self,
    root_config: Option<&RuntimeConfig>,
    op_config: Option<&RuntimeConfig>,
  ) -> Result<TestKind, TestError> {
    Ok(match self {
      TestPacketData::SuccessPacket(_) | TestPacketData::ErrorPacket(_) => {
        self.to_packet(root_config, op_config)?.into()
      }
      TestPacketData::PacketAssertion(p) => {
        let env = std::env::vars().collect();

        let ctx = LiquidJsonConfig::make_context(None, root_config, op_config, Some(&env), None).config_error()?;
        TestKind::Assertion(AssertionDef {
          port: self.port().to_owned(),
          assertions: p
            .assertions()
            .iter()
            .map(|a| {
              Ok(Assertion {
                path: a.path().cloned(),
                operator: *a.operator(),
                value: a.value().render(&ctx).config_error()?,
              })
            })
            .collect::<Result<Vec<_>, _>>()?,
        })
      }
    })
  }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TestKind {
  Exact(Packet),
  Assertion(AssertionDef),
}

#[derive(Debug, PartialEq, Clone)]
pub struct AssertionDef {
  pub(crate) port: String,
  pub(crate) assertions: Vec<Assertion>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Assertion {
  pub(crate) path: Option<String>,
  pub(crate) operator: AssertionOperator,
  pub(crate) value: Value,
}

impl From<Packet> for TestKind {
  fn from(value: Packet) -> Self {
    Self::Exact(value)
  }
}

impl TestKind {
  pub(crate) fn port(&self) -> &str {
    match self {
      TestKind::Exact(p) => p.port(),
      TestKind::Assertion(p) => &p.port,
    }
  }

  pub(crate) const fn flags(&self) -> u8 {
    match self {
      TestKind::Exact(p) => p.flags(),
      TestKind::Assertion(_) => 0,
    }
  }

  pub(crate) fn has_data(&self) -> bool {
    match self {
      TestKind::Exact(p) => p.has_data(),
      TestKind::Assertion(_) => true,
    }
  }
}
