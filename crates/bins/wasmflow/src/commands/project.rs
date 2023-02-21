use clap::Subcommand;

pub(crate) mod new;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SubCommands {
  /// Create a new boilerplate project
  #[clap(name = "new")]
  New(new::ProjectNewCommand),
}
