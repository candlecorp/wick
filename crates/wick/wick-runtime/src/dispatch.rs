use uuid::Uuid;
use wick_packet::{Invocation, PacketStream};

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
  pub(crate) fn error(tx_id: Uuid, msg: String) -> InvocationResponse {
    InvocationResponse::Error { tx_id, msg }
  }

  pub(crate) fn ok(self) -> Result<PacketStream, InvocationError> {
    match self {
      InvocationResponse::Stream { rx, .. } => Ok(rx),
      InvocationResponse::Error { msg, .. } => Err(InvocationError(msg)),
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

pub(crate) async fn engine_invoke_async(
  engine_id: Uuid,
  invocation: Invocation,
  stream: PacketStream,
) -> Result<PacketStream, DispatchError> {
  let engine = RuntimeService::for_id(&engine_id).ok_or(DispatchError::EntityNotAvailable(engine_id))?;

  let response = engine.invoke(invocation, stream)?.await?;
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
    let (_, nuid) = init_engine_from_yaml("./manifests/v0/echo.yaml").await?;

    let target = Entity::operation("self", "echo");
    let stream = packet_stream![("input", "hello")];
    let invocation = Invocation::new(Entity::test(file!()), target, None);

    let packets = engine_invoke_async(nuid, invocation, stream).await?;
    let mut packets: Vec<_> = packets.collect().await;
    debug!("{:?}", packets);
    assert_eq!(packets.len(), 2);
    let _ = packets.pop();
    let actual = packets.pop().unwrap().unwrap();
    assert_eq!(actual, Packet::encode("output", "hello"));

    Ok(())
  }

  // fn sync_send<T>()
  // where
  //   T: Sync + Send,
  // {
  // }

  // #[test_logger::test]
  // fn test_sync_send() -> Result<()> {
  //   sync_send::<InvocationResponse>();
  //   Ok(())
  // }
}
