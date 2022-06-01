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
  about = "Vino host",
  version = option_env!("VINO_VERSION").unwrap_or("0.0.0")
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
  /// Load a manifest and execute an entrypoint provider.
  #[clap(name = "run")]
  Run(run::RunCommand),
  /// Load a manifest and run the default schematic.
  #[clap(name = "invoke")]
  Invoke(invoke::InvokeCommand),
  /// Print the schematics and their accessible components for the passed manifest.
  #[clap(name = "list")]
  List(list::ListCommand),
  /// Execute a schematic with test data and assert its output.
  #[clap(name = "test")]
  Test(test::TestCommand),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct FetchOptions {
  /// Allows the use of "latest" artifact tag.
  #[clap(long = "allow-latest")]
  pub(crate) allow_latest: bool,

  /// Allows the use of HTTP registry connections to these registries.
  #[clap(long = "insecure")]
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
