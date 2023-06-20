use std::sync::Arc;

use seeded_random::Seed;
use tracing::Span;
use uuid::Uuid;
use wick_config::config::{ComponentConfiguration, ComponentConfigurationBuilder};
use wick_packet::{Entity, RuntimeConfig};

use crate::dev::prelude::*;
use crate::runtime_service::{ComponentFactory, ComponentRegistry, ServiceInit};

type Result<T> = std::result::Result<T, RuntimeError>;
#[derive(Debug, Clone)]
#[must_use]
pub struct Runtime {
  pub uid: Uuid,
  inner: Arc<RuntimeService>,
}

#[derive(Debug, derive_builder::Builder)]
#[builder(pattern = "owned", name = "RuntimeBuilder", setter(into), build_fn(skip))]
#[must_use]
#[allow(unreachable_pub)]
pub struct RuntimeInit {
  #[builder(default)]
  pub(crate) manifest: ComponentConfiguration,

  #[builder(default)]
  pub(crate) root_config: Option<RuntimeConfig>,

  #[builder(default)]
  pub(crate) allow_latest: bool,

  #[builder(default)]
  pub(crate) allowed_insecure: Vec<String>,

  #[builder(setter(strip_option))]
  pub(crate) namespace: Option<String>,

  #[builder(default)]
  pub(crate) constraints: Vec<RuntimeConstraint>,

  pub(crate) span: Span,

  #[builder(setter(custom = true))]
  pub(crate) native_components: ComponentRegistry,
}

impl Runtime {
  pub(crate) async fn new(seed: Seed, config: RuntimeInit) -> Result<Self> {
    let init = ServiceInit::new(seed, config);

    let service = RuntimeService::start(init)
      .await
      .map_err(|e| RuntimeError::InitializationFailed(e.to_string()))?;
    Ok(Self {
      uid: service.id,
      inner: service,
    })
  }

  pub async fn invoke(&self, invocation: Invocation, config: Option<RuntimeConfig>) -> Result<PacketStream> {
    let time = std::time::SystemTime::now();
    trace!(start_time=%time.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() ,"invocation start");

    let response = self.inner.invoke(invocation, config)?.await?;
    trace!(duration_ms=%time.elapsed().unwrap().as_millis(),"invocation complete");

    response.ok()
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

impl std::fmt::Debug for RuntimeBuilder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RuntimeBuilder")
      .field("allow_latest", &self.allow_latest)
      .field("allowed_insecure", &self.allowed_insecure)
      .field("manifest", &self.manifest)
      .field("namespace", &self.namespace)
      .field("native_components", &self.native_components)
      .finish()
  }
}

#[derive(Debug, Clone)]
pub enum RuntimeConstraint {
  Operation {
    entity: Entity,
    signature: OperationSignature,
  },
}

impl std::fmt::Display for RuntimeConstraint {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      RuntimeConstraint::Operation { entity, .. } => {
        write!(f, "Operation signature for {}", entity)
      }
    }
  }
}

impl RuntimeBuilder {
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Creates a new [RuntimeBuilder] from a [config::ComponentConfiguration]
  #[must_use]
  pub fn from_definition(definition: ComponentConfiguration) -> Self {
    let builder = Self::new();
    builder
      .allow_latest(definition.allow_latest())
      .allowed_insecure(definition.insecure_registries().map(|v| v.to_vec()).unwrap_or_default())
      .manifest(definition)
  }

  pub fn add_constraint(&mut self, constraint: RuntimeConstraint) -> &mut Self {
    let mut val = self.constraints.take().unwrap_or_default();
    val.push(constraint);
    self.constraints.replace(val);
    self
  }

  pub fn add_import(&mut self, component: config::ImportBinding) -> &mut Self {
    let val = self.manifest.take().unwrap_or_default();
    let mut val = ComponentConfigurationBuilder::from_base(val);
    val.add_import(component);
    self.manifest.replace(val.build().unwrap());
    self
  }

  pub fn add_resource(&mut self, resource: config::ResourceBinding) -> &mut Self {
    let val = self.manifest.take().unwrap_or_default();
    let mut val = ComponentConfigurationBuilder::from_base(val);
    val.add_resource(resource);
    self.manifest.replace(val.build().unwrap());
    self
  }

  pub fn add_native_component(&mut self, factory: Box<ComponentFactory>) -> &mut Self {
    let mut val = self.native_components.take().unwrap_or_default();
    val.add(factory);
    self.native_components.replace(val);
    self
  }

  /// Constructs an instance of a Wick [Runtime].
  pub async fn build(self, seed: Option<Seed>) -> Result<Runtime> {
    let from_span = self.span.unwrap_or_else(tracing::Span::current);
    let span = debug_span!("runtime");
    span.follows_from(from_span);
    let definition = self.manifest.unwrap_or_default();
    Runtime::new(
      seed.unwrap_or_else(new_seed),
      RuntimeInit {
        manifest: definition,
        root_config: self.root_config.unwrap_or_default(),
        allow_latest: self.allow_latest.unwrap_or_default(),
        allowed_insecure: self.allowed_insecure.unwrap_or_default(),
        native_components: self.native_components.unwrap_or_default(),
        namespace: self.namespace.unwrap_or_default(),
        constraints: self.constraints.unwrap_or_default(),
        span,
      },
    )
    .await
  }
}
