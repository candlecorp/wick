use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use vino_transport::TransportStream;
use vino_types::ProviderSignature;

use crate::error::WasmProviderError;
use crate::wasm_host::RpcProxy;

pub(crate) struct HostPool {
  factory: Arc<Mutex<dyn Fn() -> Box<dyn RpcProxy + Send + Sync> + Send + Sync + 'static>>,
  index: AtomicUsize,
  hosts: RwLock<Vec<Box<dyn RpcProxy + Send + Sync>>>,
}

impl Debug for HostPool {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("HostPool")
      .field("max", &self.hosts.read().len())
      .finish()
  }
}

impl HostPool {
  pub(crate) fn start_hosts<F>(f: F, threads: usize) -> Self
  where
    F: Fn() -> Box<dyn RpcProxy + Send + Sync> + Send + Sync + 'static,
  {
    let arcfn = Arc::new(Mutex::new(f));

    let hosts: RwLock<Vec<Box<dyn RpcProxy + Send + Sync>>> = RwLock::new(vec![]);
    let index = 0;

    let pool = Self {
      factory: arcfn,
      index: index.into(),
      hosts,
    };

    for _i in 0..threads {
      pool.expand();
    }

    pool
  }

  fn expand(&self) {
    let factory = self.factory.clone();
    let host = factory.lock()();
    let mut lock = self.hosts.write();
    lock.push(host);
  }

  fn get_next_index(&self) -> usize {
    let lock = self.hosts.read();
    let len = lock.len();
    self
      .index
      .fetch_update(Ordering::SeqCst, Ordering::SeqCst, move |old| {
        let new = old + 1;
        if new == len {
          Some(0)
        } else {
          Some(new)
        }
      })
      .unwrap()
  }

  pub(crate) fn call(
    &self,
    component_name: &str,
    input_map: &HashMap<String, Vec<u8>>,
  ) -> Result<TransportStream, WasmProviderError> {
    debug!("Invoking from pool");
    let index = self.get_next_index();
    let lock = self.hosts.read();
    let host = lock.get(index).unwrap();
    host.call(component_name, input_map)
  }

  pub(crate) fn get_components(&self) -> ProviderSignature {
    debug!("Sending message");
    let index = self.get_next_index();
    let lock = self.hosts.read();
    let host = lock.get(index).unwrap();
    host.get_components().clone()
  }
}

#[cfg(test)]
mod tests {
  use std::time::{Duration, Instant};

  use futures::future::join_all;
  use tokio::sync::mpsc::unbounded_channel;
  use tokio_stream::wrappers::UnboundedReceiverStream;
  use tokio_stream::StreamExt;
  use vino_transport::{MessageTransport, TransportWrapper};

  use super::*;

  type Result<T> = anyhow::Result<T, anyhow::Error>;
  #[test_logger::test(tokio::test)]
  async fn test() -> Result<()> {
    #[derive(Default)]
    struct Test {
      sig: ProviderSignature,
    }
    impl RpcProxy for Test {
      fn call(
        &self,
        _component_name: &str,
        _input_map: &HashMap<String, Vec<u8>>,
      ) -> crate::Result<TransportStream> {
        let (tx, rx) = unbounded_channel::<TransportWrapper>();
        let stream = UnboundedReceiverStream::new(rx);
        tokio::spawn(async move {
          tokio::time::sleep(Duration::from_millis(200)).await;
          tx.send(TransportWrapper::new(
            "test",
            MessageTransport::success(&true),
          ))
          .unwrap();
          tx.send(TransportWrapper::new_system_close()).unwrap();
        });
        Ok(TransportStream::new(stream))
      }

      fn get_components(&self) -> &ProviderSignature {
        &self.sig
      }
    }
    let pool = HostPool::start_hosts(move || Box::new(Test::default()), 5);

    let num = 5;
    let mut streams = vec![];
    let now = Instant::now();
    for _ in 0..num {
      streams.push(pool.call("test1", &HashMap::default())?);
    }
    let mut packets = vec![];
    for stream in streams {
      packets.push(stream.collect::<Vec<_>>());
    }

    let results = join_all(packets).await;
    // naive assertion checking that the pool executed all the sleeps in parallel.
    assert!(now.elapsed().as_millis() < 300);
    for packet in results {
      assert_eq!(packet.len(), 1);
    }

    Ok(())
  }
}
