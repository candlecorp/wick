use clap::Subcommand;

pub(crate) mod dotviz;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SubCommands {
  /// Generate a dotviz graph of a composite component.
  #[clap(name = "dotviz")]
  Dotviz(dotviz::Options),
}
