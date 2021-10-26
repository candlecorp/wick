use std::collections::HashMap;
use std::sync::atomic::{
  AtomicUsize,
  Ordering,
};
use std::sync::Arc;

use parking_lot::{
  Mutex,
  RwLock,
};
use vino_transport::TransportStream;
use vino_types::signatures::ProviderSignature;

use crate::error::WasmProviderError;
use crate::wasm_host::WasmHost;

pub(crate) struct HostPool {
  factory: Arc<Mutex<dyn Fn() -> WasmHost + Send + Sync + 'static>>,
  index: AtomicUsize,
  hosts: RwLock<Vec<WasmHost>>,
}

impl HostPool {
  pub(crate) fn start_hosts<F>(f: F, threads: usize) -> Self
  where
    F: Fn() -> WasmHost + Send + Sync + 'static,
  {
    let arcfn = Arc::new(Mutex::new(f));

    let hosts: RwLock<Vec<WasmHost>> = RwLock::new(vec![]);
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
    debug!("Spawning host");
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
