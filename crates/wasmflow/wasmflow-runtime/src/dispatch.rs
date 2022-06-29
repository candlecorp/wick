use uuid::Uuid;
use wasmflow_sdk::v1::Invocation;

use crate::dev::prelude::*;

#[derive(Debug)]
#[must_use]
pub(crate) enum InvocationResponse {
  Stream {
    #[allow(unused)]
    tx_id: Uuid,
    rx: TransportStream,
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

  pub(crate) fn ok(self) -> Result<TransportStream, InvocationError> {
    match self {
      InvocationResponse::Stream { rx, .. } => Ok(rx),
      InvocationResponse::Error { msg, .. } => Err(InvocationError(msg)),
    }
  }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum DispatchError {
  #[error("{0}")]
  Sdk(String),
  #[error("Entity not available {0}")]
  EntityNotAvailable(Uuid),
  #[error("Call failure {0}")]
  CallFailure(String),
}

impl From<wasmflow_sdk::v1::error::Error> for DispatchError {
  fn from(e: wasmflow_sdk::v1::error::Error) -> Self {
    DispatchError::Sdk(e.to_string())
  }
}

impl From<CollectionError> for DispatchError {
  fn from(e: CollectionError) -> Self {
    DispatchError::CallFailure(e.to_string())
  }
}

pub(crate) async fn network_invoke_async(
  network_id: Uuid,
  invocation: Invocation,
) -> Result<Vec<TransportWrapper>, DispatchError> {
  let network = NetworkService::for_id(&network_id).ok_or(DispatchError::EntityNotAvailable(network_id))?;

  let response = network.invoke(invocation)?.await?;
  match response {
    InvocationResponse::Stream { rx, .. } => {
      let messages: Vec<TransportWrapper> = rx.collect().await;
      trace!(num_messages = messages.len(), "link_call response");
      debug!(?messages, "link call response");
      Ok(messages)
    }
    InvocationResponse::Error { msg, .. } => Err(DispatchError::CallFailure(msg)),
  }
}

#[cfg(test)]
mod tests {

  use wasmflow_sdk::v1::packet::PacketMap;

  use super::*;
  use crate::test::prelude::{assert_eq, *};
  #[test_logger::test(tokio::test)]
  async fn invoke_async() -> TestResult<()> {
    let (_, nuid) = init_network_from_yaml("./manifests/v0/echo.wafl").await?;

    let target = Entity::component("self", "echo");
    let map = PacketMap::from(vec![("input", "hello")]);
    let invocation = Invocation::new_test(file!(), target, map, None);

    let packets = network_invoke_async(nuid, invocation).await?;
    debug!("{:?}", packets);
    assert_eq!(packets.len(), 2);
    let rv: String = packets[0].payload.clone().deserialize()?;
    assert_eq!(rv, "hello");

    Ok(())
  }
}
