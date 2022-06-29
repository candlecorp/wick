use std::fs::File;
use std::io::Read;

use anyhow::Result;
use clap::Args;

#[derive(Debug, Clone, Args)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct Options {
  #[clap(flatten)]
  pub(crate) logging: logger::LoggingOptions,

  /// WebAssembly module location.
  #[clap(action)]
  pub(crate) module: String,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: Options) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;

  let mut file = File::open(&opts.module)?;
  let mut buf = Vec::new();
  file.read_to_end(&mut buf)?;

  // Extract will return an error if it encounters an invalid hash in the claims
  let claims = wasmflow_wascap::extract_claims(&buf)?;
  match claims {
    Some(claims) => println!("{}", serde_json::to_string(&claims)?),
    None => error!("Error extracting claims"),
  }

  Ok(())
}
