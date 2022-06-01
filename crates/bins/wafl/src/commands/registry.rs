use clap::Subcommand;

pub(crate) mod pull;
pub(crate) mod push;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SubCommands {
  /// Push an artifact or bundle to an OCI registry .
  #[clap(name = "push")]
  Push(push::Options),

  /// Pull an artifact or architecture specific bundle item from an OCI registry .
  #[clap(name = "push")]
  Pull(pull::Options),
}
