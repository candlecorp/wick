use std::sync::Arc;

use seeded_random::Seed;
use tracing::Span;
use uuid::Uuid;
use wick_config::config::ComponentConfiguration;
use wick_config::Resolver;
use wick_packet::RuntimeConfig;

use super::{ComponentRegistry, RuntimeService, ServiceInit};
use crate::runtime::RuntimeInit;
use crate::{BoxFuture, EngineError};

#[derive()]
pub(crate) struct ChildInit {
  pub(crate) rng_seed: Seed,
  pub(crate) runtime_id: Uuid,
  pub(crate) allow_latest: bool,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) config: Option<RuntimeConfig>,
  pub(crate) resolver: Box<Resolver>,
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
      .field("config", &self.config)
      .finish()
  }
}

pub(crate) fn init_child(
  uid: Uuid,
  manifest: ComponentConfiguration,
  namespace: Option<String>,
  opts: ChildInit,
) -> BoxFuture<'static, Result<Arc<RuntimeService>, EngineError>> {
  let child_span = debug_span!("child",id=%uid);
  child_span.follows_from(opts.span);
  let config = RuntimeInit {
    manifest,
    allow_latest: opts.allow_latest,
    allowed_insecure: opts.allowed_insecure,
    namespace,
    constraints: Default::default(),
    span: child_span,
    native_components: ComponentRegistry::default(),
  };
  let init = ServiceInit::new_with_id(uid, opts.rng_seed, config);

  Box::pin(async move { RuntimeService::start(init).await })
}
