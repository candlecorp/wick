use std::sync::Arc;

use futures::Stream;
use parking_lot::{Condvar, Mutex};
use uuid::Uuid;
use wasmflow_sdk::v1::Invocation;

use crate::dev::prelude::*;

#[derive(Debug)]
#[must_use]
pub enum InvocationResponse {
  Stream { tx_id: Uuid, rx: TransportStream },
  Error { tx_id: Uuid, msg: String },
}

impl InvocationResponse {
  /// Creates a successful invocation response stream. Response include the receiving end.
  /// of an unbounded channel to listen for future output.
  pub fn stream(tx_id: Uuid, rx: impl Stream<Item = TransportWrapper> + Send + 'static) -> InvocationResponse {
    InvocationResponse::Stream {
      tx_id,
      rx: TransportStream::new(rx),
    }
  }

  /// Creates an error response.
  pub fn error(tx_id: Uuid, msg: String) -> InvocationResponse {
    InvocationResponse::Error { tx_id, msg }
  }

  pub fn tx_id(&self) -> &Uuid {
    match self {
      InvocationResponse::Stream { tx_id, .. } => tx_id,
      InvocationResponse::Error { tx_id, .. } => tx_id,
    }
  }

  pub fn ok(self) -> Result<TransportStream, InvocationError> {
    match self {
      InvocationResponse::Stream { rx, .. } => Ok(rx),
      InvocationResponse::Error { msg, .. } => Err(InvocationError(msg)),
    }
  }
}

#[derive(thiserror::Error, Debug)]
pub enum DispatchError {
  #[error("Thread died")]
  JoinFailed,
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

#[allow(unused)]
pub(crate) fn network_invoke_sync(
  network_id: Uuid,
  invocation: Invocation,
) -> Result<Vec<TransportWrapper>, DispatchError> {
  let pair = Arc::new((Mutex::new(false), Condvar::new()));
  let inner = Arc::clone(&pair);

  let handle = std::thread::spawn(move || {
    let system = tokio::runtime::Runtime::new().unwrap();
    let (lock, cvar) = &*inner;
    let mut started = lock.lock();
    *started = true;
    let re = system.block_on(network_invoke_async(network_id, invocation));
    cvar.notify_one();
    re
  });

  let (lock, cvar) = &*pair;
  let mut started = lock.lock();
  while !*started {
    std::thread::yield_now();
    cvar.wait(&mut started);
  }

  let packets = handle.join().map_err(|_| DispatchError::JoinFailed)??;
  Ok(packets)
}

#[cfg(test)]
mod tests {

  use tokio::sync::oneshot;
  use wasmflow_sdk::v1::packet::PacketMap;

  use super::*;
  use crate::test::prelude::{assert_eq, *};
  #[test_logger::test(tokio::test)]
  async fn invoke_async() -> TestResult<()> {
    let (_, nuid) = init_network_from_yaml("./manifests/v0/echo.yaml").await?;

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

  #[test_logger::test(tokio::test)]
  async fn invoke_sync() -> TestResult<()> {
    let (tx, rx) = oneshot::channel::<Uuid>();
    let (tx2, rx2) = oneshot::channel::<bool>();
    std::thread::spawn(|| {
      let system = tokio::runtime::Runtime::new().unwrap();

      let (_, nuid) = system
        .block_on(init_network_from_yaml("./manifests/v0/echo.yaml"))
        .unwrap();
      let _ = tx.send(nuid);
      let _ = system.block_on(rx2);
    });
    let nuid = rx.await?;

    let target = Entity::component("self", "echo");
    let map = PacketMap::from(vec![("input", "hello")]);
    let invocation = Invocation::new_test(file!(), target, map, None);

    let packets = network_invoke_sync(nuid, invocation)?;
    let _ = tx2.send(true);

    debug!("{:?}", packets);
    assert_eq!(packets.len(), 2);
    let rv: String = packets[0].payload.clone().deserialize()?;
    assert_eq!(rv, "hello");

    Ok(())
  }
}
