pub mod error;

use std::collections::HashMap;
use std::io::Read;
use std::net::Ipv4Addr;
use std::path::PathBuf;

use actix::SyncArbiter;
use vino_codec::messagepack::serialize;
use vino_component::Packet;
use vino_runtime::prelude::MessageTransport;
pub use vow::error::VowError as Error;

pub(crate) type Result<T> = std::result::Result<T, vow::error::VowError>;

use structopt::StructOpt;
use vow::wasm_service::{
  Call,
  WasmService,
};

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct CliOptions {
  /// Path to WebAssembly binary
  wasm: String,

  /// The name of the component to execute
  component_name: String,

  /// JSON data
  data: Option<String>,

  /// Allow :latest tags for OCI artifacts
  #[structopt(long)]
  pub latest: bool,

  /// List of insecure registries to allow pulling from
  #[structopt(long)]
  pub insecure: Vec<String>,

  /// Port to listen on
  #[structopt(short, long)]
  pub port: Option<u16>,

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

  #[structopt(flatten)]
  pub logging: logger::LoggingOptions,
}

#[actix::main]
async fn main() -> Result<()> {
  let opts = CliOptions::from_args();
  logger::Logger::init(&opts.logging)?;

  match run(opts).await {
    Ok(_) => {}
    Err(e) => {
      tracing::error!("{}", e.to_string());
    }
  }

  Ok(())
}

async fn run(opts: CliOptions) -> Result<()> {
  let data = match opts.data {
    None => {
      eprintln!("No input passed, reading from <STDIN>");
      let mut data = String::new();
      std::io::stdin().read_to_string(&mut data)?;
      data
    }
    Some(i) => i,
  };

  let json: HashMap<String, serde_json::value::Value> = serde_json::from_str(&data)?;
  let multibytes: HashMap<String, Vec<u8>> = json
    .into_iter()
    .map(|(name, val)| Ok((name, serialize(val)?)))
    .filter_map(Result::ok)
    .collect();

  let component = vino_runtime::prelude::load_wasm(&opts.wasm, opts.latest, opts.insecure).await?;

  let addr = SyncArbiter::start(5, move || WasmService::new(&component));

  let result: HashMap<String, Packet> = addr
    .send(Call {
      component: opts.component_name,
      message: MessageTransport::MultiBytes(multibytes),
    })
    .await
    .map_err(|_| Error::InternalError(200))??;

  let mut map = serde_json::Map::new();
  for (k, v) in result.into_iter() {
    let transport: MessageTransport = v.into();
    map.insert(k, transport.into_json());
  }

  println!("{}", serde_json::to_string(&map)?);

  std::process::exit(0);
}
