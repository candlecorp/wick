pub(crate) mod error;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use flow_graph_interpreter::{HandlerMap, InterpreterOptions, NamespaceHandler};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use seeded_random::{Random, Seed};
use tracing::Instrument;
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
  pub(crate) rng_seed: Seed,
  pub(crate) namespace: Option<String>,
  pub(crate) event_log: Option<PathBuf>,
  pub(crate) span: tracing::Span,
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
  pub(crate) async fn new(msg: Initialize) -> Result<Arc<Self>> {
    debug!("initializing engine service");
    let graph = flow_graph_interpreter::graph::from_def(&msg.manifest)?;
    let mut components = HandlerMap::default();
    let rng = Random::from_seed(msg.rng_seed);
    let ns = msg.namespace.unwrap_or_else(|| msg.id.to_string());

    if let ComponentImplementation::Wasm(comp) = msg.manifest.component() {
      let span = debug_span!(parent: &msg.span, "main:init");
      let collection_init = ComponentInitOptions {
        rng_seed: rng.seed(),
        runtime_id: msg.id,
        allow_latest: msg.allow_latest,
        allowed_insecure: msg.allowed_insecure.clone(),
        timeout: msg.timeout,
        resolver: Some(msg.manifest.resolver()),
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
      let main_component = initialize_component(&component, collection_init).await?;
      let signed_sig = main_component.component().list();
      let manifest_sig = msg.manifest.signature();

      expect_signature_match(
        comp.reference().location(),
        signed_sig,
        msg.manifest.source().clone().unwrap_or_else(|| "<Unknown>".to_owned()),
        &manifest_sig,
      )?;
      main_component.expose();

      components
        .add(main_component)
        .map_err(|e| EngineError::InterpreterInit(msg.manifest.source().clone(), Box::new(e)))?;
    } else if let ComponentImplementation::Composite(comp) = msg.manifest.component() {
      let span = debug_span!(parent: &msg.span, "composite:init");

      let stdlib = initialize_native_component(V0_NAMESPACE.to_owned(), rng.seed(), &span)?;
      components
        .add(stdlib)
        .map_err(|e| EngineError::InterpreterInit(msg.manifest.source().clone(), Box::new(e)))?;

      for component in comp.components().values() {
        let collection_init = ComponentInitOptions {
          rng_seed: rng.seed(),
          runtime_id: msg.id,
          allow_latest: msg.allow_latest,
          allowed_insecure: msg.allowed_insecure.clone(),
          resolver: Some(msg.manifest.resolver()),
          timeout: msg.timeout,
          span: &span,
        };
        let p = initialize_component(component, collection_init).await?;
        components
          .add(p)
          .map_err(|e| EngineError::InterpreterInit(msg.manifest.source().clone(), Box::new(e)))?;
      }
    }

    let source = msg.manifest.source().clone();
    let callback = make_link_callback(msg.id);

    let mut interpreter =
      flow_graph_interpreter::Interpreter::new(Some(rng.seed()), graph, Some(ns.clone()), Some(components), callback)
        .map_err(|e| EngineError::InterpreterInit(source, Box::new(e)))?;

    let options = InterpreterOptions {
      output_timeout: msg.timeout,
      ..Default::default()
    };
    match msg.event_log {
      Some(path) => {
        interpreter
          .start(Some(options), Some(Box::new(JsonWriter::new(path))))
          .await;
      }
      None => interpreter.start(Some(options), None).await,
    }

    let engine = Arc::new(RuntimeService {
      started_time: std::time::Instant::now(),
      id: msg.id,
      namespace: ns,
      interpreter: Arc::new(interpreter),
    });

    let mut registry = HOST_REGISTRY.lock();
    registry.insert(msg.id, engine.clone());

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
  pub(crate) span: &'a tracing::Span,
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
) -> Result<NamespaceHandler> {
  debug!(?collection, ?opts, "initializing component");
  let id = collection.id.clone();
  let span = opts.span.clone();
  match &collection.kind {
    #[allow(deprecated)]
    config::ComponentDefinition::Wasm(def) => {
      init_wasm_component(def, id, opts, Default::default())
        .instrument(span)
        .await
    }
    config::ComponentDefinition::Manifest(def) => init_manifest_component(def, id, opts).instrument(span).await,
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
        Ok(NamespaceHandler::new(id, Box::new(comp)))
      }
    },
    config::ComponentDefinition::Native(_) => unreachable!(),
  }
}

#[cfg(test)]
mod test {
  // You can find many of the engine tests in the integration tests
}
