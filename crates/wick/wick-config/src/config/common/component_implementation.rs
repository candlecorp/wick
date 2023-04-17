use std::collections::HashMap;

use wick_interface_types::TypeDefinition;

use super::BoundComponent;
use crate::config;

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(config::AssetReference)]
#[must_use]
pub enum ComponentImplementation {
  Wasm(config::WasmComponentImplementation),
  Composite(config::CompositeComponentImplementation),
}

impl ComponentImplementation {
  pub fn kind(&self) -> ComponentKind {
    match self {
      ComponentImplementation::Wasm(_) => ComponentKind::Wasm,
      ComponentImplementation::Composite(_) => ComponentKind::Composite,
    }
  }

  pub fn types(&self) -> &[TypeDefinition] {
    match self {
      ComponentImplementation::Wasm(w) => w.types(),
      ComponentImplementation::Composite(c) => c.types(),
    }
  }

  #[must_use]
  pub fn imports_owned(&self) -> HashMap<String, BoundComponent> {
    match self {
      ComponentImplementation::Wasm(_w) => HashMap::new(),
      ComponentImplementation::Composite(c) => c.import.clone(),
    }
  }

  #[must_use]
  pub fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    match self {
      ComponentImplementation::Wasm(w) => w.operation_signatures(),
      ComponentImplementation::Composite(c) => c.operation_signatures(),
    }
  }
}

impl Default for ComponentImplementation {
  fn default() -> Self {
    ComponentImplementation::Composite(config::CompositeComponentImplementation::default())
  }
}

#[derive(Debug, Clone, Copy)]
#[must_use]
pub enum ComponentKind {
  Wasm,
  Composite,
}

impl std::fmt::Display for ComponentKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ComponentKind::Wasm => write!(f, "wick/component/wasm"),
      ComponentKind::Composite => write!(f, "wick/component/composite"),
    }
  }
}
