pub(crate) mod inspect;
pub(crate) mod invoke;
pub(crate) mod list;
pub(crate) mod sign;
pub(crate) mod stats;

use std::net::Ipv4Addr;
use std::path::PathBuf;

use logger::LoggingOptions;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[must_use]
pub(crate) fn get_args() -> Cli {
  Cli::from_args()
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(
     global_settings(&[AppSettings::VersionlessSubcommands]),
     name = crate::BIN_NAME, about = "Vino controller")]
pub(crate) struct Cli {
  #[structopt(flatten)]
  pub(crate) command: CliCommand,
}

#[derive(Debug, Clone, StructOpt)]
pub(crate) enum CliCommand {
  /// Invoke a component or schematic on a provider.
  #[structopt(name = "invoke")]
  Invoke(invoke::Options),

  /// Query a provider for a list of its hosted components.
  #[structopt(name = "list")]
  List(list::Options),

  /// Query a provider for its runtime statistics.
  #[structopt(name = "stats")]
  Stats(stats::Options),

  /// Sign a WaPC component.
  #[structopt(name = "sign")]
  Sign(sign::Options),

  /// Inspect the claims of a signed WebAssembly module.
  #[structopt(name = "inspect")]
  Inspect(inspect::Options),
}

#[derive(Debug, Clone, StructOpt)]
pub(crate) struct ConnectOptions {
  /// Port to listen on.
  #[structopt(short, long)]
  pub(crate) port: u16,

  /// IP address to bind to.
  #[structopt(short, long, default_value = "127.0.0.1")]
  pub(crate) address: Ipv4Addr,

  /// Path to pem file for TLS connections.
  #[structopt(long)]
  pub(crate) pem: Option<PathBuf>,

  /// Path to client key for TLS connections.
  #[structopt(long)]
  pub(crate) key: Option<PathBuf>,

  /// Path to CA pem for TLS connections.
  #[structopt(long)]
  pub(crate) ca: Option<PathBuf>,

  /// The domain to verify against the certificate.
  #[structopt(long)]
  pub(crate) domain: Option<String>,
}
