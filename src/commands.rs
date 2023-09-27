pub(crate) mod config;
pub(crate) mod install;
pub(crate) mod invoke;
pub(crate) mod key;
pub(crate) mod list;
pub(crate) mod new;
pub(crate) mod query;
pub(crate) mod registry;
pub(crate) mod rpc;
pub(crate) mod run;
pub(crate) mod serve;
pub(crate) mod show;
pub(crate) mod test;
pub(crate) mod wasm;

use clap::{Parser, Subcommand};

use crate::options::logging::LoggingOptions;
use crate::options::GlobalOptions;

#[derive(Parser, Debug, Clone)]
#[clap(
  name = crate::BIN_NAME,
  about = crate::BIN_DESC,
  version,
)]
pub(crate) struct Cli {
  #[clap(flatten)]
  pub(crate) output: GlobalOptions,
  #[clap(flatten)]
  pub(crate) logging: LoggingOptions,
  #[clap(subcommand)]
  pub(crate) command: CliCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum CliCommand {
  /// Start a persistent host from a manifest.
  #[clap(name = "serve")]
  Serve(serve::Options),

  /// Run a wick application.
  #[clap(name = "run")]
  Run(run::Options),

  /// Invoke an operation.
  #[clap(name = "invoke")]
  Invoke(invoke::Options),

  /// Print the signature of a component.
  #[clap(name = "list")]
  List(list::Options),

  /// Run test cases against a component.
  #[clap(name = "test")]
  Test(test::Options),

  /// Create new app and component configurations.
  #[clap(subcommand, name = "new")]
  New(new::SubCommands),

  /// Install a wick app to the local system.
  #[clap(name = "install")]
  Install(install::Options),

  /// Show information about wick's configuration or manifest details.
  #[clap(subcommand, name = "show")]
  Show(show::SubCommands),

  /// Commands to help audit, explain, and validate configurations.
  #[clap(subcommand, name = "config")]
  Config(config::SubCommands),

  /// Commands for WebAssembly component.
  #[clap(subcommand, name = "wasm")]
  Wasm(wasm::SubCommands),

  /// Commands to interact with OCI registries.
  #[clap(subcommand, name = "registry", alias = "reg")]
  Registry(registry::SubCommands),

  /// Commands related to signing keys.
  #[clap(subcommand, name = "key")]
  Key(key::SubCommands),

  /// Commands to interact with running Wick instances.
  #[clap(subcommand, name = "rpc")]
  Rpc(rpc::SubCommands),

  /// Command to apply a liquid template to a JSON, YAML, or TOML file.
  #[clap(name = "query")]
  Query(query::Options),
}

#[cfg(test)]
mod tests {
  use clap::CommandFactory;

  #[test]
  fn verify_options() {
    super::Cli::command().debug_assert();
  }
}
