use std::fs::File;
use std::io::Read;

use structopt::StructOpt;

use crate::error::ControlError;
use crate::Result;
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub(crate) struct Options {
  #[structopt(flatten)]
  pub(crate) logging: super::LoggingOptions,

  /// File to read.
  pub(crate) module: String,
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle(opts: Options) -> Result<()> {
  let _guard = crate::utils::init_logger(&opts.logging)?;

  let mut file = File::open(&opts.module).map_err(ControlError::ReadFailed)?;
  let mut buf = Vec::new();
  file.read_to_end(&mut buf).map_err(ControlError::ReadFailed)?;

  // Extract will return an error if it encounters an invalid hash in the claims
  let claims = vino_wascap::extract_claims(&buf)?;
  match claims {
    Some(claims) => println!("{}", serde_json::to_string(&claims)?),
    None => error!("Error extracting claims"),
  }

  Ok(())
}
