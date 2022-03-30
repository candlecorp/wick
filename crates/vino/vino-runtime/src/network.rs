use std::sync::Arc;
use std::time::Duration;

use vino_lattice::Lattice;
use vino_manifest::HostDefinition;
use vino_wascap::KeyPair;

use crate::dev::prelude::*;
use crate::network_service::Initialize;

type Result<T> = std::result::Result<T, RuntimeError>;
#[derive(Debug)]
#[must_use]
pub struct Network {
  pub uid: String,
  inner: Arc<NetworkService>,
  #[allow(unused)]
  kp: KeyPair,
  timeout: Duration,
}

#[derive(Debug)]
#[must_use]
pub struct NetworkInit {
  uid: String,
  definition: HostDefinition,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
  kp: KeyPair,
  timeout: Duration,
  lattice: Option<Arc<Lattice>>,
  rng_seed: u64,
}

impl Network {
  pub async fn new_default(definition: HostDefinition, seed: &str) -> Result<Self> {
    Ok(NetworkBuilder::from_definition(definition, seed)?.build().await?)
  }

  #[instrument(name = "network", skip_all)]
  pub async fn new(config: NetworkInit) -> Result<Self> {
    trace!(?config, "init");
    let init = Initialize {
      id: config.uid.clone(),
      lattice: config.lattice.clone(),
      network: config.definition.clone(),
      allowed_insecure: config.allowed_insecure.clone(),
      allow_latest: config.allow_latest,
      timeout: config.timeout,
      rng_seed: config.rng_seed,
    };
    let service = NetworkService::new(init)
      .await
      .map_err(|e| RuntimeError::InitializationFailed(e.to_string()))?;
    Ok(Self {
      uid: config.uid,
      inner: service,
      kp: config.kp,
      timeout: config.timeout,
    })
  }

  #[instrument(skip_all, fields(target=?invocation.target))]
  pub async fn invoke(&self, invocation: Invocation) -> Result<TransportStream> {
    let time = std::time::Instant::now();
    trace!(start_time=?time,"invocation start");

    let response = tokio::time::timeout(self.timeout, self.inner.invoke(invocation)?)
      .await
      .map_err(|_| NetworkError::Timeout)??;
    trace!(duration=?time.elapsed().as_micros(),"invocation complete");

    Ok(response.ok()?)
  }

  pub fn get_signature(&self) -> Result<ProviderSignature> {
    let signature = self.inner.get_signature()?;
    trace!(?signature, "network signature");
    Ok(signature)
  }
}

/// The [NetworkBuilder] builds the configuration for a Vino Network.
#[derive(Debug)]
#[must_use]
pub struct NetworkBuilder {
  allow_latest: bool,
  allowed_insecure: Vec<String>,
  definition: HostDefinition,
  kp: KeyPair,
  uid: String,
  lattice: Option<Arc<Lattice>>,
  timeout: Duration,
  rng_seed: Option<u64>,
}

impl NetworkBuilder {
  /// Creates a new network builder from a [NetworkDefinition]
  pub fn from_definition(definition: HostDefinition, seed: &str) -> Result<Self> {
    let kp = keypair_from_seed(seed)?;
    let nuid = kp.public_key();
    Ok(Self {
      allow_latest: definition.host.allow_latest,
      allowed_insecure: definition.host.insecure_registries.clone(),
      definition,
      uid: nuid,
      timeout: Duration::from_secs(5),
      lattice: None,
      kp,
      rng_seed: None,
    })
  }

  pub fn timeout(self, timeout: Duration) -> Self {
    Self { timeout, ..self }
  }

  pub fn allow_latest(self, allow_latest: bool) -> Self {
    Self { allow_latest, ..self }
  }

  pub fn allow_insecure(self, allowed_insecure: Vec<String>) -> Self {
    Self {
      allowed_insecure,
      ..self
    }
  }

  pub fn lattice(self, lattice: Arc<Lattice>) -> Self {
    Self {
      lattice: Some(lattice),
      ..self
    }
  }

  pub fn with_seed(self, seed: u64) -> Self {
    Self {
      rng_seed: Some(seed),
      ..self
    }
  }

  /// Constructs an instance of a Vino host.
  pub async fn build(self) -> Result<Network> {
    Network::new(NetworkInit {
      definition: self.definition,
      uid: self.uid,
      allow_latest: self.allow_latest,
      allowed_insecure: self.allowed_insecure,
      kp: self.kp,
      timeout: self.timeout,
      lattice: self.lattice,
      rng_seed: self.rng_seed.unwrap_or_else(new_seed),
    })
    .await
  }
}
