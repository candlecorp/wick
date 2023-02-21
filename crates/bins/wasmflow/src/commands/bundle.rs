use clap::Subcommand;

pub(crate) mod pack;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SubCommands {
  /// Create a signed archive bundle.
  #[clap(name = "pack")]
  Pack(pack::BundlePackCommand),
}
