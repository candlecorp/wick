use super::{ChildInit, ComponentRegistry};
use crate::dev::prelude::*;
use crate::runtime::{RuntimeConstraint, RuntimeInit};

#[derive(Debug)]
pub(crate) struct ServiceInit {
  rng: Random,
  pub(crate) id: Uuid,
  pub(crate) manifest: ComponentConfiguration,
  pub(crate) allow_latest: bool,
  pub(crate) allowed_insecure: Vec<String>,
  pub(crate) namespace: Option<String>,
  pub(crate) constraints: Vec<RuntimeConstraint>,
  pub(crate) native_components: ComponentRegistry,
  pub(crate) span: Span,
}

impl ServiceInit {
  pub(crate) fn new(seed: Seed, config: RuntimeInit) -> Self {
    let rng = Random::from_seed(seed);
    Self {
      id: rng.uuid(),
      rng,
      manifest: config.manifest,
      allow_latest: config.allow_latest,
      allowed_insecure: config.allowed_insecure,
      namespace: config.namespace,
      constraints: config.constraints,
      native_components: config.native_components,
      span: config.span,
    }
  }

  pub(crate) fn new_with_id(id: Uuid, seed: Seed, config: RuntimeInit) -> Self {
    let rng = Random::from_seed(seed);
    Self {
      id,
      rng,
      manifest: config.manifest,
      allow_latest: config.allow_latest,
      allowed_insecure: config.allowed_insecure,
      namespace: config.namespace,
      constraints: config.constraints,
      native_components: config.native_components,
      span: config.span,
    }
  }

  pub(super) fn child_init(&self, root_config: Option<RuntimeConfig>) -> ChildInit {
    ChildInit {
      rng_seed: self.rng.seed(),
      runtime_id: self.id,
      root_config,
      allow_latest: self.allow_latest,
      allowed_insecure: self.allowed_insecure.clone(),
      resolver: self.manifest.resolver(),
      span: self.span.clone(),
    }
  }

  pub(super) fn seed(&self) -> Seed {
    self.rng.seed()
  }
}
