use clap::Subcommand;

pub(crate) mod new;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SubCommands {
  #[clap(name = "new")]
  New(new::Options),
}
