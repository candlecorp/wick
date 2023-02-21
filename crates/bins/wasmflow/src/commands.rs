pub(crate) mod bundle;
pub(crate) mod component;
pub(crate) mod invoke;
pub(crate) mod key;
pub(crate) mod list;
pub(crate) mod project;
pub(crate) mod query;
pub(crate) mod registry;
pub(crate) mod rpc;
pub(crate) mod run;
pub(crate) mod serve;
pub(crate) mod test;
pub(crate) mod wasm;

use clap::{AppSettings, Args, Parser, Subcommand};
use logger::LoggingOptions;

#[derive(Parser, Debug, Clone)]
#[clap(
  global_setting(AppSettings::DeriveDisplayOrder),
  name = crate::BIN_NAME,
  about = crate::BIN_DESC,
  version,
)]
pub(crate) struct Cli {
  #[clap(subcommand)]
  pub(crate) command: CliCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum CliCommand {
  // Core commands
  /// Start a persistent host from a manifest.
  #[clap(name = "serve")]
  Serve(serve::ServeCommand),
  /// Load a manifest and execute an entrypoint component (temporarily disabled).
  #[clap(name = "run")]
  Run(run::RunCommand),
  /// Invoke a component from a manifest or wasm module.
  #[clap(name = "invoke")]
  Invoke(invoke::InvokeCommand),
  /// Print the components in a manifest or wasm module.
  #[clap(name = "list")]
  List(list::ListCommand),
  /// Execute a component with test data and assert its output.
  #[clap(name = "test")]
  Test(test::TestCommand),

  // Commands migrated from external `wafl` cli.
  /// Commands to manage projects.
  #[clap(subcommand, name = "project", alias = "proj")]
  Project(project::SubCommands),

  /// Commands to manage components.
  #[clap(subcommand, name = "component", alias = "comp")]
  Component(component::SubCommands),

  /// Commands for WebAssembly component.
  #[clap(subcommand, name = "wasm")]
  Wasm(wasm::SubCommands),

  /// Commands for multi-architecture bundles.
  #[clap(subcommand, name = "bundle")]
  Bundle(bundle::SubCommands),

  /// Commands to interact with OCI registries.
  #[clap(subcommand, name = "registry", alias = "reg")]
  Registry(registry::SubCommands),

  /// Commands related to signing keys.
  #[clap(subcommand, name = "key")]
  Key(key::SubCommands),

  /// Commands to interact with running Wasmflow instances.
  #[clap(subcommand, name = "rpc")]
  Rpc(rpc::SubCommands),

  /// Command to query JSON, YAML, or TOML file.
  #[clap(name = "query")]
  Query(query::Options),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct FetchOptions {
  /// Allows the use of "latest" artifact tag.
  #[clap(long = "latest", action)]
  pub(crate) allow_latest: bool,

  /// Allows the use of HTTP registry connections to these registries.
  #[clap(long = "insecure", action)]
  pub(crate) insecure_registries: Vec<String>,
}

#[cfg(test)]
mod tests {
  #[test]
  fn verify_options() {
    use clap::IntoApp;
    super::Cli::command().debug_assert();
  }
}
