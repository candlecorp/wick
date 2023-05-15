use clap::Subcommand;

pub(crate) mod login;
pub(crate) mod pull;
pub(crate) mod push;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SubCommands {
  /// Push an artifact or bundle to an OCI registry.
  #[clap(name = "push")]
  Push(push::RegistryPushCommand),

  /// Pull an artifact from an OCI registry.
  #[clap(name = "pull")]
  Pull(pull::RegistryPullCommand),

  /// Save the credentials for a registry.
  #[clap(name = "login")]
  Login(login::RegistryLoginCommand),
}
