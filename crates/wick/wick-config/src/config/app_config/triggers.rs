mod cli;
mod http;
mod time;
mod wasm_command;

use std::collections::HashMap;
use std::path::Path;

pub use cli::{CliConfig, CliConfigBuilder, CliConfigBuilderError};
pub use http::{
  Contact,
  Documentation,
  HttpRouterConfig,
  HttpRouterKind,
  HttpTriggerConfig,
  HttpTriggerConfigBuilder,
  HttpTriggerConfigBuilderError,
  Info,
  License,
  Middleware,
  MiddlewareBuilder,
  MiddlewareBuilderError,
  ProxyRouterConfig,
  ProxyRouterConfigBuilder,
  ProxyRouterConfigBuilderError,
  RawRouterConfig,
  RawRouterConfigBuilder,
  RawRouterConfigBuilderError,
  RestRoute,
  RestRouterConfig,
  RestRouterConfigBuilder,
  RestRouterConfigBuilderError,
  StaticRouterConfig,
  StaticRouterConfigBuilder,
  StaticRouterConfigBuilderError,
  Tools,
  WickRouter,
};
use serde_json::Value;
pub use time::{
  ScheduleConfig,
  ScheduleConfigBuilder,
  ScheduleConfigBuilderError,
  TimeTriggerConfig,
  TimeTriggerConfigBuilder,
  TimeTriggerConfigBuilderError,
};
use wick_asset_reference::AssetReference;
use wick_packet::RuntimeConfig;

use self::common::template_config::Renderable;
use self::common::{Binding, ImportDefinition};
pub use self::wasm_command::WasmCommandConfig;
use crate::config::common;
use crate::error::ManifestError;
use crate::ExpandImports;

#[derive(Debug, Clone, derive_asset_container::AssetManager, serde::Serialize)]
#[asset(asset(AssetReference))]

/// Normalized representation of a trigger definition.
#[serde(rename_all = "kebab-case")]

pub enum TriggerDefinition {
  /// A WebAssembly command trigger.
  WasmCommand(WasmCommandConfig),
  /// A CLI trigger.
  Cli(CliConfig),
  /// An HTTP trigger.
  Http(HttpTriggerConfig),
  /// A time trigger.
  Time(TimeTriggerConfig),
}

impl TriggerDefinition {
  /// Returns the kind of trigger.
  pub const fn kind(&self) -> TriggerKind {
    match self {
      TriggerDefinition::WasmCommand(_) => TriggerKind::WasmCommand,
      TriggerDefinition::Cli(_) => TriggerKind::Cli,
      TriggerDefinition::Http(_) => TriggerKind::Http,
      TriggerDefinition::Time(_) => TriggerKind::Time,
    }
  }
}

impl Renderable for TriggerDefinition {
  fn render_config(
    &mut self,
    source: Option<&Path>,
    root_config: Option<&RuntimeConfig>,
    env: Option<&HashMap<String, String>>,
  ) -> Result<(), ManifestError> {
    match self {
      TriggerDefinition::WasmCommand(v) => v.render_config(source, root_config, env),
      TriggerDefinition::Cli(v) => v.render_config(source, root_config, env),
      TriggerDefinition::Http(v) => v.render_config(source, root_config, env),
      TriggerDefinition::Time(v) => v.render_config(source, root_config, env),
    }
  }
}

impl ExpandImports for TriggerDefinition {
  type Error = ManifestError;
  fn expand_imports(&mut self, bindings: &mut Vec<Binding<ImportDefinition>>, index: usize) -> Result<(), Self::Error> {
    match self {
      TriggerDefinition::WasmCommand(c) => c.expand_imports(bindings, index),
      TriggerDefinition::Cli(c) => c.expand_imports(bindings, index),
      TriggerDefinition::Http(c) => c.expand_imports(bindings, index),
      TriggerDefinition::Time(c) => c.expand_imports(bindings, index),
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
  /// A time trigger.
  Time,
  /// An external WebAssembly command component.
  WasmCommand,
}

impl std::fmt::Display for TriggerKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TriggerKind::Cli => f.write_str("CLI"),
      TriggerKind::Http => f.write_str("HTTP"),
      TriggerKind::Time => f.write_str("TIME"),
      TriggerKind::WasmCommand => f.write_str("WASM_COMMAND"),
    }
  }
}

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, serde::Serialize)]
#[asset(asset(AssetReference))]
#[must_use]

pub struct OperationInputConfig {
  #[asset(skip)]
  pub(crate) name: String,
  #[asset(skip)]
  pub(crate) value: Value,
}

impl OperationInputConfig {
  #[must_use]
  pub const fn name(&self) -> &String {
    &self.name
  }

  #[must_use]
  pub const fn value(&self) -> &Value {
    &self.value
  }
}
