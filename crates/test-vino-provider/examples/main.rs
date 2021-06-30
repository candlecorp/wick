use std::env;
use std::net::Ipv4Addr;
use std::sync::Arc;

use structopt::StructOpt;
use test_vino_provider::Provider;
use tokio::sync::Mutex;
use vino_provider_cli::cli::Options as CliOpts;

#[derive(Debug, Clone, StructOpt)]
pub struct Options {
  /// Port to listen on
  #[structopt(short, long)]
  pub port: u16,

  /// IP address to bind to
  #[structopt(short, long, default_value = "127.0.0.1")]
  pub address: Ipv4Addr,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  env::set_var("RUST_LOG", "trace");
  let opts = Options::from_args();

  env_logger::init();
  vino_provider_cli::init(
    Arc::new(Mutex::new(Provider::default())),
    Some(CliOpts {
      port: Some(opts.port),
      address: opts.address,
    }),
  )
  .await?;
  Ok(())
}
