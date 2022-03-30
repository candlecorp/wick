use std::sync::Arc;

use futures::Stream;
use parking_lot::{Condvar, Mutex};
use vino_transport::Invocation;

use crate::dev::prelude::*;

#[derive(Debug)]
#[must_use]
pub enum InvocationResponse {
  Stream { tx_id: String, rx: TransportStream },
  Error { tx_id: String, msg: String },
}

impl InvocationResponse {
  /// Creates a successful invocation response stream. Response include the receiving end.
  /// of an unbounded channel to listen for future output.
  pub fn stream(tx_id: String, rx: impl Stream<Item = TransportWrapper> + Send + 'static) -> InvocationResponse {
    InvocationResponse::Stream {
      tx_id,
      rx: TransportStream::new(rx),
    }
  }

  /// Creates an error response.
  pub fn error(tx_id: String, msg: String) -> InvocationResponse {
    InvocationResponse::Error { tx_id, msg }
  }

  pub fn tx_id(&self) -> &str {
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
  #[error("Thread died {0}")]
  EntityFailure(String),
  #[error("Entity not available {0}")]
  EntityNotAvailable(String),
  #[error("Call failure {0}")]
  CallFailure(String),
}

impl From<vino_entity::Error> for DispatchError {
  fn from(e: vino_entity::Error) -> Self {
    DispatchError::EntityFailure(e.to_string())
  }
}

impl From<ProviderError> for DispatchError {
  fn from(e: ProviderError) -> Self {
    DispatchError::CallFailure(e.to_string())
  }
}

#[allow(unused)]
pub(crate) async fn network_invoke_async(
  network_id: String,
  invocation: Invocation,
) -> Result<Vec<TransportWrapper>, DispatchError> {
  let network =
    NetworkService::for_id(&network_id).ok_or_else(|| DispatchError::EntityNotAvailable(network_id.clone()))?;

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
  network_id: String,
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
  use vino_transport::TransportMap;

  use super::*;
  use crate::test::prelude::{assert_eq, *};
  #[test_logger::test(tokio::test)]
  async fn invoke_async() -> TestResult<()> {
    let (_, nuid) = init_network_from_yaml("./manifests/v0/echo.yaml").await?;

    let target = Entity::component("self", "echo");
    let map = TransportMap::from(vec![("input", "hello")]);
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
    let (tx, rx) = oneshot::channel::<String>();
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
    let map = TransportMap::from(vec![("input", "hello")]);
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
