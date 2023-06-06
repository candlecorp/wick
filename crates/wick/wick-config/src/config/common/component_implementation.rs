use wick_interface_types::Field;

use crate::config;

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(asset(config::AssetReference))]
#[must_use]
/// A root-level wick component implementation.
pub enum ComponentImplementation {
  /// A wasm component.
  Wasm(config::WasmComponentImplementation),
  /// A composite component.
  Composite(config::CompositeComponentImplementation),
  /// A sql component.
  Sql(config::components::SqlComponentConfig),
  /// An http client component.
  HttpClient(config::components::HttpClientComponentConfig),
}

impl ComponentImplementation {
  /// Get the kind of component represented by this configuration.
  pub fn kind(&self) -> ComponentKind {
    match self {
      ComponentImplementation::Wasm(_) => ComponentKind::Wasm,
      ComponentImplementation::Composite(_) => ComponentKind::Composite,
      ComponentImplementation::Sql(_) => ComponentKind::Sql,
      ComponentImplementation::HttpClient(_) => ComponentKind::HttpClient,
    }
  }

  #[must_use]
  /// Get the operation signatures for this component.
  pub fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    match self {
      ComponentImplementation::Wasm(w) => w.operation_signatures(),
      ComponentImplementation::Composite(c) => c.operation_signatures(),
      ComponentImplementation::Sql(c) => c.operation_signatures(),
      ComponentImplementation::HttpClient(c) => c.operation_signatures(),
    }
  }

  #[must_use]
  /// Get the default name for this component.
  pub fn default_name(&self) -> &'static str {
    match self {
      ComponentImplementation::Wasm(_) => panic!("Wasm components must be named"),
      ComponentImplementation::Composite(_) => panic!("Composite components must be named"),
      ComponentImplementation::Sql(_) => "wick/component/sql",
      ComponentImplementation::HttpClient(_) => "wick/component/http",
    }
  }

  #[must_use]
  /// Get the associated configuration for this component.
  pub fn config(&self) -> Option<&[Field]> {
    match self {
      ComponentImplementation::Wasm(c) => Some(c.config()),
      ComponentImplementation::Composite(c) => Some(c.config()),
      ComponentImplementation::Sql(_) => None,
      ComponentImplementation::HttpClient(_) => None,
    }
  }
}

impl Default for ComponentImplementation {
  fn default() -> Self {
    ComponentImplementation::Composite(config::CompositeComponentImplementation::default())
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[must_use]
/// The kind of component represented by ComponentImplementation.
pub enum ComponentKind {
  /// A wasm component.
  Wasm,
  /// A composite component.
  Composite,
  /// A sql component.
  Sql,
  /// An http client component.
  HttpClient,
}

impl std::fmt::Display for ComponentKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ComponentKind::Wasm => write!(f, "wick/component/wasm"),
      ComponentKind::Composite => write!(f, "wick/component/composite"),
      ComponentKind::Sql => write!(f, "wick/component/sql"),
      ComponentKind::HttpClient => write!(f, "wick/component/http"),
    }
  }
}
