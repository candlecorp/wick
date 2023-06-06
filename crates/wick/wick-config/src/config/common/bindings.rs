#![allow(missing_docs)] // delete when we move away from the `property` crate.

use super::{ComponentDefinition, ComponentImplementation, HighLevelComponent, ImportDefinition, InterfaceDefinition};
use crate::config::components::WasmComponent;
use crate::config::{self};

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(config::AssetReference))]
/// A definition of a Wick Collection with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub struct ImportBinding {
  /// The namespace to reference the collection's components on.
  #[asset(skip)]
  pub(crate) id: String,
  /// The kind/type of the collection.
  pub(crate) kind: ImportDefinition,
}

impl From<ComponentImplementation> for ImportDefinition {
  fn from(implementation: ComponentImplementation) -> Self {
    match implementation {
      #[allow(deprecated)]
      ComponentImplementation::Wasm(wasm) => Self::Component(ComponentDefinition::Wasm(WasmComponent {
        reference: wasm.reference,
        config: Default::default(),
        permissions: Default::default(),
        provide: Default::default(),
      })),
      ComponentImplementation::Composite(_) => unimplemented!("Inline composite components are not yet supported."),
      ComponentImplementation::HttpClient(client) => Self::Component(ComponentDefinition::HighLevelComponent(
        HighLevelComponent::HttpClient(client),
      )),
      ComponentImplementation::Sql(c) => {
        Self::Component(ComponentDefinition::HighLevelComponent(HighLevelComponent::Sql(c)))
      }
    }
  }
}

impl ImportBinding {
  /// Create a new [ImportBinding] with specified name and [ImportDefinition].
  pub fn new(name: impl AsRef<str>, kind: ImportDefinition) -> Self {
    Self {
      id: name.as_ref().to_owned(),
      kind,
    }
  }

  /// Get the configuration object for the collection.
  #[must_use]
  pub fn config(&self) -> Option<&wick_packet::GenericConfig> {
    match &self.kind {
      ImportDefinition::Component(c) => c.config(),
      ImportDefinition::Types(_) => None,
    }
  }

  /// Initialize a new import for the specified [ComponentDefinition].
  pub fn component(name: impl AsRef<str>, component: ComponentDefinition) -> Self {
    #[allow(deprecated)]
    Self::new(name, ImportDefinition::Component(component))
  }

  /// Create a new Wasm component definition.
  pub fn wasm(name: impl AsRef<str>, component: WasmComponent) -> Self {
    #[allow(deprecated)]
    Self::new(name, ImportDefinition::Component(ComponentDefinition::Wasm(component)))
  }

  /// Create a new GrpcUrl component definition.
  pub fn grpc_url(name: impl AsRef<str>, component: config::components::GrpcUrlComponent) -> Self {
    Self::new(
      name,
      ImportDefinition::Component(ComponentDefinition::GrpcUrl(component)),
    )
  }

  /// Create a new Manifest component definition.
  pub fn manifest(name: impl AsRef<str>, component: config::components::ManifestComponent) -> Self {
    Self::new(
      name,
      ImportDefinition::Component(ComponentDefinition::Manifest(component)),
    )
  }

  /// Create a new High level component definition.
  pub fn high_level(name: impl AsRef<str>, component: HighLevelComponent) -> Self {
    Self::new(
      name,
      ImportDefinition::Component(ComponentDefinition::HighLevelComponent(component)),
    )
  }
}

#[derive(Debug, Default, Clone, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(private), mut(disable))]
#[asset(asset(crate::config::AssetReference))]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct BoundInterface {
  /// The namespace to reference the collection's components on.
  #[asset(skip)]
  pub(crate) id: String,
  /// The kind/type of the collection.
  pub(crate) kind: InterfaceDefinition,
}
