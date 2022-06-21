use std::net::Ipv4Addr;
use std::path::PathBuf;

use clap::{Args, Subcommand};

pub(crate) mod invoke;
pub(crate) mod list;
pub(crate) mod stats;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SubCommands {
  /// Invoke a component in a collection.
  #[clap(name = "invoke")]
  Invoke(invoke::Options),

  /// Query a collection for a list of its components.
  #[clap(name = "list")]
  List(list::Options),

  /// Query a collection for its runtime statistics.
  #[clap(name = "stats")]
  Stats(stats::Options),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ConnectOptions {
  /// RPC port.
  #[clap(short, long, env = wasmflow_collection_cli::options::env::WAFL_RPC_PORT,action)]
  pub(crate) port: u16,

  /// RPC address.
  #[clap(short, long, default_value = "127.0.0.1", env = wasmflow_collection_cli::options::env::WAFL_RPC_ADDRESS,action)]
  pub(crate) address: Ipv4Addr,

  /// Path to pem file for TLS connections.
  #[clap(long, action)]
  pub(crate) pem: Option<PathBuf>,

  /// Path to client key for TLS connections.
  #[clap(long, action)]
  pub(crate) key: Option<PathBuf>,

  /// Path to CA pem for TLS connections.
  #[clap(long, action)]
  pub(crate) ca: Option<PathBuf>,

  /// The domain to verify against the certificate.
  #[clap(long, action)]
  pub(crate) domain: Option<String>,
}
