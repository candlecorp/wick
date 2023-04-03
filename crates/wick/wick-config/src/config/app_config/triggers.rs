use crate::config::*;

#[derive(Debug, Clone, derive_assets::AssetManager)]
#[asset(AssetReference)]

/// Normalized representation of a trigger definition.
pub enum TriggerDefinition {
  /// A CLI trigger.
  Cli(CliConfig),
  /// An HTTP trigger.
  Http(HttpTriggerConfig),
}

impl TriggerDefinition {
  /// Returns the kind of trigger.
  pub fn kind(&self) -> TriggerKind {
    match self {
      TriggerDefinition::Cli(_) => TriggerKind::Cli,
      TriggerDefinition::Http(_) => TriggerKind::Http,
    }
  }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
#[must_use]
/// The kind of trigger.
pub enum TriggerKind {
  /// A CLI trigger.
  Cli,
  /// An Http trigger.
  Http,
}

impl std::fmt::Display for TriggerKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TriggerKind::Cli => f.write_str("CLI"),
      TriggerKind::Http => f.write_str("HTTP"),
    }
  }
}

#[derive(Debug, Clone, PartialEq, derive_assets::AssetManager)]
#[asset(AssetReference)]

/// Normalized representation of a CLI trigger configuration.
pub struct CliConfig {
  pub(crate) operation: ComponentOperationExpression,
  pub(crate) app: Option<ComponentDefinition>,
}

#[derive(Debug, Clone, derive_assets::AssetManager)]
#[asset(AssetReference)]
#[must_use]
pub struct HttpTriggerConfig {
  #[asset(skip)]
  pub(crate) resource: String,
  pub(crate) routers: Vec<HttpRouterConfig>,
}

impl HttpTriggerConfig {
  #[must_use]
  pub fn resource_id(&self) -> &str {
    &self.resource
  }
  pub fn routers(&self) -> &[HttpRouterConfig] {
    &self.routers
  }
}

#[derive(Debug, Clone, derive_assets::AssetManager)]
#[asset(AssetReference)]
#[must_use]
pub struct RawRouterConfig {
  #[asset(skip)]
  pub(crate) path: String,
  pub(crate) operation: ComponentOperationExpression,
}

impl RawRouterConfig {
  #[must_use]
  pub fn path(&self) -> &str {
    &self.path
  }
  #[must_use]
  pub fn operation(&self) -> &ComponentOperationExpression {
    &self.operation
  }
}

#[derive(Debug, Clone, derive_assets::AssetManager)]
#[asset(AssetReference)]
#[must_use]
pub struct RestRouterConfig {
  #[asset(skip)]
  pub(crate) path: String,
  pub(crate) component: ComponentDefinition,
}

impl RestRouterConfig {
  #[must_use]
  pub fn path(&self) -> &str {
    &self.path
  }
  pub fn component(&self) -> &ComponentDefinition {
    &self.component
  }
}

#[derive(Debug, Clone, derive_assets::AssetManager)]
#[asset(AssetReference)]
#[must_use]
pub enum HttpRouterConfig {
  RawRouter(RawRouterConfig),
  RestRouter(RestRouterConfig),
}

impl CliConfig {
  /// Returns the component id for the CLI trigger.
  pub fn component(&self) -> &ComponentDefinition {
    &self.operation.component
  }

  /// Returns the operation name for the CLI trigger.
  #[must_use]
  pub fn operation(&self) -> &str {
    &self.operation.operation
  }

  /// Returns the app definition for the CLI trigger.
  #[must_use]
  pub fn app(&self) -> Option<&ComponentDefinition> {
    self.app.as_ref()
  }
}
