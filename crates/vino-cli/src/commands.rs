pub(crate) mod run;
pub(crate) mod start;

use logger::options::LoggingOptions;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
     global_settings(&[AppSettings::VersionlessSubcommands]),
     name = "vino", about = "Vino host runtime")]
pub(crate) struct Cli {
  #[structopt(flatten)]
  pub(crate) command: CliCommand,
}

#[derive(Debug, Clone, StructOpt)]
pub(crate) enum CliCommand {
  /// Start a persistent host from a manifest to serve requests to schematics.
  #[structopt(name = "start")]
  Start(start::StartCommand),
  /// Load a manifest and run the default schematic.
  #[structopt(name = "run")]
  Run(run::RunCommand),
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub(crate) struct HostOptions {
  /// Allows the use of "latest" artifact tag.
  #[structopt(long = "allow-latest", env = "VINO_ALLOW_LATEST")]
  pub(crate) allow_latest: Option<bool>,

  /// Allows the use of HTTP registry connections to these registries.
  #[structopt(long = "insecure")]
  pub(crate) insecure_registries: Vec<String>,
}
