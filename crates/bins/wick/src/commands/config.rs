use clap::Subcommand;

pub(crate) mod dot;
pub(crate) mod expand;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SubCommands {
  /// Generate a dot-syntax graph of a composite component.
  #[clap(name = "dot")]
  Dot(dot::Options),
  /// Validate and output an expanded configuration.
  #[clap(name = "expand")]
  Expand(expand::Options),
}
