use crate::{v1, ComponentKind};

#[derive(Debug, Clone, PartialEq)]
/// Normalized representation of a trigger definition.
pub enum TriggerDefinition {
  /// A CLI trigger.
  Cli(CliConfig),
}

impl TriggerDefinition {
  /// Returns the kind of trigger.
  pub fn kind(&self) -> TriggerKind {
    match self {
      TriggerDefinition::Cli(_) => TriggerKind::Cli,
    }
  }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
#[must_use]
/// The kind of trigger.
pub enum TriggerKind {
  /// A CLI trigger.
  Cli,
}

impl std::fmt::Display for TriggerKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TriggerKind::Cli => f.write_str("CLI"),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
/// Normalized representation of a CLI trigger configuration.
pub struct CliConfig {
  component: Option<ComponentKind>,
  operation: String,
  app: Option<ComponentKind>,
}

impl From<v1::TriggerDefinition> for TriggerDefinition {
  fn from(trigger: v1::TriggerDefinition) -> Self {
    match trigger {
      v1::TriggerDefinition::CliTrigger(cli) => Self::Cli(CliConfig {
        component: cli.component.map(|v| v.into()),
        operation: cli.operation,
        app: cli.app.map(|v| v.into()),
      }),
    }
  }
}

impl CliConfig {
  /// Returns the component definition for the CLI trigger.
  #[must_use]
  pub fn component(&self) -> Option<&ComponentKind> {
    self.component.as_ref()
  }

  /// Returns the operation name for the CLI trigger.
  #[must_use]
  pub fn operation(&self) -> &str {
    &self.operation
  }

  /// Returns the app definition for the CLI trigger.
  #[must_use]
  pub fn app(&self) -> Option<&ComponentKind> {
    self.app.as_ref()
  }
}
