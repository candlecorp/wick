use std::sync::Arc;
use std::time::Duration;

use seeded_random::Seed;
use uuid::Uuid;
use wick_packet::OperationConfig;

use crate::dev::prelude::*;
use crate::runtime_service::{ComponentFactory, ComponentRegistry, Initialize};

type Result<T> = std::result::Result<T, RuntimeError>;
#[derive(Debug)]
#[must_use]
pub struct Runtime {
  pub uid: Uuid,
  inner: Arc<RuntimeService>,
  timeout: Duration,
}

#[derive(Debug)]
#[must_use]
pub struct RuntimeInit {
  definition: config::ComponentConfiguration,
  allow_latest: bool,
  allowed_insecure: Vec<String>,
  timeout: Duration,
  namespace: Option<String>,
  rng_seed: Seed,
  native_components: ComponentRegistry,
}

impl Runtime {
  #[instrument(name = "runtime", skip_all)]
  pub async fn new(config: RuntimeInit) -> Result<Self> {
    trace!(?config, "init");
    let init = Initialize::new(
      config.rng_seed,
      config.definition,
      config.allowed_insecure.clone(),
      config.allow_latest,
      config.timeout,
      config.native_components,
      config.namespace,
      debug_span!("runtime:new"),
    );

    let service = RuntimeService::new(init)
      .await
      .map_err(|e| RuntimeError::InitializationFailed(e.to_string()))?;
    Ok(Self {
      uid: service.id,
      inner: service,
      timeout: config.timeout,
    })
  }

  pub async fn invoke(
    &self,
    invocation: Invocation,
    stream: PacketStream,
    config: Option<OperationConfig>,
  ) -> Result<PacketStream> {
    let time = std::time::SystemTime::now();
    trace!(start_time=%time.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() ,"invocation start");

    let response = tokio::time::timeout(self.timeout, self.inner.invoke(invocation, stream, config)?)
      .await
      .map_err(|_| RuntimeError::Timeout)??;
    trace!(duration_ms=%time.elapsed().unwrap().as_millis(),"invocation complete");

    Ok(response.ok()?)
  }

  pub async fn shutdown(&self) -> Result<()> {
    trace!("runtime engine shutting down");
    self.inner.shutdown().await?;

    Ok(())
  }

  pub fn get_signature(&self) -> Result<ComponentSignature> {
    let signature = self.inner.get_signature()?;
    trace!(?signature, "runtime engine instance signature");
    Ok(signature)
  }

  #[must_use]
  pub fn namespace(&self) -> &str {
    &self.inner.namespace
  }
}

/// The [RuntimeBuilder] builds the configuration for a Wick [Runtime].
#[derive(Default)]
#[must_use]
pub struct RuntimeBuilder {
  allow_latest: bool,
  allowed_insecure: Vec<String>,
  manifest_builder: config::ComponentConfigurationBuilder,
  timeout: Duration,
  rng_seed: Option<Seed>,
  namespace: Option<String>,
  native_components: ComponentRegistry,
}

impl std::fmt::Debug for RuntimeBuilder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RuntimeBuilder")
      .field("allow_latest", &self.allow_latest)
      .field("allowed_insecure", &self.allowed_insecure)
      .field("manifest_builder", &self.manifest_builder)
      .field("timeout", &self.timeout)
      .field("rng_seed", &self.rng_seed)
      .field("namespace", &self.namespace)
      .field("native_components", &self.native_components)
      .finish()
  }
}

impl RuntimeBuilder {
  pub fn new() -> Self {
    Self {
      timeout: Duration::from_secs(5),
      ..Default::default()
    }
  }

  /// Creates a new [RuntimeBuilder] from a [config::ComponentConfiguration]
  pub fn from_definition(definition: config::ComponentConfiguration) -> Result<Self> {
    Ok(Self {
      allow_latest: definition.allow_latest(),
      allowed_insecure: definition.insecure_registries().clone(),
      manifest_builder: config::ComponentConfigurationBuilder::from_base(definition),
      timeout: Duration::from_secs(5),
      native_components: ComponentRegistry::default(),
      namespace: None,
      rng_seed: None,
    })
  }

  pub fn add_import(&mut self, component: config::ImportBinding) -> &mut Self {
    self.manifest_builder.add_import(component);
    self
  }

  pub fn add_resource(&mut self, resource: config::ResourceBinding) -> &mut Self {
    self.manifest_builder.add_resource(resource);
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

  pub fn native_component(&mut self, factory: Box<ComponentFactory>) {
    self.native_components.add(factory);
  }

  /// Constructs an instance of a Wick [Runtime].
  pub async fn build(self) -> Result<Runtime> {
    let definition = self
      .manifest_builder
      .build()
      .map_err(|e| RuntimeError::InitializationFailed(e.to_string()))?;
    Runtime::new(RuntimeInit {
      definition,
      allow_latest: self.allow_latest,
      allowed_insecure: self.allowed_insecure,
      native_components: self.native_components,
      timeout: self.timeout,
      namespace: self.namespace,
      rng_seed: self.rng_seed.unwrap_or_else(new_seed),
    })
    .await
  }
}
