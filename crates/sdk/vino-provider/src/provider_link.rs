use serde::{Deserialize, Serialize};
use vino_entity::Entity;
#[cfg(any(feature = "wasm", feature = "native"))]
use vino_transport::TransportMap;

/// An implementation that encapsulates a provider link that components can use to call out to a Vino network.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    Entity::component(self.linked_entity.name(), component).url()
  }

  #[must_use]
  /// Get the URL for the called component
  pub fn get_origin_url(&self) -> String {
    self.origin_entity.url()
  }

  /// Make a call to the linked provider.
  #[cfg(feature = "wasm")]
  pub fn call(
    &self,
    component: &str,
    payload: impl Into<TransportMap>,
  ) -> Result<crate::wasm::prelude::ProviderOutput, crate::wasm::Error> {
    let payload: TransportMap = payload.into();
    let result = crate::wasm::host_call(
      "1",
      &self.get_origin_url(),
      &self.get_component_url(component),
      &vino_codec::messagepack::serialize(&payload)?,
    )?;
    let packets: Vec<vino_transport::TransportWrapper> = vino_codec::messagepack::deserialize(&result)?;
    Ok(crate::wasm::prelude::ProviderOutput::new(packets))
  }

  /// Make a call to the linked provider.
  #[cfg(all(not(feature = "wasm"), feature = "native"))]
  pub fn call(
    &self,
    _component: &str,
    _payload: impl Into<TransportMap>,
  ) -> Result<crate::native::prelude::ProviderOutput, crate::native::Error> {
    unimplemented!("Link calls from native providers is not implemented yet")
  }
}

impl std::fmt::Display for ProviderLink {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}=>{}", self.origin_entity, self.linked_entity)
  }
}
