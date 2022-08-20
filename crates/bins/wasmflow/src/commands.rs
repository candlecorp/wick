pub(crate) mod invoke;
pub(crate) mod list;
pub(crate) mod run;
pub(crate) mod serve;
pub(crate) mod test;

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
  /// Start a persistent host from a manifest.
  #[clap(name = "serve")]
  Serve(serve::ServeCommand),
  /// Load a manifest and execute an entrypoint component (temporarily disabled).
  #[clap(name = "run", skip)]
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
