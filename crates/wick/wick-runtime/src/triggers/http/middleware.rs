use wick_config::config::{AppConfiguration, ImportBinding, WickRouter};
use wick_packet::{Entity, RuntimeConfig};

use crate::dev::prelude::RuntimeError;
use crate::triggers::{resolve_ref, ResolvedComponent};

#[derive(Debug, Clone)]
pub(crate) struct RouterMiddleware {
  pub(crate) request: Vec<(Entity, Option<RuntimeConfig>)>,
  pub(crate) response: Vec<(Entity, Option<RuntimeConfig>)>,
}

impl RouterMiddleware {
  pub(crate) fn new(
    request: Vec<(Entity, Option<RuntimeConfig>)>,
    response: Vec<(Entity, Option<RuntimeConfig>)>,
  ) -> Self {
    Self { request, response }
  }
}

pub(super) fn resolve_middleware_components(
  router_index: usize,
  app_config: &AppConfiguration,
  router: &impl WickRouter,
) -> Result<(RouterMiddleware, Vec<ImportBinding>), RuntimeError> {
  let mut request_operations = Vec::new();
  let mut response_operations = Vec::new();
  let mut bindings = Vec::new();
  if let Some(middleware) = router.middleware() {
    for (i, operation) in middleware.request().iter().enumerate() {
      let component_id = match resolve_ref(app_config, operation.component())? {
        ResolvedComponent::Ref(id, _) => id.to_owned(),
        ResolvedComponent::Inline(def) => {
          let id = format!("{}_request_middleware_{}", router_index, i);
          let binding = ImportBinding::component(&id, def.clone());
          bindings.push(binding);
          id
        }
      };
      request_operations.push((
        Entity::operation(component_id, operation.name()),
        operation.config().and_then(|v| v.value().cloned()),
      ));
    }
    for (i, operation) in middleware.response().iter().enumerate() {
      let component_id = match resolve_ref(app_config, operation.component())? {
        ResolvedComponent::Ref(id, _) => id.to_owned(),
        ResolvedComponent::Inline(def) => {
          let id = format!("{}_response_middleware_{}", router_index, i);
          let binding = ImportBinding::component(&id, def.clone());
          bindings.push(binding);
          id
        }
      };
      response_operations.push((
        Entity::operation(component_id, operation.name()),
        operation.config().and_then(|v| v.value().cloned()),
      ));
    }
  }
  let middleware = RouterMiddleware::new(request_operations, response_operations);
  Ok((middleware, bindings))
}
