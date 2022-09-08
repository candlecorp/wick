use std::path::PathBuf;

use clap::{Args, Subcommand};
use url::Url;

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
  /// RPC Url
  #[clap(short, long)]
  pub(crate) url: Url,

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

impl ConnectOptions {
  pub(crate) fn host(&self) -> String {
    self.url.host().unwrap().to_string()
  }

  pub(crate) fn port(&self) -> u16 {
    self.url.port().unwrap()
  }
}
