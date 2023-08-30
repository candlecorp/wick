use clap::Subcommand;

pub(crate) mod login;
pub(crate) mod manifest;
pub(crate) mod pull;
pub(crate) mod push;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SubCommands {
  /// Push an artifact or bundle to an OCI registry.
  #[clap(name = "push")]
  Push(push::Options),

  /// Pull an artifact from an OCI registry.
  #[clap(name = "pull")]
  Pull(pull::Options),

  /// Save the credentials for a registry.
  #[clap(name = "login")]
  Login(login::Options),

  /// Retrieve the manifest for a package.
  #[clap(name = "manifest")]
  Manifest(manifest::Options),
}
