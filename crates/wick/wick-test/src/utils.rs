use std::collections::HashMap;

use either::Either;
use wasmrs_codec::messagepack;
use wick_config::config::test_case::{ErrorPayload, PacketFlag, SuccessPayload};
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

pub(crate) trait ConfigError<OK> {
  fn config_error(self) -> Result<OK, TestError>;
}

impl<OK, E: std::fmt::Display> ConfigError<OK> for Result<OK, E> {
  fn config_error(self) -> Result<OK, TestError> {
    self.map_err(|e| TestError::Configuration(e.to_string()))
  }
}

/// Convert the [TestPacket] into a real [Packet].
pub(crate) fn gen_packet(
  p: Either<&SuccessPayload, &ErrorPayload>,
  root_config: Option<&RuntimeConfig>,
  op_config: Option<&RuntimeConfig>,
) -> Result<Packet, TestError> {
  let packet = match p {
    Either::Left(success) => Packet::new_for_port(
      success.port(),
      PacketPayload::Ok(match success.data() {
        Some(data) => {
          let ctx =
            LiquidJsonConfig::make_context(None, root_config, op_config, env().as_ref(), None).config_error()?;
          let data = data.render(&ctx).config_error()?;
          Some(
            messagepack::serialize(&data)
              .map_err(|e| TestError::Serialization(e.to_string()))?
              .into(),
          )
        }
        None => None,
      }),
      convert_flags(success.flag()),
    ),
    Either::Right(error) => Packet::new_for_port(
      error.port(),
      PacketPayload::Err(PacketError::new(
        error
          .error()
          .render(None, Some(&std::env::vars().collect()))
          .config_error()?,
      )),
      convert_flags(error.flag()),
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
    Ok(Some(config.render(None, None, Some(&env), inherent).config_error()?))
  } else {
    Ok(None)
  }
}
