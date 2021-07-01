use std::collections::HashMap;

use nkeys::KeyPair;
use serde::Serialize;

use crate::dev::prelude::*;
pub use crate::dispatch::{
  ComponentEntity,
  PortReference,
  VinoEntity,
};
pub use crate::network_provider::Provider as NetworkProvider;
use crate::network_service::Initialize;

#[derive(Debug)]
pub struct Network {
  pub id: String,
  definition: NetworkDefinition,
  addr: Addr<NetworkService>,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
  kp: KeyPair,
}

impl Network {
  #[must_use]
  pub fn new(definition: NetworkDefinition, seed: &str) -> Self {
    NetworkBuilder::new(definition, seed).build()
  }
  pub async fn init(&self) -> Result<()> {
    let kp = KeyPair::new_service();
    let seed = kp.seed()?;
    self
      .addr
      .send(Initialize {
        network_id: self.id.clone(),
        seed,
        network: self.definition.clone(),
        allowed_insecure: self.allowed_insecure.clone(),
        allow_latest: self.allow_latest,
      })
      .await??;
    Ok(())
  }
  pub async fn request<T, U>(
    &self,
    schematic: T,
    data: &HashMap<U, impl Serialize + Sync>,
  ) -> Result<HashMap<String, MessageTransport>>
  where
    T: AsRef<str> + Send + Sync,
    U: AsRef<str> + Send + Sync,
  {
    let serialized_data: HashMap<String, Vec<u8>> = data
      .iter()
      .map(|(k, v)| Ok((k.as_ref().to_owned(), mp_serialize(&v)?)))
      .filter_map(Result::ok)
      .collect();

    let time = std::time::Instant::now();
    let result = self
      .addr
      .send(crate::network_service::Request {
        schematic: schematic.as_ref().to_owned(),
        data: serialized_data,
      })
      .await??;
    trace!(
      "result for {} took {} μs",
      schematic.as_ref().to_owned(),
      time.elapsed().as_micros()
    );
    trace!("Result: {:?}", result);
    Ok(result)
  }
}

/// The HostBuilder builds the configuration for a Vino Host
#[derive(Debug)]
pub struct NetworkBuilder {
  allow_latest: bool,
  allowed_insecure: Vec<String>,
  definition: NetworkDefinition,
  kp: KeyPair,
  id: String,
}

impl NetworkBuilder {
  /// Creates a new host builder
  #[must_use]
  pub fn new(definition: NetworkDefinition, seed: &str) -> Self {
    let kp = KeyPair::from_seed(seed).unwrap();
    let network_id = kp.public_key();
    Self {
      definition,
      allow_latest: false,
      allowed_insecure: vec![],
      id: network_id,
      kp,
    }
  }

  #[must_use]
  pub fn allow_latest(self, val: bool) -> Self {
    Self {
      allow_latest: val,
      ..self
    }
  }

  #[must_use]
  pub fn allow_insecure(self, registries: Vec<String>) -> Self {
    Self {
      allowed_insecure: registries,
      ..self
    }
  }

  /// Constructs an instance of a Vino host.
  #[must_use]
  pub fn build(self) -> Network {
    let addr = crate::network_service::NetworkService::for_id(&self.id);
    Network {
      addr,
      definition: self.definition,
      id: self.id,
      allow_latest: self.allow_latest,
      allowed_insecure: self.allowed_insecure,
      kp: self.kp,
    }
  }
}
