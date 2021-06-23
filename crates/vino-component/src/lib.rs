pub mod v0;

use serde::{
  Deserialize,
  Serialize,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// The output payload that component's push out of output ports
pub enum Output<T: Serialize> {
  /// Version 0 of the payload format (unstable)
  #[serde(rename = "0")]
  V0(v0::Payload<T>),
}
