pub(crate) mod error;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use flow_graph_interpreter::{HandlerMap, InterpreterOptions, NamespaceHandler};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use seeded_random::{Random, Seed};
use tracing::Span;
use uuid::Uuid;
use wick_config::config::{ComponentConfiguration, ComponentKind, Metadata};
use wick_config::Resolver;
use wick_packet::GenericConfig;

use self::error::ConstraintFailure;
use crate::components::validation::expect_signature_match;
use crate::components::{
  init_hlc_component,
  init_manifest_component,
  init_wasm_component,
  initialize_native_component,
  make_link_callback,
};
use crate::dev::prelude::*;
use crate::runtime::{RuntimeConstraint, RuntimeInit};
use crate::{BoxFuture, V0_NAMESPACE};

type Result<T> = std::result::Result<T, EngineError>;
#[derive(Debug)]
pub(crate) struct Initialize {
  pub(crate) id: Uuid,
  pub(crate) config: RuntimeInit,
  rng: Random,
}

impl Initialize {
  pub(crate) fn new(seed: Seed, config: RuntimeInit) -> Self {
    let rng = Random::from_seed(seed);
    Self {
      id: rng.uuid(),
      rng,
      config,
    }
  }

  pub(crate) fn new_with_id(id: Uuid, seed: Seed, config: RuntimeInit) -> Self {
    let rng = Random::from_seed(seed);
    Self { id, config, rng }
  }

  fn component_init(&self, config: Option<GenericConfig>) -> ComponentInitOptions {
    ComponentInitOptions {
      rng_seed: self.rng.seed(),
      runtime_id: self.id,
      config,
      allow_latest: self.config.allow_latest,
      allowed_insecure: self.config.allowed_insecure.clone(),
      timeout: self.config.timeout,
      resolver: self.config.manifest.resolver(),
      span: self.config.span.clone(),
    }
  }

  fn seed(&self) -> Seed {
    self.rng.seed()
  }
}

#[must_use]
pub type ComponentFactory = dyn Fn(Seed) -> Result<NamespaceHandler> + Send + Sync;

#[derive()]
pub(crate) struct ComponentRegistry(Vec<Box<ComponentFactory>>);

impl ComponentRegistry {
  /// Add a component to the registry
  pub(crate) fn add(&mut self, factory: Box<ComponentFactory>) {
    self.0.push(factory);
  }

  /// Get the list of components
  #[must_use]
  pub(crate) fn inner(&self) -> &[Box<ComponentFactory>] {
    &self.0
  }
}

impl std::fmt::Debug for ComponentRegistry {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("ComponentRegistry").field(&self.0.len()).finish()
  }
}

impl Default for ComponentRegistry {
  fn default() -> Self {
    let mut list: Vec<Box<ComponentFactory>> = Vec::default();
    list.push(Box::new(|seed: Seed| {
      initialize_native_component(V0_NAMESPACE.to_owned(), seed)
    }));
    Self(list)
  }
}

#[derive(Debug)]
pub(crate) struct RuntimeService {
  #[allow(unused)]
  started_time: std::time::Instant,
  pub(crate) id: Uuid,
  pub(super) namespace: String,
  interpreter: Arc<flow_graph_interpreter::Interpreter>,
}

type ServiceMap = HashMap<Uuid, Arc<RuntimeService>>;
static HOST_REGISTRY: Lazy<Mutex<ServiceMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

