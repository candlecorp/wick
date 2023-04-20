pub(crate) mod error;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use flow_graph_interpreter::{HandlerMap, InterpreterOptions, NamespaceHandler};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use seeded_random::{Random, Seed};
use tracing::{Instrument, Span};
use uuid::Uuid;
use wick_config::config::{ComponentConfiguration, ComponentImplementation};
use wick_config::{HighLevelComponent, Resolver};

use crate::components::{
  expect_signature_match,
  init_manifest_component,
  init_wasm_component,
  initialize_native_component,
  make_link_callback,
};
use crate::dev::prelude::*;
use crate::json_writer::JsonWriter;
use crate::runtime_service::error::InternalError;
use crate::{BoxFuture, V0_NAMESPACE};

type Result<T> = std::result::Result<T, EngineError>;
#[derive(Debug)]
pub(crate) struct Initialize {
  pub(crate) id: Uuid,
  pub(crate) manifest: ComponentConfiguration,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) allow_latest: bool,
  pub(crate) timeout: Duration,
  pub(crate) native_components: ComponentRegistry,
  pub(crate) rng_seed: Seed,
  pub(crate) namespace: Option<String>,
  pub(crate) event_log: Option<PathBuf>,
  pub(crate) span: Span,
}

#[must_use]
pub type ComponentFactory = dyn Fn(Seed) -> Result<NamespaceHandler> + Send;

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
  pub(crate) async fn new(init: Initialize) -> Result<Arc<Self>> {
    debug!("initializing engine service");
    let graph = flow_graph_interpreter::graph::from_def(&init.manifest)?;
    let mut components = HandlerMap::default();
    let rng = Random::from_seed(init.rng_seed);
    let ns = init.namespace.unwrap_or_else(|| init.id.to_string());

    if let ComponentImplementation::Wasm(comp) = init.manifest.component() {
      let span = debug_span!(parent: &init.span, "main:init");
      let collection_init = ComponentInitOptions {
        rng_seed: rng.seed(),
        runtime_id: init.id,
        allow_latest: init.allow_latest,
        allowed_insecure: init.allowed_insecure.clone(),
        timeout: init.timeout,
        resolver: Some(init.manifest.resolver()),
        span: &span,
      };

      let component = config::BoundComponent::new(
        &ns,
        #[allow(deprecated)]
        config::ComponentDefinition::Wasm(config::components::WasmComponent {
          reference: comp.reference().clone(),
          config: Default::default(),
          permissions: Default::default(),
          provide: Default::default(),
        }),
      );
      if let Some(main_component) = initialize_component(&component, collection_init).await? {
        let signed_sig = main_component.component().list();
        let manifest_sig = init.manifest.signature();

        expect_signature_match(
          comp.reference().location(),
          signed_sig,
          init.manifest.source().clone().unwrap_or_else(|| "<Unknown>".to_owned()),
          &manifest_sig,
        )?;
        main_component.expose();

        components
          .add(main_component)
          .map_err(|e| EngineError::InterpreterInit(init.manifest.source().clone(), Box::new(e)))?;
      }
    } else if let ComponentImplementation::Composite(comp) = init.manifest.component() {
      let span = debug_span!(parent: &init.span, "composite:init");

      for native_comp in init.native_components.inner() {
        components
          .add(native_comp(rng.seed())?)
          .map_err(|e| EngineError::InterpreterInit(init.manifest.source().clone(), Box::new(e)))?;
      }

      for component in comp.components().values() {
        let collection_init = ComponentInitOptions {
          rng_seed: rng.seed(),
          runtime_id: init.id,
          allow_latest: init.allow_latest,
          allowed_insecure: init.allowed_insecure.clone(),
          resolver: Some(init.manifest.resolver()),
          timeout: init.timeout,
          span: &span,
        };
        if let Some(p) = initialize_component(component, collection_init).await? {
          components
            .add(p)
            .map_err(|e| EngineError::InterpreterInit(init.manifest.source().clone(), Box::new(e)))?;
        }
      }
    }

    let source = init.manifest.source().clone();
    let callback = make_link_callback(init.id);

    let mut interpreter =
      flow_graph_interpreter::Interpreter::new(Some(rng.seed()), graph, Some(ns.clone()), Some(components), callback)
        .map_err(|e| EngineError::InterpreterInit(source, Box::new(e)))?;

    let options = InterpreterOptions {
      output_timeout: init.timeout,
      ..Default::default()
    };
    match init.event_log {
      Some(path) => {
        interpreter
          .start(Some(options), Some(Box::new(JsonWriter::new(path))))
          .await;
      }
      None => interpreter.start(Some(options), None).await,
    }

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

  pub(crate) fn new_from_manifest<'a, 'b>(
    uid: Uuid,
    manifest: ComponentConfiguration,
    namespace: Option<String>,
    opts: ComponentInitOptions<'b>,
  ) -> BoxFuture<'b, Result<Arc<RuntimeService>>>
  where
    'a: 'b,
  {
    Box::pin(async move {
      let init = Initialize {
        id: uid,
        manifest,
        allowed_insecure: opts.allowed_insecure,
        allow_latest: opts.allow_latest,
        timeout: opts.timeout,
        rng_seed: opts.rng_seed,
        native_components: ComponentRegistry::default(),
        namespace,
        event_log: None,
        span: debug_span!("engine:new"),
      };
      RuntimeService::new(init).await
    })
  }

  pub(crate) fn for_id(id: &Uuid) -> Option<Arc<Self>> {
    trace!(%id, "get engine service");
    let registry = HOST_REGISTRY.lock();
    registry.get(id).cloned()
  }

  pub(crate) async fn shutdown(&self) -> std::result::Result<(), RuntimeError> {
    let _ = self.interpreter.shutdown().await;
    Ok(())
  }
}

