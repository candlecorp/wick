pub mod inspect;
pub mod invoke;
pub mod list;
pub mod sign;
pub mod stats;

use std::net::Ipv4Addr;
use std::path::PathBuf;

use logger::options::LoggingOptions;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[must_use]
pub fn get_args() -> Cli {
  Cli::from_args()
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(
     global_settings(&[AppSettings::VersionlessSubcommands]),
     name = "vino", about = "Vino host runtime")]
pub struct Cli {
  #[structopt(flatten)]
  pub command: CliCommand,
}

#[derive(Debug, Clone, StructOpt)]
pub enum CliCommand {
  /// Invoke a component or schematic on a provider
  #[structopt(name = "invoke")]
  Invoke(invoke::InvokeCommand),
  /// Query a provider for a list of its hosted components
  #[structopt(name = "list")]
  List(list::ListCommand),
  /// Query a provider for its runtime statistics
  #[structopt(name = "stats")]
  Stats(stats::StatsCommand),
  /// Sign a WaPC component
  #[structopt(name = "sign")]
  Sign(sign::SignCommand),
  /// Inspect the claims of a signed WaPC component
  #[structopt(name = "inspect")]
  Inspect(inspect::InspectCommand),
}

#[derive(Debug, Clone, StructOpt)]
pub struct ConnectOptions {
  /// Port to listen on
  #[structopt(short, long)]
  pub port: u16,

  /// IP address to bind to
  #[structopt(short, long, default_value = "127.0.0.1")]
  pub address: Ipv4Addr,

  /// Path to pem file for TLS connections
  #[structopt(long)]
  pub pem: Option<PathBuf>,

  /// Path to client key for TLS connections
  #[structopt(long)]
  pub key: Option<PathBuf>,

  /// Path to CA pem for TLS connections
  #[structopt(long)]
  pub ca: Option<PathBuf>,

  /// The domain to verify against the certificate
  #[structopt(long)]
  pub domain: Option<String>,
}
