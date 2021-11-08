pub(crate) mod list;
pub(crate) mod run;
pub(crate) mod serve;
pub(crate) mod test;

use logger::LoggingOptions;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(global_settings(
  &[
    AppSettings::VersionlessSubcommands,
    AppSettings::ColoredHelp,
    AppSettings::DeriveDisplayOrder,
    AppSettings::UnifiedHelpMessage
  ]),
  name = "vino",
  about = "Vino host",
)]
pub(crate) struct Cli {
  #[structopt(flatten)]
  pub(crate) command: CliCommand,
}

#[derive(Debug, Clone, StructOpt)]
pub(crate) enum CliCommand {
  /// Start a persistent host from a manifest.
  #[structopt(name = "serve")]
  Serve(serve::ServeCommand),
  /// Load a manifest and run the default schematic.
  #[structopt(name = "run")]
  Run(run::RunCommand),
  /// Print the schematics and their accessible components for the passed manifest.
  #[structopt(name = "list")]
  List(list::ListCommand),
  /// Execute a schematic with test data and assert its output.
  #[structopt(name = "test")]
  Test(test::TestCommand),
}

#[derive(Debug, Clone, StructOpt)]
pub(crate) struct HostOptions {
  /// Allows the use of "latest" artifact tag.
  #[structopt(long = "allow-latest")]
  pub(crate) allow_latest: bool,

  /// Allows the use of HTTP registry connections to these registries.
  #[structopt(long = "insecure")]
  pub(crate) insecure_registries: Vec<String>,
}
