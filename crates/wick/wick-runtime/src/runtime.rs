use seeded_random::Seed;
use tracing::Span;
use uuid::Uuid;
use wick_config::config::{ComponentConfiguration, ComponentConfigurationBuilder};
use wick_packet::{Entity, RuntimeConfig};
pub(crate) mod scope;

use scope::{ComponentFactory, ComponentRegistry, ScopeInit};

use crate::dev::prelude::*;

type Result<T> = std::result::Result<T, RuntimeError>;
#[derive(Debug, Clone)]
#[must_use]
pub struct Runtime {
  pub uid: Uuid,
  root: Scope,
}

#[derive(Debug, derive_builder::Builder)]
#[builder(pattern = "owned", name = "RuntimeBuilder", setter(into), build_fn(skip))]
#[must_use]
#[allow(unreachable_pub)]
pub struct RuntimeInit {
  #[builder(default)]
  pub(crate) manifest: ComponentConfiguration,

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
  pub(crate) initial_components: ComponentRegistry,
}

impl Runtime {
  pub(crate) async fn new(seed: Seed, config: RuntimeInit) -> Result<Self> {
    let init = ScopeInit::new(seed, config);

    let ns = init.namespace.as_deref().unwrap_or("__local__").to_owned();
    init.span.in_scope(|| {
      info!(id = ns, "initializing");
    });
    let span = init.span.clone();

    let start = std::time::Instant::now();
    let service = Scope::start(init)
      .await
      .map_err(|e| RuntimeError::InitializationFailed(e.to_string()))?;
    let end = std::time::Instant::now();

    span.in_scope(|| info!(id = ns, duration_ms = %end.duration_since(start).as_millis(), "initialized"));

    Ok(Self {
      uid: service.id(),
      root: service,
    })
  }

  pub async fn invoke(&self, invocation: Invocation, config: Option<RuntimeConfig>) -> Result<PacketStream> {
    let time = std::time::SystemTime::now();
    trace!(start_time=%time.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() ,"invocation start");

    let response = self.root.invoke(invocation, config)?.await?;
    trace!(duration_ms=%time.elapsed().unwrap().as_millis(),"invocation complete");

    response.ok()
  }

  fn get_scope(&self, path: Option<&[&str]>) -> Option<Scope> {
    if path.is_none() {
      return Some(self.root.clone());
    }
    let path = path.unwrap();
    let mut last_scope = self.root.clone();
    // if our first path hop is Entity::LOCAL, skip it.
    let path = if Some(&Entity::LOCAL) == path.first() {
      &path[1..]
    } else {
      path
    };
    for path in path {
      if let Some(scope) = Scope::find(Some(last_scope.id()), path) {
        last_scope = scope.clone();
      } else {
        return None;
      }
    }
    Some(last_scope)
  }

  /// `invoke_deep()` but will traverse the known scopes – starting from this [Runtime]'s root – until it
  /// finds the target scope and then invokes the operation from there.
  ///
  /// `invoke`, conversely, will only invoke operations from the context of the root scope.
  pub async fn invoke_deep(
    &self,
    path: Option<&[&str]>,
    invocation: Invocation,
    config: Option<RuntimeConfig>,
  ) -> Result<PacketStream> {
    if let Some(scope) = self.get_scope(path) {
      scope.invoke(invocation, config)?.await?.ok()
    } else {
      Err(RuntimeError::ScopeNotFound(
        path.map(|p| p.iter().copied().map(Into::into).collect()),
        Some(invocation.target),
      ))
    }
  }

  pub fn deep_signature(&self, path: Option<&[&str]>, entity: Option<&Entity>) -> Result<ComponentSignature> {
    self
      .get_scope(path)
      .and_then(|s| {
        entity.map_or_else(
          || s.get_signature().ok(),
          |entity| s.get_handler_signature(entity.component_id()).cloned(),
        )
      })
      .ok_or_else(|| {
        RuntimeError::ScopeNotFound(
          path.map(|p| p.iter().copied().map(Into::into).collect()),
          entity.cloned(),
        )
      })
  }

  pub async fn shutdown(&self) -> Result<()> {
    trace!("runtime scope shutting down");
    self.root.shutdown().await?;

    Ok(())
  }

  pub fn get_signature(&self) -> Result<ComponentSignature> {
    let signature = self.root.get_signature()?;
    trace!(?signature, "runtime scope instance signature");
    Ok(signature)
  }

  #[must_use]
  pub fn namespace(&self) -> &str {
    self.root.namespace()
  }

  pub fn render_dotviz(&self, op: &str) -> Result<String> {
    self.root.render_dotviz(op)
  }

  pub fn active_config(&self) -> &ComponentConfiguration {
    self.root.active_config()
  }
}

impl std::fmt::Debug for RuntimeBuilder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RuntimeBuilder")
      .field("allow_latest", &self.allow_latest)
      .field("allowed_insecure", &self.allowed_insecure)
      .field("manifest", &self.manifest)
      .field("namespace", &self.namespace)
      .field("initial_components", &self.initial_components)
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
    let mut val = self.initial_components.take().unwrap_or_default();
    val.add(factory);
    self.initial_components.replace(val);
    self
  }

  /// Constructs an instance of a Wick [Runtime].
  pub async fn build(self, seed: Option<Seed>) -> Result<Runtime> {
    let span = self.span.unwrap_or_else(tracing::Span::current);

    let definition = self.manifest.ok_or(RuntimeError::MissingComponentDefinition)?;
    Runtime::new(
      seed.unwrap_or_else(new_seed),
      RuntimeInit {
        manifest: definition,
        allow_latest: self.allow_latest.unwrap_or_default(),
        allowed_insecure: self.allowed_insecure.unwrap_or_default(),
        initial_components: self.initial_components.unwrap_or_default(),
        namespace: self.namespace.unwrap_or_default(),
        constraints: self.constraints.unwrap_or_default(),
        span,
      },
    )
    .await
  }
}
