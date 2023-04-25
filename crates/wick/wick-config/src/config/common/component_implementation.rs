use crate::config;

#[derive(Debug, Clone, derive_asset_container::AssetManager)]
#[asset(config::AssetReference)]
#[must_use]
pub enum ComponentImplementation {
  Wasm(config::WasmComponentImplementation),
  Composite(config::CompositeComponentImplementation),
  Sql(config::components::SqlComponentConfig),
  HttpClient(config::components::HttpClientComponentConfig),
}

impl ComponentImplementation {
  pub fn kind(&self) -> ComponentKind {
    match self {
      ComponentImplementation::Wasm(_) => ComponentKind::Wasm,
      ComponentImplementation::Composite(_) => ComponentKind::Composite,
      ComponentImplementation::Sql(_) => ComponentKind::Sql,
      ComponentImplementation::HttpClient(_) => ComponentKind::HttpClient,
    }
  }

  #[must_use]
  pub fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    match self {
      ComponentImplementation::Wasm(w) => w.operation_signatures(),
      ComponentImplementation::Composite(c) => c.operation_signatures(),
      ComponentImplementation::Sql(c) => c.operation_signatures(),
      ComponentImplementation::HttpClient(c) => c.operation_signatures(),
    }
  }

  #[must_use]
  pub fn default_name(&self) -> &'static str {
    match self {
      ComponentImplementation::Wasm(_) => panic!("Wasm components must be named"),
      ComponentImplementation::Composite(_) => panic!("Composite components must be named"),
      ComponentImplementation::Sql(_) => "wick/component/sql",
      ComponentImplementation::HttpClient(_) => "wick/component/http",
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
pub enum ComponentKind {
  Wasm,
  Composite,
  Sql,
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
