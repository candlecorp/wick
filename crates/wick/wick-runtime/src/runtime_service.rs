mod child_init;
mod component_registry;
pub(crate) mod error;
mod init;
mod utils;

pub(crate) use child_init::{init_child, ChildInit};
pub(crate) use component_registry::{ComponentFactory, ComponentRegistry};
use flow_graph_interpreter::{HandlerMap, InterpreterOptions};
pub(crate) use init::ServiceInit;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use utils::{assert_constraints, instantiate_import};
use uuid::Uuid;
use wick_config::config::{ComponentDefinition, ComponentKind};

use self::utils::instantiate_component;
use crate::components::make_link_callback;
use crate::components::validation::expect_signature_match;
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
  interpreter: Arc<flow_graph_interpreter::Interpreter>,
}

impl RuntimeService {
  pub(crate) async fn start(mut init: ServiceInit) -> Result<Arc<Self>> {
    let span = init.span.clone();
    let ns = init.namespace.clone().unwrap_or_else(|| init.id.to_string());
    let graph = init.span.in_scope(|| {
      debug!("initializing engine service");

      Ok::<_, EngineError>(flow_graph_interpreter::graph::from_def(&mut init.manifest)?)
    })?;

    let mut components = HandlerMap::default();

    if init.manifest.component().kind() == ComponentKind::Composite {
      // Initialize any native components for composite components.
      for native_comp in init.native_components.inner() {
        components
          .add(native_comp(init.seed())?)
          .map_err(|e| EngineError::InterpreterInit(init.manifest.source().map(Into::into), Box::new(e)))?;
      }
    } else {
      // Instantiate the non-composite component as an exposed, standalone component.
      let component_init = init.child_init(init.manifest.root_config().cloned());
      let def: ComponentDefinition = init.manifest.component().clone().into();

      span.in_scope(|| debug!(%ns,options=?component_init,"instantiating component"));
      if let Some(main_component) = instantiate_component(ns.clone(), &def, component_init).await? {
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
          .map_err(|e| EngineError::InterpreterInit(init.manifest.source().map(Into::into), Box::new(e)))?;
      }
    }

    for component in init.manifest.import().values() {
      let component_init = init.child_init(component.config().cloned());
      if let Some(p) = instantiate_import(component, component_init).await? {
        components
          .add(p)
          .map_err(|e| EngineError::InterpreterInit(init.manifest.source().map(Into::into), Box::new(e)))?;
      }
    }

    let source = init.manifest.source();
    let callback = make_link_callback(init.id);
    assert_constraints(&init.constraints, &components)?;

    let mut interpreter = flow_graph_interpreter::Interpreter::new(
      graph,
      Some(ns.clone()),
      None, /* TODO: FIX passing config */
      Some(components),
      callback,
      &span,
    )
    .map_err(|e| EngineError::InterpreterInit(source.map(Into::into), Box::new(e)))?;

    let options = InterpreterOptions { ..Default::default() };
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

  pub(crate) fn for_id(id: &Uuid) -> Option<Arc<Self>> {
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
