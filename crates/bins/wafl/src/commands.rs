pub(crate) mod bundle;
pub(crate) mod component;
pub(crate) mod project;
pub(crate) mod registry;
pub(crate) mod rpc;
pub(crate) mod wasm;

use clap::{AppSettings, Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[clap(
      global_setting(AppSettings::DeriveDisplayOrder),
      name = crate::BIN_NAME,
      about = crate::BIN_DESC,
      version = option_env!("WAFL_VERSION").unwrap_or("0.0.0")
    )]
pub(crate) struct Cli {
  #[clap(subcommand)]
  pub(crate) command: CliCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum CliCommand {
  /// Commands to manage projects.
  #[clap(subcommand, name = "project")]
  Project(project::SubCommands),

  /// Commands to manage components.
  #[clap(subcommand, name = "component")]
  Component(component::SubCommands),

  /// Commands for WebAssembly component.
  #[clap(subcommand, name = "wasm")]
  Wasm(wasm::SubCommands),

  /// Commands for multi-architecture bundles.
  #[clap(subcommand, name = "bundle")]
  Bundle(bundle::SubCommands),

  /// Commands to interact with registries.
  #[clap(subcommand, name = "registry")]
  Registry(registry::SubCommands),

  /// Commands to interact with running Wasmflow instances.
  #[clap(subcommand, name = "rpc")]
  Rpc(rpc::SubCommands),
}

#[cfg(test)]
mod test {
  #[test]
  fn verify_options() {
    use clap::IntoApp;
    super::Cli::command().debug_assert();
  }
}
