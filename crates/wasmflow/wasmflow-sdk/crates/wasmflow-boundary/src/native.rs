/// Utility functions that return v1 packets.
pub mod v1 {
  use serde::de::DeserializeOwned;

  use crate::incoming::IncomingPayload;

  /// Convert an [wasmflow_invocation::Invocation] into an [IncomingPayload].
  pub fn from_invocation<C>(
    invocation: wasmflow_invocation::Invocation,
  ) -> Result<IncomingPayload<wasmflow_packet::v1::PacketMap, C>, Box<dyn std::error::Error + Send + Sync>>
  where
    C: std::fmt::Debug + DeserializeOwned,
  {
    let (payload, config) = invocation.into_v1_parts().map_err(Box::new)?;

    Ok(IncomingPayload::new(0, payload, config))
  }
}
