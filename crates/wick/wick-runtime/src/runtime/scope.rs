mod child_init;
mod component_registry;
pub(crate) mod error;
mod init;
mod utils;

pub(crate) use child_init::{init_child, ChildInit};
pub(crate) use component_registry::{ComponentFactory, ComponentRegistry};
use flow_graph_interpreter::{HandlerMap, NamespaceHandler};
pub(crate) use init::ScopeInit;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use uuid::Uuid;
use wick_packet::Entity;

use crate::dev::prelude::*;

type ServiceMap = HashMap<Uuid, Scope>;
type Result<T> = std::result::Result<T, ScopeError>;

static SCOPE_REGISTRY: Lazy<Mutex<ServiceMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[must_use]
#[derive(Debug, Clone)]
pub(crate) struct Scope {
  inner: Arc<InnerScope>,
}

#[derive(Debug)]
pub(crate) struct InnerScope {
  #[allow(unused)]
  started_time: std::time::Instant,
  pub(crate) parent: Option<Uuid>,
  pub(crate) id: Uuid,
  pub(super) namespace: String,
  pub(super) active_config: ComponentConfiguration,
  interpreter: flow_graph_interpreter::Interpreter,
}

impl Scope {
  pub(crate) async fn start(mut init: ScopeInit) -> Result<Self> {
    let started_time = std::time::Instant::now();
    let (extends, components) = init.instantiate_main().await?;
    let components = init.instantiate_imports(extends, components).await?;
    let interpreter = init.init_interpreter(components).await?;

    let scope = Scope {
      inner: Arc::new(InnerScope {
        parent: init.parent,
        started_time,
        id: init.id,
        namespace: init.namespace(),
        active_config: init.manifest,
        interpreter,
      }),
    };

    let mut registry = SCOPE_REGISTRY.lock();
    registry.insert(init.id, scope.clone());

    Ok(scope)
  }

  pub(crate) fn for_id(id: &Uuid) -> Option<Self> {
    let registry = SCOPE_REGISTRY.lock();
    registry.get(id).cloned()
  }

  pub(crate) fn id(&self) -> Uuid {
    self.inner.id
  }

  pub(crate) fn namespace(&self) -> &str {
    &self.inner.namespace
  }

  pub(crate) async fn shutdown(&self) -> std::result::Result<(), RuntimeError> {
    let _ = self.inner.interpreter.shutdown().await;
    Ok(())
  }

  pub(crate) fn render_dotviz(&self, op: &str) -> std::result::Result<String, RuntimeError> {
    self.inner.interpreter.render_dotviz(op).map_err(RuntimeError::DotViz)
  }

  pub(crate) fn active_config(&self) -> &ComponentConfiguration {
    &self.inner.active_config
  }

  pub(super) fn find(parent: Option<Uuid>, ns: &str) -> Option<Scope> {
    let registry = SCOPE_REGISTRY.lock();
    registry
      .values()
      .find(|scope| scope.inner.namespace == ns && scope.inner.parent == parent)
      .cloned()
  }

  pub(super) fn get_handler_signature(&self, ns: &str) -> Option<&ComponentSignature> {
    if ns == Entity::LOCAL {
      return Some(self.inner.interpreter.signature());
    }
    self
      .inner
      .interpreter
      .components()
      .get(ns)
      .map(|c| c.component().signature())
  }
}

impl InvocationHandler for Scope {
  fn get_signature(&self) -> std::result::Result<ComponentSignature, ComponentError> {
    let mut signature = self.inner.interpreter.signature().clone();
    signature.name = Some(self.inner.id.as_hyphenated().to_string());

    Ok(signature)
  }

  fn invoke(
    &self,
    invocation: Invocation,
    config: Option<RuntimeConfig>,
  ) -> std::result::Result<BoxFuture<std::result::Result<InvocationResponse, ComponentError>>, ComponentError> {
    let tx_id = invocation.tx_id();

    let fut = self.inner.interpreter.invoke(invocation, config);
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
        return Err(ScopeError::NotFound(Context::Component, from.clone()));
      }
    }
  }

  Ok(provide)
}
