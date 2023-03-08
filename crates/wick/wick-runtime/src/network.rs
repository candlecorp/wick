use std::sync::Arc;
use std::time::Duration;

use seeded_random::{Random, Seed};
use uuid::Uuid;
use wick_config::{ComponentConfiguration, ComponentConfigurationBuilder, Flow};
use wick_packet::{Invocation, PacketStream};

use crate::dev::prelude::*;
use crate::network_service::Initialize;

type Result<T> = std::result::Result<T, RuntimeError>;
#[derive(Debug)]
#[must_use]
pub struct Network {
  pub uid: Uuid,
  inner: Arc<NetworkService>,
  timeout: Duration,
}

#[derive(Debug)]
#[must_use]
pub struct NetworkInit {
  definition: ComponentConfiguration,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
  timeout: Duration,
  // mesh: Option<Arc<Mesh>>,
  namespace: Option<String>,
  rng_seed: Seed,
}

impl Network {
  #[instrument(name = "network", skip_all)]
  pub async fn new(config: NetworkInit) -> Result<Self> {
    trace!(?config, "init");
    let rng = Random::from_seed(config.rng_seed);

    let init = Initialize {
      id: rng.uuid(),
      // mesh: config.mesh.clone(),
      manifest: config.definition,
      allowed_insecure: config.allowed_insecure.clone(),
      allow_latest: config.allow_latest,
      timeout: config.timeout,
      namespace: config.namespace,
      rng_seed: rng.seed(),
      event_log: None,
    };
    let service = NetworkService::new(init)
      .await
      .map_err(|e| RuntimeError::InitializationFailed(e.to_string()))?;
    Ok(Self {
      uid: service.id,
      inner: service,
      timeout: config.timeout,
    })
  }

  pub async fn invoke(&self, invocation: Invocation, stream: PacketStream) -> Result<PacketStream> {
    let time = std::time::SystemTime::now();
    trace!(start_time=%time.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() ,"invocation start");

    let response = tokio::time::timeout(self.timeout, self.inner.invoke(invocation, stream)?)
      .await
      .map_err(|_| NetworkError::Timeout)??;
    trace!(duration_ms=%time.elapsed().unwrap().as_millis(),"invocation complete");

    Ok(response.ok()?)
  }

  pub async fn shutdown(&self) -> Result<()> {
    trace!("network shutting down");
    self.inner.shutdown().await?;

    Ok(())
  }

  pub fn get_signature(&self) -> Result<CollectionSignature> {
    let signature = self.inner.get_signature()?;
    trace!(?signature, "network signature");
    Ok(signature)
  }
}

/// The [NetworkBuilder] builds the configuration for a Wick Network.
#[derive(Debug, Default)]
#[must_use]
pub struct NetworkBuilder {
  allow_latest: bool,
  allowed_insecure: Vec<String>,
  manifest_builder: ComponentConfigurationBuilder,
  // mesh: Option<Arc<Mesh>>,
  timeout: Duration,
  rng_seed: Option<Seed>,
  namespace: Option<String>,
}

impl NetworkBuilder {
  pub fn new() -> Self {
    Self {
      timeout: Duration::from_secs(5),
      ..Default::default()
    }
  }

  /// Creates a new network builder from a [NetworkDefinition]
  pub fn from_definition(definition: ComponentConfiguration) -> Result<Self> {
    Ok(Self {
      allow_latest: definition.allow_latest(),
      allowed_insecure: definition.insecure_registries().clone(),
      manifest_builder: ComponentConfigurationBuilder::with_base(definition),
      timeout: Duration::from_secs(5),
      // mesh: None,
      namespace: None,
      rng_seed: None,
    })
  }

  pub fn add_collection(mut self, collection: ComponentDefinition) -> Self {
    self.manifest_builder = self
      .manifest_builder
      .add_collection(collection.namespace.clone(), collection);
    self
  }

  pub fn add_flow(mut self, flow: Flow) -> Self {
    self.manifest_builder = self.manifest_builder.add_flow(flow.name.clone(), flow);
    self
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

  // pub fn mesh(self, mesh: Arc<Mesh>) -> Self {
  //   Self {
  //     mesh: Some(mesh),
  //     ..self
  //   }
  // }

  pub fn with_seed(self, seed: Seed) -> Self {
    Self {
      rng_seed: Some(seed),
      ..self
    }
  }

  pub fn namespace<T: AsRef<str>>(self, namespace: T) -> Self {
    Self {
      namespace: Some(namespace.as_ref().to_owned()),
      ..self
    }
  }

  /// Constructs an instance of a Wick host.
  pub async fn build(self) -> Result<Network> {
    let definition = self.manifest_builder.build();
    Network::new(NetworkInit {
      definition,
      allow_latest: self.allow_latest,
      allowed_insecure: self.allowed_insecure,
      timeout: self.timeout,
      namespace: self.namespace,
      // mesh: self.mesh,
      rng_seed: self.rng_seed.unwrap_or_else(new_seed),
    })
    .await
  }
}
