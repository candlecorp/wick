use crate::component_definition::ComponentOperationExpression;
use crate::{v1, ComponentDefinition};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, PartialEq)]
/// Normalized representation of a CLI trigger configuration.
pub struct CliConfig {
  operation: ComponentOperationExpression,
  app: Option<ComponentDefinition>,
}

#[derive(Debug, Clone)]
#[must_use]
pub struct HttpTriggerConfig {
  resource: String,
  routers: Vec<HttpRouterConfig>,
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

#[derive(Debug, Clone)]
#[must_use]
pub struct RawRouterConfig {
  path: String,
  operation: ComponentOperationExpression,
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

#[derive(Debug, Clone)]
#[must_use]
pub struct RestRouterConfig {
  path: String,
  component: ComponentDefinition,
}

impl RestRouterConfig {
  #[must_use]
  pub fn path(&self) -> &str {
    &self.path
  }
  #[must_use]
  pub fn component(&self) -> &ComponentDefinition {
    &self.component
  }
}

impl From<v1::TriggerDefinition> for TriggerDefinition {
  fn from(trigger: v1::TriggerDefinition) -> Self {
    match trigger {
      v1::TriggerDefinition::CliTrigger(cli) => Self::Cli(CliConfig {
        operation: cli.operation.into(),
        app: cli.app.map(|v| v.into()),
      }),
      v1::TriggerDefinition::HttpTrigger(v) => Self::Http(HttpTriggerConfig {
        resource: v.resource,
        routers: v.routers.into_iter().map(|v| v.into()).collect(),
      }),
    }
  }
}

#[derive(Debug, Clone)]
#[must_use]
pub enum HttpRouterConfig {
  RawRouter(RawRouterConfig),
  RestRouter(RestRouterConfig),
}

impl From<v1::HttpRouter> for HttpRouterConfig {
  fn from(router: v1::HttpRouter) -> Self {
    match router {
      v1::HttpRouter::RawRouter(v) => Self::RawRouter(RawRouterConfig {
        path: v.path,
        operation: v.operation.into(),
      }),
      v1::HttpRouter::RestRouter(v) => Self::RestRouter(RestRouterConfig {
        path: v.path,
        component: v.component.into(),
      }),
    }
  }
}

impl CliConfig {
  /// Returns the component id for the CLI trigger.
  #[must_use]
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
