use std::collections::HashMap;

use wasmrs_codec::messagepack;
use wick_config::config::{LiquidJsonConfig, PacketFlags, TestPacket};
use wick_packet::{Packet, PacketError, PacketPayload, RuntimeConfig, CLOSE_BRACKET, DONE_FLAG, OPEN_BRACKET};

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
            LiquidJsonConfig::make_context(None, root_config, op_config, env().as_ref()).map_err(config_error)?;
          let data = data.render(&ctx).map_err(config_error)?;
          Some(
            messagepack::serialize(&data)
              .map_err(|e| TestError::Serialization(e.to_string()))?
              .into(),
          )
        }
        None => None,
      }),
      convert_flags(data.flags()),
    ),
    TestPacket::ErrorPacket(data) => Packet::new_for_port(
      data.port(),
      PacketPayload::Err(PacketError::new(
        data
          .error()
          .render(None, Some(&std::env::vars().collect()))
          .map_err(config_error)?,
      )),
      convert_flags(data.flags()),
    ),
  };
  Ok(packet)
}

fn convert_flags(flags: Option<&PacketFlags>) -> u8 {
  let mut byte = 0;
  if let Some(flags) = flags {
    if flags.done() {
      byte |= DONE_FLAG;
    }
    if flags.open() {
      byte |= OPEN_BRACKET;
    }
    if flags.close() {
      byte |= CLOSE_BRACKET;
    }
  }
  byte
}

pub(crate) fn render_config(config: Option<&LiquidJsonConfig>) -> Result<Option<RuntimeConfig>, TestError> {
  if let Some(config) = config {
    let env = std::env::vars().collect();
    Ok(Some(config.render(None, None, Some(&env)).map_err(config_error)?))
  } else {
    Ok(None)
  }
}
