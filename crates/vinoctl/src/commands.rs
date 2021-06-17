pub mod start;

use structopt::{clap::AppSettings, StructOpt};

pub fn get_args() -> Cli {
  Cli::from_args()
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(
     global_settings(&[AppSettings::VersionlessSubcommands]),
     name = "vino", about = "Vino host runtime")]
pub struct Cli {
  #[structopt(flatten)]
  pub command: CliCommand,
}

#[derive(Debug, Clone, StructOpt)]
pub enum CliCommand {
  /// Start a long-running host with a manifest and schematics
  #[structopt(name = "start")]
  Start(start::StartCommand),
}
