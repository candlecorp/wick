use wick_config::config::WickRouter;
use wick_packet::{Entity, RuntimeConfig};

use crate::HttpError;

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

pub(super) fn resolve_middleware_components(router: &impl WickRouter) -> Result<RouterMiddleware, HttpError> {
  let mut request_operations = Vec::new();
  let mut response_operations = Vec::new();
  if let Some(middleware) = router.middleware() {
    for operation in middleware.request() {
      let component_id = operation.component_id()?;
      request_operations.push((
        Entity::operation(component_id, operation.name()),
        operation.config().and_then(|v| v.value().cloned()),
      ));
    }
    for operation in middleware.response() {
      let component_id = operation.component_id()?;
      response_operations.push((
        Entity::operation(component_id, operation.name()),
        operation.config().and_then(|v| v.value().cloned()),
      ));
    }
  }
  let middleware = RouterMiddleware::new(request_operations, response_operations);
  Ok(middleware)
}
