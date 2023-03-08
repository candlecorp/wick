use clap::Subcommand;

pub(crate) mod gen;
pub(crate) mod get;
pub(crate) mod list;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SubCommands {
  /// Generate new signing keys.
  #[clap(name = "gen")]
  Gen(gen::KeyGenCommand),

  /// List all found keys.
  #[clap(name = "list")]
  List(list::KeyListCommand),

  /// Read key data.
  #[clap(name = "get")]
  Get(get::KeyGetCommand),
}
