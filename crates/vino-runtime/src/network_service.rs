pub(crate) mod handlers;

use std::collections::HashMap;

use actix::SystemRunner;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use vino_wascap::KeyPair;

use crate::dev::prelude::*;

type Result<T> = std::result::Result<T, NetworkError>;
#[derive(Debug)]

pub(crate) struct NetworkService {
  started: bool,
  started_time: std::time::Instant,
  state: Option<State>,
  id: String,
  schematics: HashMap<String, Addr<SchematicService>>,
  definition: NetworkDefinition,
}

#[derive(Debug)]
struct State {
  kp: KeyPair,
}

impl Default for NetworkService {
  fn default() -> Self {
    NetworkService {
      started: false,
      started_time: std::time::Instant::now(),
      id: "".to_owned(),
      state: None,
      schematics: HashMap::new(),
      definition: NetworkDefinition::default(),
    }
  }
}

type ServiceMap = HashMap<String, Addr<NetworkService>>;
static HOST_REGISTRY: Lazy<Mutex<ServiceMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

impl NetworkService {
  pub(crate) fn for_id(network_id: &str) -> Addr<Self> {
    trace!("NETWORK:GET:{}", network_id);
    let sys = System::current();
    let mut registry = HOST_REGISTRY.lock();
    let addr = registry
      .entry(network_id.to_owned())
      .or_insert_with(|| NetworkService::start_service(sys.arbiter()));

    addr.clone()
  }
  pub(crate) fn get_schematic_addr(&self, id: &str) -> Result<Addr<SchematicService>> {
    self
      .schematics
      .get(id)
      .cloned()
      .ok_or_else(|| NetworkError::SchematicNotFound(id.to_owned()))
  }
  pub(crate) fn ensure_is_started(&self) -> Result<()> {
    if self.started {
      Ok(())
    } else {
      Err(NetworkError::NotStarted)
    }
  }
}

impl Supervised for NetworkService {}

impl SystemService for NetworkService {
  fn service_started(&mut self, ctx: &mut Context<Self>) {
    trace!("NETWORK:Service starting");
    ctx.set_mailbox_capacity(1000);
  }
}

impl Actor for NetworkService {
  type Context = Context<Self>;
}

#[cfg(test)]
mod test {
  // You can find many of the network tests in the integration tests
}
