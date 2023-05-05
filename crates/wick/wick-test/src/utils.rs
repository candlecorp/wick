use wasmrs_codec::messagepack;
use wick_config::config::{PacketFlags, TestPacket};
use wick_packet::{Packet, PacketError, PacketPayload, CLOSE_BRACKET, DONE_FLAG, OPEN_BRACKET};

use crate::error::TestError;

/// Convert the [TestPacket] into a real [Packet].
pub(crate) fn gen_packet(p: &TestPacket) -> Result<Packet, TestError> {
  let packet = match p {
    TestPacket::PayloadData(data) => Packet::new_for_port(
      &data.port,
      PacketPayload::Ok(match &data.data {
        Some(data) => Some(
          messagepack::serialize(data)
            .map_err(|e| TestError::ConversionFailed(e.to_string()))?
            .into(),
        ),
        None => None,
      }),
      convert_flags(data.flags),
    ),
    TestPacket::ErrorData(data) => Packet::new_for_port(
      &data.port,
      PacketPayload::Err(PacketError::new(&data.error)),
      convert_flags(data.flags),
    ),
  };
  Ok(packet)
}

fn convert_flags(flags: Option<PacketFlags>) -> u8 {
  let mut byte = 0;
  if let Some(flags) = flags {
    if flags.done {
      byte |= DONE_FLAG;
    }
    if flags.open {
      byte |= OPEN_BRACKET;
    }
    if flags.close {
      byte |= CLOSE_BRACKET;
    }
  }
  byte
}
