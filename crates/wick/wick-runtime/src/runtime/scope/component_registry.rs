use flow_graph_interpreter::NamespaceHandler;
use seeded_random::Seed;

use crate::components::initialize_native_component;
use crate::{ScopeError, V0_NAMESPACE};

pub(crate) type ComponentFactory = dyn Fn(Seed) -> Result<NamespaceHandler, ScopeError> + Send + Sync;

#[derive()]
pub(crate) struct ComponentRegistry(Vec<Box<ComponentFactory>>);

impl ComponentRegistry {
  /// Add a component to the registry
  pub(crate) fn add(&mut self, factory: Box<ComponentFactory>) {
    self.0.push(factory);
  }

  /// Get the list of components
  #[must_use]
  pub(crate) fn inner(&self) -> &[Box<ComponentFactory>] {
    &self.0
  }
}

impl std::fmt::Debug for ComponentRegistry {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("ComponentRegistry").field(&self.0.len()).finish()
  }
}

impl Default for ComponentRegistry {
  fn default() -> Self {
    let mut list: Vec<Box<ComponentFactory>> = Vec::default();
    list.push(Box::new(|seed: Seed| {
      initialize_native_component(V0_NAMESPACE.to_owned(), seed)
    }));
    Self(list)
  }
}
