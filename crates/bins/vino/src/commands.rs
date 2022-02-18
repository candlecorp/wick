pub(crate) mod list;
pub(crate) mod run;
pub(crate) mod serve;
pub(crate) mod test;

use clap::{AppSettings, Args, Parser, Subcommand};
use logger::LoggingOptions;

#[derive(Parser, Debug, Clone)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder), name = "vino", about = "Vino host")]
pub(crate) struct Cli {
  #[clap(subcommand)]
  pub(crate) command: CliCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum CliCommand {
  /// Start a persistent host from a manifest.
  #[clap(name = "serve")]
  Serve(serve::ServeCommand),
  /// Load a manifest and run the default schematic.
  #[clap(name = "run")]
  Run(run::RunCommand),
  /// Print the schematics and their accessible components for the passed manifest.
  #[clap(name = "list")]
  List(list::ListCommand),
  /// Execute a schematic with test data and assert its output.
  #[clap(name = "test")]
  Test(test::TestCommand),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HostOptions {
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
