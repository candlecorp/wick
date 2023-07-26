use std::collections::HashMap;

use wasmrs_codec::messagepack;
use wick_config::config::test_case::{PacketFlag, TestPacket};
use wick_config::config::LiquidJsonConfig;
use wick_packet::{
  InherentData,
  Packet,
  PacketError,
  PacketPayload,
  RuntimeConfig,
  CLOSE_BRACKET,
  DONE_FLAG,
  OPEN_BRACKET,
};

use crate::error::TestError;

fn env() -> Option<HashMap<String, String>> {
  Some(std::env::vars().collect())
}

#[allow(clippy::needless_pass_by_value)]
fn config_error(e: impl std::fmt::Display) -> TestError {
  TestError::Configuration(e.to_string())
}

/// Convert the [TestPacket] into a real [Packet].
pub(crate) fn gen_packet(
  p: &TestPacket,
  root_config: Option<&RuntimeConfig>,
  op_config: Option<&RuntimeConfig>,
) -> Result<Packet, TestError> {
  let packet = match p {
    TestPacket::SuccessPacket(data) => Packet::new_for_port(
      data.port(),
      PacketPayload::Ok(match data.data() {
        Some(data) => {
          let ctx =
            LiquidJsonConfig::make_context(None, root_config, op_config, env().as_ref(), None).map_err(config_error)?;
          let data = data.render(&ctx).map_err(config_error)?;
          Some(
            messagepack::serialize(&data)
              .map_err(|e| TestError::Serialization(e.to_string()))?
              .into(),
          )
        }
        None => None,
      }),
      convert_flags(data.flag()),
    ),
    TestPacket::ErrorPacket(data) => Packet::new_for_port(
      data.port(),
      PacketPayload::Err(PacketError::new(
        data
          .error()
          .render(None, Some(&std::env::vars().collect()))
          .map_err(config_error)?,
      )),
      convert_flags(data.flag()),
    ),
  };
  Ok(packet)
}

fn convert_flags(flag: Option<&PacketFlag>) -> u8 {
  let mut byte = 0;
  if let Some(flag) = flag {
    match flag {
      PacketFlag::Done => byte |= DONE_FLAG,
      PacketFlag::Open => byte |= OPEN_BRACKET,
      PacketFlag::Close => byte |= CLOSE_BRACKET,
    }
  }
  byte
}

pub(crate) fn render_config(
  config: Option<&LiquidJsonConfig>,
  inherent: Option<&InherentData>,
) -> Result<Option<RuntimeConfig>, TestError> {
  if let Some(config) = config {
    let env = std::env::vars().collect();
    Ok(Some(
      config.render(None, None, Some(&env), inherent).map_err(config_error)?,
    ))
  } else {
    Ok(None)
  }
}
