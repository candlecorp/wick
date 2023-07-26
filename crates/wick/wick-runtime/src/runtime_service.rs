mod child_init;
mod component_registry;
pub(crate) mod error;
mod init;
mod utils;

use std::path::PathBuf;

pub(crate) use child_init::{init_child, ChildInit};
pub(crate) use component_registry::{ComponentFactory, ComponentRegistry};
use flow_graph_interpreter::{HandlerMap, InterpreterOptions};
pub(crate) use init::ServiceInit;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use utils::{assert_constraints, instantiate_import};
use uuid::Uuid;
use wick_config::config::ComponentImplementation;

use crate::components::validation::expect_signature_match;
use crate::components::{init_component_implementation, make_link_callback};
use crate::dev::prelude::*;

type ServiceMap = HashMap<Uuid, Arc<RuntimeService>>;
type Result<T> = std::result::Result<T, EngineError>;

static HOST_REGISTRY: Lazy<Mutex<ServiceMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[must_use]
#[derive(Debug)]
pub(crate) struct RuntimeService {
  #[allow(unused)]
  started_time: std::time::Instant,
  pub(crate) id: Uuid,
  pub(super) namespace: String,
  pub(super) active_config: ComponentConfiguration,
  interpreter: Arc<flow_graph_interpreter::Interpreter>,
}

impl RuntimeService {
  pub(crate) async fn start(mut init: ServiceInit) -> Result<Arc<Self>> {
    let span = init.span.clone();
    let manifest_source: Option<PathBuf> = init.manifest.source().map(Into::into);
    let ns = init.namespace.clone().unwrap_or_else(|| init.id.to_string());
    let mut components = HandlerMap::default();

    let extends = if let ComponentImplementation::Composite(config) = init.manifest.component() {
      // Initialize any native components for composite components.
      for native_comp in init.native_components.inner() {
        components
          .add(native_comp(init.seed())?)
          .map_err(|e| EngineError::InterpreterInit(manifest_source.clone(), Box::new(e)))?;
      }

      config.extends()
    } else {
      // Instantiate the non-composite component as an exposed, standalone component.

      let component_init = init.child_init(init.manifest.root_config().cloned());

      span.in_scope(|| debug!(%ns,options=?component_init,"instantiating component"));

      let main_component =
        init_component_implementation(&init.manifest, ns.clone(), component_init, Default::default()).await?;
      let reported_sig = main_component.component().signature();
      let manifest_sig = init.manifest.signature()?;

      expect_signature_match(
        init.manifest.source(),
        reported_sig,
        init.manifest.source(),
        &manifest_sig,
      )?;
      main_component.expose();

      components
        .add(main_component)
        .map_err(|e| EngineError::InterpreterInit(manifest_source.clone(), Box::new(e)))?;

      &[]
    };

    for id in extends {
      if !init.manifest.import().keys().any(|k| k == id) {
        return Err(EngineError::RuntimeInit(
          manifest_source.clone(),
          format!("Inherited component '{}' not found", id),
        ));
      }
    }

    for binding in init.manifest.import().values() {
      let component_init = init.child_init(binding.config().cloned());
      if let Some(component) = instantiate_import(binding, component_init).await? {
        if extends.iter().any(|n| n == component.namespace()) {
          component.expose();
          span.in_scope(|| {
            debug!(component = component.namespace(), "extending imported component");
          });
        }
        components
          .add(component)
          .map_err(|e| EngineError::InterpreterInit(manifest_source.clone(), Box::new(e)))?;
      }
    }

    let callback = make_link_callback(init.id);
    assert_constraints(&init.constraints, &components)?;

    let graph = init.span.in_scope(|| {
      debug!("generating graph");

      flow_graph_interpreter::graph::from_def(&mut init.manifest, &components)
        .map_err(|e| EngineError::Graph(manifest_source.clone(), Box::new(e)))
    })?;

    let mut interpreter = flow_graph_interpreter::Interpreter::new(
      graph,
      Some(ns.clone()),
      None, /* TODO: FIX passing config */
      Some(components),
      callback,
      &span,
    )
    .map_err(|e| EngineError::InterpreterInit(manifest_source.clone(), Box::new(e)))?;

    let options = InterpreterOptions { ..Default::default() };
    interpreter.start(Some(options), None).await;

    let engine = Arc::new(RuntimeService {
      started_time: std::time::Instant::now(),
      id: init.id,
      namespace: ns,
      active_config: init.manifest,
      interpreter: Arc::new(interpreter),
    });

    let mut registry = HOST_REGISTRY.lock();
    registry.insert(init.id, engine.clone());

    Ok(engine)
  }

  pub(crate) fn for_id(id: &Uuid) -> Option<Arc<Self>> {
    let registry = HOST_REGISTRY.lock();
    registry.get(id).cloned()
  }

  pub(crate) async fn shutdown(&self) -> std::result::Result<(), RuntimeError> {
    let _ = self.interpreter.shutdown().await;
    Ok(())
  }

  pub(crate) fn render_dotviz(&self, op: &str) -> std::result::Result<String, RuntimeError> {
    self.interpreter.render_dotviz(op).map_err(RuntimeError::DotViz)
  }

  pub(crate) fn active_config(&self) -> &ComponentConfiguration {
    &self.active_config
  }
}

impl InvocationHandler for RuntimeService {
  fn get_signature(&self) -> std::result::Result<ComponentSignature, ComponentError> {
    let mut signature = self.interpreter.signature().clone();
    signature.name = Some(self.id.as_hyphenated().to_string());

    Ok(signature)
  }

  fn invoke(
    &self,
    invocation: Invocation,
    config: Option<RuntimeConfig>,
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