impl InvocationHandler for RuntimeService {
  fn get_signature(&self) -> std::result::Result<ComponentSignature, ComponentError> {
    let mut signature = self.interpreter.get_export_signature().clone();
    signature.name = Some(self.id.as_hyphenated().to_string());

    Ok(signature)
  }

  fn invoke(
    &self,
    msg: Invocation,
    stream: PacketStream,
  ) -> std::result::Result<BoxFuture<std::result::Result<InvocationResponse, ComponentError>>, ComponentError> {
    let tx_id = msg.tx_id;

    let fut = self.interpreter.invoke(msg, stream);
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
pub(crate) struct ComponentInitOptions<'a> {
  pub(crate) rng_seed: Seed,
  pub(crate) runtime_id: Uuid,
  pub(crate) allow_latest: bool,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) timeout: Duration,
  pub(crate) resolver: Option<Box<Resolver>>,
  #[allow(unused)]
  pub(crate) span: &'a Span,
}

impl<'a> std::fmt::Debug for ComponentInitOptions<'a> {
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

pub(crate) async fn initialize_component<'a, 'b>(
  collection: &'a config::BoundComponent,
  opts: ComponentInitOptions<'b>,
) -> Result<Option<NamespaceHandler>> {
  debug!(?collection, ?opts, "initializing component");
  let id = collection.id.clone();
  let span = opts.span.clone();
  match &collection.kind {
    #[allow(deprecated)]
    config::ComponentDefinition::Wasm(def) => Ok(Some(
      init_wasm_component(def, id, opts, Default::default())
        .instrument(span)
        .await?,
    )),
    config::ComponentDefinition::Manifest(def) => {
      Ok(Some(init_manifest_component(def, id, opts).instrument(span).await?))
    }
    config::ComponentDefinition::Reference(_) => unreachable!(),
    config::ComponentDefinition::GrpcUrl(_) => todo!(), // CollectionKind::GrpcUrl(v) => initialize_grpc_collection(v, namespace).await,
    config::ComponentDefinition::HighLevelComponent(hlc) => match hlc {
      config::HighLevelComponent::Postgres(config) => {
        if opts.resolver.is_none() {
          return Err(EngineError::InternalError(InternalError::MissingResolver));
        }
        let resolver = opts.resolver.unwrap();
        let comp = wick_sqlx::SqlXComponent::default();
        comp.validate(config, &resolver)?;
        comp
          .init(config.clone(), resolver)
          .await
          .map_err(EngineError::NativeComponent)?;
        Ok(Some(NamespaceHandler::new(id, Box::new(comp))))
      }
    },
    config::ComponentDefinition::Native(_) => Ok(None),
  }
}

#[cfg(test)]
mod test {
  // You can find many of the engine tests in the integration tests
}