impl RuntimeService {
  pub(crate) async fn new(mut init: Initialize) -> Result<Arc<Self>> {
    let span = init.config.span.clone();
    let graph = init.config.span.in_scope(|| {
      debug!("initializing engine service");
      flow_graph_interpreter::graph::from_def(&mut init.config.manifest)
    })?;

    let config = &init.config;

    let mut components = HandlerMap::default();
    let ns = config.namespace.clone().unwrap_or_else(|| init.id.to_string());

    if config.manifest.component().kind() == ComponentKind::Composite {
      for native_comp in config.native_components.inner() {
        components
          .add(native_comp(init.seed())?)
          .map_err(|e| EngineError::InterpreterInit(config.manifest.source().map(Into::into), Box::new(e)))?;
      }
    } else {
      let component_init = init.component_init(init.config.config.clone());
      let binding = config::ImportBinding::new(&ns, config.manifest.component().clone().into());

      if let Some(main_component) = instantiate_import(&binding, component_init).await? {
        let reported_sig = main_component.component().signature();
        let manifest_sig = config.manifest.signature()?;
        span.in_scope(|| debug!("manifest_sig: {:?}", config.manifest));

        expect_signature_match(
          config.manifest.source(),
          reported_sig,
          config.manifest.source(),
          &manifest_sig,
        )?;
        main_component.expose();

        span.in_scope(|| debug!("Adding main component: {}", main_component.namespace()));
        components
          .add(main_component)
          .map_err(|e| EngineError::InterpreterInit(config.manifest.source().map(Into::into), Box::new(e)))?;
      }
    }

    for component in config.manifest.import().values() {
      let component_init = init.component_init(component.config().cloned());
      if let Some(p) = instantiate_import(component, component_init).await? {
        components
          .add(p)
          .map_err(|e| EngineError::InterpreterInit(config.manifest.source().map(Into::into), Box::new(e)))?;
      }
    }

    let source = config.manifest.source();
    let callback = make_link_callback(init.id);
    assert_constraints(&config.constraints, &components)?;

    let mut interpreter = flow_graph_interpreter::Interpreter::new(
      Some(init.seed()),
      graph,
      Some(ns.clone()),
      Some(components),
      callback,
      &span,
    )
    .map_err(|e| EngineError::InterpreterInit(source.map(Into::into), Box::new(e)))?;

    let options = InterpreterOptions {
      output_timeout: config.timeout,
      ..Default::default()
    };
    interpreter.start(Some(options), None).await;

    let engine = Arc::new(RuntimeService {
      started_time: std::time::Instant::now(),
      id: init.id,
      namespace: ns,
      interpreter: Arc::new(interpreter),
    });

    let mut registry = HOST_REGISTRY.lock();
    registry.insert(init.id, engine.clone());

    Ok(engine)
  }

  pub(crate) fn init_child(
    uid: Uuid,
    manifest: ComponentConfiguration,
    namespace: Option<String>,
    opts: ComponentInitOptions,
  ) -> BoxFuture<'static, Result<Arc<RuntimeService>>> {
    let child_span = debug_span!("child",id=%uid);
    child_span.follows_from(opts.span);
    let config = RuntimeInit {
      manifest,
      config: opts.config,
      allow_latest: opts.allow_latest,
      allowed_insecure: opts.allowed_insecure,
      timeout: opts.timeout,
      namespace,
      constraints: Default::default(),
      span: child_span,
      native_components: ComponentRegistry::default(),
    };
    let init = Initialize::new_with_id(uid, opts.rng_seed, config);

    Box::pin(async move { RuntimeService::new(init).await })
  }

  pub(crate) fn for_id(id: &Uuid) -> Option<Arc<Self>> {
    let registry = HOST_REGISTRY.lock();
    registry.get(id).cloned()
  }

  pub(crate) async fn shutdown(&self) -> std::result::Result<(), RuntimeError> {
    let _ = self.interpreter.shutdown().await;
    Ok(())
  }
}

fn assert_constraints(constraints: &[RuntimeConstraint], components: &HandlerMap) -> Result<()> {
  for constraint in constraints {
    #[allow(irrefutable_let_patterns)]
    if let RuntimeConstraint::Operation { entity, signature } = constraint {
      let handler = components
        .get(entity.component_id())
        .ok_or_else(|| EngineError::InvalidConstraint(ConstraintFailure::ComponentNotFound(entity.clone())))?;
      let sig = handler.component().signature();
      let op = sig
        .get_operation(entity.operation_id())
        .ok_or_else(|| EngineError::InvalidConstraint(ConstraintFailure::OperationNotFound(entity.clone())))?;
      for field in &signature.inputs {
        op.inputs
          .iter()
          .find(|sig_field| sig_field.name == field.name)
          .ok_or_else(|| {
            EngineError::InvalidConstraint(ConstraintFailure::InputNotFound(entity.clone(), field.name.clone()))
          })?;
      }
      for field in &signature.outputs {
        op.outputs
          .iter()
          .find(|sig_field| sig_field.name == field.name)
          .ok_or_else(|| {
            EngineError::InvalidConstraint(ConstraintFailure::OutputNotFound(entity.clone(), field.name.clone()))
          })?;
      }
    }
  }
  Ok(())
}

impl InvocationHandler for RuntimeService {
  fn get_signature(&self) -> std::result::Result<ComponentSignature, ComponentError> {
    let mut signature = self.interpreter.get_export_signature().clone();
    signature.name = Some(self.id.as_hyphenated().to_string());

    Ok(signature)
  }

