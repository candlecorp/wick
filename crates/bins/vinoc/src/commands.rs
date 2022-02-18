pub(crate) mod inspect;
pub(crate) mod invoke;
pub(crate) mod list;
pub(crate) mod pull;
pub(crate) mod push;
pub(crate) mod sign;
pub(crate) mod stats;

use std::net::Ipv4Addr;
use std::path::PathBuf;

use clap::{AppSettings, Args, Parser, Subcommand};
use logger::LoggingOptions;

#[derive(Parser, Debug, Clone)]
#[clap(
     global_setting(AppSettings::DeriveDisplayOrder),
     name = crate::BIN_NAME, about = "Vino controller")]
pub(crate) struct Cli {
  #[clap(subcommand)]
  pub(crate) command: CliCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum CliCommand {
  /// Invoke a component or schematic on a provider.
  #[clap(name = "invoke")]
  Invoke(invoke::Options),

  /// Query a provider for a list of its hosted components.
  #[clap(name = "list")]
  List(list::Options),

  /// Query a provider for its runtime statistics.
  #[clap(name = "stats")]
  Stats(stats::Options),

  /// Sign a WaPC component.
  #[clap(name = "sign")]
  Sign(sign::Options),

  /// Inspect the claims of a signed WebAssembly module.
  #[clap(name = "inspect")]
  Inspect(inspect::Options),

  /// Push an artifact or bundle to an OCI registry .
  #[clap(name = "push")]
  Push(push::Options),

  /// Pull an artifact or architecture specific bundle item from an OCI registry .
  #[clap(name = "push")]
  Pull(pull::Options),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ConnectOptions {
  /// RPC port.
  #[clap(short, long, env = vino_provider_cli::options::env::VINO_RPC_PORT)]
  pub(crate) port: u16,

  /// RPC address.
  #[clap(short, long, default_value = "127.0.0.1", env = vino_provider_cli::options::env::VINO_RPC_ADDRESS)]
  pub(crate) address: Ipv4Addr,

  /// Path to pem file for TLS connections.
  #[clap(long)]
  pub(crate) pem: Option<PathBuf>,

  /// Path to client key for TLS connections.
  #[clap(long)]
  pub(crate) key: Option<PathBuf>,

  /// Path to CA pem for TLS connections.
  #[clap(long)]
  pub(crate) ca: Option<PathBuf>,

  /// The domain to verify against the certificate.
  #[clap(long)]
  pub(crate) domain: Option<String>,
}

#[cfg(test)]
mod test {
  #[test]
  fn verify_options() {
    use clap::IntoApp;
    super::Cli::command().debug_assert();
  }
}
