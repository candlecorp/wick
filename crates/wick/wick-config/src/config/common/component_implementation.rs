use wick_interface_types::TypeDefinition;

use crate::config;

#[derive(Debug, Clone, derive_assets::AssetManager)]
#[asset(config::AssetReference)]
#[must_use]
pub enum ComponentImplementation {
  Wasm(config::WasmComponentConfiguration),
  Composite(config::CompositeComponentConfiguration),
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
}

impl Default for ComponentImplementation {
  fn default() -> Self {
    ComponentImplementation::Composite(config::CompositeComponentConfiguration::default())
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