  fn invoke(
    &self,
    invocation: Invocation,
    config: Option<GenericConfig>,
  ) -> std::result::Result<BoxFuture<std::result::Result<InvocationResponse, ComponentError>>, ComponentError> {
    let tx_id = invocation.tx_id;

    let fut = self.interpreter.invoke(invocation, config);
    let task = async move {
      match fut.await {
        Ok(response) => Ok(InvocationResponse::Stream { tx_id, rx: response }),
        Err(e) => {
          error!("{}", e);
          Ok(InvocationResponse::error(
            tx_id,
            format!("Internal error invoking schematic: {}", e),
          ))
        }
      }
    };
    Ok(Box::pin(task))
  }
}

#[derive()]
pub(crate) struct ComponentInitOptions {
  pub(crate) rng_seed: Seed,
  pub(crate) runtime_id: Uuid,
  pub(crate) allow_latest: bool,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) timeout: Duration,
  pub(crate) config: Option<GenericConfig>,
  pub(crate) resolver: Box<Resolver>,
  #[allow(unused)]
  pub(crate) span: Span,
}

impl std::fmt::Debug for ComponentInitOptions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ComponentInitOptions")
      .field("rng_seed", &self.rng_seed)
      .field("runtime_id", &self.runtime_id)
      .field("allow_latest", &self.allow_latest)
      .field("allowed_insecure", &self.allowed_insecure)
      .field("timeout", &self.timeout)
      .finish()
  }
}

pub(crate) async fn instantiate_import(
  binding: &config::ImportBinding,
  opts: ComponentInitOptions,
) -> Result<Option<NamespaceHandler>> {
  opts.span.in_scope(|| debug!(?binding, ?opts, "initializing component"));
  let id = binding.id().to_owned();
  match binding.kind() {
    config::ImportDefinition::Component(c) => {
      match c {
        #[allow(deprecated)]
        config::ComponentDefinition::Wasm(def) => Ok(Some(
          init_wasm_component(
            def.reference(),
            Some(def.permissions().clone()),
            id,
            opts,
            Default::default(),
          )
          .await?,
        )),
        config::ComponentDefinition::Manifest(def) => Ok(Some(init_manifest_component(def, id, opts).await?)),
        config::ComponentDefinition::Reference(_) => unreachable!(),
        config::ComponentDefinition::GrpcUrl(_) => todo!(), // CollectionKind::GrpcUrl(v) => initialize_grpc_collection(v, namespace).await,
        config::ComponentDefinition::HighLevelComponent(hlc) => {
          init_hlc_component(id, Metadata::default(), hlc.clone(), opts.resolver)
            .await
            .map(Some)
        }
        config::ComponentDefinition::Native(_) => Ok(None),
      }
    }
    config::ImportDefinition::Types(_) => Ok(None),
  }
}

#[cfg(test)]
mod test {
  // You can find many of the engine tests in the integration tests

  use anyhow::Result;
  use wick_packet::Entity;

  use super::*;

  struct TestComponent {
    signature: ComponentSignature,
  }

  impl TestComponent {
    fn new() -> Self {
      Self {
        signature: component! {
          name: "test",
          version: "0.0.1",
          operations: {
            "testop" => {
              inputs: {
                "in" => "object",
              },
              outputs: {
                "out" => "object",
              },
            },
          }
        },
      }
    }
  }

  impl flow_component::Component for TestComponent {
    fn handle(
      &self,
      _invocation: Invocation,
      _data: Option<GenericConfig>,
      _callback: Arc<flow_component::RuntimeCallback>,
    ) -> flow_component::BoxFuture<std::result::Result<PacketStream, flow_component::ComponentError>> {
      todo!()
    }

    fn signature(&self) -> &ComponentSignature {
      &self.signature
    }
  }

  #[test]
  fn test_constraints() -> Result<()> {
    let mut components = HandlerMap::default();

    components.add(NamespaceHandler::new("test", Box::new(TestComponent::new())))?;

    let constraints = vec![RuntimeConstraint::Operation {
      entity: Entity::operation("test", "testop"),
      signature: operation!(
        "testop" => {
          inputs: {
            "in" => "object",
          },
          outputs: {
            "out" => "object",
          },
        }
      ),
    }];

    assert_constraints(&constraints, &components)?;

    let constraints = vec![RuntimeConstraint::Operation {
      entity: Entity::operation("test", "testop"),
      signature: operation!(
        "testop" => {
          inputs: {
            "otherin" => "object",
          },
          outputs: {
            "otherout" => "object",
          },
        }
      ),
    }];

    let result = assert_constraints(&constraints, &components);

    assert!(result.is_err());

    Ok(())
  }
}
