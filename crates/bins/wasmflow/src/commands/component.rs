use clap::Subcommand;

pub(crate) mod new;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SubCommands {
  /// Create a new component schema.
  #[clap(name = "new")]
  New(new::Options),
}
