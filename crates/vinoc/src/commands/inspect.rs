use std::fs::File;
use std::io::Read;

use structopt::StructOpt;

use crate::Result;
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct InspectCommand {
  #[structopt(flatten)]
  pub logging: super::LoggingOptions,

  /// File to read
  pub(crate) module: String,

  #[structopt(flatten)]
  common: GenerateCommon,
}

#[derive(Debug, Clone, StructOpt)]
struct GenerateCommon {
  /// Location of key files for signing. Defaults to $VINO_KEYS ($HOME/.vino/keys)
  #[structopt(long = "directory", env = "VINO_KEYS", hide_env_values = true)]
  directory: Option<String>,

  /// Indicates the token expires in the given amount of days. If this option is left off, the token will never expire
  #[structopt(short = "x", long = "expires")]
  expires_in_days: Option<u64>,

  /// Period in days that must elapse before this token is valid. If this option is left off, the token will be valid immediately
  #[structopt(short = "b", long = "nbf")]
  not_before_days: Option<u64>,
}

pub async fn handle_command(command: InspectCommand) -> Result<()> {
  crate::utils::init_logger(&command.logging)?;

  let mut file = File::open(&command.module)?;
  let mut buf = Vec::new();
  file.read_to_end(&mut buf)?;

  // Extract will return an error if it encounters an invalid hash in the claims
  let claims = vino_wascap::extract_claims(&buf)?;
  match claims {
    Some(claims) => println!("{}", serde_json::to_string(&claims)?),
    None => error!("Error extracting claims"),
  }

  Ok(())
}
