use std::collections::HashMap;

use flow_component::LocalScope;
use wick_config::config::Permissions;
use wick_packet::RuntimeConfig;

#[derive(Clone, Default, derive_builder::Builder)]
#[builder(default)]
#[non_exhaustive]
pub struct ComponentSetup {
  #[builder(setter(strip_option), default)]
  pub engine: Option<wasmtime::Engine>,
  #[builder(setter(), default)]
  pub config: Option<RuntimeConfig>,
  #[builder(setter(), default)]
  pub buffer_size: Option<u32>,
  #[builder(setter(), default)]
  pub callback: LocalScope,
  #[builder(default, setter(into))]
  pub provided: HashMap<String, String>,
  #[builder(default, setter(into))]
  pub imported: HashMap<String, String>,
  #[builder(setter(), default)]
  pub permissions: Option<Permissions>,
}

impl std::fmt::Debug for ComponentSetup {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ComponentSetup")
      .field("config", &self.config)
      .field("buffer_size", &self.buffer_size)
      .field("provided", &self.provided)
      .field("imported", &self.provided)
      .finish()
  }
}
