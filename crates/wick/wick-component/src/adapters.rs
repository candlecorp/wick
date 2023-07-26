/// Utility functions for binary operations (operations with two inputs).
pub mod binary;
/// Utility functions for unary operations (operations with one input).
pub mod unary;

use wick_packet::PacketPayload;

/// Encode a [Result] type into a raw [PacketPayload]
pub fn encode<T: serde::Serialize, E: std::fmt::Display>(val: Result<T, E>) -> PacketPayload {
  match val {
    Ok(v) => PacketPayload::encode(v),
    Err(e) => PacketPayload::err(e.to_string()),
  }
}
