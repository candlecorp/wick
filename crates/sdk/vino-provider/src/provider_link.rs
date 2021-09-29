use serde::{
  Deserialize,
  Serialize,
};
use vino_entity::Entity;

/// An implementation that encapsulates a provider link that components can use to call out to a Vino network.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
pub struct ProviderLink {
  linked_entity: Entity,
  origin_entity: Entity,
}

impl ProviderLink {
  /// Constructor for a [ProviderLink]
  pub fn new(linked_entity: Entity, origin_entity: Entity) -> Self {
    Self {
      linked_entity,
      origin_entity,
    }
  }

  #[must_use]
  /// Get the URL for the called component
  pub fn get_component_url(&self, component: &str) -> String {
    Entity::Component(self.linked_entity.name(), component.to_owned()).url()
  }

  #[must_use]
  /// Get the URL for the called component
  pub fn get_origin_url(&self) -> String {
    self.origin_entity.url()
  }
}
