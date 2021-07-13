use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::Arc;

use structopt::StructOpt;
use tokio::sync::Mutex;
use vino_collection_inmemory::provider::Provider;
use vino_provider_cli::cli::Options as CliOpts;

#[derive(Debug, Clone, StructOpt)]
pub struct Options {
  /// Port to listen on
  #[structopt(short, long)]
  pub port: Option<u16>,

  /// IP address to bind to
  #[structopt(short, long, default_value = "127.0.0.1")]
  pub address: Ipv4Addr,

  /// Path to pem file for TLS
  #[structopt(long)]
  pub pem: Option<PathBuf>,

  /// Path to key file for TLS
  #[structopt(long)]
  pub key: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> vino_collection_inmemory::Result<()> {
  let opts = Options::from_args();

  env_logger::init();
  vino_provider_cli::init_cli(
    Arc::new(Mutex::new(Provider::default())),
    Some(CliOpts {
      port: opts.port,
      address: opts.address,
      pem: opts.pem,
      ca: None,
      key: opts.key,
    }),
  )
  .await?;
  Ok(())
}
