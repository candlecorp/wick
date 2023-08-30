use clap::Subcommand;

pub(crate) mod env;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SubCommands {
  /// Show environment details.
  #[clap(name = "env")]
  Env(env::Options),
}
