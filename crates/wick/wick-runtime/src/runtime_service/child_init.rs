use std::sync::Arc;

use flow_graph_interpreter::HandlerMap;
use seeded_random::Seed;
use tracing::Span;
use uuid::Uuid;
use wick_config::config::ComponentConfiguration;
use wick_packet::{Entity, RuntimeConfig};

use super::{ComponentRegistry, RuntimeService, ServiceInit};
use crate::runtime::RuntimeInit;
use crate::{BoxFuture, EngineError};

#[derive()]
pub(crate) struct ChildInit {
  pub(crate) rng_seed: Seed,
  pub(crate) runtime_id: Uuid,
  pub(crate) allow_latest: bool,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) root_config: Option<RuntimeConfig>,
  pub(crate) provided: Option<HandlerMap>,
  #[allow(unused)]
  pub(crate) span: Span,
}

impl std::fmt::Debug for ChildInit {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ComponentInitOptions")
      .field("rng_seed", &self.rng_seed)
      .field("runtime_id", &self.runtime_id)
      .field("allow_latest", &self.allow_latest)
      .field("allowed_insecure", &self.allowed_insecure)
      .field("root_config", &self.root_config)
      .finish()
  }
}

pub(crate) fn init_child(
  uid: Uuid,
  manifest: ComponentConfiguration,
  namespace: Option<String>,
  opts: ChildInit,
) -> BoxFuture<'static, Result<Arc<RuntimeService>, EngineError>> {
  let child_span = info_span!(parent:opts.span,"runtime:child",id=%uid);
  let mut components = ComponentRegistry::default();

  Box::pin(async move {
    for req in manifest.requires() {
      let ns = req.id();
      if let Some(handler) = opts.provided.as_ref().and_then(|p| p.get(ns).cloned()) {
        components.add(Box::new(move |_| Ok(handler.clone())));
      } else {
        return Err(EngineError::RequirementUnsatisfied(Entity::component(ns)));
      }
    }

    let config = RuntimeInit {
      manifest,
      allow_latest: opts.allow_latest,
      allowed_insecure: opts.allowed_insecure,
      namespace,
      constraints: Default::default(),
      span: child_span,
      initial_components: components,
    };
    let init = ServiceInit::new_with_id(uid, opts.rng_seed, config);

    RuntimeService::start(init).await
  })
}
