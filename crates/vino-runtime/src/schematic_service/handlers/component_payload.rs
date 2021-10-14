use crate::dev::prelude::*;

#[derive(Clone, Debug)]
pub struct ComponentPayload {
  pub tx_id: String,
  pub instance: String,
  pub payload_map: TransportMap,
}
