use clap::Subcommand;

pub(crate) mod inspect;
pub(crate) mod sign;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SubCommands {
  /// Sign a WebAssembly module.
  #[clap(name = "sign")]
  Sign(sign::Options),

  /// Inspect the claims of a signed WebAssembly module.
  #[clap(name = "inspect")]
  Inspect(inspect::Options),
}
