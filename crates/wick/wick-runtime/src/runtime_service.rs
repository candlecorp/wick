mod child_init;
mod component_registry;
pub(crate) mod error;
mod init;
mod utils;

pub(crate) use child_init::{init_child, ChildInit};
pub(crate) use component_registry::{ComponentFactory, ComponentRegistry};
use flow_graph_interpreter::{HandlerMap, NamespaceHandler};
pub(crate) use init::ServiceInit;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use uuid::Uuid;

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
    let started_time = std::time::Instant::now();
    let (extends, components) = init.instantiate_main().await?;
    let components = init.instantiate_imports(extends, components).await?;
    let interpreter = init.init_interpreter(components).await?;

    let engine = Arc::new(RuntimeService {
      started_time,
      id: init.id,
      namespace: init.namespace(),
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

fn generate_provides_handlers(
  provides: Option<&HashMap<String, String>>,
  available: &HandlerMap,
) -> Result<HandlerMap> {
  let mut provide = HandlerMap::default();
  if let Some(provides) = provides {
    for (to, from) in provides {
      if let Some(handler) = available.get(from) {
        let ns_handler = NamespaceHandler::new_from_shared(to, handler.component().clone());
        let _ = provide.add(ns_handler); // Can't fail, we just created the map and are iterating over unique keys.
      } else {
        return Err(EngineError::NotFound(Context::Component, from.clone()));
      }
    }
  }

  Ok(provide)
}
