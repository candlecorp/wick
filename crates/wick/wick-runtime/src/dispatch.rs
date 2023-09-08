use uuid::Uuid;
use wick_packet::{Invocation, PacketStream, RuntimeConfig};

use crate::dev::prelude::*;

#[derive(Debug)]
#[must_use]
pub(crate) enum InvocationResponse {
  Stream {
    #[allow(unused)]
    tx_id: Uuid,
    rx: PacketStream,
  },
  Error {
    #[allow(unused)]
    tx_id: Uuid,
    msg: String,
  },
}

impl InvocationResponse {
  /// Creates an error response.
  pub(crate) const fn error(tx_id: Uuid, msg: String) -> InvocationResponse {
    InvocationResponse::Error { tx_id, msg }
  }

  #[allow(clippy::missing_const_for_fn)]
  pub(crate) fn ok(self) -> Result<PacketStream, RuntimeError> {
    match self {
      InvocationResponse::Stream { rx, .. } => Ok(rx),
      InvocationResponse::Error { msg, .. } => Err(RuntimeError::InvocationError(msg)),
    }
  }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum DispatchError {
  #[error("Entity not available {0}")]
  EntityNotAvailable(Uuid),
  #[error("Call failure {0}")]
  CallFailure(String),
}

impl From<ComponentError> for DispatchError {
  fn from(e: ComponentError) -> Self {
    DispatchError::CallFailure(e.to_string())
  }
}

pub(crate) async fn scope_invoke_async(
  scope_id: Uuid,
  invocation: Invocation,
  config: Option<RuntimeConfig>,
) -> Result<PacketStream, DispatchError> {
  let scope = Scope::for_id(&scope_id).ok_or(DispatchError::EntityNotAvailable(scope_id))?;

  let response = scope.invoke(invocation, config)?.await?;
  match response {
    InvocationResponse::Stream { rx, .. } => Ok(rx),
    InvocationResponse::Error { msg, .. } => Err(DispatchError::CallFailure(msg)),
  }
}

#[cfg(test)]
mod tests {

  use anyhow::Result;
  use wick_packet::{packet_stream, Entity, Packet};

  use super::*;
  use crate::test::prelude::{assert_eq, *};
  #[test_logger::test(tokio::test)]
  async fn invoke_async() -> Result<()> {
    let (_, nuid) = init_scope_from_yaml("./manifests/v0/echo.yaml").await?;

    let target = Entity::operation("self", "echo");
    let stream = packet_stream![("input", "hello")];
    let invocation = Invocation::test(file!(), target, stream, None)?;

    let packets = scope_invoke_async(nuid, invocation, Default::default()).await?;
    let mut packets: Vec<_> = packets.collect().await;
    debug!("{:?}", packets);
    assert_eq!(packets.len(), 2);
    let _ = packets.pop();
    let actual = packets.pop().unwrap().unwrap();
    assert_eq!(actual, Packet::encode("output", "hello"));

    Ok(())
  }
}
