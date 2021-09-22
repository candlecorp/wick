pub mod error;
pub(crate) mod handlers;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use vino_lattice::lattice::Lattice;
use vino_manifest::Loadable;
use vino_wascap::KeyPair;

use crate::dev::prelude::*;
use crate::network_service::handlers::initialize::Initialize;

type Result<T> = std::result::Result<T, NetworkError>;
#[derive(Debug)]

pub(crate) struct NetworkService {
  started: bool,
  started_time: std::time::Instant,
  state: Option<State>,
  uid: String,
  schematics: HashMap<String, Addr<SchematicService>>,
  definition: NetworkDefinition,
  lattice: Option<Arc<Lattice>>,
  allow_latest: bool,
  insecure: Vec<String>,
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
      uid: "".to_owned(),
      state: None,
      schematics: HashMap::new(),
      definition: NetworkDefinition::default(),
      lattice: None,
      allow_latest: false,
      insecure: vec![],
    }
  }
}

type ServiceMap = HashMap<String, Addr<NetworkService>>;
static HOST_REGISTRY: Lazy<Mutex<ServiceMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

impl NetworkService {
  pub(crate) fn for_id(uid: &str) -> Addr<Self> {
    trace!("NETWORK:GET:{}", uid);
    let sys = System::current();
    let mut registry = HOST_REGISTRY.lock();
    let addr = registry
      .entry(uid.to_owned())
      .or_insert_with(|| NetworkService::start_service(sys.arbiter()));

    addr.clone()
  }
  pub(crate) async fn start_from_manifest(
    location: &str,
    seed: &str,
    allow_latest: bool,
    allowed_insecure: Vec<String>,
    lattice: Option<Arc<Lattice>>,
    timeout: Duration,
  ) -> Result<String> {
    let bytes = vino_loader::get_bytes(location, allow_latest, &allowed_insecure).await?;
    let manifest = vino_manifest::HostManifest::load_from_bytes(&bytes)?;
    let def = NetworkDefinition::from(manifest.network());
    let kp = KeyPair::from_seed(seed).unwrap();

    let addr = NetworkService::for_id(&kp.public_key());
    let init = Initialize {
      network: def,
      network_uid: kp.public_key(),
      seed: seed.to_owned(),
      allowed_insecure,
      allow_latest,
      lattice,
      timeout,
    };
    addr.send(init).await.map_err(|_| InternalError::E5001)??;

    Ok(kp.public_key())
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
