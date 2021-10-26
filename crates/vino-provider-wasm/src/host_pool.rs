use std::{collections::HashMap, sync::Arc};

use parking_lot::{Mutex, RwLock};
use vino_transport::TransportStream;
use vino_types::signatures::ProviderSignature;

use crate::{error::WasmProviderError, wasm_host::WasmHost};

pub(crate) struct HostPool {
  factory: Arc<Mutex<dyn Fn() -> WasmHost + Send + Sync + 'static>>,
  iter: Arc<RwLock<Box<dyn Iterator<Item = WasmHost> + Send + Sync + 'static>>>,
  hosts: Vec<WasmHost>,
}

impl HostPool {
  pub(crate) fn start_hosts<F>(f: F, threads: u8) -> Self
  where
    F: Fn() -> WasmHost + Send + Sync + 'static,
  {
    let arcfn = Arc::new(Mutex::new(f));

    let addresses: Vec<WasmHost> = vec![];

    let mut pool = Self {
      factory: arcfn,
      iter: Arc::new(RwLock::new(Box::new(addresses.clone().into_iter()))),
      hosts: addresses,
    };

    for _i in 0..threads {
      pool.spawn_new();
    }

    pool
  }

  fn next_host(&self) -> WasmHost {
    let next = { self.iter.write().next() };
    match next {
      Some(addr) => addr,
      None => {
        let mut iter = self.hosts.clone().into_iter();
        let addr = match iter.next() {
          Some(addr) => addr,
          None => {
            panic!("No addresses left for actor");
          }
        };
        *self.iter.write() = Box::new(iter);
        addr
      }
    }
  }

  fn spawn_new(&mut self) {
    debug!("Spawning host");
    let factory = self.factory.clone();
    let host = factory.lock()();
    self.hosts.push(host);
  }

  pub(crate) fn call(
    &self,
    component_name: &str,
    input_map: &HashMap<String, Vec<u8>>,
  ) -> Result<TransportStream, WasmProviderError> {
    log::debug!("Invoking from pool");

    let host = self.next_host();
    host.call(component_name, input_map)
  }

  pub(crate) fn get_components(&self) -> ProviderSignature {
    log::debug!("Sending message");

    let host = self.next_host();
    host.get_components().clone()
  }
}
